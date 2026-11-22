#!/usr/bin/env python3
"""Read cartalith-trace JSON lines (user://logs/trace-*.jsonl). Standard library only.

  trace_report.py top  FILE... [-n 20]      spans by total and by p95 elapsed_us
  trace_report.py tree FILE    [--run N]    span tree of the Nth (default last) gen::run, or any root
  trace_report.py diff A B     [--pct 5]    per-span-name total deltas between two runs of one seed
  trace_report.py grep FILE... key=value    records whose field `key` (or name/target/op) equals value
"""
import argparse, json, sys


def load(paths):
    out = []
    for p in paths:
        with open(p, encoding="utf-8") as f:
            for ln in f:
                try:
                    out.append(json.loads(ln))
                except ValueError:
                    pass  # a half-written last line
    return out


def spans(recs):
    return [r for r in recs if r.get("t") == "span"]


def totals(recs):
    t = {}
    for s in spans(recs):
        t.setdefault(s["name"], []).append(s["elapsed_us"])
    return t


def p95(v):
    v = sorted(v)
    return v[min(len(v) - 1, int(len(v) * 0.95))]


def top(a):
    t = totals(load(a.files))
    rows = [(n, len(v), sum(v), p95(v)) for n, v in t.items()]
    print(f"{'span':32} {'n':>6} {'total_ms':>10} {'p95_ms':>10}")
    for n, c, tot, q in sorted(rows, key=lambda r: -r[2])[: a.n]:
        print(f"{n:32} {c:6} {tot/1000:10.1f} {q/1000:10.1f}")


def tree(a):
    ss = spans(load(a.files))
    kids = {}
    for s in ss:
        kids.setdefault(s["parent"], []).append(s)
    ids = {s["id"] for s in ss}
    runs = [s for s in ss if s["name"] == "gen::run"] or [s for s in ss if s["parent"] not in ids]
    if not runs:
        sys.exit("no spans")
    root = runs[a.run if a.run is not None else -1]

    def show(s, d):
        f = " ".join(f"{k}={v}" for k, v in s.get("f", {}).items())
        print(f"{'  '*d}{s['name']}  {s['elapsed_us']/1000:.1f} ms  {f}")
        for c in sorted(kids.get(s["id"], []), key=lambda c: c["ts_us"]):
            show(c, d + 1)

    show(root, 0)
    inner = sum(c["elapsed_us"] for c in kids.get(root["id"], []))
    print(f"children sum {inner/1000:.1f} ms of {root['elapsed_us']/1000:.1f} ms ({100*inner/max(1,root['elapsed_us']):.1f} %)")


def diff(a):
    x, y = totals(load([a.a])), totals(load([a.b]))
    print(f"{'span':32} {'a_ms':>10} {'b_ms':>10} {'delta':>8}")
    for n in sorted(set(x) | set(y)):
        ta, tb = sum(x.get(n, [])), sum(y.get(n, []))
        pct = 100 * (tb - ta) / ta if ta else float("inf")
        flag = "  <--" if abs(pct) >= a.pct else ""
        print(f"{n:32} {ta/1000:10.1f} {tb/1000:10.1f} {pct:+7.1f}%{flag}")


def grep(a):
    k, _, v = a.expr.partition("=")
    for r in load(a.files):
        got = r.get("f", {}).get(k, r.get(k))
        if str(got) == v:
            print(json.dumps(r))


ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
sp = ap.add_subparsers(dest="cmd", required=True)
p = sp.add_parser("top"); p.add_argument("files", nargs="+"); p.add_argument("-n", type=int, default=20); p.set_defaults(fn=top)
p = sp.add_parser("tree"); p.add_argument("files", nargs="+"); p.add_argument("--run", type=int); p.set_defaults(fn=tree)
p = sp.add_parser("diff"); p.add_argument("a"); p.add_argument("b"); p.add_argument("--pct", type=float, default=5); p.set_defaults(fn=diff)
p = sp.add_parser("grep"); p.add_argument("files", nargs="+"); p.set_defaults(fn=grep)
a = ap.parse_args()
if a.cmd == "grep":
    a.expr = a.files.pop()
a.fn(a)
