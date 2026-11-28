# inkling-2-mini-bench

[English](README.md) | **日本語**

**Inkling 2 Mini** をローカルの OpenCode のバックエンドとして使えるか、日本語の OCR ができるか、32GB の MacBook で動くかを確かめるベンチ。

> **状況：RTX 5090 で第1回の計測が完了（2026-11-23）。** レポートは [`docs/reports/2026-11-23-first-pass-rtx5090.ja.md`](docs/reports/2026-11-23-first-pass-rtx5090.ja.md)。
> 要点：`reasoning_effort` 0.2 で Rust の pass@1 が **5/5**（Qwen3.8-27B は 0/5、DeepSeek-V4-Flash は 2/5）。effort 0.9 以上では推論が止まらず、上限トークンを使い切る。合成した請求書の OCR は誤りゼロ。デコード 94 tok/s。tool の選択精度は 60%。MacBook と OpenCode での計測は未実施。

## 対象

| | |
|---|---|
| モデル | `thinkingmachines/Inkling-2-Mini`（2026-11-20 公開、Apache 2.0） |
| 構造 | ループ型 MoE。全体 26.4B、1トークンあたり 3.4B が動く（128 の expert から 8 を選ぶ）。4ブロック中の中間2ブロックを重みを共有したまま3回まわす（実効 5.4B）。vision は ViT。入力はテキストと画像、出力はテキスト。最大 1M ctx（開発元の主張） |
| 重み（GGUF） | `unsloth/Inkling-2-Mini-GGUF` の **`Mainline_Support/`** 配下。`UD-Q4_K_M` 16.61GB と `mmproj-F16` 0.64GB |
| 重み（ネイティブ） | NVFP4（最後の4層は MXFP8）、19.53GB と MTP ヘッド 0.64GB |
| ハード | **RTX 5090 32GB ×1**（本計測。比較表と同じ条件）、長いコンテキストを試すときは RTX PRO 6000 96GB、ノート PC の軸は **Apple M2 Pro 32GB** |
| サーバー | llama.cpp **b12611 以降**（それより古いビルドでも読み込めてしまい、出力だけが黙って壊れる） |
| クライアント | OpenCode（OpenAI 互換のプロバイダとして接続） |

## 4つの問い

| | 問い | 場所 |
|---|---|---|
| A | コーディング、**特に Rust**。変更していない同じタスクで、Qwen3.8-27B（Rust は 0/5）を上回るか | `bench/` |
| B | 日本語文書の **OCR（vision）**。単体と、Tesseract と組み合わせた場合 | `ocr/` |
| C | **OpenCode の日常使い**で快適かつ安全に使えるか | `opencode/` |
| D | **32GB の MacBook で動くか**。動くならどれだけ落ちるか | `docs/SETUP.md` |

`bench/tasks/` は [letusfly85/coding-agent-bench](https://github.com/letusfly85/coding-agent-bench) から [qwen3.8-27b-bench](https://github.com/wonder-soft/qwen3.8-27b-bench) を経由して、**変更せずに**持ってきている。そのため A 系の数値は Qwen3.8-27B、Qwen3.6-27B、DeepSeek-V4-Flash の数値とそのまま並べられる。変えたのはスクリプトの既定のモデル名だけ。

## このモデルに固有の注意点

- **thinking はオン・オフではなく連続値。** チャットテンプレートが `reasoning_effort` を 0.0〜0.99（既定 0.9）で受け取り、リクエストごとに `chat_template_kwargs` で渡す。モデルカードには「最小にしても推論が完全には消えない」「デバッグ中に推論がループすることがある」とある。どちらも計測する。
- **GGUF の配置が2種類ある。** ルート直下のファイルは Unsloth のランタイムが必要。本家の llama.cpp で使うのは `Mainline_Support/*.gguf_file` で、拡張子を `.gguf` に直してから使う。`01_download_model.sh` がこの処理をする。
- **サンプリング。** モデルカードの推奨は temp 1.0 で切り詰めなし。比較表は cab のコーディング用設定（0.6 / 0.95 / 20）で取っている。ここでの既定は cab の設定で、`MODEL_CARD_SAMPLING=1` は別の計測として区別して記録する。
- **モデルカードが認めているリスク：** 確認なしで破壊的な操作をし、結果を誤って報告することがある。C 系は使い捨てのサンドボックスで、OpenCode の権限確認をオンにして動かす。

## 構成

```
serving/   00_preflight  01_download_model  02_serve_llamacpp  03_smoke  04_serve_vllm
bench/     tasks/（cab のタスク、変更なし）、scripts/
ocr/       gen_synthetic.py  run_ocr.py  score.py  fixtures/
opencode/  opencode.json.example
docs/      PLAN.ja.md  SETUP.md  reports/
```

## ライセンス

このリポジトリのコードとドキュメントは [Apache License 2.0](LICENSE) です。モデルの重みはそれぞれのライセンス（Inkling 2 Mini は Apache 2.0）に従い、ここでは再配布しません。
