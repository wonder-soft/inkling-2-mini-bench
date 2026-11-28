#!/usr/bin/env bash
# Prove the endpoint works before blaming the model for anything the harness did.
# An earlier run attributed garbled output to a quant when the real cause was a
# parser that could not read the model's output format. Hit the raw API first.
set -uo pipefail
cd "$(dirname "$0")"
[ -f env.sh ] && . ./env.sh || . ./env.example.sh
BASE=${BASE:-http://127.0.0.1:$PORT}
SMOKE_IMG=${SMOKE_IMG:-../ocr/fixtures/smoke.png}

post() { curl -s "$BASE/v1/chat/completions" -H 'Content-Type: application/json' -d @-; }

echo "== /v1/models =="
curl -s "$BASE/v1/models" | python3 -m json.tool || exit 1

echo
echo "== reasoning_effort actually changes behaviour =="
# Inkling's thinking is a continuous knob, not on/off. If the template kwarg
# does not reach the template, every effort level gives the same reasoning
# length and an effort sweep silently measures nothing.
for EFF in 0.0 0.2 0.9; do
  python3 - "$SERVED_NAME" "$EFF" <<'PY' | post | python3 -c '
import json,sys
d=json.load(sys.stdin); m=d["choices"][0]["message"]
r=m.get("reasoning_content") or ""; c=m.get("content") or ""
leak = "<|content_thinking|>" in c
print(f"  effort={sys.argv[1]:<4} reasoning_chars={len(r):>6}  content={c[:60]!r}" + ("   <-- THINKING LEAKED INTO content" if leak else ""))
' "$EFF"
import json,sys
print(json.dumps({"model": sys.argv[1],
  "messages":[{"role":"user","content":"A bat and a ball cost 1.10 in total. The bat costs 1.00 more than the ball. How much is the ball?"}],
  "max_tokens": 4096, "temperature": 0.6,
  "chat_template_kwargs": {"reasoning_effort": float(sys.argv[2])}}))
PY
done
echo "  Expect reasoning_chars to grow with effort. Note the card says 0.0 does"
echo "  not fully disable reasoning, so 0.0 > 0 chars is not by itself a failure."
echo "  If reasoning_chars is 0 everywhere and content holds the thoughts, the"
echo "  reasoning parser is not wired for this template: fix before any A-run."

echo
echo "== tool call round-trip =="
post <<JSON | python3 -c '
import json,sys
d=json.load(sys.stdin)
tc=d["choices"][0]["message"].get("tool_calls") or []
print(json.dumps(tc, indent=2)[:600] if tc else "  NONE  <-- tool parser not wired (missing --jinja?)")'
{"model":"$SERVED_NAME",
 "messages":[{"role":"user","content":"What is the weather in Tokyo? Use the tool."}],
 "tools":[{"type":"function","function":{"name":"get_weather","description":"Get weather for a city",
   "parameters":{"type":"object","properties":{"city":{"type":"string"}},"required":["city"]}}}],
 "max_tokens":2048,"temperature":0}
JSON

echo
echo "== vision round-trip (needs WANT_MMPROJ=1) =="
if [ -f "$SMOKE_IMG" ]; then
  python3 - "$SERVED_NAME" "$SMOKE_IMG" <<'PY' | post | python3 -c '
import json,sys
d=json.load(sys.stdin)
if "error" in d: print("  ERROR:", d["error"]); sys.exit()
c=d["choices"][0]["message"].get("content") or ""
print("  content:", repr(c[:200]))
print("  PASS" if "4729" in c else "  FAIL: expected the number 4729 from the image")'
import base64,json,sys
b=base64.b64encode(open(sys.argv[2],"rb").read()).decode()
print(json.dumps({"model":sys.argv[1],"max_tokens":2048,"temperature":0,
  "chat_template_kwargs":{"reasoning_effort":0.2},
  "messages":[{"role":"user","content":[
    {"type":"image_url","image_url":{"url":"data:image/png;base64,"+b}},
    {"type":"text","text":"Transcribe all text in this image exactly."}]}]}))
PY
else
  echo "  skipped: $SMOKE_IMG not found"
fi
