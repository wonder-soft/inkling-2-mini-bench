#!/usr/bin/env python3
"""Run one OCR condition over a directory of <id>.png and write a JSONL.

Conditions (B4):
  vlm        image -> model
  tesseract  image -> tesseract (jpn+eng, or jpn_vert for vertical samples)
  hybrid     image + tesseract's text -> model ("correct this OCR draft")

Task (B1 vs B2):
  transcribe  free text, scored by CER
  extract     invoice fields as JSON, scored by field exact match

    python3 -I ocr/run_ocr.py --in ocr/data/synth --cond vlm --task transcribe \
        --out results/ocr/vlm-transcribe.jsonl --effort 0.2
"""
import argparse
import base64
import json
import subprocess
import time
import urllib.request
from pathlib import Path

PROMPT_TRANSCRIBE = ("Transcribe all text in this image exactly, in reading order. "
                     "Output only the text. Do not translate, summarise or add anything.")
PROMPT_EXTRACT = ("This is a Japanese invoice. Return ONLY a JSON object with keys: "
                  "invoice_no, issue_date, buyer, seller, subtotal, tax, total, "
                  "items (list of {name, qty, unit_price, amount}). Numbers as integers.")
PROMPT_HYBRID = ("Below is a draft produced by a conventional OCR engine for the attached image. "
                 "It may contain errors. Using the image as the source of truth, output the "
                 "corrected {what}.\n\n--- OCR draft ---\n{draft}")


def tesseract(png, vertical):
    lang = "jpn_vert" if vertical else "jpn+eng"
    t0 = time.time()
    r = subprocess.run(["tesseract", str(png), "-", "-l", lang, "--psm", "5" if vertical else "6"],
                       capture_output=True, text=True)
    return r.stdout.strip(), time.time() - t0


def vlm(base, model, png, prompt, effort, max_tokens):
    b64 = base64.b64encode(png.read_bytes()).decode()
    body = {"model": model, "temperature": 0, "max_tokens": max_tokens,
            "chat_template_kwargs": {"reasoning_effort": effort},
            "messages": [{"role": "user", "content": [
                {"type": "image_url", "image_url": {"url": f"data:image/png;base64,{b64}"}},
                {"type": "text", "text": prompt}]}]}
    req = urllib.request.Request(f"{base}/v1/chat/completions", data=json.dumps(body).encode(),
                                 headers={"Content-Type": "application/json"})
    t0 = time.time()
    with urllib.request.urlopen(req, timeout=1800) as r:
        d = json.load(r)
    dt = time.time() - t0
    m = d["choices"][0]["message"]
    return (m.get("content") or "").strip(), dt, {
        "reasoning_chars": len(m.get("reasoning_content") or ""),
        "usage": d.get("usage"), "finish_reason": d["choices"][0].get("finish_reason")}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--in", dest="inp", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--cond", choices=["vlm", "tesseract", "hybrid"], required=True)
    ap.add_argument("--task", choices=["transcribe", "extract"], default="transcribe")
    ap.add_argument("--base-url", default="http://127.0.0.1:8000")
    ap.add_argument("--model", default="inkling-2-mini")
    ap.add_argument("--effort", type=float, default=0.2)
    ap.add_argument("--max-tokens", type=int, default=8192)
    ap.add_argument("--limit", type=int, default=0)
    a = ap.parse_args()

    pngs = sorted(Path(a.inp).glob("*.png"))
    if a.task == "extract":
        pngs = [p for p in pngs if p.name.startswith("invoice")]
    if a.limit:
        pngs = pngs[:a.limit]
    Path(a.out).parent.mkdir(parents=True, exist_ok=True)
    base_prompt = PROMPT_EXTRACT if a.task == "extract" else PROMPT_TRANSCRIBE
    what = "JSON" if a.task == "extract" else "transcription"

    with open(a.out, "w") as fo:
        for p in pngs:
            vertical = p.name.startswith("vertical")
            rec = {"id": p.stem, "cond": a.cond, "task": a.task, "model": a.model,
                   "effort": a.effort}
            try:
                if a.cond == "tesseract":
                    out, dt = tesseract(p, vertical)
                    rec.update(output=out, seconds=round(dt, 3))
                elif a.cond == "vlm":
                    out, dt, meta = vlm(a.base_url, a.model, p, base_prompt, a.effort, a.max_tokens)
                    rec.update(output=out, seconds=round(dt, 3), **meta)
                else:
                    draft, dt_t = tesseract(p, vertical)
                    prompt = base_prompt + "\n\n" + PROMPT_HYBRID.format(what=what, draft=draft)
                    out, dt_v, meta = vlm(a.base_url, a.model, p, prompt, a.effort, a.max_tokens)
                    rec.update(output=out, seconds=round(dt_t + dt_v, 3),
                               seconds_tesseract=round(dt_t, 3), **meta)
            except Exception as e:  # record, never drop: a missing row reads as a pass
                rec.update(output=None, error=repr(e))
            fo.write(json.dumps(rec, ensure_ascii=False) + "\n")
            fo.flush()
            print(rec["id"], rec.get("seconds"), rec.get("error", ""))


if __name__ == "__main__":
    main()
