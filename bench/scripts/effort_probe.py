#!/usr/bin/env python3
"""M0-2 / A5 quick probe: does reasoning_effort move reasoning length and time?

One prompt, several effort levels, n runs each. Prints one row per run.
    python3 bench/scripts/effort_probe.py --efforts 0.0,0.2,0.7,0.9,0.99 --n 2
"""
import argparse
import json
import time
import urllib.request

PROMPT = ("Write a Rust function that parses an ISO-8601 duration string like P3DT4H12M "
          "into total seconds without external crates. Handle errors with a custom enum. "
          "Include unit tests.")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--base-url", default="http://127.0.0.1:8000")
    ap.add_argument("--model", default="inkling-2-mini")
    ap.add_argument("--efforts", default="0.0,0.2,0.7,0.9,0.99")
    ap.add_argument("--n", type=int, default=1)
    ap.add_argument("--max-tokens", type=int, default=20000)
    ap.add_argument("--prompt", default=PROMPT)
    a = ap.parse_args()
    for eff in [float(x) for x in a.efforts.split(",")]:
        for i in range(a.n):
            body = {"model": a.model, "messages": [{"role": "user", "content": a.prompt}],
                    "max_tokens": a.max_tokens, "temperature": 0.6, "top_p": 0.95, "top_k": 20,
                    "chat_template_kwargs": {"reasoning_effort": eff}}
            req = urllib.request.Request(a.base_url + "/v1/chat/completions", json.dumps(body).encode(),
                                         {"Content-Type": "application/json"})
            t = time.time()
            d = json.load(urllib.request.urlopen(req, timeout=1800))
            ch = d["choices"][0]
            m = ch["message"]
            r = len(m.get("reasoning_content") or "")
            c = len(m.get("content") or "")
            tok = d["usage"]["completion_tokens"]
            tps = d.get("timings", {}).get("predicted_per_second", 0)
            print(f"effort={eff:<5} run={i} reasoning_chars={r:>6} content_chars={c:>6} "
                  f"completion_tok={tok:>6} wall={time.time() - t:6.1f}s decode={tps:.1f}tok/s "
                  f"finish={ch.get('finish_reason')}", flush=True)


if __name__ == "__main__":
    main()
