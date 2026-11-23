"""Ruling HM literal mutants. Exact literal replace, occurs exactly once, BUILD_ERROR apart from SURVIVED, restore in finally.
Run: python -I mutants.py [name prefix ...] from anywhere; ROOT and the target dir are this lane's."""
import subprocess, sys, pathlib, hashlib, os
ROOT = pathlib.Path("C:/cw/hm/cartalith-native")
BANK = ROOT / "crates/cartalith-godot/src/bank_site.rs"
ADAPT = ROOT / "crates/cartalith-civ/src/urban_adapter.rs"
FREEZE = ROOT / "crates/cartalith-civ/src/town_freeze.rs"
LABELS = ROOT / "crates/cartalith-civ/src/labels.rs"
ENV = dict(os.environ, CARGO_TARGET_DIR="C:/cw/hm_target")
LIB = ["cargo", "test", "-p", "cartalith-godot", "--lib", "bank_site"]
CIV = ["cargo", "test", "-p", "cartalith-civ", "--lib", "--", "urban_adapter", "town_freeze", "labels"]
MUTANTS = [
    ("C1 the box grows by once the distance, not twice", ADAPT, "let grow = 2.0 * js_max(0.0, origin?.river_m);", "let grow = 1.0 * js_max(0.0, origin?.river_m);", CIV),
    ("C2 ROAD_BANK_CLEAR_CELLS 2.0 -> 0.0", ADAPT, "pub const ROAD_BANK_CLEAR_CELLS: f64 = 2.0;", "pub const ROAD_BANK_CLEAR_CELLS: f64 = 0.0;", CIV),
    ("C3 the pin moved the wrong way", ADAPT, "Vec2::new(base.x - (m.x * cs - m.y * sn), base.y - (m.x * sn + m.y * cs))", "Vec2::new(base.x + (m.x * cs - m.y * sn), base.y + (m.x * sn + m.y * cs))", CIV),
    ("C4 the drawn width not taken", ADAPT, "river_width_m = js_max(river_width_m, f64::from(wide) / gpm);", "river_width_m = js_max(river_width_m, 0.0);", CIV),
    ("C5 other drawn rivers not stamped", ADAPT, "        stamp(&mut mask, &mut water_cells, path, *width_m);", "        let _ = (path, width_m);", CIV),
    ("C6 a far-bank road point is left", ADAPT, "Some(k) if !bank[k] => {", "Some(k) if water.mask[k] != 0 => {", CIV),
    ("C7 the water read round the cell, not the origin", ADAPT, "let (bx, by) = o.origin.map_or((px, py), |b| b.at);", "let (bx, by) = o.origin.map_or((px, py), |_| (px, py));", CIV),
    ("C8 URBAN_RULES_VERSION 2 -> 1", FREEZE, "pub const URBAN_RULES_VERSION: u32 = 2;", "pub const URBAN_RULES_VERSION: u32 = 1;", CIV),
    ("C9 the origin left out of the switch record", FREEZE, '("civ_bank_origin", origin.is_some()),', '("civ_bank_origin", false),', CIV),
    ("C10 a label anchored at the cell whatever it is given", LABELS, "world.settlement_at.get(i).copied().unwrap_or(", "None.unwrap_or(", CIV),
    ("C11 every drawn run counts as carrying a width", ADAPT, "o.river_smooth.is_some_and(|r| r.iter().any(|d| !d.widths.is_empty()))", "o.river_smooth.is_some_and(|r| r.iter().any(|d| d.widths.is_empty()))", CIV),
    ("C12 the pin taken from the generator's, not the box centre", ADAPT, "let base = if ctx.water.is_some() { Vec2::new(t.wm / 2.0, t.hm / 2.0) } else { t.pin };", "let base = t.pin;", CIV),
    ("C13 TOWN_CHANNEL_MAX_M 46 -> 4600 (an unmoved town's box never grows)", ADAPT, "pub const TOWN_CHANNEL_MAX_M: f64 = 46.0;", "pub const TOWN_CHANNEL_MAX_M: f64 = 4600.0;", CIV),
    ("C14 an unmoved town's box grows by half the river's width", ADAPT, "Some(rw) => (Some((SITE_WM + rw, SITE_HM + rw)), (SITE_WM + rw, SITE_HM + rw)),", "Some(rw) => (Some((SITE_WM + 0.5 * rw, SITE_HM + 0.5 * rw)), (SITE_WM + 0.5 * rw, SITE_HM + 0.5 * rw)),", CIV),
    ("G1 ORIGIN_MIN_CLEAR_M 60 -> 6", BANK, "pub const ORIGIN_MIN_CLEAR_M: f64 = 60.0;", "pub const ORIGIN_MIN_CLEAR_M: f64 = 6.0;", LIB),
    ("G2 ORIGIN_BANK_GAP_M 30 -> 0", BANK, "pub const ORIGIN_BANK_GAP_M: f64 = 30.0;", "pub const ORIGIN_BANK_GAP_M: f64 = 0.0;", LIB),
    ("G3 the ground minimum ignored", BANK, "Some(min) => Some((f64::from(point.width_cells) / 2.0).max(min)),", "Some(_) => Some(f64::from(point.width_cells) / 2.0),", LIB),
    ("G4 the market's bank ignored", BANK, "(Some(n), _) => n,", "(Some(_), _) => hit.n,", LIB),
    ("G5 a clear centre across from its market stays", BANK, "let m = market?;", "let m = market.filter(|_| false)?;", LIB),
    ("G6 the same-bank rule dropped", BANK, "&& !(q.same_bank && market.is_some_and(", "&& !(false && market.is_some_and(", LIB),
    ("G7 the roads' bank reversed", BANK, ".map(|n| (n.0 * hit.n.0 + n.1 * hit.n.1).signum())", ".map(|n| -(n.0 * hit.n.0 + n.1 * hit.n.1).signum())", LIB),
    ("G8 a hidden river still has banks", BANK, " && drawn(segment_hit(geom, ri, i, a).point)", "", LIB),
    ("G9 the reach widths not read for the town", BANK, "ground: Some(min), reach_widths: Some(index.reach_widths(geom, k)), same_bank: false };", "ground: Some(min), reach_widths: None, same_bank: false };", LIB),
    ("G10 the look reaches the ground width", BANK, "    f64::from(geom.runs.iter().flat_map(|r| r.widths.iter().copied()).fold(0.0f32, f32::max)) / 2.0", "    f64::from(geom.runs.iter().flat_map(|r| r.widths.iter().copied()).fold(0.0f32, f32::max)) / 4.0", LIB),
    ("G11 the reach by half its length", BANK, "let lo = at.partition_point(|&d| d < at[i] - reach);", "let lo = at.partition_point(|&d| d < at[i] - reach * 0.1);", LIB),
    ("G12 the last fallback (any clear bank) removed", BANK, "two(q).or_else(|| if q.same_bank && market.is_some() { two(&BankQuery { same_bank: false, ..*q }) } else { None })", "two(q)", LIB),
]
only = sys.argv[1:]
for name, path, old, new, cmd in MUTANTS:
    if only and not any(name.startswith(o + " ") for o in only):
        continue
    src = path.read_bytes()
    text = src.decode("utf-8")
    assert text.count(old) == 1, (name, text.count(old))
    try:
        path.write_bytes(text.replace(old, new).encode("utf-8"))
        r = subprocess.run(cmd, cwd=ROOT, env=ENV, capture_output=True, text=True, encoding="utf-8", errors="replace")
        out = r.stdout + r.stderr
        if "error[" in out or "error: could not compile" in out:
            verdict = "BUILD_ERROR"
        elif r.returncode != 0 and "test result: FAILED" in out:
            failed = [l.split()[1] for l in out.splitlines() if l.startswith("test ") and l.rstrip().endswith("FAILED")]
            verdict = "KILLED by " + ", ".join(sorted(set(failed)))
        elif "test result: ok" in out and " 0 passed" not in out.split("test result: ok")[1][:40]:
            verdict = "SURVIVED"
        else:
            verdict = "UNCLEAR rc=%d" % r.returncode
        print(name, "->", verdict, flush=True)
    finally:
        path.write_bytes(src)
        assert hashlib.sha256(path.read_bytes()).hexdigest() == hashlib.sha256(src).hexdigest()
print("done")
