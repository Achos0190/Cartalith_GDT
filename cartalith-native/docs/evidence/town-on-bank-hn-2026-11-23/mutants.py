"""Ruling HN: named literal mutants. Each replaces one exact literal (it must occur exactly once), runs the tests
named for it, and must turn them red. The file is restored in a `finally`. A mutant that does not build is reported
as BUILD_ERROR, never as killed.

  python -I mutants.py [name-prefix]        (run from anywhere; set CARGO_TARGET_DIR first)

Verdicts go to stdout and to mutants_result.txt beside this file."""
import pathlib, subprocess, sys, hashlib

ROOT = pathlib.Path("C:/cw/hn/cartalith-native")
U, C, G = "crates/cartalith-urban/src/", "crates/cartalith-civ/src/", "crates/cartalith-godot/src/"
# (name, file, old, new, crate, test filter)
M = [
    ("U1 rampart sweep keeps the largest piece only", U + "cleanup.rs", "prune_components(g, market, market.is_some());", "prune_components(g, market, false);", "cartalith-urban", "the_rampart_sweep"),
    ("U2 rampart sweep ignores the market", U + "cleanup.rs", "prune_components(g, market, market.is_some());", "prune_components(g, None, market.is_some());", "cartalith-urban", "the_rampart_sweep"),
    ("U3 wet sweep: the quay is not exempt", U + "cleanup.rs", 'let live: Vec<usize> = g.edges.iter().filter(|e| e.alive && e.cls != "quay").map(|e| e.id).collect();', 'let live: Vec<usize> = g.edges.iter().filter(|e| e.alive && e.cls != "quayX").map(|e| e.id).collect();', "cartalith-urban", "only_a_bridge"),
    ("U4 wet sweep: a street's ends are not sampled", U + "cleanup.rs", "(0..=n).any(|i| f(a.lerp(b, i as f64 / n as f64)))", "(1..n).any(|i| f(a.lerp(b, i as f64 / n as f64)))", "cartalith-urban", "only_a_bridge"),
    ("U5 wet sweep: every wet street seeds a bridge", U + "cleanup.rs", "site.river.windows(2).any(|w| crate::geom::seg_int(a, b, w[0], w[1]).is_some())\n        })\n        .collect();\n    stack.sort_unstable();", "true || site.river.windows(2).any(|w| crate::geom::seg_int(a, b, w[0], w[1]).is_some())\n        })\n        .collect();\n    stack.sort_unstable();", "cartalith-urban", "only_a_bridge"),
    ("U6 wet sweep: a street off the box stays", U + "cleanup.rs", "any_sample(g, id, &|p| site.off_box(p))).collect();", "any_sample(g, id, &|_| false)).collect();", "cartalith-urban", "only_a_bridge"),
    ("U7 past a river's end: the last segment is not read", U + "site.rs", "|| (bi == n - 2 && (p - r[n - 1]).dot(r[n - 2] - r[n - 1]) < 0.0)", "|| (false && (p - r[n - 1]).dot(r[n - 2] - r[n - 1]) < 0.0)", "cartalith-urban", "ground_past_a_rivers_end"),
    ("U8 past a river's end keeps the line's own side", U + "site.rs", "Some(m) if self.past_river_end(p) => self.bank_side_of_line(m),", "Some(_) if self.past_river_end(p) => self.bank_side_of_line(p),", "cartalith-urban", "ground_past_a_rivers_end"),
    ("U9 WATER_GATE_KEEP_M 15 to 1500", U + "wallhug.rs", "pub const WATER_GATE_KEEP_M: f64 = 15.0;", "pub const WATER_GATE_KEEP_M: f64 = 1500.0;", "cartalith-urban", "a_walls_vertices_in_the_water"),
    ("U10 a gated wall run is the one taken out", U + "wallhug.rs", "        if !water_gates.iter().any(|&g| run.windows(2).any(|s| dist_pt_seg(g, s[0], s[1]) <= WATER_GATE_KEEP_M)) {", "        if water_gates.iter().any(|&g| run.windows(2).any(|s| dist_pt_seg(g, s[0], s[1]) <= WATER_GATE_KEEP_M)) {", "cartalith-urban", "a_walls_vertices_in_the_water"),
    ("U13 a wall run that would make the ring cross itself is taken out anyway", U + "wallhug.rs", "        if out.len() >= 3 && !ring_self_crosses(&out) {\n            any = true;", "        if out.len() >= 3 {\n            any = true;", "cartalith-urban", "a_walls_vertices_in_the_water"),
    ("U11 BANK_SITES_LOW_SHARE 0.97 to 0.5", U + "riverbank.rs", "pub const BANK_SITES_LOW_SHARE: f64 = 0.97;", "pub const BANK_SITES_LOW_SHARE: f64 = 0.5;", "cartalith-urban", "a_village_on_the_maps_wide_water"),
    ("U12 market_only answers the box corner", U + "generate.rs", "    (s.anchors.market, s.road_bank_moved)\n}", "    (Vec2::new(0.0, 0.0), s.road_bank_moved)\n}", "cartalith-urban", "market_only_is_the_laid_markets"),
    ("U14 the wet street sweep is never called", U + "generate.rs", "    if opts.bank_sites {\n        crate::cleanup::sweep_wet_streets(&site, &mut g, anchors.market);", "    if false {\n        crate::cleanup::sweep_wet_streets(&site, &mut g, anchors.market);", "cartalith-urban", "the_bank_site_rules_reach_the_layout"),
    ("U15 a wet wall run is not tried vertex by vertex", U + "wallhug.rs", "            for d in 0..len {\n                gone[(i + d) % n] = true;", "            for d in 0..0 {\n                gone[(i + d) % n] = true;", "cartalith-urban", "a_walls_vertices_in_the_water"),
    ("U16 ground past a river's end is never given the market's bank", U + "generate.rs", "    if opts.bank_sites {\n        site.bank_ref = Some(anchors.market);", "    if false {\n        site.bank_ref = Some(anchors.market);", "cartalith-urban", "the_bank_site_rules_reach_the_layout"),
    ("U17 the wall is never taken off the water", U + "generate.rs", "    if opts.bank_sites {\n        crate::wallhug::wall_off_water(&mut wall_state, &site);", "    if false {\n        crate::wallhug::wall_off_water(&mut wall_state, &site);", "cartalith-urban", "the_bank_site_rules_reach_the_layout"),
    ("U18 the rampart sweep is never given the market", U + "generate.rs", "opts.bank_sites.then_some(anchors.market));", "None);", "cartalith-urban", "the_bank_site_rules_reach_the_layout"),
    ("U19 a house off the box is never swept", U + "generate.rs", "|| (opts.bank_sites && buildings[i].poly.iter().any(|&v| site.off_box(v)))", "|| (false && buildings[i].poly.iter().any(|&v| site.off_box(v)))", "cartalith-urban", "the_bank_site_rules_reach_the_layout"),
    ("C1 MAP_WATER_TAPS: the centre alone", C + "urban_adapter.rs", "pub const MAP_WATER_TAPS: [(f64, f64); 5] = [(0.5, 0.5), (0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)];", "pub const MAP_WATER_TAPS: [(f64, f64); 5] = [(0.5, 0.5), (0.5, 0.5), (0.5, 0.5), (0.5, 0.5), (0.5, 0.5)];", "cartalith-civ", "the_towns_lake_and_sea_cells"),
    ("C2 the map's water test is not asked", C + "urban_adapter.rs", "                    drawn(x, y)\n                }),", "                    drawn(x, y) && false\n                }),", "cartalith-civ", "the_towns_lake_and_sea_cells"),
    ("C3 a junction town takes the nearer run", C + "urban_adapter.rs", ".min_by(|a, b| mean_width(b.1).total_cmp(&mean_width(a.1)).then(a.2.1.total_cmp(&b.2.1)))", ".min_by(|a, b| mean_width(b.1).total_cmp(&mean_width(b.1)).then(a.2.1.total_cmp(&b.2.1)))", "cartalith-civ", "at_a_junction"),
    ("C4 DRAWN_WIDTH_MARGIN_M 8 to 80", C + "urban_adapter.rs", "pub const DRAWN_WIDTH_MARGIN_M: f64 = 8.0;", "pub const DRAWN_WIDTH_MARGIN_M: f64 = 80.0;", "cartalith-civ", "at_a_junction"),
    ("C5 BANK_CENTRE_CLEAR_M 20 to 3", C + "urban_adapter.rs", "pub const BANK_CENTRE_CLEAR_M: f64 = 20.0;", "pub const BANK_CENTRE_CLEAR_M: f64 = 3.0;", "cartalith-civ", "at_a_junction"),
    ("C6 the market offsets answer the box centre", C + "urban_adapter.rs", "                let (m, moved) = cartalith_urban::generate::market_only(c.seed, o, front);\n                push(m, wm, hm);", "                let (_, moved) = cartalith_urban::generate::market_only(c.seed, o, front);\n                push(Vec2::new(wm / 2.0, hm / 2.0), wm, hm);", "cartalith-civ", "the_market_is_known"),
    ("C7 URBAN_RULES_VERSION 3 to 4", C + "town_freeze.rs", "pub const URBAN_RULES_VERSION: u32 = 3;", "pub const URBAN_RULES_VERSION: u32 = 4;", "cartalith-civ", "the_rules_version"),
    ("C8 the bank-site layout rules are never turned on", C + "urban_adapter.rs", "        bank_sites: ctx.origin_m.is_some(),", "        bank_sites: false,", "cartalith-civ", "the_bank_site_layout_rules"),
    ("G1 the sea test inverted", G + "town_water.rs", "if bil(&|i| f64::from(field[i])) < self.sea_level {", "if bil(&|i| f64::from(field[i])) > self.sea_level {", "cartalith-godot", "the_point_test_is"),
    ("G2 the lake shore's sign", G + "town_water.rs", "_ if bil(&|i| f64::from(self.shore[i])) > 0.0 => LAKE,", "_ if bil(&|i| f64::from(self.shore[i])) < 0.0 => LAKE,", "cartalith-godot", "the_point_test_is"),
    ("G3 four lake corners read land", G + "town_water.rs", "            4 => LAKE,", "            4 => LAND,", "cartalith-godot", "the_point_test_is"),
    ("G4 the marker never goes to its dry market", G + "bank_site.rs", "        .or_else(|| (q.dry)(m.0, m.1).then_some(m))", "        .or_else(|| None::<(f64, f64)>)", "cartalith-godot", "the_marker_comes_to_its_markets_bank"),
    ("G5 the clear market itself is not taken", G + "bank_site.rs", "    if clear(m) {\n        return Some(m);\n    }", "    if false && clear(m) {\n        return Some(m);\n    }", "cartalith-godot", "the_marker_comes_to_its_markets_bank"),
    ("G6 a road's landing past the reach is followed", G + "bank_site.rs", "        .filter(|&p| off(p) <= reach)", "        .filter(|&p| off(p) <= reach * 1000.0)", "cartalith-godot", "a_town_stands_where_its_road"),
    ("G7 the road's landing is never taken", G + "bank_site.rs", "        .min_by(|&x, &y| off(x).total_cmp(&off(y)));\n    // Else the bank", "        .min_by(|&x, &y| off(x).total_cmp(&off(y)))\n        .filter(|_| false);\n    // Else the bank", "cartalith-godot", "a_town_stands_where_its_road"),
    ("G8 a town is found round its cell plus its origin without the half cell", G + "town_layout_bridge.rs", "let c = origin.map_or((cell.0 as f64 + 0.5, cell.1 as f64 + 0.5), |b| (b.at.0 + 0.5, b.at.1 + 0.5));", "let c = origin.map_or((cell.0 as f64 + 0.5, cell.1 as f64 + 0.5), |b| (b.at.0, b.at.1));", "cartalith-godot", "a_town_is_found_round"),
    ("G9 census: GATE_NEAR_M 15 to 1500", G + "town_measures.rs", "pub(crate) const GATE_NEAR_M: f64 = 15.0;", "pub(crate) const GATE_NEAR_M: f64 = 1500.0;", "cartalith-godot", "the_footprint_measure"),
    ("G10 census: BRIDGE_ON_EDGE_M 1.5 to 1500", G + "town_measures.rs", "pub(crate) const BRIDGE_ON_EDGE_M: f64 = 1.5;", "pub(crate) const BRIDGE_ON_EDGE_M: f64 = 1500.0;", "cartalith-godot", "the_footprint_measure"),
    ("G11 census: OVER_STEP_M 4 to 400", G + "town_measures.rs", "pub(crate) const OVER_STEP_M: f64 = 4.0;", "pub(crate) const OVER_STEP_M: f64 = 400.0;", "cartalith-godot", "the_footprint_measure"),
    ("G12 an older archive is not read as saved without the switch", G + "params.rs", "state.get(NATIVE_PARAMS_KEY).and_then(|v| v.as_object()).is_none_or(|o| !o.contains_key(key))", "state.get(NATIVE_PARAMS_KEY).and_then(|v| v.as_object()).is_some_and(|o| !o.contains_key(key))", "cartalith-godot", "an_older_archive_reads_as_saved_without"),
]

