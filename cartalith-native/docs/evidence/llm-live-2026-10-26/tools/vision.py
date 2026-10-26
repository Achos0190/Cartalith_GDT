"""Optional vision check for LLM-6c stage 2: ask a local vision model, through LM Studio's
OpenAI-compatible API (NOT through Cartalith -- Ruling DI P5 still bars map images in the app),
to describe a before and an after render, and what changed between them.

Usage: python -I vision.py <model id> <before.jpg> <after.jpg> <out.json> [base url]

Synthetic Cartalith renders only. Writes the prompts, the answers, timings and the server's
usage figures. Never sends anything but the two images and the fixed questions below.
"""
import base64
import json
import sys
import time
import urllib.request

QUESTIONS = [
    ("before", "This is a generated fantasy map. Describe the land and sea in three sentences: how much is sea, where the land is, any mountains or volcanoes.", ["before"]),
    ("after", "This is a generated fantasy map. Describe the land and sea in three sentences: how much is sea, where the land is, any mountains or volcanoes.", ["after"]),
    ("diff", "These are two renders of the same generated fantasy map, before and after a settings change. In two or three sentences, say what changed between the first and the second image.", ["before", "after"]),
]


def b64(path):
    with open(path, "rb") as f:
        return base64.b64encode(f.read()).decode("ascii")


def main():
    model, before, after, out = sys.argv[1:5]
    base = sys.argv[5] if len(sys.argv) > 5 else "http://127.0.0.1:1234/v1"
    imgs = {"before": b64(before), "after": b64(after)}
    res = {"model": model, "before": before, "after": after, "answers": []}
    for tag, q, which in QUESTIONS:
        content = [{"type": "text", "text": q}] + [
            {"type": "image_url", "image_url": {"url": "data:image/jpeg;base64," + imgs[w]}} for w in which]
        body = {"model": model, "messages": [{"role": "user", "content": content}], "max_tokens": 400,
                "temperature": 0.2, "stream": False, "reasoning_effort": "none"}
        req = urllib.request.Request(base + "/chat/completions", data=json.dumps(body).encode(),
                                     headers={"Content-Type": "application/json"})
        t0 = time.time()
        with urllib.request.urlopen(req, timeout=600) as r:
            j = json.loads(r.read())
        res["answers"].append({"q": tag, "question": q, "seconds": round(time.time() - t0, 1),
                               "text": j["choices"][0]["message"].get("content", ""), "usage": j.get("usage", {})})
    with open(out, "w", encoding="utf-8") as f:
        json.dump(res, f, indent=1, ensure_ascii=True)
    print("wrote", out)


if __name__ == "__main__":
    main()
