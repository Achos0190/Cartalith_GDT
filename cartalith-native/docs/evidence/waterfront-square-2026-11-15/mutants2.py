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
    ('plaza.rs', 'if let Some((d, m)) = riverbank_square(site, anchors, g, l0 * t, wd0 * t) {', 'if let Some((d, m)) = riverbank_square(site, anchors, g, l0, wd0) {', 'riverbank square unscaled'),
    ('riverbank.rs', 'pub const BACK_EXISTING_M: (f64, f64) = (20.0, 70.0);', 'pub const BACK_EXISTING_M: (f64, f64) = (20.0, 64.0);', 'existing high down'),
    ('riverbank.rs', 'pub const BACK_PARALLEL_DEG: f64 = 30.0;', 'pub const BACK_PARALLEL_DEG: f64 = 60.0;', 'parallel angle'),
    ('riverbank.rs', 'pub const BACK_BREAK_DROP: usize = 2;', 'pub const BACK_BREAK_DROP: usize = 1;', 'break drop'),
    ('riverbank.rs', ' || waterfront.iter().any(|w| w.dist(q) <= RIVER_STEP / 2.0))', ')', 'embankment ignores waterfront'),
    ('riverbank.rs', 'Some(q) if run.last().is_none_or(|l| (l.1 && q.1) || seg_ok(l.0, q.0)) => run.push(q),', 'Some(q) if run.last().is_none_or(|l| seg_ok(l.0, q.0)) => run.push(q),', 'existing pair checked'),
    ('riverbank.rs', 'let joins = k + 1 < run.len() && !(run[k].1 && run[k + 1].1);', 'let joins = k + 1 < run.len();', 'street over existing'),
    ('riverbank.rs', 'let in_wall = |p: Vec2| wall.is_none_or(|r| r.len() < 3 || point_in_poly(p, r));', 'let in_wall = |_p: Vec2| true;', 'outside the wall'),
    ('village.rs', 'for e in g.edges.iter().filter(|e| e.alive && (site.riverbank() || !waterside(e.cls))) {', 'for e in g.edges.iter().filter(|e| e.alive) {', 'harbour tofts'),
    ('riverbank.rs', '(reached && side_of(site, b) == side && !in_plaza(b) && in_wall(b)).then_some((b, false, w))', '(reached && side_of(site, b) == side && in_wall(b)).then_some((b, false, w))', 'plaza point'),
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
