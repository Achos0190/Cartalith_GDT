"""LLM-6c stage 1: model comparison on generation-parameter requests (offline experiment).

Standalone, stdlib only.  Run with:  python -I run_eval.py --repo <repo> --out <dir> --models <names...>
Re-runnable by model name; results are appended to <out>/results.jsonl and finished
(model, suite, prompt, rep) cells are skipped, so a re-run only fills what is missing.
Local models talk to Ollama at http://localhost:11434.  Claude models (claude-*) use the
Anthropic Messages API; the key is read in-process from Windows Credential Manager
(target Cartalith/LLM/anthropic), never printed or written, and scrubbed from all output.
Images go to local Ollama (Ministral) and, by owner allowance for this offline experiment only, to Claude.
"""
import argparse, base64, ctypes, ctypes.wintypes as wt, json, os, sys, time, urllib.request, urllib.error

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import catalogue as C

OLLAMA = "http://localhost:11434"
NUM_CTX = 16384            # set explicitly for every local model (Ollama's own default is smaller)
TOOL_TEMP, PROSE_TEMP, VISION_TEMP = 0.3, 0.7, 0.2
CLAUDE_CALL_CAP = 150  # successful generations only
MAX_TURNS = 5

PROSE = [
    ("harbour", "Describe a small harbour town on a foggy morning, in about 150 words."),
    ("watchtower", "Write a short scene of about 150 words set in a ruined watchtower on a windswept ridge."),
    ("volcanic_island", "Write a one-paragraph history of a volcanic island and the people who settled it."),
]
TOOLS = [
    ("no_sea", "I don't want any sea, change the map"),
    ("more_volcanoes", "I want more volcanoes"),
    ("drier_lower", "drier climate, lower mountains"),
    ("cold_big_mountains", "make the world colder with big mountain ranges"),
    ("islands", "make it look like a chain of islands"),
    ("imp_river", "add a river here"),
    ("imp_prettier", "make the map prettier"),
    ("imp_pacific", "delete the Pacific"),
]
IMPOSSIBLE = {"imp_river", "imp_prettier", "imp_pacific"}
# expected direction per prompt: key -> +1 (should rise) / -1 (should fall).  A labelled judgement
# built from GENERATION_PARAMETERS.md's "what it does" column; used ONLY for scoring, never shown to a model.
EXPECT = {
    "no_sea": {"sea_level": -1},
    "more_volcanoes": {"volc.count": +1, "world_structure.hotspot_density": +1},
    "drier_lower": {"climate.ocean": -1, "climate.ocean_hum": -1, "climate.evap": -1, "climate.rain_k": -1,
                    "climate.rain_dep": +1, "peak_m": -1, "tect.alpha": -1, "world_structure.tectonic_energy": -1},
    "cold_big_mountains": {"climate.equator_temp": -1, "climate.pole_temp": -1, "climate.lapse_rate": +1,
                           "peak_m": +1, "tect.alpha": +1, "world_structure.tectonic_energy": +1},
    "islands": {"world_structure.fragmentation": +1, "sea_level": +1, "world_structure.continentality": -1},
}
REFERENCE = {"no_sea", "drier_lower", "cold_big_mountains"}  # the unambiguous-direction prompts

SECRET = [None]


def scrub(s):
    s = str(s)
    if SECRET[0]:
        s = s.replace(SECRET[0], "[REDACTED]")
    return s.replace("sk" + "-ant", "[REDACTED-PREFIX]")


def read_cred(target="Cartalith/LLM/anthropic"):
    class CRED(ctypes.Structure):
        _fields_ = [("Flags", wt.DWORD), ("Type", wt.DWORD), ("TargetName", wt.LPWSTR), ("Comment", wt.LPWSTR),
                    ("LastWritten", wt.FILETIME), ("CredentialBlobSize", wt.DWORD),
                    ("CredentialBlob", ctypes.POINTER(ctypes.c_ubyte)), ("Persist", wt.DWORD),
                    ("AttributeCount", wt.DWORD), ("Attributes", ctypes.c_void_p),
                    ("TargetAlias", wt.LPWSTR), ("UserName", wt.LPWSTR)]
    adv = ctypes.WinDLL("advapi32", use_last_error=True)
    adv.CredReadW.argtypes = [wt.LPCWSTR, wt.DWORD, wt.DWORD, ctypes.POINTER(ctypes.POINTER(CRED))]
    p = ctypes.POINTER(CRED)()
    if not adv.CredReadW(target, 1, 0, ctypes.byref(p)):
        raise RuntimeError("credential not found")
    n = p.contents.CredentialBlobSize
    raw = bytes(p.contents.CredentialBlob[i] for i in range(n))
    adv.CredFree(p)
    return raw.decode("utf-8").strip()


