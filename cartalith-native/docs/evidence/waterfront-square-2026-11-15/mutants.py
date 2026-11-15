"""Mutation run for TOWN_GENERATOR_JUDGEMENT.md changes 5 (waterfront) and 6 (market square).

Exact literal replace (the file's own line endings kept), occurs-exactly-once assert,
BUILD_ERROR separate from SURVIVED, every file restored in a finally and checked by hash.
usage: python -I mutants.py <out.txt>
"""
import hashlib
import os
import subprocess
import sys

ROOT = 'C:/cw/waterfront/cartalith-native'
U = ROOT + '/crates/cartalith-urban/src/'
NL = chr(10)
CRLF = chr(13) + chr(10)
M = [
    ('plaza.rs', 'pub const SQUARE_LAW_SHARE: f64 = 0.004;', 'pub const SQUARE_LAW_SHARE: f64 = 0.005;', 'square share'),
    ('plaza.rs', 'pub const SETTLED_LAW: (f64, f64) = (0.12, 0.714);', 'pub const SETTLED_LAW: (f64, f64) = (0.12, 0.7);', 'law exponent'),
    ('plaza.rs', 'pub const SETTLED_LAW: (f64, f64) = (0.12, 0.714);', 'pub const SETTLED_LAW: (f64, f64) = (0.13, 0.714);', 'law coefficient'),
    ('plaza.rs', 'const DRAW_MEAN_AREA_M2: f64 = 67.5 * 33.0;', 'const DRAW_MEAN_AREA_M2: f64 = 67.5 * 34.0;', 'draw mean'),
    ('plaza.rs', 'const SQUARE_SHRINK: f64 = 0.85;', 'const SQUARE_SHRINK: f64 = 0.95;', 'shrink step'),
    ('plaza.rs', '    while s > 1.0 {', '    while s > 1.2 {', 'tries floor'),
    ('plaza.rs', 'let scale = scale && pop >= PLAZA_MARKET_POP && square_scale(pop) > 1.0;', 'let scale = scale && square_scale(pop) > 1.0;', 'greens scaled'),
    ('plaza.rs', 'let scale = scale && pop >= PLAZA_MARKET_POP && square_scale(pop) > 1.0;', 'let scale = scale && pop >= PLAZA_MARKET_POP && square_scale(pop) >= 1.0;', 'floor >='),
    ('plaza.rs', 'let k = if scale { square_scale(pop) } else { 1.0 };', 'let k = 1.0;', 'scale ignored'),
    ('plaza.rs', '    if scale && !placed {', '    if false {', 'unplaced size kept'),
    ('plaza.rs', 'let scale = scale && pop >= PLAZA_MARKET_POP && square_scale(pop) > 1.0;', 'let scale = scale && pop >= PLAZA_MARKET_POP;', 'small squares shrunk'),
    ('plaza.rs', 'if scale && !placed && pop >= PLAZA_MARKET_POP && !site.riverbank() {', 'if false {', 'landlocked search off'),
    ('plaza.rs', 'if let Some((d, m)) = riverbank_square(site, anchors, g, l0 * t, wd0 * t) {', 'if let Some((d, m)) = riverbank_square(site, anchors, g, l0, wd0) {', 'riverbank square unscaled'),
    ('riverbank.rs', 'pub const BACK_STREET_M: (f64, f64) = (35.0, 45.0);', 'pub const BACK_STREET_M: (f64, f64) = (30.0, 45.0);', 'back depth low'),
    ('riverbank.rs', 'pub const BACK_STREET_M: (f64, f64) = (35.0, 45.0);', 'pub const BACK_STREET_M: (f64, f64) = (35.0, 50.0);', 'back depth high'),
    ('riverbank.rs', 'pub const BACK_EXISTING_M: (f64, f64) = (20.0, 70.0);', 'pub const BACK_EXISTING_M: (f64, f64) = (24.0, 70.0);', 'existing low'),
    ('riverbank.rs', 'pub const BACK_EXISTING_M: (f64, f64) = (20.0, 70.0);', 'pub const BACK_EXISTING_M: (f64, f64) = (20.0, 64.0);', 'existing high down'),
    ('riverbank.rs', 'pub const BACK_EXISTING_M: (f64, f64) = (20.0, 70.0);', 'pub const BACK_EXISTING_M: (f64, f64) = (20.0, 78.0);', 'existing high up'),
    ('riverbank.rs', 'pub const BACK_PARALLEL_DEG: f64 = 30.0;', 'pub const BACK_PARALLEL_DEG: f64 = 60.0;', 'parallel angle'),
    ('riverbank.rs', 'pub const BACK_RUN_MIN_M: f64 = 60.0;', 'pub const BACK_RUN_MIN_M: f64 = 0.0;', 'run min'),
    ('riverbank.rs', 'pub const WATER_LANE_GAP_M: (f64, f64) = (60.0, 120.0);', 'pub const WATER_LANE_GAP_M: (f64, f64) = (60.0, 300.0);', 'lane gap'),
    ('riverbank.rs', 'pub const BACK_BREAK_DROP: usize = 2;', 'pub const BACK_BREAK_DROP: usize = 1;', 'break drop'),
    ('riverbank.rs', 'let reached = town.iter().any(|&(a, e, own)| own && dist_pt_seg(b, a, e) <= EMBANKMENT_REACH);', 'let reached = town.iter().any(|&(a, e, _own)| dist_pt_seg(b, a, e) <= EMBANKMENT_REACH);', 'primary reaches'),
    ('riverbank.rs', ' || waterfront.iter().any(|w| w.dist(q) <= RIVER_STEP / 2.0))', ')', 'embankment ignores waterfront'),
    ('riverbank.rs', 'Some(q) if run.last().is_none_or(|l| (l.1 && q.1) || seg_ok(l.0, q.0)) => run.push(q),', 'Some(q) if run.last().is_none_or(|l| seg_ok(l.0, q.0)) => run.push(q),', 'existing pair checked'),
    ('riverbank.rs', 'let joins = k + 1 < run.len() && !(run[k].1 && run[k + 1].1);', 'let joins = k + 1 < run.len();', 'street over existing'),
    ('riverbank.rs', 'Some((t, _)) if t * BACK_EXISTING_M.1 < BACK_EXISTING_M.0 => None,', 'Some((t, _)) if t * BACK_EXISTING_M.1 < 0.0 => None,', 'at-water arm'),
    ('riverbank.rs', '(reached && side_of(site, b) == side && !in_plaza(b)).then_some((b, false, w))', '(reached && side_of(site, b) == side).then_some((b, false, w))', 'plaza point'),
    ('riverbank.rs', 'if dry_on_side_w(site, b, w, side, WATER_LANE_W) && !crosses_wall(b, w, wall) && !crosses_plaza(b, w) {', 'if dry_on_side_w(site, b, w, side, WATER_LANE_W) && !crosses_plaza(b, w) {', 'lane wall'),
    ('riverbank.rs', 'if ties.is_empty() && !run.iter().any(|t| t.1) {', 'if false {', 'unjoined run laid'),
    ('riverbank.rs', 'let in_wall = |p: Vec2| wall.is_none_or(|r| r.len() < 3 || point_in_poly(p, r));', 'let in_wall = |_p: Vec2| true;', 'outside the wall'),
    ('generate.rs', 'let waterfront = if riverbank && pop_target >= BOTH_BANKS_MIN_POP && profile.planning != "radial" {', 'let waterfront = if riverbank && profile.planning != "radial" {', 'village waterfront'),
    ('village.rs', 'for e in g.edges.iter().filter(|e| e.alive && (site.riverbank() || !waterside(e.cls))) {', 'for e in g.edges.iter().filter(|e| e.alive && !waterside(e.cls)) {', 'no waterfront tofts'),
    ('village.rs', 'for e in g.edges.iter().filter(|e| e.alive && (site.riverbank() || !waterside(e.cls))) {', 'for e in g.edges.iter().filter(|e| e.alive) {', 'harbour tofts'),
]


