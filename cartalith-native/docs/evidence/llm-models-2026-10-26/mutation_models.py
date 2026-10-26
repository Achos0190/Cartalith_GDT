"""Mutation test for src/models.rs and src/settings.rs (literal replace, exactly once, restore in finally)."""
import hashlib, os, subprocess, sys

ROOT = "C:/cw/ml1/cartalith-native"
M = ROOT + "/crates/cartalith-llm/src/models.rs"
S = ROOT + "/crates/cartalith-llm/src/settings.rs"
ENV = dict(os.environ, CARGO_TARGET_DIR="C:/cw/ml1t")

MUTS = [
    (M, 'Some("embeddings") => ModelKind::Embedding', 'Some("embeddings") => ModelKind::Chat'),
    (M, 'Some("not-loaded") => Some(false)', 'Some("not-loaded") => Some(true)'),
    (M, 'pub const MAX_ENTRIES: usize = 500;', 'pub const MAX_ENTRIES: usize = 501;'),
    (M, 'matches!(f.code, "unauthorized" | "forbidden" | "redirect" | "too_large")', 'matches!(f.code, "forbidden" | "redirect" | "too_large")'),
    (S, 'if need_model && s.model.trim().is_empty()', 'if s.model.trim().is_empty()'),
    (S, 'if need_model && s.model.trim().is_empty()', 'if false && s.model.trim().is_empty()'),
    (M, '404 | 405 => ListFailure::http("not_found", status)', '405 => ListFailure::http("not_found", status)'),
    (M, 'pub const MAX_PAGES: u8 = 5;', 'pub const MAX_PAGES: u8 = 6;'),
    (M, 'caps.contains(&"embedding")', 'caps.contains(&"embeddingx")'),
    (M, 's.trim() != s ||', ''),
    (M, 'Header::new("x-api-key", k.expose().to_owned(), true)', 'Header::new("Authorization", k.expose().to_owned(), true)'),
    (M, 'e.loaded != Some(true)', 'e.loaded == Some(true)'),
    (M, '|| s.chars().any(is_hostile_char)', ''),
    (M, 'format!("{}/api/v0/models", origin_root(base_url)?)', 'format!("{}/api/v1/models", origin_root(base_url)?)'),
    (M, '        if self.retried {\n            return Step::Failed(f);\n        }\n', ''),
    (M, 'Fault::Connect => Step::Failed(ListFailure { code: "unreachable", status: None, detail }),', 'Fault::Connect => { self.retried = true; return self.request(None); }'),
    (M, '.filter(|c| cursor_ok(c))', ''),
    (M, 'if body.len() > MAX_BODY_BYTES {', 'if body.len() > MAX_BODY_BYTES * 2 {'),
    (M, '(ApiStyle::AnthropicMessages, _) => vec![ModelSource::AnthropicModels],', '(ApiStyle::AnthropicMessages, _) => vec![ModelSource::OpenAiModels],'),
    (M, '300..=399 => ListFailure::http("redirect", status)', '300..=399 => ListFailure::http("http", status)'),
]


def sha(p):
    return hashlib.sha256(open(p, 'rb').read()).hexdigest()


def run():
    r = subprocess.run(["cargo", "test", "-j", "2", "-p", "cartalith-llm", "--lib", "--test", "models_wires"], cwd=ROOT, env=ENV, capture_output=True, text=True)
    out = r.stdout + r.stderr
    if r.returncode == 0:
        return "SURVIVED"
    if "could not compile" in out or "error[" in out:
        return "BUILD_ERROR"
    return "KILLED"


before = {M: sha(M), S: sha(S)}
res = []
for path, a, b in MUTS:
    orig = open(path, encoding='utf-8', newline='').read()
    assert orig.count(a) == 1, ("not exactly once", a, orig.count(a))
    try:
        open(path, 'w', encoding='utf-8', newline='').write(orig.replace(a, b))
        res.append((run(), a[:70]))
    finally:
        open(path, 'w', encoding='utf-8', newline='').write(orig)
    print(res[-1], flush=True)
assert before == {M: sha(M), S: sha(S)}, "file not restored"
print("SUMMARY", {k: sum(1 for r in res if r[0] == k) for k in ("KILLED", "SURVIVED", "BUILD_ERROR")})
