#!/usr/bin/env bash
# Verify the box can hold this model before spending the download.
# Works on a Linux GPU server and on Apple silicon (for the D-series MacBook runs).
set -uo pipefail
cd "$(dirname "$0")"
[ -f env.sh ] && . ./env.sh || . ./env.example.sh

FAIL=0
ok()   { printf '  \033[32mOK\033[0m   %s\n' "$1"; }
bad()  { printf '  \033[31mFAIL\033[0m %s\n' "$1"; FAIL=1; }
warn() { printf '  \033[33mWARN\033[0m %s\n' "$1"; }

if [ "$(uname -s)" = "Darwin" ]; then
  echo "== Apple silicon =="
  CHIP=$(sysctl -n machdep.cpu.brand_string)
  MEM_GIB=$(( $(sysctl -n hw.memsize) / 1024 / 1024 / 1024 ))
  echo "  $CHIP, ${MEM_GIB} GiB unified memory"
  # The model card's floor is 32 GB. macOS reserves part of unified memory for
  # itself; by default the GPU may wire only ~2/3-3/4 of it.
  [ "$MEM_GIB" -ge 32 ] && ok "meets the 32 GB floor" || bad "below the model card's 32 GB floor"
  WIRED=$(sysctl -n iogpu.wired_limit_mb 2>/dev/null || echo 0)
  [ "$WIRED" = "0" ] && warn "iogpu.wired_limit_mb is default. Record it; raising it is a separate, labelled run." \
                     || ok "iogpu.wired_limit_mb=$WIRED"
else
  echo "== GPU =="
  if ! command -v nvidia-smi >/dev/null; then
    bad "nvidia-smi not found"
  else
    nvidia-smi --query-gpu=index,name,memory.total,compute_cap --format=csv,noheader | sed 's/^/  /'
    VRAM_MIB=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits | head -1)
    VRAM_GIB=$((VRAM_MIB / 1024))
    [ "$VRAM_GIB" -ge 30 ] && ok "VRAM ${VRAM_GIB} GiB on GPU0" \
                           || bad "VRAM ${VRAM_GIB} GiB — UD-Q4_K_M is 16.6 GB before KV and mmproj"
    # NVFP4 tensor cores are Blackwell (compute capability 10.x / 12.x). On older
    # cards the native checkpoint either will not load in vLLM or is emulated.
    CC=$(nvidia-smi --query-gpu=compute_cap --format=csv,noheader | head -1 | cut -d. -f1)
    [ "${CC:-0}" -ge 10 ] && ok "Blackwell-class GPU (cc ${CC}.x) — native NVFP4 possible" \
                          || warn "cc ${CC}.x is pre-Blackwell — skip 04_serve_vllm.sh, GGUF only"
    N=$(nvidia-smi --list-gpus | wc -l | tr -d ' ')
    [ "$N" -eq 1 ] || warn "$N GPUs visible. Baseline is ONE card — CUDA_VISIBLE_DEVICES=0."
  fi
fi

echo "== llama.cpp =="
if ! command -v "$LLAMA_SERVER" >/dev/null; then
  bad "$LLAMA_SERVER not on PATH"
else
  VER=$("$LLAMA_SERVER" --version 2>&1 | grep -m1 -i 'version')
  echo "  $VER"
  # Two formats seen in the wild: "version: 12611 (abc)" and
  # "version: 1.0.0-dev (build 12652, commit abc)". Prefer the explicit build.
  BUILD=$(sed -n 's/.*(build \([0-9][0-9]*\).*/\1/p' <<<"$VER")
  [ -n "$BUILD" ] || BUILD=$(sed -n 's/.*version: \([0-9][0-9]*\) .*/\1/p' <<<"$VER")
  if [ -z "$BUILD" ]; then
    warn "could not parse build number; confirm by hand it is >= b$LLAMA_MIN_BUILD"
  elif [ "$BUILD" -ge "$LLAMA_MIN_BUILD" ]; then
    ok "build b$BUILD >= b$LLAMA_MIN_BUILD"
  else
    bad "build b$BUILD < b$LLAMA_MIN_BUILD: looped layers lose NVFP4 scales, output is garbage (ppl ~130)"
  fi
  H=$("$LLAMA_SERVER" --help 2>&1)
  for FLAG in --jinja --mmproj --reasoning-format --chat-template-kwargs; do
    grep -q -- "$FLAG" <<<"$H" && ok "supports $FLAG" || bad "no $FLAG — rebuild from master"
  done
fi

echo "== disk =="
mkdir -p "$MODEL_DIR" 2>/dev/null || true
AVAIL_GIB=$(df -Pk "$MODEL_DIR" 2>/dev/null | tail -1 | awk '{print int($4/1024/1024)}')
echo "  MODEL_DIR=$MODEL_DIR (${AVAIL_GIB} GiB free)"
# 16.6 GB GGUF + 0.6 GB mmproj + 20 GB native checkpoint + artifacts.
[ "${AVAIL_GIB:-0}" -ge 80 ] && ok "${AVAIL_GIB} GiB free, need >=80" \
                             || warn "${AVAIL_GIB} GiB free — 80 GiB covers GGUF + native checkpoint; GGUF alone needs ~30"

echo "== toolchains (the bench builds real projects) =="
# A missing toolchain does not error out; every sample reports build=fail and
# reads as a model result. That produced a fake 0/5 on an earlier run.
for C in python3 go cargo scala-cli java; do
  command -v "$C" >/dev/null && ok "$C" || bad "$C missing — its task will report a FAKE 0/N"
done

echo "== OCR =="
if command -v tesseract >/dev/null; then
  L=$(tesseract --list-langs 2>/dev/null)
  for LANG in eng jpn jpn_vert; do
    grep -qx "$LANG" <<<"$L" && ok "tesseract $LANG" || bad "tesseract $LANG missing — B4 cannot run (see docs/SETUP.md)"
  done
else
  warn "tesseract not installed — only needed for the OCR (B4) runs"
fi

echo
[ "$FAIL" -eq 0 ] && echo "PREFLIGHT: pass" || { echo "PREFLIGHT: FAIL — fix the above before downloading"; exit 1; }
