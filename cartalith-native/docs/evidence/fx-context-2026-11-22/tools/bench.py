"""Controlled speed bench, no app: python bench.py <ctx>. Model already loaded by the caller. 3 reps of a ~5 600-token
prompt (distinct per rep so no prefix cache) and 200 generated tokens, streamed; prints first-token s, prefill tok/s
(prompt tokens / first-token s) and generation tok/s (completion tokens / (total - first)). reasoning_effort none."""
import http.client, json, sys, time
words = "river ridge basin delta plateau glacier harbour pass moor fen cliff dune steppe tundra canyon lagoon atoll estuary".split()
def prompt(rep):
    return ("Rep %d. Write a long list of invented terrain names, one per line, numbered, no commentary.\n" % rep) + " ".join(words[(i * 7 + rep) % len(words)] + str(i) for i in range(1150))
for rep in range(3):
    body = json.dumps({"model": "qwen3.8-27b", "messages": [{"role": "user", "content": prompt(rep)}], "max_tokens": 200, "temperature": 0, "stream": True,
                       "stream_options": {"include_usage": True}, "reasoning_effort": "none"})
    t0 = time.time(); c = http.client.HTTPConnection("127.0.0.1", 1234, timeout=900)
    c.request("POST", "/v1/chat/completions", body, {"Content-Type": "application/json"}); r = c.getresponse()
    first = None; usage = {}
    for ln in r.read().decode("utf8").splitlines() if False else iter(lambda: r.readline().decode("utf8"), ""):
        if ln.startswith("data: {"):
            d = json.loads(ln[6:])
            if first is None and d.get("choices") and d["choices"][0].get("delta", {}).get("content"):
                first = time.time() - t0
            if d.get("usage"): usage = d["usage"]
    tot = time.time() - t0
    p, cpl = usage.get("prompt_tokens"), usage.get("completion_tokens")
    print("ctx", sys.argv[1], "rep", rep, "prompt", p, "completion", cpl, "first_s %.1f" % first, "prefill_tps %.0f" % (p / first), "gen_tps %.1f" % (cpl / (tot - first)))
