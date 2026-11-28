#!/usr/bin/env bash
# Serve the NATIVE checkpoint (NVFP4, last 4 layers MXFP8) with vLLM.
# Blackwell only. This is the vendor's intended numerics; the GGUF path is the
# comparable one. The GGUF-vs-native gap is itself a measurement (E1).
#
# vLLM support for the looped architecture is UNVERIFIED as of writing. If the
# model class is not registered, record the vLLM version and stop: do not patch
# around it inside a benchmark run.
set -uo pipefail
cd "$(dirname "$0")"
[ -f env.sh ] && . ./env.sh || . ./env.example.sh

CKPT="$MODEL_DIR/Inkling-2-Mini"
[ -d "$CKPT" ] || { echo "missing $CKPT — NATIVE=1 ./01_download_model.sh"; exit 1; }

ARGS=(
  serve "$CKPT"
  --served-model-name "$SERVED_NAME"
  --host 127.0.0.1 --port "$VLLM_PORT"
  --max-model-len "$CTX"
  --gpu-memory-utilization 0.92
  --enable-auto-tool-choice
  --trust-remote-code
)
echo "== vllm =="
python3 -c 'import vllm; print("  vllm", vllm.__version__)' || { echo "vllm not installed"; exit 1; }
printf '  vllm\n'; printf '    %s\n' "${ARGS[@]}"
echo
echo "NOTE: tool-call parser and reasoning parser names for this model are not"
echo "known yet. Find them in the vLLM release notes and add them here."
exec vllm "${ARGS[@]}"
