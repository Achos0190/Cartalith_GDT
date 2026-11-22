"""Exact-literal mutation runner (MISTAKES.md): each mutant replaces a literal that occurs exactly once, runs the named
cargo test filter, and is KILLED (tests fail), SURVIVED (tests pass) or BUILD_ERROR (does not compile). The file is
restored in a finally. Run: python -I mutants.py [name ...]"""
import os, pathlib, subprocess, sys

ROOT = pathlib.Path("C:/cw/tl/cartalith-native")
CIV = ROOT / "crates/cartalith-civ/src/town_freeze.rs"
BR = ROOT / "crates/cartalith-godot/src/town_layout_bridge.rs"
RULES = ROOT / "crates/cartalith-urban/src/rules.rs"
CT = ("cartalith-civ", "town_freeze")
GT = ("cartalith-godot", "town_layout")

M = [
    ("rules_version", CIV, "pub const URBAN_RULES_VERSION: u32 = 1;", "pub const URBAN_RULES_VERSION: u32 = 2;", CT),
    ("a_rules_constant", RULES, "        segment_length_median: 56.0,", "        segment_length_median: 57.0,", CT),
    ("quantum", CIV, "pub const Q_PER_M: f64 = 10.0;", "pub const Q_PER_M: f64 = 20.0;", CT),
    ("tone_quantum", CIV, "pub const Q_TONE: f64 = 1_000.0;", "pub const Q_TONE: f64 = 100.0;", CT),
    ("orient_bits_dropped", CIV, 'g.insert("orient".into(), bits(*orient));', 'g.insert("orient".into(), bits(0.0));', CT),
    ("switch_record_wrong", CIV, '("urban_parallel_streets", parallel_streets),', '("urban_parallel_streets", wall_hug),', CT),
    ("absent_switch_reads_on", CIV, ".filter(|k| recorded.get(*k).copied().unwrap_or(false)", ".filter(|k| recorded.get(*k).copied().unwrap_or(true)", CT),
    ("site_hash_ignores_terrain", CIV, "            for v in &t.grid {\n                s.h(*v);", "            for v in &t.grid {\n                let _ = v;", CT),
    ("site_hash_ignores_mask", CIV, "        s.0.extend_from_slice(&w.mask);", "        let _ = &w.mask;", CT),
    ("site_hash_ignores_route_ends", CIV, "    match &ctx.route_ends {\n        Some(r) => {\n            s.tag(1);\n            s.pts(r);", "    match &ctx.route_ends {\n        Some(r) => {\n            s.tag(1);\n            let _ = r;", CT),
    ("site_hash_ignores_paths", CIV, "            for p in pp {\n                s.pts(p);", "            for p in pp {\n                let _ = p;", CT),
    ("rounding_too_loose", CIV, "if (p - q).abs() > 2 || (sa[i % 2] - sb[i % 2]).abs() > 2 {", "if (p - q).abs() > 20 || (sa[i % 2] - sb[i % 2]).abs() > 20 {", CT),
    ("gate_bearing_wide", CIV, "pub const GATE_BEARING_DEG: f64 = 15.0;", "pub const GATE_BEARING_DEG: f64 = 45.0;", CT),
    ("predictor_off_by_one", CIV, "                (ip[k - 1].0 + ip[k - 3].0 - ip[k - 2].0, ip[k - 1].1 + ip[k - 3].1 - ip[k - 2].1)\n            } else {\n                last\n            };\n            d.push", "                (ip[k - 1].0 + ip[k - 3].0 - ip[k - 2].0 + 1, ip[k - 1].1 + ip[k - 3].1 - ip[k - 2].1)\n            } else {\n                last\n            };\n            d.push", CT),
    ("mismatch_silently_accepted", BR, "        if tf::layout_hash(geom) != stored {", "        if false && tf::layout_hash(geom) != stored {", GT),
    ("unknown_enc_accepted", BR, "        if enc != tf::ENC {", "        if false && enc != tf::ENC {", GT),
    ("older_save_writes_a_member", BR, "    if t.rows.is_empty() && t.orphans.is_empty() {\n        return None;", "    if t.rows.is_empty() && t.orphans.is_empty() {\n        return Some(String::new());", GT),
    ("newer_version_read", BR, "    if v > DOC_VERSION {", "    if v > DOC_VERSION + 5 {", GT),
    ("pin_dropped_on_relay", BR, "                // Stays exactly where it is, re-anchored to whatever it now stands on (scope §4.4 step 5).\n                out.push((m, None, na, true, here));", "                // Stays exactly where it is, re-anchored to whatever it now stands on (scope §4.4 step 5).\n                let _ = (m, na, here);", GT),
    ("dead_anchor_kept", BR, "Some((poi, s)) if live_pois.contains(&poi) && anchor_in_range(&layout, &s.a) => {", "Some((poi, s)) if anchor_in_range(&layout, &s.a) => {", GT),
    ("chip_threshold", BR, "pub(crate) const POP_CHIP_RATIO: f64 = 2.0;", "pub(crate) const POP_CHIP_RATIO: f64 = 1.8;", GT),
    ("cheap_tier_misses_world", BR, '    if recorded.get("site_hash") != now.get("site_hash") {', '    if false && recorded.get("site_hash") != now.get("site_hash") {', GT),
]

ENV = dict(os.environ, CARGO_TARGET_DIR="C:/cw/tl_target")


def run(crate_filter):
    crate, flt = crate_filter
    p = subprocess.run(["cargo", "test", "-p", crate, "--lib", flt], cwd=ROOT, env=ENV, capture_output=True, text=True, errors="replace")
    out = p.stdout + p.stderr
    if "error[" in out or "could not compile" in out:
        return "BUILD_ERROR", out
    return ("SURVIVED" if p.returncode == 0 else "KILLED"), out


def main(names):
    res = []
    for name, f, old, new, tgt in M:
        if names and name not in names:
            continue
        src = f.read_text(encoding="utf-8")
        assert src.count(old) == 1, (name, src.count(old))
        try:
            f.write_text(src.replace(old, new), encoding="utf-8")
            v, out = run(tgt)
        finally:
            f.write_text(src, encoding="utf-8")
        failed = [l for l in out.splitlines() if l.startswith("test ") and l.endswith("FAILED")]
        print(f"{v:12s} {name}: {'; '.join(failed)[:200]}", flush=True)
        res.append((name, v))
    print("summary:", {v: sum(1 for _, x in res if x == v) for v in ("KILLED", "SURVIVED", "BUILD_ERROR")})


if __name__ == "__main__":
    main(sys.argv[1:])
