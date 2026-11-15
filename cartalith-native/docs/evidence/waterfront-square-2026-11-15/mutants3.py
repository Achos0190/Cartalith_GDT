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
