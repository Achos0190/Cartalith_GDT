"""LM Studio load-setting tuning for the LLM-6c stage-2 local arms (owner instruction 2026-10-26).

Usage: python -I tune.py <out.jsonl> <model key> <config name> [--cli "<lms load flags>"] [--rest '<json>']

For one configuration: unloads everything, loads the model either through the `lms load` CLI
(flags given verbatim) or the LM Studio REST endpoint `/api/v1/models/load` (JSON body merged
over {"model": key, "echo_load_config": true}), records the load config LM Studio echoes, the
GPU adapter memory (dedicated and shared, every adapter, from Windows' `GPU Adapter Memory`
counters) once loaded and once after the runs, then sends a FIXED prompt of about 400 tokens
three times (thinking off via `reasoning_effort: "none"`, temperature 0, `max_tokens` 256,
streamed) and records time to first content token and generation speed
(completion tokens / (end - first token)). One warm-up request is sent first and excluded.

Synthetic text only. Writes one JSON line per configuration. Prints ASCII.
"""
import json
import statistics
import subprocess
import sys
import time
import urllib.request

LMS = r"C:\Users\Vincent\.lmstudio\bin\lms.exe"
BASE = "http://127.0.0.1:1234"
# About 400 tokens of fixed, synthetic prompt (measured from the server's prompt_tokens; stated per run).
PARA = ("The valley of Ostmere lies between two ridges of grey granite. Its river, the Lenne, runs north to a cold "
        "inland sea, and every spring the snowmelt floods the lower meadows where the ferrymen keep their boats. ")
PROMPT = ("Here is a description of a fictional region.\n" + PARA * 9 +
          "\nWrite a long, detailed travel guide to this region, in plain prose, at least 400 words.")


def gpu_mem():
    ps = ("$c=Get-Counter '\\GPU Adapter Memory(*)\\Dedicated Usage','\\GPU Adapter Memory(*)\\Shared Usage';"
          "$c.CounterSamples | % { '{0}|{1}|{2}' -f $_.Path, $_.InstanceName, [int64]$_.CookedValue }")
    out = subprocess.run(["powershell", "-NoProfile", "-Command", ps], capture_output=True, text=True).stdout
    mem = {}
    for line in out.splitlines():
        parts = line.strip().split("|")
        if len(parts) != 3:
            continue
        kind = "dedicated" if "dedicated" in parts[0].lower() else "shared"
        mem.setdefault(parts[1], {})[kind] = round(int(parts[2]) / 2**30, 2)
    return mem


def post(path, body, timeout=600):
    req = urllib.request.Request(BASE + path, data=json.dumps(body).encode(), headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read())


def one_run(model):
    body = {"model": model, "messages": [{"role": "user", "content": PROMPT}], "max_tokens": 256, "temperature": 0,
            "stream": True, "stream_options": {"include_usage": True}, "reasoning_effort": "none"}
    req = urllib.request.Request(BASE + "/v1/chat/completions", data=json.dumps(body).encode(),
                                 headers={"Content-Type": "application/json"})
    t0 = time.time()
    first = None
    usage = {}
    with urllib.request.urlopen(req, timeout=900) as r:
        for raw in r:
            line = raw.decode("utf-8", "replace").strip()
            if not line.startswith("data:") or line == "data: [DONE]":
                continue
            j = json.loads(line[5:])
            if j.get("usage"):
                usage = j["usage"]
            for ch in j.get("choices", []):
                if ch.get("delta", {}).get("content") and first is None:
                    first = time.time()
    end = time.time()
    n = usage.get("completion_tokens", 0)
    return {"ttft_s": round(first - t0, 3) if first else None, "total_s": round(end - t0, 2),
            "prompt_tokens": usage.get("prompt_tokens"), "completion_tokens": n,
            "tok_s": round((n - 1) / (end - first), 2) if first and n > 1 and end > first else None}


