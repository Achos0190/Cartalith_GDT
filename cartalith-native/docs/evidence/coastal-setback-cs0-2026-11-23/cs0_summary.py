"""CS-0 summary: the per-world tables and bar baselines from the probe dumps and the census TSVs.

  python -I cs0_summary.py DUMPS_DIR OUT_DIR > summary.txt

DUMPS_DIR holds `cs0_probe.gd`'s `w<seed>_<gw>.jsonl`; OUT_DIR holds `cs0_hazard_census`'s `rows_<seed>_<gw>.tsv`.
Reads only; every number printed is computed here from those files (no constant is restated)."""
import csv, json, pathlib, sys

dumps, out = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
WORLDS = [("483920", "384"), ("912345", "384"), ("246371", "384"), ("83744", "384"), ("483920", "2048"), ("532187", "2048")]
REACH = 23.0   # HARBOUR_REACH_M, FK's one-street bar (research 2.3)


def q(v, f):
    v = sorted(v)
    return v[round((len(v) - 1) * f)] if v else float("nan")


print("== Bar baselines from the app path (cs0_probe.gd: town_bank_census and town_debug_json, civ.bank_sites on) ==")
for seed, gw in WORLDS:
    lines = (dumps / ("w%s_%s.jsonl" % (seed, gw))).read_text().splitlines()
    rows = [json.loads(l) for l in lines[1:]]
    laid = [r for r in rows if r["c"].get("laid")]
    housed = [100.0 * r["c"]["pop"] / max(1.0, r["c"]["pop_target"]) for r in laid]
    s = lambda k: sum(int(r["c"].get(k, 0)) for r in laid)
    across = sum(1 for r in laid if any(r["c"].get("side_" + z) == -1 for z in ("x1", "x4", "x16", "x64")))
    print("%s at %s: settlements %d, laid %d; ports (badge) %d; housed p10 %.1f %%, under 80 %% %d; buildings in the town's own water %d; "
          "over a drawn river b/w/s %d/%d/%d; over drawn sea b/w/s %d/%d/%d; over drawn lake b/w/s %d/%d/%d; markers across a drawn river from their market (any zoom) %d"
          % (seed, gw, len(rows), len(laid), sum(1 for r in rows if r["coastal"]), q(housed, 0.1), sum(1 for h in housed if h < 80.0), s("wet"),
             s("over_b"), s("over_w"), s("over_s"), s("sea_over_b"), s("sea_over_w"), s("sea_over_s"), s("lake_over_b"), s("lake_over_w"), s("lake_over_s"), across))

print("\n== Per-world classes (cs0_hazard_census rows) ==")
allrows = []
for seed, gw in WORLDS:
    p = out / ("rows_%s_%s.tsv" % (seed, gw))
    rows = list(csv.DictReader(p.open(), delimiter="\t"))
    allrows += rows
    for w in ("coast", "river"):
        rr = [r for r in rows if r["water"] == w]
        c = {k: sum(1 for r in rr if r["class"] == k) for k in "abcd"}
        print("%s at %s, %s: %d towns; a %d, b %d, c %d, d %d" % (seed, gw, w, len(rr), c["a"], c["b"], c["c"], c["d"]))

for size in ("384", "2048"):
    for w in ("coast", "river"):
        rr = [r for r in allrows if r["gw"] == size and r["water"] == w]
        c = {k: sum(1 for r in rr if r["class"] == k) for k in "abcd"}
        n = max(1, len(rr))
        print("ALL %s %s: %d towns; a %d (%.0f %%), b %d (%.0f %%), c %d (%.0f %%), d %d (%.0f %%); market to water p90 %.0f m, max %.0f m"
              % (size, w, len(rr), c["a"], 100 * c["a"] / n, c["b"], 100 * c["b"] / n, c["c"], 100 * c["c"] / n, c["d"], 100 * c["d"] / n,
                 q([float(r["market_m"]) for r in rr], 0.9), max([float(r["market_m"]) for r in rr] or [float("nan")])))

