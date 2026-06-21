#!/usr/bin/env python3
"""Polysyllable audit for Markdown files — useful when SMOG is above target.

Usage:
    python polysyllables.py path/to/article.md [skip_term1 skip_term2 ...]

    Extra args after the path are added to the skip list (project-specific
    proper nouns and acronyms that can't be replaced).
"""
import re, sys
from collections import Counter

def extract_prose(path):
    raw = open(path, encoding="utf-8").read()
    t = re.sub(r"```.*?```", " ", raw, flags=re.DOTALL)
    t = re.sub(r"`[^`]*`", " code ", t)
    t = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", t)
    t = re.sub(r"^#{1,6}\s*", "", t, flags=re.MULTILINE)
    t = re.sub(r"^\s*[-*]\s+", "", t, flags=re.MULTILINE)
    t = re.sub(r"[*_>]", " ", t)
    return re.sub(r"\s+", " ", t).strip()

def syllables(word):
    w = re.sub(r"[^a-z]", "", word.lower())
    if not w: return 0
    count = len(re.findall(r"[aeiouy]+", w))
    if w.endswith("e") and not w.endswith("le") and count > 1:
        count -= 1
    return max(1, count)

path = sys.argv[1] if len(sys.argv) > 1 else "article.md"
extra_skip = {s.lower() for s in sys.argv[2:]}

BASE_SKIP = {"dubbridge", "okf", "adr", "jwt", "yaml", "mermaid",
             "github", "google", "spacy", "textstat", "readme", "ci", "qa"}
skip = BASE_SKIP | extra_skip

prose = extract_prose(path)
words = re.findall(r"[A-Za-z]+(?:'[A-Za-z]+)?", prose)

poly = Counter(w.lower() for w in words
               if w.lower() not in skip and syllables(w) >= 3)

print(f"\nFile: {path}")
print(f"Polysyllables (3+ syl), most frequent first:\n")
for word, count in poly.most_common(30):
    print(f"  {count:>3}x  {word}  ({syllables(word)} syl)")
