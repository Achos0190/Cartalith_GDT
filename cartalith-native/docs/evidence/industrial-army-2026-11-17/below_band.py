"""Item 2 of Ruling FU: why 164 of 290 industrial factions read below the 5-10 %
standing band (`ECOLOGICAL_ARMY_TERM_SCOPE.md` section 10.12). Reads `IA row tech`
lines of `g1_baseline.rs::ia_dump_industrial_rows` (all six ag-tech rows, 20
worlds, the model WITH Ruling FU's C3). Pins nothing.

Chain 1: standing = pop*(1-a)*g(E)*fee/u, g(E) = min(E, 1).  Field and levy do not
read g below 1, so re-pricing g re-prices standing alone (control below).
usage: python -I below_band.py ia_dump_run.log
"""
import sys, statistics as st
rows = []
for line in open(sys.argv[1], encoding="utf-8", errors="replace"):
    if not line.startswith("IA row tech"):
        continue
    t = line.split()
    d = {"tech": t[3], "gov": t[t.index("gov") + 1]}
    for k in "pop E s L a cf fee u standing field levy".split():
        d[k] = float(t[t.index(k) + 1])
    i = t.index("band"); d["band"] = (float(t[i + 1]), float(t[i + 2]))
    d["era"] = t[t.index("era") + 1]
    rows.append(d)
live = [r for r in rows if r["pop"] > 0]
techs = ["subsistence", "traditionalAgrarian", "advancedAgrarian", "improvedAgrarian", "earlyIndustrial", "industrial"]
for r in live:
    own = r["pop"] * (1 - r["a"]) * min(r["E"], 1.0) * r["fee"] / r["u"]
    assert abs(own - r["standing"]) <= 1e-4 * max(1.0, r["standing"]), "control fails"
print("control: chain 1 reconstructs the model's standing on", len(live), "readings")
def share(r, g): return r["standing"] * g(r["E"]) / min(r["E"], 1.0) / (r["pop"] * r["cf"])
def verd(r, g):
    x = share(r, g)
    return "below" if x < r["band"][0] else "above" if x > r["band"][1] else "within"
def ordered(r, g):
    s = r["standing"] * g(r["E"]) / min(r["E"], 1.0)
    return s < r["field"] < r["levy"]
cands = {
    "baseline g=E": lambda e: min(e, 1.0),
    "A floor 0.5": lambda e: max(min(e, 1.0), 0.5),
    "A' floor 0.67": lambda e: max(min(e, 1.0), 0.67),
    "A'' floor 0.8": lambda e: max(min(e, 1.0), 0.8),
    "B sqrt": lambda e: min(e, 1.0) ** 0.5,
    "B' E^0.75": lambda e: min(e, 1.0) ** 0.75,
    "flat g=1": lambda e: 1.0,
}
ind = [r for r in live if r["tech"] == "industrial"]
below = [r for r in ind if verd(r, cands["baseline g=E"]) == "below"]
print(f"industrial sweep: {len(ind)} live, below {len(below)}; median E of below {st.median(r['E'] for r in below):.3f}; E<1 on {sum(r['E'] < 1 for r in below)}; E at the floor 0.25 on {sum(abs(r['E'] - 0.25) < 1e-6 for r in below)}")
print("below by government:", {g: sum(r['gov'] == g for r in below) for g in sorted({r['gov'] for r in below})})
fix = [r for r in below if share(r, lambda e: 1.0) >= r["band"][0]]
print(f"attribution: with g=1 alone {len(fix)} of {len(below)} clear the band's lower edge; {len(below) - len(fix)} stay below (median s of those {st.median(r['s'] for r in below if r not in fix):.3f})" if len(below) > len(fix) else f"attribution: g=1 clears all {len(below)}")
print("\n%-16s" % "candidate", "  ".join("%-22s" % t for t in techs[3:]))
for n, g in cands.items():
    cells = []
    for t in techs[3:]:
        rr = [r for r in live if r["tech"] == t]
        v = {"within": 0, "above": 0, "below": 0}
        for r in rr: v[verd(r, g)] += 1
        un = sum(not ordered(r, g) for r in rr)
        cells.append("%d/%d/%d un%d" % (v["within"], v["above"], v["below"], un))
    print("%-16s" % n, "  ".join("%-22s" % c for c in cells))
print("\nrows moved by a candidate that is NOT bracket-gated (E < 1 readings), per row of", len(live) // 6, "live:")
for t in techs:
    rr = [r for r in live if r["tech"] == t]
    print(f"  {t}: E<1 on {sum(r['E'] < 1 for r in rr)} of {len(rr)}")
print("\nbelow-band counts per row (baseline):", {t: sum(verd(r, cands['baseline g=E']) == 'below' for r in live if r['tech'] == t) for t in techs})