print("\n== B-FK baselines (badged harbours with the drawn sea in the box) ==")
for size in ("384", "2048"):
    h = [r for r in allrows if r["gw"] == size and r["water"] == "coast" and r["badge"] == "true"]
    # A quay is "at its water" at any water the map draws (a river-mouth quay lines the river, not the sea).
    gap = lambda r: float(r["quay_any_water_m"]) if r["quay_any_water_m"] != "none" else float(r["landing_m"])
    over = [r for r in h if gap(r) > REACH]
    noreason = [r for r in h if float(r["landing_m"]) > REACH and r["class"] in ("b", "d")]
    print("%s: harbours %d; B-FK1 (quay over 23 m from any drawn water, else nearest street or building over 23 m from the drawn sea) %d %s; "
          "B-FK3 (nearest street or building over 23 m, no reason: class b or d) %d %s"
          % (size, len(h), len(over), [(r["seed"], r["i"], r["x"] + ":" + r["y"], round(gap(r))) for r in over],
             len(noreason), [(r["seed"], r["i"], r["x"] + ":" + r["y"], r["landing_m"]) for r in noreason]))

print("\n== What 'set back' is made of (the market over the line, but the houses?) ==")
for size in ("384", "2048"):
    for w, line in (("coast", 100.0), ("river", 120.0)):
        for k in "ad":
            rr = [r for r in allrows if r["gw"] == size and r["water"] == w and r["class"] == k]
            shore = [r for r in rr if float(r["landing_m"]) <= REACH]
            near = [r for r in rr if float(r["market_m"]) <= 1.5 * line]
            print("%s %s class %s: %d; of them the nearest street or building within 23 m of the water %d; market within 1.5x the line (%.0f m) %d"
                  % (size, w, k, len(rr), len(shore), 1.5 * line, len(near)))

print("\n== River class (c): where the floodplain reason comes from ==")
for size in ("384", "2048"):
    rr = [r for r in allrows if r["gw"] == size and r["water"] == "river" and r["class"] == "c"]
    lf6 = sum(1 for r in rr if r["mkt_landform"] == "6")
    fl = sum(1 for r in rr if float(r["mkt_flood"]) > 0.55)
    home_safe = sum(1 for r in rr if float(r["home_flood"]) <= 0.55 and r["home_landform"] != "6")
    moved = sum(1 for r in rr if r["origin_m"] != "-")
    cross = sum(1 for r in rr if (float(r["home_flood"]) <= 0.55 and r["home_landform"] != "6") and r["origin_m"] != "-")
    safe = sum(1 for r in rr if r["safe_in_cap"] == "true")
    print("%s: %d river towns in class c; market cell floodplain (landform 6) %d, flood over 0.55 %d; settlement's own cell clear of both %d; "
          "laid round an HM origin off the cell %d (of them with a clear own cell %d); reason-free ground within the 500 m cap %d"
          % (size, len(rr), lf6, fl, home_safe, moved, cross, safe))

print("\n== Go/no-go inputs (coast at 2048) ==")
rr = [r for r in allrows if r["gw"] == "2048" and r["water"] == "coast"]
reason = [r for r in rr if r["market_reasons"] != "none" or r["class"] == "a" or (r["rule_core_m"] == "none") or (r["rule_core_m"] != "none" and float(r["rule_core_m"]) > 100.0)]
res = sum(1 for r in rr if r["h_resolves"] == "true")
cv = sum(1 for r in rr if r["cell_varies"] == "true")
print("coastal towns at 2048: %d; with a reason (the rule moves or holds back the core) %d (%.0f %%); the height reason splits the land in the cap %d; the cell fields vary %d"
      % (len(rr), len(reason), 100.0 * len(reason) / max(1, len(rr)), res, cv))
for size in ("384", "2048"):
    for w in ("coast", "river"):
        rr = [r for r in allrows if r["gw"] == size and r["water"] == w]
        print("%s %s: height reason splits %d of %d; cell fields vary %d of %d" % (size, w, sum(1 for r in rr if r["h_resolves"] == "true"), len(rr), sum(1 for r in rr if r["cell_varies"] == "true"), len(rr)))

print("\n== The FK row's towns on the app path ==")
for seed, gw, cell in (("483920", "384", ("380", "209")), ("483920", "2048", ("1323", "81")), ("483920", "2048", ("1323", "80"))):
    for r in allrows:
        if r["seed"] == seed and r["gw"] == gw and (r["x"], r["y"]) == cell:
            print(seed, gw, "#" + r["i"], cell, r["kind"], r["water"], "class", r["class"], "market", r["market_m"], "quay", r["quay_m"], "nearest street/building", r["landing_m"],
                  "reasons front/market", r["front_reasons"], "/", r["market_reasons"], "rule core", r["rule_core_m"], "exposure", r["exposure"])
