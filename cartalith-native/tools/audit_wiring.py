#!/usr/bin/env python3
"""Reachability audit: what is built but not wired to anything.

This port has now found the same class of defect five separate times --
something ported, golden-tested, registered as done, and called by nothing.
Every previous find was by eye, mid-task, by accident. This answers it
mechanically instead.

Four questions:

  A. Which `pub fn` in the workspace has no caller outside test code?
  B. Which `#[func]` on the GDExtension does no `.gd` file ever name?
  C. Which name does the shell ask `world_gen` for that the engine does not
     define?  (A stale `_has("...")` guard silently disables a feature
     forever: the shell degrades instead of failing, so nothing is red.)
  D. Which `func` in `engine_bridge.gd` does nothing else call?

Run:  python cartalith-native/tools/audit_wiring.py [--json]

## What this is NOT

It is a *flagger*, not a verdict. A hit is one of four things and only
reading tells you which:

  * a superseded wrapper kept on purpose (`jp_plan` -> `jp_plan_full`,
    `place_settlements` -> `..._with_water_edge_snap`);
  * a deliberate non-use, disclosed at the call site (`civ_zoom_pick_r`);
  * ordinary accessor surface on a data structure (most of
    `cartalith-spatial`);
  * an actual gap.

So it prints counts and a list; it does not fail a build. Making it a red
test would need an allowlist, and an allowlist of 60-odd entries is a second
place for the truth to rot.

Test code means `#[cfg(test)] mod ... { }` (brace-matched, not
regex-guessed) plus everything under `tests/`, `examples/` and `benches/`.
`.gd` files starting with `_` are probes and screenshot scripts, not the
shell, and are excluded from C and D.
"""
from __future__ import annotations

import argparse
import json
import re
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRATES = ROOT / "crates"
GD = ROOT / "godot-project"
GODOT_SRC = CRATES / "cartalith-godot" / "src"

# Methods every Godot `Object` answers to; the shell may legitimately call
# these on `world_gen` and they are not engine `#[func]`s.
OBJECT_BUILTINS = {
    "has_method", "call", "call_deferred", "get", "set", "connect",
    "disconnect", "is_connected", "free", "queue_free", "get_class",
    "emit_signal", "notification", "get_property_list", "get_method_list",
}


def strip_comments(src: str) -> str:
    """Blank out `//` and `/* */` so a doc comment is never read as a call."""
    out: list[str] = []
    i, n, in_str = 0, len(src), False
    while i < n:
        c = src[i]
        if in_str:
            if c == "\\":
                out.append("  ")
                i += 2
                continue
            if c == '"':
                in_str = False
            out.append(c)
            i += 1
        elif c == '"':
            in_str = True
            out.append(c)
            i += 1
        elif c == "/" and i + 1 < n and src[i + 1] == "/":
            j = src.find("\n", i)
            j = n if j < 0 else j
            out.append(" " * (j - i))
            i = j
        elif c == "/" and i + 1 < n and src[i + 1] == "*":
            j = src.find("*/", i + 2)
            j = n if j < 0 else j + 2
            out.append("".join(ch if ch == "\n" else " " for ch in src[i:j]))
            i = j
        else:
            out.append(c)
            i += 1
    return "".join(out)


