//! The API key type and the redactor that keeps it out of error text.
//!
//! WHY: LLM_ASSISTANT_SCOPE.md s.4 rule 2 (Ruling DD item 4): the key is never
//! in an error string or a `Debug` dump, and the UI shows the last four
//! characters only. Persistence of the key (OS store, env var) is LLM-2, not
//! here; this crate only holds a key in memory for one request.
//!
//! MUST NEVER: derive `Debug`/`Display` on [`ApiKey`], or add an accessor that
//! returns the key other than [`ApiKey::expose`], whose only callers are the
//! request builder (the header) and the redactor (the needle).

use std::fmt;

use crate::error::{LlmError, Scrubbed};

/// What replaces the key in scrubbed text.
pub const REDACTED: &str = "[redacted-key]";

/// Keys of at most this many characters show no characters at all in
/// [`ApiKey::masked`]. Labelled judgement: the scope asks for "last four", but
/// four characters of a six-character key (the dummy `ollama`) is most of the
/// key; 8 keeps at least half hidden.
pub const MASK_MIN_LEN_FOR_SUFFIX: usize = 8;

/// A provider API key held in memory. `Debug` prints `ApiKey(****)`; there is
/// no `Display`. `Clone` exists because the decoder and the request builder
/// each hold one for the duration of a request.
#[derive(Clone)]
pub struct ApiKey(String);

impl ApiKey {
    /// Validate and wrap a key. Rejects empty keys (no value is not a
    /// plausible key; a keyless local server passes `None` to the builder
    /// instead) and any character outside printable ASCII `!`..`~`, because a
    /// space, CR or LF would split or inject headers. Errors never echo it.
    pub fn new(raw: &str) -> Result<Self, LlmError> {
        if raw.is_empty() {
            return Err(LlmError::InvalidKey("key is empty"));
        }
        if !raw.bytes().all(|b| (0x21..=0x7E).contains(&b)) {
            return Err(LlmError::InvalidKey(
                "key contains a space, control or non-ASCII character",
            ));
        }
        Ok(ApiKey(raw.to_owned()))
    }

    /// The raw key. Callers: the header builder and the redactor only.
    pub(crate) fn expose(&self) -> &str {
        &self.0
    }

    /// What the UI may show: the last four characters, or `****` for a short
    /// key (see [`MASK_MIN_LEN_FOR_SUFFIX`]).
    pub fn masked(&self) -> String {
        let n = self.0.chars().count();
        if n <= MASK_MIN_LEN_FOR_SUFFIX {
            "****".to_owned()
        } else {
            let tail: String = self.0.chars().skip(n - 4).collect();
            format!("****{tail}")
        }
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey(****)")
    }
}

/// Removes a key from text before it can enter an error. With no key (a
/// keyless local server) it still produces [`Scrubbed`] values, unchanged.
#[derive(Clone)]
pub struct Redactor {
    /// Raw key plus its JSON-escaped form when that differs (a key holding `"`
    /// or `\` appears escaped inside a JSON error body).
    needles: Vec<String>,
}

impl fmt::Debug for Redactor {
    /// Never prints the needles: they are the key.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Redactor({} needle(s))", self.needles.len())
    }
}

impl Redactor {
    /// A redactor for `key`, or a pass-through one for `None`.
    pub fn new(key: Option<&ApiKey>) -> Self {
        let mut needles = Vec::new();
        if let Some(k) = key {
            needles.push(k.expose().to_owned());
            let json = serde_json::to_string(k.expose()).unwrap_or_default();
            let escaped = json.trim_matches('"').to_owned();
            if !escaped.is_empty() && escaped != k.expose() {
                needles.push(escaped);
            }
        }
        Redactor { needles }
    }

    /// Replace every occurrence of the key with [`REDACTED`]. Longest needle
    /// first so the escaped form is not half-replaced by the raw one.
    pub fn scrub(&self, text: &str) -> Scrubbed {
        let mut order: Vec<&String> = self.needles.iter().collect();
        order.sort_by_key(|n| std::cmp::Reverse(n.len()));
        let mut out = text.to_owned();
        for n in order {
            out = out.replace(n.as_str(), REDACTED);
        }
        Scrubbed::new_unchecked(out)
    }
}
