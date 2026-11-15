"""Mutation harness (MISTAKES.md: exact literal replace, occurs-once, BUILD_ERROR separate, restore in finally)."""
import subprocess, sys, os, json
ROOT = 'C:/cw/tautwall/cartalith-native'
U = ROOT + '/crates/cartalith-urban/src/'
M = [
 # (file, old, new, label)
 (U+'wallhug.rs', 'pub const TAUT_POCKET_MAX_M2: f64 = 8_000.0;', 'pub const TAUT_POCKET_MAX_M2: f64 = 11_000.0;', 'pocket cap up'),
 (U+'wallhug.rs', 'pub const TAUT_POCKET_MAX_M2: f64 = 8_000.0;', 'pub const TAUT_POCKET_MAX_M2: f64 = 6_000.0;', 'pocket cap down'),
 (U+'wallhug.rs', 'pub const TAUT_MIN_RUN_M: f64 = 40.0;', 'pub const TAUT_MIN_RUN_M: f64 = 30.0;', 'min run down'),
 (U+'wallhug.rs', 'pub const TAUT_MAX_RUN_M: f64 = 90.0;', 'pub const TAUT_MAX_RUN_M: f64 = 110.0;', 'max run up'),
 (U+'wallhug.rs', 'pub const TAUT_MAX_TURN: f64 = 75.0 * PI / 180.0;', 'pub const TAUT_MAX_TURN: f64 = 130.0 * PI / 180.0;', 'max turn up'),
 (U+'wallhug.rs', 'pub const TAUT_SIMPLIFY_M: f64 = 1.0;', 'pub const TAUT_SIMPLIFY_M: f64 = 2.0;', 'simplify tol up'),
 (U+'wallhug.rs', 'pub const TAUT_SIMPLIFY_M: f64 = 1.0;', 'pub const TAUT_SIMPLIFY_M: f64 = 0.3;', 'simplify tol down'),
 (U+'wallhug.rs', 'pub const OPEN_PATCH_MAX_M2: f64 = 10_000.0;', 'pub const OPEN_PATCH_MAX_M2: f64 = 100_000.0;', 'open patch cap up'),
 (U+'wallhug.rs', 'pub const OPEN_REACH_M: f64 = 40.0;', 'pub const OPEN_REACH_M: f64 = 30.0;', 'open reach down'),
 (U+'wallhug.rs', 'pub const TAUT_POCKET_STEPS: [f64; 3] = [1.0, 0.25, 0.0];', 'pub const TAUT_POCKET_STEPS: [f64; 3] = [1.0, 1.0, 1.0];', 'pocket steps flat'),
 (U+'wallhug.rs', '            if turn(u, c, w) <= 0.0 {', '            if turn(u, c, w) <= -1e9 {', 'no reflex fill'),
 (U+'wallhug.rs', '                    if t >= 0.0 && s >= 0.0 && ang <= TAUT_MAX_TURN {', '                    if ang <= TAUT_MAX_TURN {', 'merge without forward test'),
 (U+'wallhug.rs', '                    if blocked(&v, &[(u, w)], &[(ip + n - 1) % n, ip, i, inx], [u, v[i], w], &[ip, i, inx]) {', '                    if false {', 'fill never blocked'),
 (U+'wallhug.rs', '            let is_out = side < -1e-6;', '            let is_out = false;', 'simplify_out two-sided'),
 (U+'wallhug.rs', '            if strict && !open_ok {', '            if false && strict && !open_ok {', 'no strict pass'),
 (U+'wallhug.rs', '            let hull = tauten(&hull, TAUT_POCKET_MAX_M2);', '            let hull = hull;', 'PLACEHOLDER'),
 (U+'fortify.rs', '        let mut r = if bmh.taut { taut_loop(&base) } else { corner_cut(&chaikin(&chaikin(&base, true), true), 1.75, 4) };', '        let mut r = corner_cut(&chaikin(&chaikin(&base, true), true), 1.75, 4);', 'spans branch chaikin'),
 (U+'fortify.rs', '            let mut r = if bmh.taut { taut_loop(&hull) } else { corner_cut(&chaikin(&chaikin(&hull, true), true), 1.75, 4) };\n            if !bmh.taut {\n                r = simplify(&r, 2.0);\n            }\n            land_arc = r.clone();', '            let mut r = corner_cut(&chaikin(&chaikin(&hull, true), true), 1.75, 4);\n            r = simplify(&r, 2.0);\n            land_arc = r.clone();', 'all-land branch chaikin'),
 (U+'fortify.rs', '            if bmh.taut {\n                let whole', '            if false {\n                let whole', 'one-bank no taut simplify'),
 (U+'village.rs', 'pub const STREET_PER_HEAD_M: f64 = 0.25;', 'pub const STREET_PER_HEAD_M: f64 = 0.5;', 'street per head up'),
 (U+'village.rs', 'pub const ROAD_REACH_M: f64 = 40.0;', 'pub const ROAD_REACH_M: f64 = 80.0;', 'road reach up'),
 (U+'village.rs', 'pub const TOFT_DEPTH_M: f64 = 30.0;', 'pub const TOFT_DEPTH_M: f64 = 25.0;', 'toft depth'),
 (U+'village.rs', 'pub const TOFT_FRONTAGE: (f64, f64) = (11.0, 0.22);', 'pub const TOFT_FRONTAGE: (f64, f64) = (8.0, 0.22);', 'toft frontage'),
 (U+'village.rs', 'const TOFT_MIN_DEPTH_M: f64 = 6.0;', 'const TOFT_MIN_DEPTH_M: f64 = 13.0;', 'toft min depth'),
 (U+'village.rs', 'const VERGE_M: f64 = 1.4;', 'const VERGE_M: f64 = 2.4;', 'verge'),
 (U+'village.rs', 'pub const TOFT_REACH_RF: f64 = 3.0;', 'pub const TOFT_REACH_RF: f64 = 4.0;', 'toft reach'),
 (U+'village.rs', '    if have >= need {\n        return Vec::new();\n    }', '', 'no early return'),
 (U+'village.rs', '        if have + out.len() >= need {\n            break;\n        }', '', 'no stop at need'),
 (U+'village.rs', '        if !accepts(g, site, &greens, &taken, &quad) {\n            continue;\n        }', '', 'no accepts'),
 (U+'village.rs', '        if len < 6.0 || crate::geom::dist_pt_seg(market, a, b) > reach {', '        if len < 6.0 {', 'no reach filter'),
 (U+'plaza.rs', 'pub const GREEN_RIVER_CLEAR_M: f64 = 4.0;', 'pub const GREEN_RIVER_CLEAR_M: f64 = 15.0;', 'green clear up'),
 (U+'plaza.rs', '            .or_else(|| square_near(site, anchors, g, l, wd, GREEN_RIVER_CLEAR_M, false, true))', '', 'no other-bank green'),
 (U+'plaza.rs', '    if green_at_market\n        && pop < PLAZA_MARKET_POP', '    if false && green_at_market\n        && pop < PLAZA_MARKET_POP', 'green not moved'),
 (U+'plaza.rs', "(!has_line || !same_bank || site.bank_side(p) == mkt_side);", "(!has_line || site.bank_side(p) == mkt_side);", 'same_bank ignored'),
 (U+'growth.rs', '        let budget = if opts.road_reach.is_some() { target_len - placed_len } else { target_len / epochs as f64 };', '        let budget = target_len / epochs as f64;', 'budget per epoch'),
 (U+'growth.rs', '            if let Some(reach) = opts.road_reach\n                && !prim_edges', '            if let Some(reach) = opts.road_reach.filter(|_| false)\n                && !prim_edges', 'no road reach test'),
 (U+'cleanup.rs', '    if let Some(p) = keep\n', '    if let Some(p) = keep.filter(|_| false)\n', 'prune ignores keep'),
 (U+'generate.rs', '    if profile.planning != "radial" && !street_village {', '    if profile.planning != "radial" {', 'courtyards for villages'),
 (U+'generate.rs', '    let street_village = opts.street_village && pop_target < crate::plaza::PLAZA_MARKET_POP && profile.planning != "radial";', '    let street_village = opts.street_village && profile.planning != "radial";', 'sv for towns'),
 (U+'plaza.rs', '    let in_channel = |p: Vec2| has_line && site.river_dist(p) < site.river_w / 2.0 + 1.0;', '    let in_channel = |_p: Vec2| false;', 'green channel guard off'),
 (U+'plaza.rs', 'pub const ROAD_ON_M: f64 = 3.0;', 'pub const ROAD_ON_M: f64 = 30.0;', 'road on up'),
 (U+'plaza.rs', '        if len < l && !(village && len >= SQUARE_SLIDE_M) {', '        if len < l {', 'no short edges'),
 (U+'village.rs', '    if rk {\n        let n = quad.len();', '    if false {\n        let n = quad.len();', 'toft channel guard off'),
 (U+'riverbank.rs', 'pub const VILLAGE_MAX_SCALE: f64 = 8.0;', 'pub const VILLAGE_MAX_SCALE: f64 = 6.0;', 'max scale down'),
 (U+'riverbank.rs', '    if t.street_village && t.pop < t.pop_target * VILLAGE_LOW_SHARE {', '    if false && t.street_village && t.pop < t.pop_target * VILLAGE_LOW_SHARE {', 'no short refit'),
 (U+'riverbank.rs', '        if t.pop <= hi && t.pop > best.pop {', '        if t.pop <= hi && t.pop > best.pop + 1e9 {', 'short refit keeps first'),
]
M = [m for m in M if m[3] != 'PLACEHOLDER']
only = sys.argv[1:] 
res = {}
for f, old, new, lab in M:
    if only and lab not in only: continue
    src = open(f, encoding='utf-8').read()
    n = src.count(old)
    if n != 1:
        res[lab] = f'NOT-UNIQUE({n})'; print(lab, res[lab], flush=True); continue
    try:
        open(f, 'w', encoding='utf-8').write(src.replace(old, new))
        env = dict(os.environ, CARGO_TARGET_DIR='C:/cw/tautwall_target')
        b = subprocess.run(['cargo', 'test', '-p', 'cartalith-urban', '--no-run'], cwd=ROOT, env=env, capture_output=True, text=True)
        if b.returncode != 0:
            res[lab] = 'BUILD_ERROR'
        else:
            r = subprocess.run(['cargo', 'test', '-p', 'cartalith-urban', '--', '--test-threads', '8'], cwd=ROOT, env=env, capture_output=True, text=True)
            fails = [l for l in r.stdout.splitlines() if l.startswith('test ') and l.endswith('FAILED')]
            res[lab] = ('KILLED by ' + ', '.join(x.split()[1] for x in fails[:3])) if r.returncode != 0 else 'SURVIVED'
    finally:
        open(f, 'w', encoding='utf-8').write(src)
    print(lab, '->', res[lab], flush=True)
json.dump(res, open('C:/cw/tautwall_mut/results.json', 'w'), indent=1)
