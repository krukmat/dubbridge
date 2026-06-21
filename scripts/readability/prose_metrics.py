#!/usr/bin/env python3
"""Prose quality metrics for Markdown files: passive voice, lexical density,
long sentences, adverb density (via spaCy).

Usage:
    python prose_metrics.py path/to/article.md
"""
import re, sys, spacy

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

nlp = spacy.load("en_core_web_sm")
doc = nlp(prose)

sentences = list(doc.sents)
n_sent = len(sentences)
tokens = [t for t in doc if t.is_alpha]
n_tokens = len(tokens)

passive_sents = [s for s in sentences if any(t.dep_ == "nsubjpass" for t in s)]
passive_pct = 100 * len(passive_sents) / n_sent

content_pos = {"NOUN", "VERB", "ADJ", "ADV"}
aux_deps = {"aux", "auxpass", "cop"}
content = [t for t in doc if t.is_alpha and t.pos_ in content_pos
           and t.dep_ not in aux_deps and not t.is_stop]
lexical_density = 100 * len(content) / n_tokens

word_counts = [len([t for t in s if t.is_alpha]) for s in sentences]
long_sents = [(wc, s) for wc, s in zip(word_counts, sentences) if wc > 25]
avg_len = sum(word_counts) / n_sent

adverbs_ly = [t.text for t in doc if t.tag_ == "RB" and t.text.lower().endswith("ly")]
adverb_density = 100 * len(adverbs_ly) / n_tokens

print(f"\nFile: {path}")
print(f"Prose tokens ............. {n_tokens}")
print(f"Sentences ................ {n_sent}")
print(f"Avg sentence length ...... {avg_len:.1f} words")
print()
print("=" * 55)
print(f"Passive voice ............ {len(passive_sents)}/{n_sent}  ({passive_pct:.1f}%)")
print(f"  target: < 10%  {'✓ OK' if passive_pct < 10 else '✗ above target'}")
print()
print(f"Lexical density .......... {lexical_density:.1f}%")
print(f"  target: 40-55% for accessible prose")
print(f"  {'✓ conversational' if lexical_density < 55 else ('✗ dense' if lexical_density > 60 else '~ borderline')}")
print()
print(f"Long sentences (>25w) .... {len(long_sents)}/{n_sent}  ({100*len(long_sents)/n_sent:.1f}%)")
print(f"  target: < 10%  {'✓ OK' if len(long_sents)/n_sent < 0.10 else '✗ above target'}")
print()
print(f"Adverb density (-ly) ..... {len(adverbs_ly)} adverbs  ({adverb_density:.1f}% of tokens)")
print("=" * 55)

if passive_sents:
    print("\nPassive sentences:")
    for s in passive_sents[:5]:
        print(f"  • {s.text.strip()[:90]}...")

if long_sents:
    print(f"\nLong sentences (top 5):")
    for wc, s in sorted(long_sents, key=lambda x: -x[0])[:5]:
        print(f"  [{wc}w] {s.text.strip()[:90]}...")

if adverbs_ly:
    print(f"\n-ly adverbs: {', '.join(adverbs_ly)}")