def run(crate, filt):
    p = subprocess.run(["cargo", "test", "-p", crate, "--lib", "--", filt], cwd=ROOT, capture_output=True, text=True)
    out = p.stdout + p.stderr
    if "error[" in out or "could not compile" in out: return "BUILD_ERROR"
    if "test result: FAILED" in out: return "KILLED"
    if "test result: ok" in out and " 0 passed" not in out: return "SURVIVED"
    return "NO_TEST_RAN"

only = sys.argv[1] if len(sys.argv) > 1 else ""
res = []
for name, f, old, new, crate, filt in M:
    if not name.startswith(only): continue
    path = ROOT / f
    raw = path.read_bytes(); h = hashlib.sha256(raw).hexdigest()
    text = raw.decode("utf-8"); crlf = "\r\n" in text
    t = text.replace("\r\n", "\n")
    n = t.count(old)
    if n != 1:
        res.append((name, "LITERAL_COUNT_%d" % n)); print(res[-1], flush=True); continue
    try:
        m = t.replace(old, new)
        path.write_bytes((m.replace("\n", "\r\n") if crlf else m).encode("utf-8"))
        v = run(crate, filt)
    finally:
        path.write_bytes(raw)
    assert hashlib.sha256(path.read_bytes()).hexdigest() == h, "not restored: " + f
    res.append((name, v)); print(res[-1], flush=True)
killed = sum(1 for _, v in res if v == "KILLED")
lines = ["%-70s %s" % r for r in res] + ["", "%d mutants, %d killed" % (len(res), killed)]
if not only:
    (pathlib.Path(__file__).parent / "mutants_result.txt").write_text("\n".join(lines) + "\n")
print("\n".join(lines[-2:]))
