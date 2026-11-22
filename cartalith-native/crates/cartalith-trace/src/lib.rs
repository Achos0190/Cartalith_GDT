//! Opt-in structured tracing (`TOOLING_PLAN.md` M1, Ruling GP item 3).
//!
//! **Off by default.** Until [`start`] is called no `tracing` subscriber exists, so every span and
//! event macro in the workspace is a disabled static callsite (one cached atomic load). [`start`]
//! installs one global subscriber whose only layer is [`Trace`]: it writes JSON lines to
//! `trace-<yyyymmdd-hhmmss>.jsonl` in a directory the caller names, through a bounded channel to a
//! writer thread that rotates (5 files of 8 MiB), so a slow disk never stalls generation (a full
//! channel drops the record and counts it).
//!
//! **Privacy is structural, not a promise.** The layer records only fields whose *key* is in
//! [`NUM_KEYS`] (numbers and bools) or [`STR_KEYS`] (strings that also pass [`is_name`], lowercase
//! `[a-z0-9_.:]`, 48 bytes at most, so a note sentence or a `sk-...` key cannot pass). Every other
//! field is dropped and only counted (`dropped`). An event's free-text `message` is kept only when it
//! arrived through the `log` bridge from `wgpu`/`naga`/`ash`. Never add a key that can hold user
//! text, a path, a prompt or a secret.
//!
//! Must never: be called per cell (a span wraps a stage, not an iteration), call into Godot (it is
//! used from worker threads), or panic.

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber, span};
use tracing_core::LevelFilter;
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;

/// Numeric/boolean field keys that may be logged: counts, sizes, durations, seed, indexes.
pub const NUM_KEYS: &[&str] = &[
    "cells", "w", "h", "gw", "gh", "seed", "n", "count", "bytes", "bytes_in", "bytes_out", "tokens", "ms", "hit",
    "idx", "threads", "level", "tier",
];
/// String field keys that may be logged, and then only when [`is_name`] holds for the value.
pub const STR_KEYS: &[&str] = &["op", "stage", "kind", "form", "tool", "name", "backend"];
/// Log-bridge targets whose `message` text is kept (library diagnostics, never user text).
const MESSAGE_TARGETS: &[&str] = &["wgpu", "naga", "ash"];
const FILE_BYTES: u64 = 8 * 1024 * 1024;
const FILES_KEPT: usize = 5;

/// True for an operation-style name: `[a-z0-9_.:]`, 1..=48 bytes. Rejects spaces, capitals and `-`.
pub fn is_name(s: &str) -> bool {
    !s.is_empty() && s.len() <= 48 && s.bytes().all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'.' | b':'))
}

/// JSON-escapes `s` into `out` (quotes included).
fn esc(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// `yyyymmdd-hhmmss` (UTC) for a Unix time (Howard Hinnant's civil-from-days).
pub fn stamp(secs: u64) -> String {
    let (days, rem) = ((secs / 86_400) as i64, secs % 86_400);
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}{m:02}{d:02}-{:02}{:02}{:02}", rem / 3600, rem % 3600 / 60, rem % 60)
}

/// Collects the allowlisted fields of one span or event as a JSON object body.
#[derive(Default)]
struct Fields {
    json: String,
    dropped: u32,
    message: Option<String>,
    log_target: Option<String>,
}

impl Fields {
    fn put(&mut self, key: &str, write: impl FnOnce(&mut String)) {
        if !self.json.is_empty() {
            self.json.push(',');
        }
        esc(&mut self.json, key);
        self.json.push(':');
        write(&mut self.json);
    }
    fn num(&mut self, f: &Field, write: impl FnOnce(&mut String)) {
        if NUM_KEYS.contains(&f.name()) {
            self.put(f.name(), write);
        } else {
            self.dropped += 1;
        }
    }
}

impl Visit for Fields {
    fn record_u64(&mut self, f: &Field, v: u64) {
        self.num(f, |o| {
            let _ = write!(o, "{v}");
        });
    }
    fn record_i64(&mut self, f: &Field, v: i64) {
        self.num(f, |o| {
            let _ = write!(o, "{v}");
        });
    }
    fn record_f64(&mut self, f: &Field, v: f64) {
        self.num(f, |o| {
            let _ = write!(o, "{}", if v.is_finite() { v } else { 0.0 });
        });
    }
    fn record_bool(&mut self, f: &Field, v: bool) {
        self.num(f, |o| {
            let _ = write!(o, "{v}");
        });
    }
    fn record_str(&mut self, f: &Field, v: &str) {
        match f.name() {
            "log.target" => self.log_target = Some(v.to_owned()),
            "message" => self.message = Some(v.to_owned()),
            k if STR_KEYS.contains(&k) && is_name(v) => self.put(k, |o| esc(o, v)),
            _ => self.dropped += 1,
        }
    }
    fn record_debug(&mut self, f: &Field, v: &dyn std::fmt::Debug) {
        // `message` arrives here (a `format_args!`); every other debug-formatted value is dropped.
        if f.name() == "message" {
            self.message = Some(format!("{v:?}"));
        } else if f.name() != "log.module_path" && f.name() != "log.file" && f.name() != "log.line" {
            self.dropped += 1;
        }
    }
}

