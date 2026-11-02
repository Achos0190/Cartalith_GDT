"""Summarise bench.js JSONL output: per file, median [min..max] (n) of each metric path, and the
v2.72/v2.71 ratio of medians. Usage: python3 -I summarize.py <file.jsonl> <metric.path> [...]
A path is dot-separated; a list index may be given as a number. '*' as the only path prints every
numeric leaf found in the first record."""
import json, sys, statistics

def get(d, path):
    for p in path.split('.'):
        if isinstance(d, list):
            try: d = d[int(p)]
            except Exception: return None
        elif isinstance(d, dict):
            if p not in d: return None
            d = d[p]
        else:
            return None
    return d if isinstance(d, (int, float)) and not isinstance(d, bool) else None

def leaves(d, pre=''):
    if isinstance(d, dict):
        for k, v in d.items(): yield from leaves(v, pre + ('.' if pre else '') + str(k))
    elif isinstance(d, list):
        if len(d) <= 12:
            for i, v in enumerate(d): yield from leaves(v, pre + '.' + str(i))
    elif isinstance(d, (int, float)) and not isinstance(d, bool):
        yield pre

def fmt(x):
    if x is None: return '-'
    a = abs(x)
    return f'{x:.0f}' if a >= 100 else f'{x:.1f}' if a >= 10 else f'{x:.2f}' if a >= 0.1 else f'{x:.3f}'

def main():
    fn, paths = sys.argv[1], sys.argv[2:]
    recs = [json.loads(l) for l in open(fn) if l.strip()]
    recs = [r for r in recs if 'fatal' not in r]
    files = sorted({r['file'] for r in recs})
    if paths == ['*']:
        paths = list(leaves(recs[0]))
    print(f'{"metric":58s} ' + '  '.join(f'{f:>30s}' for f in files) + '   ratio(B/A)')
    for p in paths:
        meds = []
        cells = []
        for f in files:
            vals = [get(r, p) for r in recs if r['file'] == f]
            vals = [v for v in vals if v is not None]
            if not vals:
                cells.append(f'{"-":>30s}'); meds.append(None); continue
            m = statistics.median(vals)
            meds.append(m)
            cells.append(f'{fmt(m) + " [" + fmt(min(vals)) + ".." + fmt(max(vals)) + "] n=" + str(len(vals)):>30s}')
        ratio = (meds[1] / meds[0]) if len(meds) == 2 and meds[0] not in (None, 0) and meds[1] is not None else None
        print(f'{p:58s} ' + '  '.join(cells) + (f'   {ratio:.3f}' if ratio is not None else ''))

main()
