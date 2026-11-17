"""Offline re-pricing of the industrial standing army from `IA row` lines
(g1_baseline.rs::ia_dump_industrial_rows). Pins nothing.

Chain 1 is  standing = pop*(1-a)*Efactor*fee(s)/upkeep  (manpower.rs,
manpower_with_levy_bonus); field and levy read neither fee nor upkeep, so a
candidate that changes only the fee ceiling or the upkeep re-prices standing
analytically: standing' = standing * fee'/fee * u/u'.  The control below checks
the formula against the model's own standing on every row before any candidate
is read (a re-price that cannot reproduce the baseline proves nothing).

usage: python -I analyse.py ia_dump_run.log
"""
import re, sys, statistics as st, math

FLOOR, CEIL = 0.04, 0.16
rows = []
for line in open(sys.argv[1], encoding="utf-8", errors="replace"):
    if not line.startswith("IA row"):
        continue
    t = line.split()
    d = {"world": "/".join(t[2:5]), "f": t[5], "gov": t[7]}
    for k in "pop E s L a cf fee u standing field levy".split():
        d[k] = float(t[t.index(k) + 1])
    i = t.index("band")
    d["band"] = (float(t[i + 1]), float(t[i + 2]))
    d["era"] = t[t.index("era") + 1]
    d["order"] = t[-1]
    rows.append(d)
live = [r for r in rows if r["pop"] > 0]
print(f"rows {len(rows)}, non-empty {len(live)}")
assert len(live) >= 288, "silently short dump"

def fee(s, c): return FLOOR + (c - FLOOR) * s
def reprice(r, c=CEIL, u=None):
    uu = r["u"] if u is None else u(r)
    return r["standing"] * fee(r["s"], c) / fee(r["s"], CEIL) * r["u"] / uu

# control: the model's own standing from its own drivers
bad = 0
for r in live:
    e = r["E"]
    be = e if e <= 1.0 else 1.0
    own = r["pop"] * (1 - r["a"]) * be * r["fee"] / r["u"]
    if abs(own - r["standing"]) > 1e-3 * max(1.0, r["standing"]) * 1e-2:
        bad += 1
assert bad == 0, f"chain-1 reconstruction fails on {bad} rows"
print("control: chain 1 reconstructs the model's standing on every row")

def ordered(r, stand): return stand < r["field"] and r["field"] < r["levy"]
def share(r, stand): return stand / (r["pop"] * r["cf"])
def verdict(r, stand):
    x = share(r, stand)
    return "below" if x < r["band"][0] else "above" if x > r["band"][1] else "within"

def report(name, c=CEIL, u=None, detail=False):
    un = [r for r in live if not ordered(r, reprice(r, c, u))]
    v = {"within": 0, "above": 0, "below": 0}
    for r in live:
        v[verdict(r, reprice(r, c, u))] += 1
    med = st.median(share(r, reprice(r, c, u)) for r in live)
    print(f"{name}: unordered {len(un)} of {len(live)}; standing verdicts {v}; median {100*med:.2f} %")
    if detail:
        for r in un:
            x = reprice(r, c, u)
            print(f"   leaf {r['world']} f{r['f']} {r['gov']} E {r['E']:.3f} s {r['s']:.3f} L {r['L']:.3f} standing {100*share(r,x):.2f} % of citizens, band {r['band']}, era {r['era']}, standing/field {x/r['field']:.3f}")
    return un

print("\n== 1. baseline leaves ==")
leaves = report("baseline", detail=True)
eras = {}
for r in live: eras[r["era"]] = eras.get(r["era"], 0) + 1
print("eras over the 290:", eras)
print("state capacity of the live factions: min %.3f median %.3f max %.3f" % (min(r["s"] for r in live), st.median(r["s"] for r in live), max(r["s"] for r in live)))
over = [r for r in live if share(r, r["standing"]) > r["band"][1]]
print(f"standing above its own band, ordered or not: {len(over)} (of which unordered {len([r for r in over if not ordered(r, r['standing'])])})")
by_gov = {}
for r in live:
    g = by_gov.setdefault(r["gov"], [0, 0, 0])
    g[0] += 1; g[1] += not ordered(r, r["standing"]); g[2] += share(r, r["standing"]) > r["band"][1]
print("by government [n, unordered, above band]:", by_gov)

print("\n== 2. attribution: each cause isolated ==")
print("-- upkeep alone (a level change: every polity scaled alike)")
for u in [1.10, 1.15, 1.20, 1.25, 1.30, 1.324, 1.33, 1.40, 1.50]:
    report(f"upkeep {u}", u=lambda r, u=u: u)
print("-- ceiling alone, industrial brackets (a slope change: the spread over s compressed)")
for c in [0.15, 0.14, 0.13, 0.128, 0.125, 0.12, 0.11]:
    report(f"ceiling {c}", c=c)

print("\n== 3. candidates on the 290-faction sweep ==")
FEE_REF = 0.09056  # geometric-mean fee of the upkeep derivation set; driver_space_run.log "DS derivation set"
report("C1 upkeep 1.33 (level; band top at s 0.91)", u=lambda r: 1.33)
report("C2prime ceiling 0.08 with upkeep re-derived to 0.6785 (rule-consistent slope change)", c=0.08, u=lambda r: 0.6785)
report("C2 industrial ceiling 0.128 (slope; band top at s 0.91)", c=0.128)
for th in (0.25, 0.5, 0.75, 1.0):
    report(f"C3 upkeep scaled by (fee/fee_ref)^{th}", u=lambda r, th=th: r["u"] * (fee(r["s"], CEIL) / FEE_REF) ** th)
print("\n== 4. minimal single change that clears the sweep's leaves ==")

for u in [1.07 + 0.001 * k for k in range(0, 40)]:
    if not [r for r in live if not ordered(r, reprice(r, u=lambda r, u=u: u))]:
        print(f"upkeep alone clears the sweep from {u:.3f}"); break
for c in [0.16 - 0.0005 * k for k in range(0, 200)]:
    if not [r for r in live if not ordered(r, reprice(r, c=c))]:
        print(f"ceiling alone first clears the sweep at {c:.4f}"); break
print("highest fee on the sweep %.5f against EXTRACTION_CEILING %.2f (the ceiling is never reached: state capacity clamps at 0.95 and tops out at 0.910 here)" % (max(r["fee"] for r in live), CEIL))
# how far is each leaf over its field army, the margin a fix must buy
print("margins standing/field of the leaves:", [round(r['standing']/r['field'], 4) for r in leaves])
# which factions are above their band, by cause: s-tail (high fee) vs low-E
print("above-band factions: s median %.3f, E median %.3f" % (st.median(r["s"] for r in over), st.median(r["E"] for r in over)))
print("below-band factions: s median %.3f, E median %.3f" % (st.median(r["s"] for r in live if share(r, r["standing"]) < r["band"][0]), st.median(r["E"] for r in live if share(r, r["standing"]) < r["band"][0])))