/// What a span remembers between open and close.
struct SpanData {
    start: Instant,
    fields: String,
}

enum Msg {
    Line(Vec<u8>),
    Flush(SyncSender<()>),
}

/// Process-wide state: the runtime level (0 off .. 5 trace), the writer's channel, a drop counter.
struct State {
    level: AtomicU8,
    tx: Mutex<Option<SyncSender<Msg>>>,
    lost: AtomicU64,
    t0: Instant,
}

static STATE: OnceLock<State> = OnceLock::new();
fn state() -> &'static State {
    STATE.get_or_init(|| State { level: AtomicU8::new(0), tx: Mutex::new(None), lost: AtomicU64::new(0), t0: Instant::now() })
}

fn level_num(l: &Level) -> u8 {
    match *l {
        Level::ERROR => 1,
        Level::WARN => 2,
        Level::INFO => 3,
        Level::DEBUG => 4,
        Level::TRACE => 5,
    }
}

fn parse_level(s: &str) -> u8 {
    match s.trim().to_ascii_lowercase().as_str() {
        "error" => 1,
        "warn" => 2,
        "info" | "on" | "1" | "true" => 3,
        "debug" => 4,
        "trace" => 5,
        _ => 0,
    }
}

/// True while records are being written. Cheap; safe from any thread.
pub fn is_on() -> bool {
    STATE.get().is_some_and(|s| s.level.load(Ordering::Relaxed) != 0)
}

/// The layer: turns spans and events into allowlisted JSON lines.
struct Trace;

fn send(line: String) {
    let s = state();
    let guard = s.tx.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(tx) = guard.as_ref()
        && let Err(TrySendError::Full(_) | TrySendError::Disconnected(_)) = tx.try_send(Msg::Line(line.into_bytes()))
    {
        s.lost.fetch_add(1, Ordering::Relaxed);
    }
}

fn head(out: &mut String, kind: &str, meta: &tracing::Metadata<'_>) {
    let s = state();
    let _ = write!(
        out,
        "{{\"ts_us\":{},\"t\":\"{kind}\",\"level\":\"{}\",\"target\":",
        s.t0.elapsed().as_micros(),
        meta.level()
    );
    esc(out, meta.target());
    let _ = write!(out, ",\"thread\":\"{:?}\"", std::thread::current().id());
}

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for Trace {
    fn enabled(&self, m: &tracing::Metadata<'_>, _: Context<'_, S>) -> bool {
        level_num(m.level()) <= state().level.load(Ordering::Relaxed)
    }
    fn max_level_hint(&self) -> Option<LevelFilter> {
        Some(match state().level.load(Ordering::Relaxed) {
            0 => LevelFilter::OFF,
            1 => LevelFilter::ERROR,
            2 => LevelFilter::WARN,
            3 => LevelFilter::INFO,
            4 => LevelFilter::DEBUG,
            _ => LevelFilter::TRACE,
        })
    }
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        let mut f = Fields::default();
        attrs.record(&mut f);
        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(SpanData { start: Instant::now(), fields: f.json });
        }
    }
    fn on_close(&self, id: span::Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(&id) else { return };
        let ext = span.extensions();
        let Some(d) = ext.get::<SpanData>() else { return };
        let mut o = String::new();
        head(&mut o, "span", span.metadata());
        let _ = write!(o, ",\"name\":\"{}\",\"id\":{},\"parent\":{},\"elapsed_us\":{},\"f\":{{{}}}}}\n",
            span.name(), id.into_u64(), span.parent().map_or(0, |p| p.id().into_u64()), d.start.elapsed().as_micros(), d.fields);
        send(o);
    }
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut f = Fields::default();
        event.record(&mut f);
        let mut o = String::new();
        head(&mut o, "event", event.metadata());
        let _ = write!(o, ",\"name\":\"{}\",\"parent\":{}", event.metadata().name().split(' ').next().unwrap_or(""),
            ctx.event_span(event).map_or(0, |s| s.id().into_u64()));
        if let (Some(m), Some(t)) = (&f.message, &f.log_target)
            && MESSAGE_TARGETS.iter().any(|p| t.starts_with(p))
        {
            o.push_str(",\"log_target\":");
            esc(&mut o, t);
            o.push_str(",\"message\":");
            esc(&mut o, &m.chars().take(300).collect::<String>());
        }
        let _ = write!(o, ",\"f\":{{{}}},\"dropped\":{}}}\n", f.json, f.dropped);
        send(o);
    }
}

