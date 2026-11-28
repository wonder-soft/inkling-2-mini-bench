#!/usr/bin/env bash
# Fetch one mainline-layout GGUF (and optionally the vision projector).
#   NATIVE=1 ./01_download_model.sh   also fetches the NVFP4 checkpoint for vLLM
set -euo pipefail
cd "$(dirname "$0")"
[ -f env.sh ] && . ./env.sh || . ./env.example.sh

if ! command -v hf >/dev/null; then
  pip install -q --break-system-packages "huggingface_hub[cli,hf_transfer]" \
    || pip install -q "huggingface_hub[cli,hf_transfer]"
fi
export HF_HUB_ENABLE_HF_TRANSFER=${HF_HUB_ENABLE_HF_TRANSFER:-1}
mkdir -p "$MODEL_DIR"

# Mainline_Support/ files are published as *.gguf_file so that tools listing the
# repo do not mix the two layouts. Download under that name, then rename.
REMOTE="$GGUF_SUBDIR/${GGUF_FILE}_file"
echo "== $GGUF_REPO :: $REMOTE =="
hf download "$GGUF_REPO" "$REMOTE" --local-dir "$MODEL_DIR"
mv -f "$MODEL_DIR/$REMOTE" "$MODEL_DIR/$GGUF_FILE"

if [ "${WANT_MMPROJ:-0}" = "1" ]; then
  echo "== $GGUF_REPO :: $MMPROJ_FILE (vision) =="
  hf download "$GGUF_REPO" "$MMPROJ_FILE" --local-dir "$MODEL_DIR"
fi

if [ "${NATIVE:-0}" = "1" ]; then
  echo "== $HF_REPO_NATIVE (native NVFP4, ~20 GB) =="
  hf download "$HF_REPO_NATIVE" --local-dir "$MODEL_DIR/Inkling-2-Mini"
fi

echo
ls -lh "$MODEL_DIR"/*.gguf 2>/dev/null | sed 's/^/  /'
echo
echo "Next: ./02_serve_llamacpp.sh   (WANT_MMPROJ=1 for OCR runs)"