# --------------------------------------------------------------------------- transports
def http_json(url, body, headers, stream=False):
    req = urllib.request.Request(url, data=json.dumps(body).encode("utf8"), headers=headers, method="POST")
    return urllib.request.urlopen(req, timeout=900)


def ollama_chat(model, messages, tools, temp, num_predict, images=False):
    think = None
    if model.endswith("#nothink"):   # a spec suffix: same model, thinking switched off (think:false)
        model, think = model[:-8], False
    """Streams one Ollama /api/chat turn.  Returns dict(content, thinking, tool_calls, first_token_s, total_s, stats)."""
    body = {"model": model, "messages": messages, "stream": True,
            "options": {"temperature": temp, "num_ctx": NUM_CTX, "num_predict": num_predict}}
    if think is not None:
        body["think"] = think
    if tools:
        body["tools"] = [{"type": "function", "function": t} for t in tools]
    t0 = time.time(); first = None
    content, thinking, calls, final = [], [], [], {}
    try:
        resp = http_json(OLLAMA + "/api/chat", body, {"Content-Type": "application/json"})
        for line in resp:
            if not line.strip():
                continue
            j = json.loads(line)
            m = j.get("message") or {}
            if (m.get("content") or m.get("thinking") or m.get("tool_calls")) and first is None:
                first = time.time() - t0
            if m.get("content"): content.append(m["content"])
            if m.get("thinking"): thinking.append(m["thinking"])
            if m.get("tool_calls"): calls.extend(m["tool_calls"])
            if j.get("done"): final = j
    except urllib.error.HTTPError as e:
        return dict(error="HTTP %s %s" % (e.code, scrub(e.read()[:300])), total_s=time.time() - t0)
    except Exception as e:
        return dict(error=scrub(repr(e))[:300], total_s=time.time() - t0)
    tot = time.time() - t0
    st = {k: final.get(k) for k in ("eval_count", "eval_duration", "prompt_eval_count", "prompt_eval_duration",
                                    "load_duration", "total_duration", "done_reason")}
    return dict(content="".join(content), thinking="".join(thinking),
                tool_calls=[{"name": c["function"]["name"], "arguments": c["function"].get("arguments")} for c in calls],
                first_token_s=first, total_s=tot, stats=st)


def claude_chat(model, system, messages, tools, temp, key, state):
    body = {"model": model, "max_tokens": 1024, "system": system, "messages": messages, "temperature": temp}
    if tools:
        body["tools"] = [{"name": t["name"], "description": t["description"], "input_schema": t["parameters"]} for t in tools]
    hdr = {"x-api-key": key, "anthropic-version": "2023-06-01", "content-type": "application/json"}
    if state["calls"] >= CLAUDE_CALL_CAP:
        return dict(error="call cap reached", total_s=0)
    t0 = time.time()
    if state.get("temperature_dropped"):
        body.pop("temperature", None)   # this API rejected `temperature`; do not waste a call re-learning it
    for attempt in (0, 1):
        try:
            j = json.loads(http_json("https://api.anthropic.com/v1/messages", body, hdr).read())
            state["calls"] += 1  # only SUCCESSFUL generations count against the cap (a 400 costs no tokens)
            break
        except urllib.error.HTTPError as e:
            msg = scrub(e.read()[:400])
            if e.code == 400 and "temperature" in msg and "temperature" in body and attempt == 0:
                del body["temperature"]; state["temperature_dropped"] = True; state["rejected_400"] = state.get("rejected_400", 0) + 1; continue
            return dict(error="HTTP %s %s" % (e.code, msg), total_s=time.time() - t0)
        except Exception as e:
            return dict(error=scrub(repr(e))[:300], total_s=time.time() - t0)
    state["in"] += j["usage"].get("input_tokens", 0); state["out"] += j["usage"].get("output_tokens", 0)
    text = "".join(b.get("text", "") for b in j["content"] if b["type"] == "text")
    calls = [dict(id=b["id"], name=b["name"], arguments=b["input"]) for b in j["content"] if b["type"] == "tool_use"]
    return dict(content=text, tool_calls=calls, raw_content=j["content"], first_token_s=None, total_s=time.time() - t0,
                stats=dict(input_tokens=j["usage"].get("input_tokens"), output_tokens=j["usage"].get("output_tokens"),
                           stop_reason=j.get("stop_reason")))


