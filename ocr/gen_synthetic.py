#!/usr/bin/env python3
"""Generate synthetic Japanese documents with exact ground truth.

Every name, address, amount and item here is invented, and the prose is
public-domain (Aozora Bunko). Nothing in this set comes from a real document,
which is what lets the images and their ground truth live in a public repo.

Output (per sample):
  <out>/<id>.png    the rendered page
  <out>/<id>.json   {"id", "kind", "text", "fields", "params"}

Kinds:
  invoice   horizontal layout, header + line-item table + totals  (B1, B2)
  vertical  tategaki prose in columns, right to left              (B1)

Fonts: pass OFL-licensed fonts (Noto Sans JP / Noto Serif JP). Do not commit
images rendered with a proprietary system font.

    python3 -I ocr/gen_synthetic.py --font /path/NotoSansJP-Regular.ttf \
        --font-serif /path/NotoSerifJP-Regular.ttf --out ocr/data/synth --n 40
"""
import argparse
import json
import random
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont

# Deliberately generic placeholder names. Do not add real organisations.
COMPANIES = ["サンプル商事株式会社", "架空工業株式会社", "例示システム合同会社",
             "テスト物産株式会社", "モデル精機株式会社", "ダミー電子株式会社"]
PEOPLE = ["山田 太郎", "佐藤 花子", "鈴木 一郎", "高橋 次郎", "田中 三恵"]
ITEMS = [("ステンレスボルト M8×30", 120), ("六角ナット M8", 35), ("平ワッシャー 8mm", 12),
         ("配線ダクト 2m", 1850), ("端子台 12極", 2400), ("保守作業費（半日）", 45000),
         ("設計レビュー費", 80000), ("ケーブル LAN Cat6 5m", 980), ("梱包・送料", 1500)]
# Public domain (Natsume Soseki, Aozora Bunko).
PROSE = ("吾輩は猫である。名前はまだ無い。どこで生れたかとんと見当がつかぬ。"
         "何でも薄暗いじめじめした所でニャーニャー泣いていた事だけは記憶している。"
         "吾輩はここで始めて人間というものを見た。しかもあとで聞くとそれは書生という"
         "人間中で一番獰悪な種族であったそうだ。")
DIGITS_KANJI = str.maketrans("0123456789", "〇一二三四五六七八九")


def yen(n):
    return f"¥{n:,}"


def make_invoice(rng, font_path, idx):
    W, H = 1240, 1754  # A4 at 150 dpi
    img = Image.new("RGB", (W, H), "white")
    d = ImageDraw.Draw(img)
    f_title = ImageFont.truetype(font_path, 56)
    f = ImageFont.truetype(font_path, 26)
    lines = []

    def text(xy, s, font=f):
        d.text(xy, s, fill="black", font=font)
        lines.append(s)

    buyer, seller = rng.sample(COMPANIES, 2)
    no = f"INV-{rng.randint(2026000, 2026999)}"
    date = f"2026年{rng.randint(1, 12)}月{rng.randint(1, 28)}日"
    text((W // 2 - 110, 80), "請求書", f_title)
    text((80, 200), f"{buyer} 御中")
    text((800, 200), f"請求番号 {no}")
    text((800, 240), f"発行日 {date}")
    text((800, 300), seller)
    text((800, 340), f"担当 {rng.choice(PEOPLE)}")

    rows = rng.sample(ITEMS, rng.randint(3, 6))
    y = 460
    cols = [80, 640, 800, 960]
    d.rectangle([70, y - 10, W - 70, y + 40], outline="black")
    for x, h in zip(cols, ["品目", "数量", "単価", "金額"]):
        text((x, y), h)
    subtotal = 0
    items = []
    for name, price in rows:
        y += 56
        qty = rng.randint(1, 50)
        amt = qty * price
        subtotal += amt
        items.append({"name": name, "qty": qty, "unit_price": price, "amount": amt})
        d.line([70, y - 10, W - 70, y - 10], fill="gray")
        for x, s in zip(cols, [name, str(qty), yen(price), yen(amt)]):
            text((x, y), s)
    tax = subtotal // 10
    total = subtotal + tax
    y += 100
    for label, v in [("小計", subtotal), ("消費税(10%)", tax), ("合計", total)]:
        text((800, y), f"{label} {yen(v)}")
        y += 44

    fields = {"invoice_no": no, "issue_date": date, "buyer": buyer, "seller": seller,
              "subtotal": subtotal, "tax": tax, "total": total, "items": items}
    return img, "\n".join(lines), fields


def make_vertical(rng, font_path, idx):
    W, H = 1240, 1754
    img = Image.new("RGB", (W, H), "white")
    d = ImageDraw.Draw(img)
    size = 40
    f = ImageFont.truetype(font_path, size)
    start = rng.randint(0, 20)
    body = PROSE[start:] + f"昭和{rng.randint(1, 60)}年{rng.randint(1, 12)}月記。".translate(DIGITS_KANJI)
    per_col = (H - 240) // (size + 6)
    cols = [body[i:i + per_col] for i in range(0, len(body), per_col)]
    x = W - 140
    for col in cols:
        for j, ch in enumerate(col):
            d.text((x, 120 + j * (size + 6)), ch, fill="black", font=f)
        x -= size + 30
    return img, "".join(cols), {}


def degrade(rng, img, level):
    """level 0 = clean render. 1..3 = scanner-ish blur, rotation, noise."""
    if level == 0:
        return img
    img = img.rotate(rng.uniform(-0.6, 0.6) * level, expand=False, fillcolor="white")
    img = img.filter(ImageFilter.GaussianBlur(0.4 * level))
    px = img.load()
    for _ in range(3000 * level):
        x, y = rng.randrange(img.width), rng.randrange(img.height)
        g = rng.randint(120, 200)
        px[x, y] = (g, g, g)
    return img


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--font", required=True, help="OFL sans font (Noto Sans JP)")
    ap.add_argument("--font-serif", help="OFL serif font for vertical prose (Noto Serif JP)")
    ap.add_argument("--out", default="ocr/data/synth")
    ap.add_argument("--n", type=int, default=40)
    ap.add_argument("--seed", type=int, default=20261123)
    a = ap.parse_args()

    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    rng = random.Random(a.seed)
    for i in range(a.n):
        kind = "invoice" if i % 2 == 0 else "vertical"
        level = (i // 2) % 4
        if kind == "invoice":
            img, text, fields = make_invoice(rng, a.font, i)
        else:
            img, text, fields = make_vertical(rng, a.font_serif or a.font, i)
        img = degrade(rng, img, level)
        sid = f"{kind}-{i:03d}-d{level}"
        img.save(out / f"{sid}.png")
        (out / f"{sid}.json").write_text(json.dumps(
            {"id": sid, "kind": kind, "text": text, "fields": fields,
             "params": {"seed": a.seed, "degrade": level}},
            ensure_ascii=False, indent=1))
    print(f"wrote {a.n} samples to {out}")


if __name__ == "__main__":
    main()
