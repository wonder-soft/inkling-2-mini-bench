#!/usr/bin/env bash
# Start llama-server. Prints the exact invocation first: copy it into the
# report, because a number without its serving flags is not reproducible.
set -uo pipefail
cd "$(dirname "$0")"
[ -f env.sh ] && . ./env.sh || . ./env.example.sh

GGUF_PATH="$MODEL_DIR/$GGUF_FILE"
[ -f "$GGUF_PATH" ] || { echo "missing $GGUF_PATH — run ./01_download_model.sh"; exit 1; }

ARGS=(
  --model "$GGUF_PATH"
  --alias "$SERVED_NAME"
  --host 127.0.0.1 --port "$PORT"
  --n-gpu-layers "$NGL"
  --ctx-size "$CTX"
  --parallel "$N_PARALLEL"
  --flash-attn "$FLASH_ATTN"
  --cache-type-k "$KV_TYPE_K"
  --cache-type-v "$KV_TYPE_V"
  # Mandatory: without --jinja the server uses a generic template, tool_calls
  # come back empty and OpenCode looks like the model refuses to call tools.
  --jinja
  --reasoning-format "$REASONING_FORMAT"
  # Server-wide default only. Bench scripts override it per request.
  --chat-template-kwargs "{\"reasoning_effort\": $REASONING_EFFORT}"
)
# Bound to 127.0.0.1 on purpose. On a remote GPU server, reach it through an SSH tunnel
# rather than exposing an unauthenticated port.

if [ "${WANT_MMPROJ:-0}" = "1" ]; then
  [ -f "$MODEL_DIR/$MMPROJ_FILE" ] || { echo "missing $MMPROJ_FILE — WANT_MMPROJ=1 ./01_download_model.sh"; exit 1; }
  ARGS+=(--mmproj "$MODEL_DIR/$MMPROJ_FILE")
fi

echo "== serving =="
printf '  %s\n' "$LLAMA_SERVER"; printf '    %s\n' "${ARGS[@]}"
echo
echo "  quant     : $GGUF_FILE"
echo "  context   : $CTX (kv ${KV_TYPE_K}/${KV_TYPE_V})"
echo "  effort    : $REASONING_EFFORT (default; per-request overrides win)"
echo "  vision    : ${WANT_MMPROJ:-0}"
echo
exec "$LLAMA_SERVER" "${ARGS[@]}"