/// Writer thread: appends lines, flushes when idle, rotates at [`FILE_BYTES`], keeps [`FILES_KEPT`].
fn writer(rx: Receiver<Msg>, dir: PathBuf, first: PathBuf) {
    let open = |p: &Path| std::fs::OpenOptions::new().create(true).append(true).open(p).ok();
    let (mut file, mut written, mut seq) = (open(&first), 0u64, 0u32);
    let base = first.file_stem().and_then(|s| s.to_str()).unwrap_or("trace").to_owned();
    loop {
        match rx.recv_timeout(std::time::Duration::from_millis(250)) {
            Ok(Msg::Line(b)) => {
                if written + b.len() as u64 > FILE_BYTES {
                    seq += 1;
                    file = open(&dir.join(format!("{base}-{seq}.jsonl")));
                    written = 0;
                    prune(&dir);
                }
                if let Some(f) = file.as_mut() {
                    let _ = f.write_all(&b);
                }
                written += b.len() as u64;
            }
            Ok(Msg::Flush(ack)) => {
                if let Some(f) = file.as_mut() {
                    let _ = f.flush();
                }
                let _ = ack.send(());
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if let Some(f) = file.as_mut() {
                    let _ = f.flush();
                }
            }
            Err(_) => return,
        }
    }
}

/// Deletes the oldest `trace-*.jsonl` beyond [`FILES_KEPT`] (names sort by time).
fn prune(dir: &Path) {
    let mut v: Vec<_> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("trace-"))
        .map(|e| e.path())
        .collect();
    v.sort();
    for p in v.iter().take(v.len().saturating_sub(FILES_KEPT)) {
        let _ = std::fs::remove_file(p);
    }
}

/// Turns tracing on at `level` ("error".."trace"; "info"/"on" = info), writing into `dir`
/// (created). Returns the file path, or `None` if the level is unparseable/off, the directory cannot
/// be made, or another tracing subscriber is already installed. Idempotent: a second call restarts
/// the writer in `dir`. Installing the subscriber is the only global side effect.
pub fn start(level: &str, dir: &Path) -> Option<PathBuf> {
    let lv = parse_level(level);
    if lv == 0 || std::fs::create_dir_all(dir).is_err() {
        return None;
    }
    let s = state();
    static INSTALLED: OnceLock<bool> = OnceLock::new();
    if !*INSTALLED.get_or_init(|| tracing::subscriber::set_global_default(tracing_subscriber::registry().with(Trace)).is_ok()) {
        return None;
    }
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let path = dir.join(format!("trace-{}.jsonl", stamp(secs)));
    let (tx, rx) = sync_channel(8192);
    let (d2, p2) = (dir.to_owned(), path.clone());
    std::thread::Builder::new().name("cartalith-trace".into()).spawn(move || writer(rx, d2, p2)).ok()?;
    *s.tx.lock().unwrap_or_else(|e| e.into_inner()) = Some(tx);
    s.level.store(lv, Ordering::Relaxed);
    tracing_core::callsite::rebuild_interest_cache();
    Some(path)
}

/// Turns tracing off again (records stop; the file is flushed). Callsites go back to disabled.
pub fn stop() {
    if let Some(s) = STATE.get() {
        s.level.store(0, Ordering::Relaxed);
        tracing_core::callsite::rebuild_interest_cache();
        flush();
        *s.tx.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
}

/// Blocks (up to 2 s) until everything sent so far is on disk. For tests, `stop` and shutdown.
pub fn flush() {
    let Some(s) = STATE.get() else { return };
    let tx = s.tx.lock().unwrap_or_else(|e| e.into_inner()).clone();
    if let Some(tx) = tx {
        let (ack, wait) = sync_channel(1);
        if tx.send(Msg::Flush(ack)).is_ok() {
            let _ = wait.recv_timeout(std::time::Duration::from_secs(2));
        }
    }
}

/// Records dropped because the writer's channel was full.
pub fn lost() -> u64 {
    STATE.get().map_or(0, |s| s.lost.load(Ordering::Relaxed))
}

/// Forwards one `log` record into the stream (the desktop logger calls this while tracing is on).
pub fn forward_log(record: &log::Record<'_>) {
    let _ = tracing_log::format_trace(record);
}

/// Opens an `info` event from the shell: `op` must pass [`is_name`]; `ms` and `n` are the only
/// numbers the shell hook passes. Returns false (and logs nothing) when off or the name is refused.
pub fn shell_event(op: &str, ms: f64, n: i64) -> bool {
    if !is_on() || !is_name(op) {
        return false;
    }
    tracing::info!(target: "shell", op = op, ms = ms, n = n);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Protects the privacy gate: a sentence, a key shape and a path are not names; an op is.
    #[test]
    fn is_name_rejects_text_and_keys() {
        assert!(is_name("menu.run:generate_world"));
        assert!(!is_name("The queen hides the sword"));
        assert!(!is_name("sk-CANARY123456"));
        assert!(!is_name("C:/Users/x"));
        assert!(!is_name(""));
        assert!(!is_name(&"a".repeat(49)));
        assert!(is_name(&"a".repeat(48)));
    }

    /// Protects the file-name stamp against a calendar slip (a literal date, leap-year day).
    #[test]
    fn stamp_is_a_utc_calendar_date() {
        assert_eq!(stamp(0), "19700101-000000");
        assert_eq!(stamp(1_709_210_096), "20240229-123456");
    }

    #[test]
    fn esc_escapes_quotes_and_controls() {
        let mut s = String::new();
        esc(&mut s, "a\"b\\c\n");
        assert_eq!(s, "\"a\\\"b\\\\c\\u000a\"");
    }
}
