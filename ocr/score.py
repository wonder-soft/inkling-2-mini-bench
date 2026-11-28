#!/usr/bin/env python3
"""Score an OCR JSONL against ground truth.

transcribe: CER = edit_distance(pred, gold) / len(gold), after NFKC and with all
            whitespace removed (layout whitespace is not what B1 measures).
extract:    per-field exact match; items scored as a multiset of
            (name, qty, unit_price, amount).

    python3 -I ocr/score.py --gold ocr/data/synth --pred results/ocr/vlm-transcribe.jsonl
"""
import argparse
import json
import re
import statistics
import unicodedata
from collections import Counter, defaultdict
from pathlib import Path


def norm(s):
    s = unicodedata.normalize("NFKC", s or "")
    return re.sub(r"\s+", "", s)


def edit_distance(a, b):
    prev = list(range(len(b) + 1))
    for i, ca in enumerate(a, 1):
        cur = [i]
        for j, cb in enumerate(b, 1):
            cur.append(min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (ca != cb)))
        prev = cur
    return prev[-1]


def parse_json(s):
    if not s:
        return None
    m = re.search(r"\{.*\}", s, re.S)
    try:
        return json.loads(m.group(0)) if m else None
    except json.JSONDecodeError:
        return None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--gold", required=True)
    ap.add_argument("--pred", required=True)
    a = ap.parse_args()

    rows = [json.loads(l) for l in open(a.pred)]
    by_group = defaultdict(list)
    field_hits = Counter()
    field_n = Counter()
    errors = 0
    for r in rows:
        gold = json.loads((Path(a.gold) / f"{r['id']}.json").read_text())
        if r.get("output") is None:
            errors += 1
        if r["task"] == "transcribe":
            g = norm(gold["text"])
            cer = edit_distance(norm(r.get("output")), g) / max(1, len(g))
            by_group[(gold["kind"], gold["params"]["degrade"])].append(cer)
        else:
            pred = parse_json(r.get("output")) or {}
            for k, v in gold["fields"].items():
                field_n[k] += 1
                if k == "items":
                    key = lambda it: (norm(str(it.get("name"))), it.get("qty"), it.get("unit_price"), it.get("amount"))
                    ok = Counter(map(key, pred.get("items") or [])) == Counter(map(key, v))
                elif k.endswith("_date"):
                    # The prompt does not fix a format; models answer ISO
                    # (2026-03-27) for 2026年3月27日. Compare the numbers.
                    ok = [int(x) for x in re.findall(r"\d+", str(pred.get(k)))] == \
                         [int(x) for x in re.findall(r"\d+", str(v))]
                else:
                    ok = norm(str(pred.get(k))) == norm(str(v))
                field_hits[k] += ok

    secs = [r["seconds"] for r in rows if r.get("seconds") is not None]
    print(f"rows={len(rows)} errors={errors} "
          f"median_seconds={statistics.median(secs) if secs else 'n/a'}")
    if by_group:
        print("\nCER by (kind, degrade)  — lower is better")
        for (kind, lvl), v in sorted(by_group.items()):
            print(f"  {kind:<9} d{lvl}  n={len(v):<3} mean={statistics.mean(v):.4f} median={statistics.median(v):.4f}")
        allv = [x for v in by_group.values() for x in v]
        print(f"  ALL           n={len(allv):<3} mean={statistics.mean(allv):.4f}")
    if field_n:
        print("\nField exact match")
        for k in field_n:
            print(f"  {k:<11} {field_hits[k]}/{field_n[k]}")


if __name__ == "__main__":
    main()
