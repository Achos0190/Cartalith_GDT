import subprocess, os, sys
ROOT = 'C:/cw/f12/cartalith-native/'
ENV = dict(os.environ, CARGO_TARGET_DIR='C:/cw/f12_target')
EA = ROOT + 'crates/cartalith-llm/src/edit_actions.rs'
PF = ROOT + 'crates/cartalith-llm/src/params_flow.rs'
RE = ROOT + 'crates/cartalith-terrain/src/region_edit.rs'
M = [
 ('F12-1 lazy ridge back for strokes', EA, 'let mapping = if boxed {', 'let mapping = if !boxed {', 'cartalith-llm'),
 ('F12-2 stroke kind wording dropped', EA, 'format!("Region kind: a painted stroke (a path of dabs), {size}', 'format!("Region kind: a painted thing, {size}', 'cartalith-llm'),
 ('F12-3 ridge word removed from hint table', EA, 'words: &["ridge", "ridges", ', 'words: &["ridges", ', 'cartalith-llm'),
 ('F12-4 raise branch left open', EA, '"required": ["operation", "region_id"], "additionalProperties": false})', '"required": ["operation", "region_id"], "additionalProperties": true})', 'cartalith-llm'),
 ('F12-5 mesa example over the limit', EA, 'amount_m\\":200,\\"reason\\":\\"Flat top.', 'amount_m\\":600,\\"reason\\":\\"Flat top.', 'cartalith-llm'),
 ('F12-6 profile slices reversed', EA, 'r.profile.iter().map(|m| m.map_or("-".to_owned()', 'r.profile.iter().rev().map(|m| m.map_or("-".to_owned()', 'cartalith-llm'),
 ('F12-7 contradictory peak line restored', EA, 'Raising stops at {:.0} m above sea (the world\'s height scale)', 'The world\'s peak altitude is {:.0} m (the world\'s height scale)', 'cartalith-llm'),
 ('F12-7b highest cell sentence dropped', EA, ' The highest cell anywhere in this world is {h:.0} m.', '', 'cartalith-llm'),
 ('F12-8 second revise allowed', PF, 'if self.revised {\n            return Err("The model has already', 'if false {\n            return Err("The model has already', 'cartalith-llm'),
 ('F12-8b preview feedback claims applied', EA, 'json!({"status": "previewed", "applied": false,', 'json!({"status": "previewed", "applied": true,', 'cartalith-llm'),
 ('F12-8c preview whitelist widened to land_fraction', EA, 'for k in ["min_m", "mean_m", "max_m"] {', 'for k in ["min_m", "mean_m", "max_m", "land_fraction"] {', 'cartalith-llm'),
 ('F12-2b stroke length drops the round ends', RE, 'length_cells: length + 2.0 * mask.path_r', 'length_cells: length + mask.path_r', 'cartalith-terrain'),
 ('F12-6b profile axis sign flipped', RE, 'axis: [(b[0] - a[0]) / d, (b[1] - a[1]) / d] })', 'axis: [(b[0] - a[0]) / d, (a[1] - b[1]) / d] })', 'cartalith-terrain'),
 ('F12-4b amount_m required again (grammar order fault)', EA, '"required": ["operation", "region_id"], "additionalProperties": false})', '"required": ["operation", "region_id", "amount_m"], "additionalProperties": false})', 'cartalith-llm'),
 ('F12-compass west/east swapped', EA, 'format!("{} to {}", NAMES[(to + 4) % 8], NAMES[to])', 'format!("{} to {}", NAMES[to], NAMES[(to + 4) % 8])', 'cartalith-llm'),
]
ONLY = sys.argv[1:]
res = []
for name, path, old, new, crate in M:
    if ONLY and not any(name.startswith(o) for o in ONLY):
        continue
    src = open(path, encoding='utf8', newline='').read()
    old_n = old.replace('\n', '\r\n') if '\r\n' in src else old
    new_n = new.replace('\n', '\r\n') if '\r\n' in src else new
    if src.count(old_n) != 1:
        res.append((name, 'NOT_ONCE %d' % src.count(old_n))); continue
    try:
        open(path, 'w', encoding='utf8', newline='').write(src.replace(old_n, new_n))
        r = subprocess.run(['cargo', 'test', '-p', crate, '--lib'], cwd=ROOT, env=ENV, capture_output=True, text=True)
        out = r.stdout + r.stderr
        if 'error[' in out or 'could not compile' in out:
            res.append((name, 'BUILD_ERROR'))
        elif r.returncode != 0:
            failed = [l.split()[1] for l in out.splitlines() if l.startswith('test ') and l.endswith('FAILED')]
            res.append((name, 'KILLED by ' + ', '.join(failed[:3])))
        else:
            res.append((name, 'SURVIVED'))
    finally:
        open(path, 'w', encoding='utf8', newline='').write(src)
for n, r in res:
    print(n, '=>', r)
