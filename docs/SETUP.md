# Setup

## GPU server (RTX 5090 32GB x1)

Template: any CUDA 12.8+ PyTorch image. Container disk >= 100 GB on **local**
disk (not the network volume: llama.cpp mmaps the GGUF).

```bash
# toolchains the coding tasks build with (missing ones report a FAKE 0/N)
apt-get update && apt-get install -y build-essential cmake git curl \
  tesseract-ocr tesseract-ocr-jpn tesseract-ocr-jpn-vert fonts-noto-cjk
# go, rust (cargo), scala-cli + JDK: install per their docs

# llama.cpp, mainline, b12611 or newer
git clone https://github.com/ggml-org/llama.cpp && cd llama.cpp
cmake -B build -DGGML_CUDA=ON
cmake --build build --config Release -j --target llama-server llama-cli llama-mtmd-cli llama-perplexity
export PATH=$PWD/build/bin:$PATH && cd ..

git clone <this repo> /root/inkling-2-mini-bench && cd /root/inkling-2-mini-bench
cp serving/env.example.sh serving/env.sh       # edit in place; gitignored
serving/00_preflight.sh
WANT_MMPROJ=1 serving/01_download_model.sh
WANT_MMPROJ=1 serving/02_serve_llamacpp.sh     # in tmux
serving/03_smoke.sh                            # M0, must pass before anything else
```

From the laptop, tunnel instead of exposing the port:
`ssh -N -L 8000:127.0.0.1:8000 <GPU server ssh target>` (the target goes in your shell
history, never in this repo).

### OCR data

Fonts must be OFL (Noto). On the server, `fonts-noto-cjk` provides them:

```bash
python3 -I ocr/gen_synthetic.py \
  --font /usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc \
  --font-serif /usr/share/fonts/opentype/noto/NotoSerifCJK-Regular.ttc \
  --out ocr/data/synth --n 40
for c in vlm tesseract hybrid; do
  python3 -I ocr/run_ocr.py --in ocr/data/synth --cond $c --task transcribe \
    --out results/ocr/$c-transcribe.jsonl
  python3 -I ocr/score.py --gold ocr/data/synth --pred results/ocr/$c-transcribe.jsonl
done
```

`ocr/data/` is gitignored. The generator is deterministic (`--seed`), so the
seed in the report is enough to reproduce the set.

Known limitation: vertical text is drawn glyph by glyph without vertical
alternates, so `ー` and `。` keep their horizontal shapes. Ground truth is
unaffected, but it is easier than real tategaki print.

## MacBook (Apple M2 Pro, 32 GB)

```bash
brew install cmake tesseract tesseract-lang
git clone https://github.com/ggml-org/llama.cpp && cd llama.cpp
cmake -B build            # Metal is the default on macOS
cmake --build build --config Release -j --target llama-server llama-cli llama-perplexity
```

Then the same `serving/` scripts with `MODEL_DIR=$HOME/models` etc. in `env.sh`.

Memory budget: macOS lets the GPU wire only part of unified memory by default
(roughly two thirds on a 32 GB machine). UD-Q4_K_M (16.6 GB) + mmproj (0.6 GB) +
KV must fit inside that, so start with a small `CTX` (16384) and raise it. If it
does not fit, try `UD-IQ3_XXS` (11.7 GB). Raising `iogpu.wired_limit_mb` is a
separate, labelled run: record the value.

Record for every D-run: chip, memory, macOS version, llama.cpp build, quant,
CTX, plugged in or on battery.
