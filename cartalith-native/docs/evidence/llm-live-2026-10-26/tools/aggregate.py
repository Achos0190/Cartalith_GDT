"""Tables for the LLM-6c stage-2 live comparison (REPORT.md), from the probe's JSON lines.

Usage: python -I aggregate.py <dir with <arm>.jsonl files> <arm> [<arm> ...] > tables.md

Reads only what `_llmlive_params_probe.gd` wrote; computes nothing about the world itself.
Direction expectations are a LABELLED JUDGEMENT from GENERATION_PARAMETERS.md (listed in
EXPECT below); `climate.rain_dep` is deliberately not scored as a "drier" key
(LLM_ASSISTANT_SCOPE.md s.5b.8 item 4). A changed key outside EXPECT is neither right nor wrong.
Prints UTF-8 whatever the console's code page (MISTAKES.md: a cp1252 console crashed on model text).
"""
import json
import sys
from pathlib import Path

# key -> expected sign of (new - old) for each scored prompt. Labelled judgement.
EXPECT = {
    "no_sea": {"sea_level": -1},
    "drier_lower": {"climate.ocean": -1, "climate.ocean_hum": -1, "climate.evap": -1, "climate.rain_k": -1,
                    "peak_m": -1, "tect.alpha": -1, "world_structure.tectonic_energy": -1},
    "more_volcanoes": {"volc.count": +1},
}


def load(d, arm):
    recs = [json.loads(l) for l in Path(d, f"{arm}.jsonl").read_text(encoding="utf-8").splitlines() if l.strip()]
    return recs


def fmt(v):
    if isinstance(v, bool):
        return "on" if v else "off"
    if isinstance(v, float):
        return f"{v:.3g}" if abs(v) < 100 else f"{v:.0f}"
    return str(v)


def changes(rec):
    card = rec.get("card") or {}
    out = []
    for c in card.get("valid", []):
        out.append(f"`{c['key']}` {fmt(c.get('old', '?'))} -> {fmt(c['new'])}")
    rej = [f"{r.get('key', '?')} {r.get('rule', '')}" for r in card.get("rejected", [])]
    if card.get("structural"):
        rej.append(f"{card.get('structural_rule', '')} {card['structural']}")
    return out, rej


def direction(rec):
    exp = EXPECT.get(rec["id"])
    if not exp:
        return ""
    right = wrong = 0
    for c in (rec.get("card") or {}).get("valid", []):
        k = c["key"]
        if k in exp and isinstance(c.get("old"), (int, float)) and isinstance(c.get("new"), (int, float)):
            s = (c["new"] > c["old"]) - (c["new"] < c["old"])
            right += s == exp[k]
            wrong += s == -exp[k]
    return f"{right}/{right + wrong}" if right + wrong else "none scored"


def stat_line(s):
    if not s or "land_fraction" not in s:
        return "-"
    return (f"land {s['land_fraction']*100:.1f}%, lake {s['lake_fraction']*100:.1f}%, "
            f"mean land {s.get('mean_land_elev_m', float('nan')):.0f} m, max {s.get('max_land_elev_m', float('nan')):.0f} m, "
            f"volcanoes hit {s.get('volcanoes_hit', '-')}")


def secs(ms):
    return f"{ms/1000:.1f}" if isinstance(ms, (int, float)) else "-"


def main():
    sys.stdout.reconfigure(encoding="utf-8")  # model text carries arrows and dashes
    d = sys.argv[1]
    arms = sys.argv[2:]
    data = {a: load(d, a) for a in arms}
    print("## Base worlds\n")
    print("| Arm | Seed | Compute adapter | Generate s | Field digest | Statistics |")
    print("|---|---|---|---|---|---|")
    for a, recs in data.items():
        for r in recs:
            if r["kind"] == "base":
                dev = ", ".join(u.get("name", "?") for u in r["regen"].get("device_usage", [])) or "-"
                print(f"| {a} | {r['seed']} | {dev} | {secs(r['regen']['generate_ms'])} | `{r['digest'].get('field', '-')}` | {stat_line(r['stats'])} |")
    print("\n## Per request\n")
    ids = []
    for recs in data.values():
        for r in recs:
            if r["kind"] == "request" and (r["id"], r["seed"]) not in ids:
                ids.append((r["id"], r["seed"]))
    for rid, seed in ids:
        req = next(r["request"] for recs in data.values() for r in recs if r["kind"] == "request" and r["id"] == rid)
        print(f"### `{rid}` on seed {seed}: \"{req}\"\n")
        print("| Arm | Outcome | Changes proposed (valid) | Refused | Direction (scored keys) | After regenerate | First output s | Total s | Tokens in/out | Undo one step | Base restored |")
        print("|---|---|---|---|---|---|---|---|---|---|---|")
        for a, recs in data.items():
            r = next((x for x in recs if x["kind"] == "request" and x["id"] == rid and x["seed"] == seed), None)
            if r is None:
                print(f"| {a} | not run | | | | | | | | | |")
                continue
            ch, rej = changes(r)
            ev = r.get("events", [])
            if any(e["e"] == "failed" for e in ev):
                outcome = "failed: " + next(e["message"] for e in ev if e["e"] == "failed")[:120]
            elif r.get("outcome") == "applied":
                outcome = "proposal, applied"
            elif r.get("card"):
                outcome = "proposal, nothing valid"
            elif any(e["e"] == "answered" for e in ev):
                outcome = "answered in words"
            else:
                outcome = r.get("outcome", "?")
            toks = f"{r.get('prompt_tokens', '-')}/{r.get('completion_tokens', '-')}"
            undo = "-" if r.get("outcome") != "applied" else ("yes" if r.get("undo_params_equal") and not r.get("undo_param_keys_differing") else "NO")
            base = "-" if r.get("outcome") != "applied" else ("yes" if r.get("base_digest_equal") else "NO " + ",".join(r.get("base_digest_keys_differing", [])))
            print(f"| {a} | {outcome} | {'<br>'.join(ch) or '-'} | {', '.join(rej) or '-'} | {direction(r)} | {stat_line(r.get('stats_after'))} | "
                  f"{secs(r.get('first_output_ms'))} | {secs(r.get('total_ms'))} | {toks} | {undo} | {base} |")
        print()
    print("## Model words for requests answered without a proposal, and summaries\n")
    for rid, seed in ids:
        for a, recs in data.items():
            r = next((x for x in recs if x["kind"] == "request" and x["id"] == rid and x["seed"] == seed), None)
            if r is None:
                continue
            said = next((e.get("text", "") for e in r.get("events", []) if e["e"] == "answered"), "")
            summ = (r.get("card") or {}).get("summary", "")
            txt = (said or summ or r.get("model_text", "")).replace("\n", " ").strip()
            if txt:
                print(f"- `{rid}` s{seed} **{a}**: {txt[:600]}")
    print()


if __name__ == "__main__":
    main()
