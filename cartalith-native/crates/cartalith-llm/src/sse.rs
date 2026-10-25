//! Server-Sent Events framing, incremental, over raw byte chunks.
//!
//! WHY: both adapters stream over SSE (LLM_INTEGRATION_RESEARCH.md s.3; the
//! framing is shared, the payloads are not). Godot's `HTTPClient` hands the
//! caller arbitrary byte chunks, so a `data:` line, a CRLF pair or a UTF-8
//! character can be cut at any byte. This parser buffers until a line is
//! complete and only then decodes it, which is what makes a split chunk
//! harmless. Follows the WHATWG SSE line grammar (`data`, `event`, comments
//! starting `:`, blank line dispatches, one leading space stripped); `id` and
//! `retry` are ignored because this crate never reconnects (LLM_ASSISTANT_SCOPE
//! s.4 rule 7: no automatic retry beyond one, and that retry restarts the
//! request rather than resuming).
//!
//! MUST NEVER: decode a partial line (a multi-byte character may be cut), or
//! dispatch a message whose terminating blank line has not arrived (the WHATWG
//! rule: an incomplete event at end of stream is discarded).

use crate::error::LlmError;

/// Largest single SSE line accepted. Labelled judgement: a tool-call argument
/// delta or a text delta is a few hundred bytes; 1 MiB is far above any honest
/// line and bounds memory against a hostile or broken server.
pub const MAX_LINE_BYTES: usize = 1 << 20;

/// Largest total payload of one event (all its lines). Labelled judgement,
/// 4 MiB, same reason.
pub const MAX_EVENT_BYTES: usize = 4 << 20;

/// One dispatched SSE message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SseMessage {
    /// The `event:` field, `None` when absent (OpenAI-style streams omit it;
    /// Anthropic's typed events set it).
    pub event: Option<String>,
    /// The `data:` lines joined with `\n`.
    pub data: String,
}

/// Incremental SSE parser. Feed chunks with [`SseParser::push`].
#[derive(Debug, Default)]
pub struct SseParser {
    /// Bytes received but not yet part of a complete line.
    buf: Vec<u8>,
    /// `data:` lines of the event being assembled.
    data: Vec<String>,
    /// `event:` of the event being assembled.
    event: Option<String>,
    /// Payload bytes of the event being assembled, for the size guard.
    event_bytes: usize,
}

impl SseParser {
    /// A fresh parser.
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed a chunk; returns every message the chunk completed (possibly none).
    ///
    /// Errors: [`LlmError::InvalidUtf8`] for a complete line that is not UTF-8,
    /// [`LlmError::Oversized`] past the size guards. After an error the parser
    /// should be discarded.
    pub fn push(&mut self, chunk: &[u8]) -> Result<Vec<SseMessage>, LlmError> {
        self.buf.extend_from_slice(chunk);
        let mut out = Vec::new();
        let mut start = 0usize;
        while let Some(rel) = self.buf[start..].iter().position(|&b| b == b'\n' || b == b'\r') {
            let end = start + rel;
            let mut next = end + 1;
            if self.buf[end] == b'\r' {
                // A CR may be the first half of CRLF; with nothing after it we
                // cannot tell yet, so wait for the next chunk.
                if next >= self.buf.len() {
                    break;
                }
                if self.buf[next] == b'\n' {
                    next += 1;
                }
            }
            let line = std::str::from_utf8(&self.buf[start..end])
                .map_err(|_| LlmError::InvalidUtf8)?
                .to_owned();
            start = next;
            if let Some(msg) = self.line(&line)? {
                out.push(msg);
            }
        }
        self.buf.drain(..start);
        if self.buf.len() > MAX_LINE_BYTES {
            return Err(LlmError::Oversized);
        }
        Ok(out)
    }

    /// Bytes buffered without a complete line, plus whether an event was
    /// started but not dispatched. Non-zero at end of stream means the server
    /// cut off mid-event; the decoder uses its own end marker, so this is for
    /// diagnostics and tests.
    pub fn has_partial(&self) -> bool {
        !self.buf.is_empty() || !self.data.is_empty()
    }