# --------------------------------------------------------------------------- tool loop
def run_tool_case(model, prompt, rows, key, cstate):
    is_claude = model.startswith("claude-")
    shown_groups, schema_called, turns = [], False, []
    msgs = [{"role": "user", "content": prompt}] if is_claude else \
           [{"role": "system", "content": C.SYSTEM}, {"role": "user", "content": prompt}]
    tools = [C.tool_get_schema()]
    final = None; schema_errors = 0; bad_tool = 0
    for turn in range(MAX_TURNS):
        r = claude_chat(model, C.SYSTEM, msgs, tools, TOOL_TEMP, key, cstate) if is_claude else \
            ollama_chat(model, msgs, tools, TOOL_TEMP, 4096)
        turns.append({k: v for k, v in r.items() if k != "raw_content"})
        if r.get("error"):
            return dict(outcome="error", error=r["error"], turns=turns)
        calls = r["tool_calls"]
        if not calls:
            return dict(outcome="no_tool_after_schema" if schema_called else "no_tool_call", turns=turns,
                        text=r.get("content", ""), shown_groups=shown_groups, schema_errors=schema_errors)
        if is_claude:
            msgs.append({"role": "assistant", "content": r["raw_content"]})
        else:
            msgs.append({"role": "assistant", "content": r.get("content", ""),
                         "tool_calls": [{"function": {"name": c["name"], "arguments": c["arguments"]}} for c in calls]})
        results = []
        setcall = next((c for c in calls if c["name"] == "set_generation_params"), None)
        if setcall is not None and schema_called:
            v = C.validate(setcall["arguments"], rows, shown_groups)
            return dict(outcome="proposal", args=setcall["arguments"], verdict=v, turns=turns,
                        shown_groups=shown_groups, n_calls_in_final_turn=len(calls), schema_errors=schema_errors,
                        text=r.get("content", ""))
        for c in calls:
            if c["name"] == "get_param_schema":
                a = c["arguments"]
                if isinstance(a, str):
                    try: a = json.loads(a)
                    except Exception: a = {}
                gs = a.get("groups") if isinstance(a, dict) else None
                if not isinstance(gs, list) or not gs or len(gs) > 3 or any(g not in C.GROUPS for g in gs):
                    schema_errors += 1
                    res = {"error": "groups must be 1 to 3 of: " + ", ".join(C.GROUPS)}
                else:
                    schema_called = True; shown_groups = list(dict.fromkeys(gs))
                    ents = C.group_entries(rows, shown_groups)
                    res = {"groups": shown_groups, "settings": ents}
                    tools = [C.tool_get_schema(), C.tool_set_params({e["key"] for e in ents})]
            else:
                bad_tool += 1
                res = {"error": "unknown or not-yet-available tool %s; call get_param_schema first" % c["name"]}
            results.append((c, json.dumps(res, ensure_ascii=False)))
        if is_claude:
            msgs.append({"role": "user", "content": [{"type": "tool_result", "tool_use_id": c["id"], "content": res_} for c, res_ in results]})
        else:
            for c, res_ in results:
                msgs.append({"role": "tool", "tool_name": c["name"], "content": res_})
    return dict(outcome="max_turns", turns=turns, shown_groups=shown_groups, schema_errors=schema_errors)


def score_direction(pid, rows, verdict):
    exp = EXPECT.get(pid)
    if not exp:
        return None
    right = wrong = 0; hits = []
    for k, v in verdict["valid"]:
        if k in exp:
            cur = rows[k].get("current")
            if cur is None or isinstance(v, bool):
                continue
            d = (v > cur) - (v < cur)
            if d == exp[k]: right += 1; hits.append((k, "ok"))
            elif d != 0: wrong += 1; hits.append((k, "WRONG"))
    return dict(right=right, wrong=wrong, hits=hits)


# --------------------------------------------------------------------------- vision
def vision_cases(repo):
    ev = os.path.join(repo, "cartalith-native/docs/evidence")
    pick = json.load(open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "vision_images.json")))
    return [(p["id"], os.path.join(ev, p["file"]), p["q"]) for p in pick]


