"""Ruling GT (2026-11-22): literal mutants for the timeline ladder.

Rules (MISTAKES.md "Mutation-test"): exact literal replace that must occur exactly once, a BUILD_ERROR is reported apart from a
SURVIVED, every file is restored in a `finally` from the text read before the edit, and the file's hash is compared with the
pre-run value at the end (zero residue).

  python -I cartalith-native/tools/ladder_mutants.py rust            # each Rust mutant against `cargo test` (names the test that must go red)
  python -I cartalith-native/tools/ladder_mutants.py shell-apply     # applies every GDScript mutant at once (they are independent), prints them
  python -I cartalith-native/tools/ladder_mutants.py shell-restore   # restores them

The GDScript mutants are applied TOGETHER and `tools/ladder_drive.gd` is run once on the desktop (and the swipe mutant on the
phone): each mutant is aimed at a different check, so the run must print one FAIL per mutant and the control run (no mutants) none.
Each entry names the check that must go red.
"""
import hashlib, json, pathlib, subprocess, sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
STATE = pathlib.Path("C:/cw/ladder_mutants_state.json")

# (file, exact old text, new text, the test (Rust) or drive check (GDScript) that must go red)
RUST = [
    ("crates/cartalith-civ/src/timeline_items.rs", "if imported_at(n.entity, &n.relative_path, &key) {", "if false && imported_at(n.entity, &n.relative_path, &key) {",
     "cartalith-civ", "linked_and_imported_note_counts_once"),
    ("crates/cartalith-civ/src/timeline_items.rs", ".filter(|i| !i.private && !matches!(i.source, ItemSource::Snapshot | ItemSource::Ownership))", ".filter(|i| !matches!(i.source, ItemSource::Snapshot | ItemSource::Ownership))",
     "cartalith-civ", "consolidated_document_is_marked_ordered_and_leaves_out_private"),
    ("crates/cartalith-civ/src/timeline_items.rs", "        if l == GENERATED_MARKER {\n            return true;", "        if l != \"\" {\n            return true;",
     "cartalith-civ", "only_a_marked_frontmatter_counts_as_generated"),
    ("crates/cartalith-vault/src/lib.rs", "        if !is_ours(&existing) {\n            return Ok(false);", "        if false {\n            return Ok(false);",
     "cartalith-vault", "write_generated_never_replaces_a_file_it_does_not_own"),
    ("crates/cartalith-civ/src/timeline_items.rs", "    it.color = e.color.clone();\n    it", "    it",
     "cartalith-civ", "items_carry_their_chronos_kind_and_colour"),
]
SHELL = [
    ("godot-project/shell/chronos_lanes.gd", "const LANDMARKS_MAX := 5", "const LANDMARKS_MAX := 6", "L0: at most five landmark ticks (T5)"),
    ("godot-project/shell/chronos_items.gd", 'if String(d.get("ev_kind", "")) == "period":', 'if String(d.get("ev_kind", "")) == "periodx":', "L0: the strip shows the vault's 5 `@` periods as bands"),
    ("godot-project/shell/chronos_lanes.gd", 'var c := String(it.get("color", ""))', 'var c := ""', "T10: the tagged mark is painted red, an untagged one is not"),
    ("godot-project/shell/chronos_items.gd", "(ev as InputEventKey).keycode == KEY_ESCAPE and level > 0", "(ev as InputEventKey).keycode == KEY_F23 and level > 0", "Esc steps down from L1 to L0"),
    ("godot-project/shell/chronos_items.gd", "var dflt := 0 if _compact else 1", "var dflt := 1 if _compact else 0", "it rests at L1 on the desktop (Ruling GT, T4)"),
    ("godot-project/shell/chronos_lanes.gd", "elif hit.size() > 1 and level >= 1:", "elif hit.size() > 1 and level >= 9:", "L1: a cluster jumps to L2"),
    ("godot-project/shell/vault_window.gd", "ed.text_edit.editable = false", "ed.text_edit.editable = true", "L3: it is read-only with a banner"),
    ("godot-project/shell/chronos_prompt.gd", '"keep", "ignore":', '"keepx", "ignorex":', "Keep applies on click, with an Undo"),
    ("godot-project/shell/chronos_items.gd", "if not _s_kinds.is_empty() and not _s_kinds.has(_kind_word(d)):", "if not _s_kinds.is_empty() and _s_kinds.has(_kind_word(d)):", "L2: Kind = Note leaves exactly the vault's events"),
    ("godot-project/shell/chronos_lanes.gd", "on_swipe.call(-1 if d.y < 0.0 else 1)", "on_swipe.call(1 if d.y < 0.0 else -1)", "phone: a swipe up on the strip climbs to L1"),
    ("godot-project/shell/chronos_items.gd", 'var d := {"kinds": PackedStringArray(["Ownership"])}', 'var d := {"kinds": PackedStringArray(["Ownershipx"])}', "Ownership spans join the model when a place is selected"),
]


def sha(p):
    return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()


def _nl(raw: bytes, s: str) -> bytes:
    """`s` as bytes in the file's own newline style (a Windows checkout may hold CRLF)."""
    return (s.replace("\n", "\r\n") if b"\r\n" in raw else s).encode("utf-8")


def rust():
    results = []
    for rel, old, new, crate, test in RUST:
        p = ROOT / rel
        before = p.read_bytes()
        ob, nb = _nl(before, old), _nl(before, new)
        assert before.count(ob) == 1, f"{rel}: the literal must occur exactly once: {old[:50]!r} ({before.count(ob)})"
        try:
            p.write_bytes(before.replace(ob, nb))
            r = subprocess.run(["cargo", "test", "-p", crate, "--lib", test], cwd=ROOT, capture_output=True, text=True, timeout=1500)
            out = r.stdout + r.stderr
            if "error[" in out or "could not compile" in out:
                verdict = "BUILD_ERROR"
            elif r.returncode != 0 and f"{test} ... FAILED" in out:
                verdict = "KILLED"
            else:
                verdict = "SURVIVED"
        finally:
            p.write_bytes(before)
        assert p.read_bytes() == before
        results.append((rel, test, verdict))
        print(f"{verdict:12} {test}  <- {old[:60]!r}", flush=True)
    return results


def shell_apply():
    state = {}
    only = {int(x) for x in sys.argv[2].split(",")} if len(sys.argv) > 2 else None  # indexes into SHELL
    for k, (rel, old, new, check) in enumerate(SHELL):
        if only is not None and k not in only:
            continue
        p = ROOT / rel
        raw = p.read_bytes()
        ob, nb = _nl(raw, old), _nl(raw, new)
        assert raw.count(ob) == 1, f"{rel}: {old!r} occurs {raw.count(ob)} times"
        state.setdefault(rel, raw.hex())  # the ORIGINAL bytes, kept from the first mutant of each file
        p.write_bytes(raw.replace(ob, nb))
        print(f"APPLIED  {rel}: {old!r} -> must turn red: {check}")
    STATE.write_text(json.dumps({r: [h, sha(ROOT / r)] for r, h in state.items()}), encoding="utf-8")


def shell_restore():
    state = json.loads(STATE.read_text(encoding="utf-8"))
    for rel, (hexed, _h) in state.items():
        (ROOT / rel).write_bytes(bytes.fromhex(hexed))
        print("RESTORED", rel, sha(ROOT / rel)[:12])
    STATE.unlink()


if __name__ == "__main__":
    {"rust": rust, "shell-apply": shell_apply, "shell-restore": shell_restore}[sys.argv[1]]()