    /// Apply one complete line to the event being assembled.
    fn line(&mut self, line: &str) -> Result<Option<SseMessage>, LlmError> {
        if line.is_empty() {
            // Blank line: dispatch if any data arrived, always reset.
            let event = self.event.take();
            let data = std::mem::take(&mut self.data);
            self.event_bytes = 0;
            if data.is_empty() {
                return Ok(None);
            }
            return Ok(Some(SseMessage { event, data: data.join("\n") }));
        }
        if line.starts_with(':') {
            // Comment / keep-alive. Kept for legibility only: without this branch a
            // `:` line splits into the empty field name, which the match below
            // ignores, so deleting it is an *equivalent* mutant (no test can
            // tell, by construction; checked 2026-10-25).
            return Ok(None);
        }
        let (field, value) = match line.split_once(':') {
            Some((f, v)) => (f, v.strip_prefix(' ').unwrap_or(v)),
            None => (line, ""),
        };
        self.event_bytes += line.len();
        if self.event_bytes > MAX_EVENT_BYTES {
            return Err(LlmError::Oversized);
        }
        match field {
            "data" => self.data.push(value.to_owned()),
            "event" => self.event = Some(value.to_owned()),
            _ => {} // id, retry, unknown: ignored, see module docs
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all(chunks: &[&[u8]]) -> Vec<SseMessage> {
        let mut p = SseParser::new();
        let mut v = Vec::new();
        for c in chunks {
            v.extend(p.push(c).unwrap());
        }
        v
    }

    /// Protects: the basic grammar, with LF, CRLF and CR terminators giving the
    /// same messages (providers and proxies differ).
    #[test]
    fn all_three_line_endings_parse_alike() {
        let lf = all(&[b"data: a\n\ndata: b\n\n"]);
        let crlf = all(&[b"data: a\r\n\r\ndata: b\r\n\r\n"]);
        let cr = all(&[b"data: a\r\rdata: b\r\r\n"]);
        let want: Vec<String> = vec!["a".into(), "b".into()];
        for got in [lf, crlf, cr] {
            assert_eq!(got.iter().map(|m| m.data.clone()).collect::<Vec<_>>(), want);
        }
    }

    /// Protects: the LLM-1 bar "a `data:` line split across chunks parses",
    /// cut at every byte offset, including inside a CRLF and inside a
    /// multi-byte character.
    #[test]
    fn a_stream_cut_at_every_byte_parses_identically() {
        let whole = "event: x\r\ndata: h\u{e9}llo \u{4e16}\u{754c}\r\ndata: line2\r\n\r\n: ping\r\ndata: z\r\n\r\n";
        let bytes = whole.as_bytes();
        let want = all(&[bytes]);
        assert_eq!(want.len(), 2);
        assert_eq!(want[0].event.as_deref(), Some("x"));
        assert_eq!(want[0].data, "h\u{e9}llo \u{4e16}\u{754c}\nline2");
        for cut in 1..bytes.len() {
            let got = all(&[&bytes[..cut], &bytes[cut..]]);
            assert_eq!(got, want, "cut at byte {cut}");
        }
    }

    /// Protects: an event with no terminating blank line is not dispatched.
    #[test]
    fn an_unterminated_event_is_not_dispatched() {
        let mut p = SseParser::new();
        assert!(p.push(b"data: partial\n").unwrap().is_empty());
        assert!(p.has_partial());
    }

    /// Protects: a complete line that is not UTF-8 is an error, not lossy text.
    #[test]
    fn invalid_utf8_is_an_error() {
        let mut p = SseParser::new();
        assert_eq!(p.push(b"data: \xff\xfe\n\n"), Err(LlmError::InvalidUtf8));
    }

    /// Protects: the memory guard. Mutation: raise `MAX_LINE_BYTES` past the
    /// test size and this goes red.
    #[test]
    fn an_endless_line_is_cut_off() {
        let mut p = SseParser::new();
        let big = vec![b'a'; 1_100_000];
        assert_eq!(p.push(&big), Err(LlmError::Oversized));
    }

    /// Protects: the exact guard values, as literals (not against themselves).
    #[test]
    fn guard_constants_are_the_documented_literals() {
        assert_eq!(MAX_LINE_BYTES, 1_048_576);
        assert_eq!(MAX_EVENT_BYTES, 4_194_304);
    }
}
