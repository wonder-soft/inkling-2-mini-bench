# inkling-2-mini-bench

**English** | [日本語](README.ja.md)

Is **Inkling 2 Mini** usable as a local OpenCode backend, can it do Japanese
OCR, and does it run on a 32 GB MacBook?

> **Status: first pass measured on RTX 5090, 2026-11-23.** Report (Japanese):
> [`docs/reports/2026-11-23-first-pass-rtx5090.ja.md`](docs/reports/2026-11-23-first-pass-rtx5090.ja.md).
> Headline: Rust pass@1 **5/5** at `reasoning_effort` 0.2 (Qwen3.8-27B 0/5,
> DeepSeek-V4-Flash 2/5); effort 0.9+ runs away and exhausts the token budget;
> clean synthetic invoices OCR'd with zero errors; 94 tok/s decode; tool
> selection 60%. MacBook and OpenCode runs are still open.

## Target

| | |
|---|---|
| Model | `thinkingmachines/Inkling-2-Mini` (released 2026-11-20, Apache 2.0) |
| Architecture | Looped MoE: 26.4B total / 3.4B active, 8 of 128 experts per token; middle 2 of 4 blocks run 3x with shared weights (5.4B effective active). Vision via ViT. Text + image in, text out. Up to 1M ctx (vendor claim) |
| Weights (GGUF) | `unsloth/Inkling-2-Mini-GGUF`, **`Mainline_Support/`** files, `UD-Q4_K_M` 16.61 GB + `mmproj-F16` 0.64 GB |
| Weights (native) | NVFP4 (last 4 layers MXFP8), 19.53 GB + 0.64 GB MTP head |
| Hardware | **RTX 5090 32GB x1** (primary, matches the comparison table), RTX PRO 6000 96GB for long-context probes, **Apple M2 Pro 32GB** for the laptop axis |
| Server | llama.cpp **b12611 or newer** (older builds load the model and silently produce garbage) |
| Client | OpenCode via OpenAI-compatible provider |

## Four questions

| | Question | Where |
|---|---|---|
| A | Coding, **Rust in particular**: does it beat Qwen3.8-27B (0/5 on Rust) on the same unmodified tasks? | `bench/` |
| B | **OCR / vision** on Japanese documents, alone and combined with Tesseract | `ocr/` |
| C | Is it pleasant and safe for **daily OpenCode use**? | `opencode/` |
| D | Does it **run on a 32 GB MacBook**, and how much does it lose there? | `docs/SETUP.md` |

`bench/tasks/` is carried over **unmodified** from
[letusfly85/coding-agent-bench](https://github.com/letusfly85/coding-agent-bench)
via [qwen3.8-27b-bench](https://github.com/wonder-soft/qwen3.8-27b-bench), so
A-series numbers sit directly beside Qwen3.8-27B, Qwen3.6-27B and
DeepSeek-V4-Flash. Only the scripts' default model name changed.

## Things specific to this model

- **Thinking is a dial, not a switch.** The chat template takes
  `reasoning_effort` in [0.0, 0.99] (default 0.9), per request via
  `chat_template_kwargs`. The card says the minimum does not fully disable
  reasoning and that the model can loop while debugging. Both are measured.
- **Two GGUF layouts.** Root files need Unsloth's runtime. Mainline llama.cpp
  needs `Mainline_Support/*.gguf_file`, renamed to `.gguf`. `01_download_model.sh` does this.
- **Sampling.** The card recommends temp 1.0 with no truncation. The
  comparison table used cab's coding preset (0.6 / 0.95 / 20). Default here is
  cab's preset; `MODEL_CARD_SAMPLING=1` is a separate, labelled run.
- **Known risk from the card:** it may take destructive actions without
  confirmation and misreport outcomes. C-series runs happen in a disposable
  sandbox with OpenCode's permission prompts on.

## Layout

```
serving/   00_preflight  01_download_model  02_serve_llamacpp  03_smoke  04_serve_vllm
bench/     tasks/ (unmodified cab set), scripts/
ocr/       gen_synthetic.py  run_ocr.py  score.py  fixtures/
opencode/  opencode.json.example
docs/      PLAN.ja.md  SETUP.md  reports/
```

## Harness notes

- `bench/scripts/run_coding_task.py` sends `REASONING_EFFORT` (env) per request
  as `chat_template_kwargs`. This is the only behavioural change to the scripts.
- `bench/scripts/thinking_sweep.sh` is inherited and still assumes the Qwen-style
  on/off server flags. For A5 use `REASONING_EFFORT=<x> bench/scripts/run_repeat.sh ...`
  per level instead, until it is ported.

## License

The code and documents in this repository are under the [Apache License 2.0](LICENSE). Model weights are under their own licenses (Inkling 2 Mini: Apache 2.0) and are not redistributed here.
