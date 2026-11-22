"""Literal mutants for Ruling FX: exact replace occurring once, BUILD_ERROR apart from SURVIVED, restore in finally.
python mutants.py   (run from cartalith-native; CARGO_TARGET_DIR set by the caller)"""
import hashlib, subprocess, sys
L = 'crates/cartalith-llm/src/'
M = [
 ("keep step", L + 'catalogue.rs', '        e.remove("step");\n', '', ['-p', 'cartalith-godot', '--test', 'param_catalogue', 'the_schema_result_is_trimmed']),
 ("keep float type", L + 'catalogue.rs', 'if e.get("type") == Some(&json!("float")) {', 'if false {', ['-p', 'cartalith-godot', '--test', 'param_catalogue', 'the_schema_result_is_trimmed']),
 ("always send default", L + 'catalogue.rs', 'if e.get("default") == e.get("current") {', 'if false {', ['-p', 'cartalith-godot', '--test', 'param_catalogue', 'the_schema_result_is_trimmed']),
 ("keep advanced desc", L + 'catalogue.rs', '            e.remove("desc");\n', '', ['-p', 'cartalith-godot', '--test', 'param_catalogue', 'the_schema_result_is_trimmed']),
 ("advanced string back", L + 'catalogue.rs', 'e.insert("advanced".into(), json!(true));', 'e.insert("advanced".into(), json!("ruled behaviour"));', ['-p', 'cartalith-godot', '--test', 'param_catalogue', 'the_schema_result_is_trimmed']),
 ("repeat re-sends", L + 'params_flow.rs', '!self.shown_keys.iter().any(|s| s == key)', 'true', ['-p', 'cartalith-llm', '--lib', 'a_repeated_schema_round']),
 ("repeat note dropped", L + 'params_flow.rs', 'if settings.is_empty() {', 'if false {', ['-p', 'cartalith-llm', '--lib', 'a_repeated_schema_round']),
 ("design window 8192", L + 'action_tools.rs', 'pub const DESIGN_WINDOW_TOKENS: u32 = 12288;', 'pub const DESIGN_WINDOW_TOKENS: u32 = 8192;', ['-p', 'cartalith-llm', '--lib', 'the_guard_refuses_with_numbers']),
 ("design window 16384", L + 'action_tools.rs', 'pub const DESIGN_WINDOW_TOKENS: u32 = 12288;', 'pub const DESIGN_WINDOW_TOKENS: u32 = 16384;', ['-p', 'cartalith-llm', '--lib', 'the_guard_refuses_with_numbers']),
]
for name, path, old, new, cmd in M:
    s = open(path, encoding='utf-8').read(); h = hashlib.sha256(s.encode()).hexdigest()
    assert s.count(old) == 1, (name, s.count(old))
    try:
        open(path, 'w', encoding='utf-8').write(s.replace(old, new))
        r = subprocess.run(['cargo', 'test'] + cmd, capture_output=True, text=True)
        out = r.stdout + r.stderr
        if 'error[' in out or 'could not compile' in out: v = 'BUILD_ERROR'
        elif r.returncode != 0 and 'FAILED' in out: v = 'KILLED'
        elif r.returncode == 0: v = 'SURVIVED'
        else: v = 'BUILD_ERROR'
    finally:
        open(path, 'w', encoding='utf-8').write(s)
    assert hashlib.sha256(open(path, encoding='utf-8').read().encode()).hexdigest() == h
    print(name, v, flush=True)
