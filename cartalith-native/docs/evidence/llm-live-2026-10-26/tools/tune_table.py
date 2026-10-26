"""Markdown table of the LM Studio tuning rows written by tune.py.

Usage: python -I tune_table.py <tune.jsonl> [model key filter]

Memory columns are the RX 7800 XT adapter (the largest dedicated usage of any adapter in the
row), read after the three timed runs. Medians of three runs with min..max.
"""
import json
import sys


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    rows = [json.loads(l) for l in open(sys.argv[1], encoding="utf-8") if l.strip()]
    filt = sys.argv[2] if len(sys.argv) > 2 else None
    print("| Model | Setting | Load s | First token s (median, min..max) | Generation tok/s (median, min..max) | Dedicated GiB | Shared GiB | Prompt tokens |")
    print("|---|---|---|---|---|---|---|---|")
    for r in rows:
        if filt and r["model"] != filt:
            continue
        mem = r.get("mem_after") or {}
        big = max(mem.values(), key=lambda m: m.get("dedicated", 0)) if mem else {}
        rng = lambda k: "..".join(f"{v:g}" for v in r.get(k + "_range", [])) or "-"
        pt = (r.get("runs") or [{}])[0].get("prompt_tokens", "-")
        err = r.get("load_error") or r.get("run_error") or ""
        print(f"| {r['model']} | {r['config']}{' (' + err[:80] + ')' if err else ''} | {r.get('load_s', '-')} | "
              f"{r.get('ttft_s_median', '-')} ({rng('ttft_s')}) | {r.get('tok_s_median', '-')} ({rng('tok_s')}) | "
              f"{big.get('dedicated', '-')} | {big.get('shared', '-')} | {pt} |")


if __name__ == "__main__":
    main()
