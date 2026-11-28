#!/usr/bin/env bash
# Copy to serving/env.sh (gitignored) and adjust. Sourced by every script here.
#
# Host-specific values (hostnames, ports, volume names, tokens) go in the
# gitignored env.sh only.
#
# CHANGING A VALUE: edit the assignment in place. Do not append overrides at the
# end of env.sh. `export X=...` at the end silently beats the sweep scripts, and
# `export X=${X:-...}` at the end is a no-op. Both were hit on earlier runs.

# --- where things live -------------------------------------------------------
export HF_HOME=${HF_HOME:-/root/.cache/huggingface}
export MODEL_DIR=${MODEL_DIR:-/root/models}
export BENCH_REPO=${BENCH_REPO:-/root/inkling-2-mini-bench}
export BENCH_OUT=${BENCH_OUT:-/root/results}

# --- model -------------------------------------------------------------------
# Use the files under Mainline_Support/. The root GGUFs in unsloth's repo use
# Unsloth's own looped layout and only load in Unsloth Studio / Unsloth's
# llama.cpp prebuilts. Mainline_Support/ holds the same tensors in the
# `inkling2_loop` layout that mainline llama.cpp reads. They are published with
# a `.gguf_file` extension on purpose; 01_download_model.sh renames them.
#
# UD-Q4_K_M is the PRIMARY quant because it is the closest match to the Q4_K_M
# used for Qwen3.8-27B / Qwen3.6-27B in the comparison table. Same quant class,
# same task files, same card means the model is the only thing moving.
#
#   Inkling-2-Mini-UD-Q4_K_M.gguf     16.61 GB   <- baseline-matching
#   Inkling-2-Mini-UD-Q4_K_XL.gguf    17.63 GB   quality at roughly the same size
#   Inkling-2-Mini-UD-Q8_K_XL.gguf    22.87 GB   quality-ceiling probe (32 GB card)
#   Inkling-2-Mini-UD-IQ3_XXS.gguf    11.66 GB   MacBook headroom probe
#
# The native checkpoint (thinkingmachines/Inkling-2-Mini, NVFP4 + last 4 layers
# MXFP8, 19.53 GB + 0.64 GB MTP head) is served by vLLM instead: 04_serve_vllm.sh.
export GGUF_REPO=${GGUF_REPO:-unsloth/Inkling-2-Mini-GGUF}
export GGUF_SUBDIR=${GGUF_SUBDIR:-Mainline_Support}
export GGUF_FILE=${GGUF_FILE:-Inkling-2-Mini-UD-Q4_K_M.gguf}
export SERVED_NAME=${SERVED_NAME:-inkling-2-mini}

# Vision projector. Needed for every OCR run (ocr/), not for the coding tasks.
# mmproj files live at the repo root, not under Mainline_Support/.
export MMPROJ_FILE=${MMPROJ_FILE:-mmproj-F16.gguf}
export WANT_MMPROJ=${WANT_MMPROJ:-0}

# --- serving -----------------------------------------------------------------
export PORT=${PORT:-8000}
export NGL=${NGL:-99}
# The model card claims up to 1M tokens. The real ceiling on one 32 GB card is
# UNMEASURED; finding it is M1. Do not quote this default as a result.
export CTX=${CTX:-131072}
export KV_TYPE_K=${KV_TYPE_K:-q8_0}
export KV_TYPE_V=${KV_TYPE_V:-q8_0}
export FLASH_ATTN=${FLASH_ATTN:-on}
export N_PARALLEL=${N_PARALLEL:-1}

# --- thinking ----------------------------------------------------------------
# Inkling does NOT use an on/off switch. Its chat template takes
# `reasoning_effort`, a number in [0.0, 0.99] or one of
#   none=0.0 minimal=0.1 low=0.2 medium=0.7 high=0.9 xhigh=0.99 max=0.99
# and defaults to 0.9 when unset. It is passed per request through
# chat_template_kwargs, so no server restart is needed to sweep it.
#
# The model card warns that the minimum does not fully disable reasoning, and
# that the model can get stuck in long reasoning loops while debugging. Both are
# measured (A5): count the runs that exhaust MAX_TOKENS inside the thinking
# block. That was exactly how Qwen3.8-27B (think) scored 0/5 on Rust.
export REASONING_EFFORT=${REASONING_EFFORT:-0.9}
# Thinking is emitted as <|content_thinking|>. Whether llama.cpp routes it into
# message.reasoning_content for this template is UNVERIFIED; 03_smoke.sh checks.
export REASONING_FORMAT=${REASONING_FORMAT:-deepseek}

# --- sampling (keep identical across every run being compared) ---------------
# Model card / unsloth: temperature 1.0, no truncation (top_p 1.0, top_k off,
# min_p 0). The comparison table was taken with cab's coding preset
# (temp 0.6 / top_p 0.95 / top_k 20). Default here is cab's preset, for
# comparability. MODEL_CARD_SAMPLING=1 switches to the card values for a
# separate, labelled run. Never mix the two in one table.
if [ "${MODEL_CARD_SAMPLING:-0}" = "1" ]; then
  export TEMP=1.0 TOP_P=1.0 TOP_K=0 MIN_P=0.0
else
  export TEMP=${TEMP:-0.6}
  export TOP_P=${TOP_P:-0.95}
  export TOP_K=${TOP_K:-20}
  export MIN_P=${MIN_P:-0.0}
fi
export MAX_TOKENS=${MAX_TOKENS:-24000}

# --- binary ------------------------------------------------------------------
# Mainline llama.cpp must be b12611 or newer (ggml-org/llama.cpp#32704). Older
# builds drop the per-expert NVFP4 scales on the 2nd and 3rd passes through the
# looped layers: it loads, it answers, and perplexity is ~130 instead of ~7.65.
# That is a silent failure that reads as "the model is bad". 00_preflight.sh
# refuses older builds.
export LLAMA_MIN_BUILD=${LLAMA_MIN_BUILD:-12611}
export LLAMA_SERVER=${LLAMA_SERVER:-llama-server}
export LLAMA_CLI=${LLAMA_CLI:-llama-cli}
export LLAMA_PPL=${LLAMA_PPL:-llama-perplexity}

# --- vLLM (native NVFP4 checkpoint) ------------------------------------------
export HF_REPO_NATIVE=${HF_REPO_NATIVE:-thinkingmachines/Inkling-2-Mini}
export VLLM_PORT=${VLLM_PORT:-8001}
