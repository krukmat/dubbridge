#!/usr/bin/env python3
"""Grade-level readability metrics for Markdown files (via textstat).

Usage:
    python readability_grade.py path/to/article.md
"""
import re, sys, textstat

def extract_prose(path):
    raw = open(path, encoding="utf-8").read()
    t = re.sub(r"```.*?```", " ", raw, flags=re.DOTALL)
    t = re.sub(r"`[^`]*`", " code ", t)
    t = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", t)
    t = re.sub(r"^#{1,6}\s*", "", t, flags=re.MULTILINE)
    t = re.sub(r"^\s*[-*]\s+", "", t, flags=re.MULTILINE)
    t = re.sub(r"[*_>]", " ", t)
    return re.sub(r"\s+", " ", t).strip()

path = sys.argv[1] if len(sys.argv) > 1 else "article.md"
prose = extract_prose(path)
textstat.set_lang("en_US")

metrics = [
    ("Flesch Reading Ease",         textstat.flesch_reading_ease(prose),         "higher = easier (0-100)"),
    ("Flesch-Kincaid Grade",        textstat.flesch_kincaid_grade(prose),        "US school grade"),
    ("Gunning Fog",                 textstat.gunning_fog(prose),                 "years of education"),
    ("SMOG Index",                  textstat.smog_index(prose),                  "grade (best >=30 sentences)"),
    ("Coleman-Liau Index",          textstat.coleman_liau_index(prose),          "US grade"),
    ("Automated Readability (ARI)", textstat.automated_readability_index(prose), "US grade"),
    ("Dale-Chall (new)",            textstat.dale_chall_readability_score(prose),"<=7 ~ 9th-10th grade"),
    ("Linsear Write",               textstat.linsear_write_formula(prose),       "US grade"),
]

print(f"\nFile: {path}")
print(f"prose words .......... {textstat.lexicon_count(prose)}")
print(f"sentences ............ {textstat.sentence_count(prose)}")
print(f"syllables ............ {textstat.syllable_count(prose)}")
print(f"difficult words ...... {textstat.difficult_words(prose)}  (Dale-Chall list)")
print("=" * 55)
for name, val, note in metrics:
    print(f"{name:<30} {val:>6.1f}   {note}")
print("=" * 55)
print(f"Consensus grade ...... {textstat.text_standard(prose, float_output=True):.1f}  "
      f"(\"{textstat.text_standard(prose)}\")\n")