def write_fields(path, fields):
    """LM Studio's per-model load defaults (the file its GUI writes when a user saves load
    settings for one model file). WHY: KV-cache type, CPU threads, mmap and batch size are not
    exposed by `lms load` or the REST load endpoint (measured 2026-10-26: the REST endpoint
    refuses the keys by name, and `--engine-config-file` is vLLM-only), so this file is the
    only lever. `n_threads` and `n_ctx` in the server log confirmed it is read; keys it does
    not echo are confirmed or refuted by their effect (VRAM, speed), and the report says which."""
    import os
    os.makedirs(os.path.dirname(path), exist_ok=True)
    body = {"preset": "", "operation": {"fields": []},
            "load": {"fields": [{"key": k, "value": v} for k, v in fields.items()]}}
    with open(path, "w", encoding="utf-8") as f:
        json.dump(body, f)


def main():
    out, model, name = sys.argv[1:4]
    cli = rest = fields = None
    if "--cli" in sys.argv:
        cli = sys.argv[sys.argv.index("--cli") + 1]
    if "--rest" in sys.argv:
        rest = json.loads(sys.argv[sys.argv.index("--rest") + 1])
    if "--fields" in sys.argv:
        fields = json.loads(sys.argv[sys.argv.index("--fields") + 1])
        write_fields(sys.argv[sys.argv.index("--fields-file") + 1], fields)
    subprocess.run([LMS, "unload", "--all"], capture_output=True)
    time.sleep(3)
    rec = {"config": name, "model": model, "cli": cli, "rest": rest, "fields": fields, "mem_idle": gpu_mem()}
    t0 = time.time()
    if rest is not None:
        body = {"model": model, "echo_load_config": True}
        body.update(rest)
        try:
            lc = post("/api/v1/models/load", body)
            cfg = dict(lc.get("load_config", {}))
            cfg.pop("prompt_template", None)
            rec["load_config"] = cfg
        except urllib.error.HTTPError as e:
            rec["load_error"] = e.read().decode("utf-8", "replace")[:400]
    else:
        p = subprocess.run([LMS, "load", model, "-y"] + (cli.split() if cli else []), capture_output=True, text=True,
                           encoding="utf-8", errors="replace")
        rec["load_stdout"] = (p.stdout + p.stderr)[-400:]
        # What the engine actually got, from LM Studio's own server log (newest file).
        import glob, os
        logs = sorted(glob.glob(r"C:\Users\Vincent\.lmstudio\server-logs\*\*.log"), key=os.path.getmtime)
        if logs:
            tail = open(logs[-1], encoding="utf-8", errors="replace").read().splitlines()[-600:]
            keep = [l.strip()[:200] for l in tail if any(s in l for s in ("n_threads =", "n_ctx_slot", "n_gpu_layers", "MTP draft"))]
            rec["engine_log"] = keep[-4:]
    rec["load_s"] = round(time.time() - t0, 1)
    if "load_error" not in rec:
        rec["mem_loaded"] = gpu_mem()
        try:
            one_run(model)  # warm-up, excluded
            runs = [one_run(model) for _ in range(3)]
            rec["runs"] = runs
            for k in ("ttft_s", "tok_s"):
                vals = [r[k] for r in runs if r[k] is not None]
                if vals:
                    rec[k + "_median"] = statistics.median(vals)
                    rec[k + "_range"] = [min(vals), max(vals)]
        except Exception as e:  # a failed config is data, not a crash
            rec["run_error"] = repr(e)[:400]
        rec["mem_after"] = gpu_mem()
    with open(out, "a", encoding="utf-8") as f:
        f.write(json.dumps(rec) + "\n")
    print(name, "load", rec.get("load_s"), "ttft", rec.get("ttft_s_median"), rec.get("ttft_s_range"),
          "tok/s", rec.get("tok_s_median"), rec.get("tok_s_range"), rec.get("load_error", ""), rec.get("run_error", ""))


if __name__ == "__main__":
    main()
