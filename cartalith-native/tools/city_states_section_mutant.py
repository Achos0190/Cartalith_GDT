"""Ruling HX / HY literal mutants (exact replace, occurs once, restored in `finally`).
  python -I cartalith-native/tools/city_states_section_mutant.py hx|hy|restore
`hx` flips the City-states list's sort so row 0 is no longer the most populous seat (`city_states_section_drive.gd` goes red on
"row 0 names the most populous seat"). `hy` drops `civ.border_friction` from `NUMERIC_RULES` (the `project_bridge.rs` restore test
goes red on 0.6). Run the probe or `cargo test` yourself between `hx|hy` and `restore`; this only applies and undoes the edit."""
import pathlib, sys
R = pathlib.Path(__file__).resolve().parents[1]
M = {
    "hx": (R / "godot-project/shell/workspaces/civilization_workspace.gd",
           'int(x.get("population", 0)) > int(y.get("population", 0))', 'int(x.get("population", 0)) < int(y.get("population", 0))'),
    "hy": (R / "crates/cartalith-godot/src/params.rs",
           'NUMERIC_RULES: &[&str] = &["civ.border_friction"]', 'NUMERIC_RULES: &[&str] = &[]'),
}
k = sys.argv[1]
if k == "restore":
    for p, a, b in M.values():
        t = p.read_text(encoding="utf-8")
        if b in t: p.write_text(t.replace(b, a), encoding="utf-8")
else:
    p, a, b = M[k]; t = p.read_text(encoding="utf-8")
    assert t.count(a) == 1, "literal must occur exactly once"
    p.write_text(t.replace(a, b), encoding="utf-8")
print("ok", k)
