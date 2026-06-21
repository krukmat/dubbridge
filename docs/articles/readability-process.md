# Readability Measurement Process for Markdown Articles

Applied to: `okf-practical-repository-knowledge.md`
Language: English

---

## Overview

Two complementary measurement passes:

1. **Grade-level metrics** (via `textstat`) — how hard is the text to read?
2. **Prose quality metrics** (via `spaCy`) — what structural patterns affect clarity?

---

## Setup

```bash
python3 -m venv readvenv
source readvenv/bin/activate
pip install textstat spacy
python -m spacy download en_core_web_sm
```

---

## Step 1 — Preprocessing

Both scripts share the same preprocessing step: strip non-prose content from
Markdown before measuring. Measuring raw Markdown inflates difficulty scores
because code blocks, YAML, and diagram syntax are not natural language.

```python
import re

def extract_prose(path):
    raw = open(path, encoding="utf-8").read()
    t = re.sub(r"```.*?```", " ", raw, flags=re.DOTALL)  # fenced blocks
    t = re.sub(r"`[^`]*`", " code ", t)                   # inline code
    t = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", t)        # links → label
    t = re.sub(r"^#{1,6}\s*", "", t, flags=re.MULTILINE)  # headings
    t = re.sub(r"^\s*[-*]\s+", "", t, flags=re.MULTILINE) # bullets
    t = re.sub(r"[*_>]", " ", t)                          # emphasis/quotes
    return re.sub(r"\s+", " ", t).strip()
```

---

## Step 2 — Grade-level metrics (`textstat`)

Run `scripts/readability_grade.py` against any Markdown file.

```python
import textstat
from preprocessing import extract_prose  # step 1 above

prose = extract_prose("path/to/article.md")
textstat.set_lang("en_US")

metrics = [
    ("Flesch Reading Ease",         textstat.flesch_reading_ease(prose)),
    ("Flesch-Kincaid Grade",        textstat.flesch_kincaid_grade(prose)),
    ("Gunning Fog",                 textstat.gunning_fog(prose)),
    ("SMOG Index",                  textstat.smog_index(prose)),
    ("Coleman-Liau Index",          textstat.coleman_liau_index(prose)),
    ("Automated Readability (ARI)", textstat.automated_readability_index(prose)),
    ("Dale-Chall",                  textstat.dale_chall_readability_score(prose)),
    ("Linsear Write",               textstat.linsear_write_formula(prose)),
]

for name, val in metrics:
    print(f"{name:<30} {val:>6.1f}")

print(f"\nConsensus: {textstat.text_standard(prose)}")
```

### Reference targets for general audience

| Metric | Target |
|---|---|
| Flesch Reading Ease | ≥ 70 |
| Flesch-Kincaid Grade | ≤ 8 |
| Gunning Fog | ≤ 10 |
| Consensus (`text_standard`) | 7th–8th grade |

### What each metric is sensitive to

| Metric | Driven by |
|---|---|
| Flesch / F-K | Sentence length + syllables per word |
| Gunning Fog | Sentence length + words with 3+ syllables |
| SMOG | Words with 3+ syllables (almost exclusively) |
| Dale-Chall | Words outside a fixed ~3,000-word "easy" list — penalizes proper nouns |
| Coleman-Liau / ARI | Characters per word + sentence length |

**Practical implication:** SMOG and Dale-Chall do not respond well to splitting
sentences. To move them, reduce polysyllabic words. Dale-Chall has a hard floor
when the text contains unavoidable proper nouns (project names, acronyms).

---

## Step 3 — Prose quality metrics (`spaCy`)

Run `scripts/prose_metrics.py` against any Markdown file.

```python
import re, spacy
from preprocessing import extract_prose

prose = extract_prose("path/to/article.md")
nlp = spacy.load("en_core_web_sm")
doc = nlp(prose)

sentences = list(doc.sents)
tokens = [t for t in doc if t.is_alpha]

# Passive voice: detect nsubjpass dependency
passive = [s for s in sentences if any(t.dep_ == "nsubjpass" for t in s)]
passive_pct = 100 * len(passive) / len(sentences)

# Lexical density: content words / total tokens
content_pos = {"NOUN", "VERB", "ADJ", "ADV"}
aux_deps = {"aux", "auxpass", "cop"}
content = [t for t in doc if t.is_alpha and t.pos_ in content_pos
           and t.dep_ not in aux_deps and not t.is_stop]
lexical_density = 100 * len(content) / len(tokens)

# Long sentences
word_counts = [len([t for t in s if t.is_alpha]) for s in sentences]
long_sents = [(wc, s) for wc, s in zip(word_counts, sentences) if wc > 25]
long_pct = 100 * len(long_sents) / len(sentences)

# Adverb density
adverbs_ly = [t.text for t in doc if t.tag_ == "RB" and t.text.lower().endswith("ly")]
```

### Reference targets

| Metric | Target | Notes |
|---|---|---|
| Passive voice | < 10% | Higher is valid for impersonal/objective tone |
| Lexical density | 40–55% | Below 40% = too thin; above 60% = dense |
| Long sentences (>25w) | < 10% | Watch for lists with semicolons — tokenizers read them as one sentence |
| Adverb density (-ly) | < 2% | High count signals weak verbs |

### Notes on passive voice

A high passive voice % is not always a defect. For divulgative or impersonal
writing it signals objective tone. Evaluate in context before reducing.

---

## Step 4 — Polysyllable audit

When SMOG is above target, run `scripts/polysyllables.py` to find replaceable
words. Always exclude proper nouns and fixed technical terms first.

```python
import re
from collections import Counter
from preprocessing import extract_prose

SKIP = {"projectname", "acronym1", "acronym2"}  # customize per project

def syllables(word):
    w = re.sub(r"[^a-z]", "", word.lower())
    if not w: return 0
    count = len(re.findall(r"[aeiouy]+", w))
    if w.endswith("e") and not w.endswith("le") and count > 1:
        count -= 1
    return max(1, count)

prose = extract_prose("path/to/article.md")
words = re.findall(r"[A-Za-z]+(?:'[A-Za-z]+)?", prose)

poly = Counter(w.lower() for w in words
               if w.lower() not in SKIP and syllables(w) >= 3)

for word, count in poly.most_common(30):
    print(f"{count:>3}x  {word}  ({syllables(word)} syl)")
```

Common replacements found useful:

| Original | Replacement |
|---|---|
| documentation | the docs / the files |
| information | the data / the content |
| mislabeled | wrong label |
| trustworthy | reliable |
| automatic | on its own |
| ordinary | plain / normal |

---

## Results on `okf-practical-repository-knowledge.md`

| Pass | Flesch RE | F-K Grade | Gunning Fog | Consensus |
|---|---|---|---|---|
| Original (technical) | 58.9 | 10.0 | 12.8 | 9th grade |
| After rewrite (general audience) | 61.8 | 9.6 | 11.8 | 9th grade |
| After sentence splitting + word simplification | **69.7** | **7.5** | **9.7** | **8th–9th grade** |

| Prose metric | Value | Assessment |
|---|---|---|
| Passive voice | 17.1% | Intentional — objective/divulgative tone |
| Lexical density | 43.8% | Conversational |
| Long sentences >25w | 14.6% | Two are semicolon lists, not real long sentences |
| Adverb density | 0.8% | Low and functional |

---

## Lessons learned

- **Preprocess first.** Markdown code blocks and diagrams inflate every metric.
- **SMOG and Dale-Chall have a hard floor** when the text contains unavoidable
  proper nouns or technical terms. Don't chase them past that floor.
- **Sentence splitting moves Flesch and F-K fast.** It has almost no effect on
  SMOG.
- **Tokenizers misread semicolon lists as long sentences.** Verify manually
  before splitting.
- **Passive voice is a tone decision,** not just a readability number.
- **Consensus grade (`text_standard`) is the most stable single number** to
  track across edits — it aggregates all metrics and smooths outliers.