def strip_test_mods(src: str) -> str:
    """Remove every `#[cfg(test)] mod name { .. }` by matching braces."""
    out = src
    while True:
        m = re.search(r"#\[cfg\(test\)\]\s*mod\s+\w+\s*\{", out)
        if not m:
            return out
        i, depth = m.end() - 1, 0
        while i < len(out):
            if out[i] == "{":
                depth += 1
            elif out[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        out = out[: m.start()] + out[i + 1 :]


def uses(name: str, blob: str) -> int:
    return len(re.findall(r"(?<![A-Za-z0-9_])" + re.escape(name) + r"(?![A-Za-z0-9_])", blob))


def audit() -> dict:
    prod: list[str] = []
    defs: dict[str, list[str]] = defaultdict(list)
    engine_funcs: dict[str, str] = {}

    for p in CRATES.rglob("*.rs"):
        parts = set(p.parts)
        is_test = bool(parts & {"tests", "examples", "benches"})
        text = strip_comments(p.read_text(encoding="utf-8", errors="replace"))
        rel = p.relative_to(ROOT).as_posix()
        if is_test:
            continue
        prod.append(strip_test_mods(text))
        for m in re.finditer(r"\bpub(?:\(crate\))?\s+(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)", text):
            defs[m.group(1)].append(f"{rel}:{text.count(chr(10), 0, m.start()) + 1}")
        if p.is_relative_to(GODOT_SRC):
            for m in re.finditer(r"#\[func\][^\n]*\n\s*(?:pub\s+)?fn\s+([a-z_][a-z0-9_]*)", text):
                engine_funcs[m.group(1)] = f"{rel}:{text.count(chr(10), 0, m.start()) + 1}"

    prod_blob = "\n".join(prod)
    unwired = [
        {"name": n, "sites": s}
        for n, s in sorted(defs.items())
        if uses(n, prod_blob) <= len(s)  # only the definition(s) themselves
    ]

    gd_all = sorted(GD.rglob("*.gd"))
    gd_shell = [p for p in gd_all if not p.name.startswith("_")]
    gd_blob = "\n".join(p.read_text(encoding="utf-8", errors="replace") for p in gd_all)

    unreached = [
        {"name": n, "at": at} for n, at in sorted(engine_funcs.items()) if uses(n, gd_blob) == 0
    ]

    asked: dict[str, set[str]] = defaultdict(set)
    for p in gd_shell:
        t = p.read_text(encoding="utf-8", errors="replace")
        for pat in (
            r'_has\(\s*"([a-z_][a-z0-9_]*)"\s*\)',
            r"world_gen\.([a-z_][a-z0-9_]*)",
            r'world_gen\.has_method\(\s*"([a-z_][a-z0-9_]*)"\s*\)',
        ):
            for m in re.finditer(pat, t):
                asked[m.group(1)].add(p.name)
    stale = {
        n: sorted(f)
        for n, f in sorted(asked.items())
        if n not in engine_funcs and n not in OBJECT_BUILTINS
    }

    bridge = GD / "shell" / "engine_bridge.gd"
    bt = bridge.read_text(encoding="utf-8", errors="replace")
    others = "\n".join(
        p.read_text(encoding="utf-8", errors="replace") for p in gd_all if p != bridge
    )
    dead = [
        {"name": m.group(1), "line": bt.count("\n", 0, m.start()) + 1}
        for m in re.finditer(r"^func\s+([a-z_][a-z0-9_]*)\s*\(", bt, re.M)
        if not m.group(1).startswith("_") and uses(m.group(1), others) == 0
    ]

    return {
        "pub_fn_total": len(defs),
        "pub_fn_unwired": unwired,
        "engine_func_total": len(engine_funcs),
        "engine_func_unreached": unreached,
        "shell_asks_for_missing": stale,
        "engine_bridge_uncalled": dead,
    }


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--json", action="store_true", help="machine-readable output")
    r = audit()
    if ap.parse_args().json:
        print(json.dumps(r, indent=1))
        return

    print(f"A. pub fn with no production caller: {len(r['pub_fn_unwired'])} of {r['pub_fn_total']}")
    by_crate: dict[str, list[str]] = defaultdict(list)
    for f in r["pub_fn_unwired"]:
        by_crate[f["sites"][0].split("/")[1]].append(f["name"])
    for crate in sorted(by_crate):
        print(f"   {crate} ({len(by_crate[crate])}): {', '.join(sorted(by_crate[crate]))}")

    print(f"\nB. #[func] no .gd names: {len(r['engine_func_unreached'])} of {r['engine_func_total']}")
    for f in r["engine_func_unreached"]:
        print(f"   {f['name']:<34} {f['at']}")

    print(f"\nC. shell asks world_gen for methods that do not exist: {len(r['shell_asks_for_missing'])}")
    for n, files in r["shell_asks_for_missing"].items():
        print(f"   {n:<34} <- {', '.join(files)}")
    if not r["shell_asks_for_missing"]:
        print("   (none -- no stale _has() guard is silently disabling a feature)")

    print(f"\nD. engine_bridge.gd funcs nothing calls: {len(r['engine_bridge_uncalled'])}")
    for f in r["engine_bridge_uncalled"]:
        print(f"   {f['name']:<34} engine_bridge.gd:{f['line']}")


if __name__ == "__main__":
    main()