def sha(p):
    return hashlib.sha1(open(p, 'rb').read()).hexdigest()


def main():
    out = open(sys.argv[1], 'w')
    before = {f: sha(U + f) for f in set(m[0] for m in M)}
    for f, old, new, name in M:
        p = U + f
        raw = open(p, 'rb').read()
        src = raw.decode('utf-8')
        if CRLF in src:
            old, new = old.replace(NL, CRLF), new.replace(NL, CRLF)
        n = src.count(old)
        if n != 1:
            out.write(f'{name}: SKIP (occurs {n} times)' + NL)
            out.flush()
            continue
        try:
            open(p, 'wb').write(src.replace(old, new).encode('utf-8'))
            r = subprocess.run('cargo test -p cartalith-urban --lib -- riverbank:: plaza:: village::', shell=True, cwd=ROOT,
                               capture_output=True, text=True, env={**os.environ, 'CARGO_TARGET_DIR': 'C:/cw/waterfront_target'})
            t = r.stdout + r.stderr
            if 'could not compile' in t or 'error[' in t:
                v = 'BUILD_ERROR'
            elif 'test result: FAILED' in t:
                v = 'KILLED by ' + ', '.join(l.split()[1] for l in t.splitlines() if l.startswith('test ') and l.endswith('FAILED'))
            elif 'test result: ok' in t:
                v = 'SURVIVED'
            else:
                v = 'UNKNOWN'
            out.write(f'{name}: {v}' + NL)
            out.flush()
        finally:
            open(p, 'wb').write(raw)
    for f, h in before.items():
        assert sha(U + f) == h, f + ' not restored'
    out.write('all files restored (hashes match)' + NL)


main()
