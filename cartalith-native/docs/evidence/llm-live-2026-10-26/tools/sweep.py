"""Runs the tuning sweep for one LM Studio model file (ROCm llama.cpp runtime only, owner 2026-10-26).

Usage: python -I sweep.py <out.jsonl> <model key> <per-model config path> <set> [n_ctx_list]

<set> is "base" (one knob at a time against a fixed reference) or a JSON object of extra
configurations {name: [fields, cli]}. Deletes the per-model config file when it ends, so LM
Studio's own defaults are as they were before the sweep (the file did not exist).
"""
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
OFF = "llm.load.llama.acceleration.offloadRatio"
CTX = "llm.load.contextLength"
PAR = "llm.load.numParallelSessions"
FA = "llm.load.llama.flashAttention"
KT = "llm.load.llama.kCacheQuantizationType"
VT = "llm.load.llama.vCacheQuantizationType"
THR = "llm.load.llama.cpuThreadPoolSize"
EB = "llm.load.llama.evalBatchSize"
MMAP = "llm.load.llama.tryMmap"
KEEP = "llm.load.llama.keepModelInMemory"


def run(out, key, cfgpath, name, fields=None, cli=""):
    args = [sys.executable, "-I", os.path.join(HERE, "tune.py"), out, key, name, "--cli", cli]
    if fields is None:
        if os.path.exists(cfgpath):
            os.remove(cfgpath)
    else:
        args += ["--fields", json.dumps(fields), "--fields-file", cfgpath]
    subprocess.run(args)


def main():
    out, key, cfgpath, which = sys.argv[1:5]
    ref = {OFF: 1, CTX: 8192, PAR: 1, FA: True}
    try:
        if which == "base":
            run(out, key, cfgpath, "defaults: lms load, no flags, no saved settings")
            run(out, key, cfgpath, "--gpu max -c 16384 (first stage-2 runs)", None, "--gpu max -c 16384")
            run(out, key, cfgpath, "ref: offload 1, ctx 8192, parallel 1, FA on, KV f16, MTP default", dict(ref))
            run(out, key, cfgpath, "ref + ctx 16384", {**ref, CTX: 16384})
            run(out, key, cfgpath, "ref + FA off", {**ref, FA: False})
            run(out, key, cfgpath, "ref + KV q8_0/q8_0", {**ref, KT: {"checked": True, "value": "q8_0"}, VT: {"checked": True, "value": "q8_0"}})
            run(out, key, cfgpath, "ref + KV q4_0/q4_0", {**ref, KT: {"checked": True, "value": "q4_0"}, VT: {"checked": True, "value": "q4_0"}})
            run(out, key, cfgpath, "ref + threads 8", {**ref, THR: 8})
            run(out, key, cfgpath, "ref + eval batch 512", {**ref, EB: 512})
            run(out, key, cfgpath, "ref + no mmap, keep in memory", {**ref, MMAP: False, KEEP: True})
            run(out, key, cfgpath, "ref + MTP off", dict(ref), "--no-speculative-draft-mtp")
            run(out, key, cfgpath, "ref + MTP draft max 5", dict(ref), "--speculative-draft-mtp --speculative-draft-max-tokens 5")
            run(out, key, cfgpath, "ref + parallel 4", {**ref, PAR: 4})
        else:
            for name, (fields, cli) in json.loads(which).items():
                run(out, key, cfgpath, name, fields, cli)
    finally:
        if os.path.exists(cfgpath):
            os.remove(cfgpath)


if __name__ == "__main__":
    main()