def main():
    global CLAUDE_CALL_CAP
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", required=True); ap.add_argument("--out", required=True)
    ap.add_argument("--models", nargs="+", required=True)
    ap.add_argument("--suites", nargs="+", default=["tools", "prose", "vision"])
    ap.add_argument("--reps", type=int, default=3)
    ap.add_argument("--cap", type=int, default=150, help="max total Claude API calls across all runs in results.jsonl")
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
    CLAUDE_CALL_CAP = a.cap
    rows = C.load(a.repo)
    resf = os.path.join(a.out, "results.jsonl")
    done, claude_calls, cin, cout = set(), 0, 0, 0
    if os.path.exists(resf):
        for ln in open(resf, encoding="utf8"):
            r = json.loads(ln); done.add((r["model"], r["suite"], r["pid"], r["rep"]))
            claude_calls += r.get("api_calls", 0); cin += r.get("in_tokens", 0); cout += r.get("out_tokens", 0)
    cstate = dict(calls=claude_calls, **{"in": cin, "out": cout})
    key = None
    if any(m.startswith("claude-") for m in a.models):
        key = read_cred(); SECRET[0] = key
    out = open(resf, "a", encoding="utf8")

    def emit(rec):
        out.write(scrub(json.dumps(rec, ensure_ascii=False)) + "\n"); out.flush()

    for m in a.models:
        is_claude = m.startswith("claude-")
        if not is_claude:  # warm-up: load the model once, untimed in the results
            w = ollama_chat(m, [{"role": "user", "content": "Reply with the single word: ready"}], None, 0.0, 32)
            print("warmup", m, scrub(w.get("error") or "ok"), flush=True)
            if w.get("error"):
                continue
        for suite in a.suites:
            if suite == "vision" and not (is_claude or "ministral" in m.lower()):
                continue
            if suite == "tools":
                cases = [(pid, p) for pid, p in TOOLS]
            elif suite == "prose":
                cases = PROSE
            else:
                cases = [(pid, (path, q)) for pid, path, q in vision_cases(a.repo)]
            for pid, p in cases:
                for rep in range(a.reps if suite != "vision" else 1):
                    if (m, suite, pid, rep) in done:
                        continue
                    c0 = (cstate["calls"], cstate["in"], cstate["out"])
                    rec = dict(model=m, suite=suite, pid=pid, rep=rep, ts=time.strftime("%Y-%m-%dT%H:%M:%S"))
                    if suite == "tools":
                        rec["prompt"] = p
                        rec.update(run_tool_case(m, p, rows, key, cstate))
                        if rec.get("verdict"):
                            rec["direction"] = score_direction(pid, rows, rec["verdict"])
                    elif suite == "prose":
                        rec["prompt"] = p
                        if is_claude:
                            r = claude_chat(m, "You are a helpful writing assistant.", [{"role": "user", "content": p}], None, PROSE_TEMP, key, cstate)
                        else:
                            r = ollama_chat(m, [{"role": "user", "content": p}], None, PROSE_TEMP, 2048)
                        rec["turns"] = [{k: v for k, v in r.items() if k != "raw_content"}]
                    else:
                        path, q = p
                        img = base64.b64encode(open(path, "rb").read()).decode()
                        imgs = [img]
                        rec["prompt"] = q; rec["image"] = os.path.relpath(path, a.repo).replace("\\", "/")
                        if is_claude:  # owner allowed synthetic evidence images to Claude (this experiment only)
                            blocks = [{"type": "image", "source": {"type": "base64", "media_type": "image/jpeg", "data": img}},
                                      {"type": "text", "text": q}]
                            r = claude_chat(m, "You describe synthetic fantasy-map renders.", [{"role": "user", "content": blocks}], None, VISION_TEMP, key, cstate)
                            r.pop("raw_content", None)
                        else:
                            r = ollama_chat(m, [{"role": "user", "content": q, "images": imgs}], None, VISION_TEMP, 1024)
                        rec["turns"] = [r]
                    rec["api_calls"] = cstate["calls"] - c0[0]; rec["in_tokens"] = cstate["in"] - c0[1]; rec["out_tokens"] = cstate["out"] - c0[2]
                    emit(rec)
                    print(m[-28:], suite, pid, rep, rec.get("outcome", ""), "calls", cstate["calls"], flush=True)
    print("claude calls so far", cstate["calls"], "in", cstate["in"], "out", cstate["out"])
    if cstate.get("temperature_dropped"):
        print("NOTE: temperature rejected by the Claude API and dropped")


if __name__ == "__main__":
    main()
