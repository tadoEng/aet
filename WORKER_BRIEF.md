# AET — Worker Agent Brief

**Version:** v3.0 (research-updated)
**Phase:** 1 — Core Pipeline
**Stack:** Rust (stable), Typst CLI, CSV, TOML

---

## Objective

Build `aet` — a local CLI tool that:
1. Reads `article.toml` + `vocab.csv` from an article folder.
2. Validates all fields.
3. Exports three Anki TSV files.
4. Exports one PDF via Typst CLI.
5. Displays a session timer at the end of every command.

---

## Final 19-Column Schema

### Column order (locked)

```
id, term, ipa, category, type,
definition_en, meaning_vi, source_sentence, my_sentence, priority,
collocation_pattern, ielts_topics, skill_use, difficulty,
card_types, origin, review_status, anki_export, tags
```

### Split

```
──── USER FILLS (10) ───────────────────────────────────────
id               term              ipa
category         type              definition_en
meaning_vi       source_sentence   my_sentence
priority

──── CODEX FILLS (9) ───────────────────────────────────────
collocation_pattern    ielts_topics      skill_use
difficulty             card_types        origin
review_status          anki_export       tags
```

### Field controlled vocabularies

**`type`:**
```
word | noun_phrase | verb_phrase | collocation |
fixed_phrase | prepositional_pattern | reporting_verb | sentence_frame
```

**`priority`:** `P1 | P2 | P3`

**`card_types`:** semicolon-separated subset of `basic | cloze | production`

**`origin`:** `teacher | codex | user | manual`

**`review_status`:** `draft | reviewed | approved | rejected`

**`anki_export`:** `yes | no`

**`skill_use`:** semicolon-separated subset of `reading | writing-task1 | writing-task2 | speaking-part2 | speaking-part3 | listening`

**`difficulty`:** `B1 | B2 | C1 | C2`

**`ielts_topics`:** semicolon-separated from canonical list:
```
environment | poverty-global-issues | law-crimes | business-economics |
health-medicine | education | family-relationships-teenagers |
urban-rural | personality | physical-appearance | work-jobs | government-policy
```

---

## Validation Rules

Run on `aet validate <article-folder>`.

**Required files:**
- `article.toml`
- `vocab.csv`

**Required CSV columns:** all 19 in correct order.

**Per-row rules:**

| Field | Rule |
|---|---|
| `id` | Non-empty, unique within file |
| `term` | Non-empty |
| `type` | Must be a valid controlled value |
| `definition_en` | Non-empty |
| `meaning_vi` | Non-empty |
| `source_sentence` | Non-empty |
| `priority` | Must be P1, P2, or P3 |
| `origin` | Must be a valid controlled value |
| `review_status` | Must be a valid controlled value |
| `anki_export` | Must be `yes` or `no` |
| `ipa` | Optional — may be empty |
| `my_sentence` | Optional — WARN if empty on P1+approved rows, do not block |
| `ielts_topics` | Each semicolon-separated value must be in canonical list |
| `card_types` | Each semicolon-separated value must be `basic`, `cloze`, or `production` |

**Article-level validation:**
- Compute `IpaMode` from all `ipa` fields:
  ```rust
  pub enum IpaMode {
      Absent,   // all ipa fields are empty
      Mixed,    // some present, some empty
      Present,  // all ipa fields are non-empty
  }
  ```
- Output `ipa_mode: absent | mixed | present` in validation summary.
- Cross-check against `has_ipa` in `article.toml [stats]` — warn if they disagree, do not block.

**CLI output format:**
```
✓ article.toml valid
✓ vocab.csv: 115 rows, 19 columns
✓ IDs unique
✓ ipa_mode: mixed (114 present, 1 absent)
⚠ 12 P1+approved rows have empty my_sentence — fill after review cycles
✓ All ielts_topics valid
✓ All type values valid
✓ Validation passed (0 errors, 12 warnings)
[aet] Session time today: 3 min.  ✓ Within 20-min budget.
```

---

## Anki Export Specification

### Export filter (applies to ALL three files)

Export a row only when ALL of:
- `priority = P1` (default) OR `--all-priorities` flag is set
- `review_status = approved`
- `anki_export = yes`
- `source_sentence` is not empty

### TSV column format (all three files)

All three Anki export files use **4 columns**:

```
front <TAB> back <TAB> extra <TAB> tags
```

**Design rationale (Ali Abdaal Anki Masterclass, 2024):**
The `back` should be clean and atomic — just the answer. Everything needed to understand context goes in `extra`, which Anki displays below the back. This is how serious Anki users use the app. The `source_sentence`, `collocation_pattern`, and topic context are reference material, not the answer itself.

---

### File 1: `anki-basic.tsv`

Tests passive meaning. Always generated.

**Front:**
```
What does "{term}" mean?
```

**Back (clean — answer only):**
```
{definition_en}
{meaning_vi}
```

**Extra (reference panel):**
```
Source: {source_sentence}
Pattern: {collocation_pattern}
Topic: {ielts_topics joined with " · "}
```

**Tags:** `AET article::{article_id} topic::{ielts_topics[0]} priority::P1`

---

### File 2: `anki-cloze.tsv`

Tests vocabulary in context. Always generated.

**Text:** Replace `{term}` in `source_sentence` with `{{c1::term}}`.

**Cloze fallback:** If the exact term string is not found in `source_sentence`:
1. Try case-insensitive match.
2. Try matching the first word of the term only (for collocations).
3. If no match: wrap the whole `source_sentence` as a cloze with the `term` as the answer in `extra`.

**Back (extra — reference panel):**
```
{term} = {definition_en}
Pattern: {collocation_pattern}
Topic: {ielts_topics joined with " · "}
```

**Tags:** same rules as basic.

---

### File 3: `anki-production.tsv`

Tests active production. Always generated (even when `my_sentence` is empty).

**Research basis (Decision D8):** The production card is a *prompt card*. The back stays clean — no model answer. Context goes in `extra`. This implements Swain's Output Hypothesis: the learner must attempt genuine production before checking the reference. Showing a model answer removes the "push" that drives productive gains.

**Front:**
```
Use "{term}" in an IELTS-style sentence about {ielts_topics[0]}.
```

**Back (clean — minimal anchor):**
```
{definition_en}
{meaning_vi}
```

**Extra (reference panel — no model answer):**
```
Pattern:    {collocation_pattern}
Source:     {source_sentence}
Topic:      {ielts_topics joined with " · "}
```

**If `my_sentence` is non-empty** (post-review log entry), append to extra:
```
Your previous sentence:
{my_sentence}
```

This shows the learner's own past production as a record, not as an answer to copy.

**Usage note (for README, not CLI):**
Import all three TSV files into Anki. In Anki, suspend new production cards immediately after import. Unsuspend a production card for a given term only after its basic card has been answered correctly 2+ times (Anki "young" status). This implements Decision D9: production activates after receptive maturity.

---

## Typst PDF Specification

### Invocation
```bash
typst compile \
  --input vocab_json="<path_to_data.json>" \
  --input ipa_mode="absent|mixed|present" \
  templates/typst/vocab-pack.typ \
  dist/{article_id}/vocabulary.pdf
```

### JSON data format passed to Typst
```json
{
  "article": {
    "id": "dengue-bangladesh",
    "title": "Deadly Dengue Fever Outbreak in Bangladesh Strains Scarce Resources",
    "source_name": "The New York Times",
    "date": "2023-09-25"
  },
  "ipa_mode": "mixed",
  "categories": [
    {
      "name": "Outbreak, Disease & Public Health",
      "entries": [...]
    }
  ]
}
```

### IPA-optional table rendering

| `ipa_mode` | Table columns |
|---|---|
| `Absent` | Term \| Definition (EN) \| Example \| Vietnamese |
| `Mixed` | Term \| IPA \| Definition (EN) \| Example \| Vietnamese — show "—" for missing IPA |
| `Present` | Term \| IPA \| Definition (EN) \| Example \| Vietnamese |

This is a **template-level decision** — the column count changes at the article level, not per row.

### Typst column widths (approximate, A4)

For `Present` / `Mixed` mode:

| Column | Width |
|---|---|
| Term | 22% |
| IPA | 18% |
| Definition (EN) | 25% |
| Example | 22% |
| Vietnamese | 13% |

For `Absent` mode (no IPA):

| Column | Width |
|---|---|
| Term | 25% |
| Definition (EN) | 28% |
| Example | 28% |
| Vietnamese | 19% |

### Required PDF features
- A4 page size
- UTF-8 Vietnamese and IPA unicode
- Category headings (from `category` field, grouped in order of first appearance)
- Multi-page table continuation
- Page numbers
- Clean text wrapping for long entries
- Optional: total entry count in footer

---

## CLI Specification

### Binary
```
aet
```

### Commands for v0.1

```bash
aet validate data/articles/dengue-bangladesh
aet build data/articles/dengue-bangladesh
aet build data/articles/dengue-bangladesh --only anki
aet build data/articles/dengue-bangladesh --only pdf
aet build data/articles/dengue-bangladesh --all-priorities
```

### `aet build` output
```text
dist/{article_id}/
  anki-basic.tsv
  anki-cloze.tsv
  anki-production.tsv
  vocabulary.pdf
```

### Exit codes
```
0 = success
1 = validation error (blocks build)
2 = build error (file write, Typst invocation)
```

### CLI prefix convention
```
✓ = success
⚠ = warning (non-blocking)
✗ = error (blocking)
ℹ = info
```

### Session timer output (every command)
```
[aet] Done in 2.3s.
[aet] Session time today: 18 min.  ✓ Within 20-min budget.
```

If over 20 minutes:
```
[aet] ⚠ Session time today: 24 min. Stop here — continue tomorrow.
```

---

## Session Timer Implementation

**Temp file:** `/tmp/aet-session-YYYY-MM-DD.json`

```json
{
  "date": "2026-05-18",
  "commands": [
    { "cmd": "validate", "article": "dengue-bangladesh", "ts": 1747526400, "elapsed_ms": 340 },
    { "cmd": "build",    "article": "dengue-bangladesh", "ts": 1747527100, "elapsed_ms": 2300 }
  ],
  "total_elapsed_ms": 2640
}
```

**Rules:**
- Read file on startup, check date. If date mismatch or file missing: start fresh.
- Record current command start time with `std::time::Instant`.
- On completion: read file again, add elapsed, write back.
- Display cumulative total in minutes.
- No external crate needed.

---

## Crate Layout

```
crates/
  aet-core/
    src/
      models/
        article.rs      ← ArticleMeta, IpaMode
        vocab.rs        ← VocabEntry, Priority, CardType, Origin, ReviewStatus, IpaMode
      parser/
        csv.rs
        toml.rs
      validator.rs
  aet-anki/
    src/
      basic.rs
      cloze.rs
      production.rs     ← reference panel format (no model answer — see Decision D8)
      lib.rs
  aet-typst/
    src/
      lib.rs
      vocab_pack.rs     ← IpaMode-aware renderer
  aet-cli/
    src/
      main.rs
      commands/
        validate.rs
        build.rs
      session_timer.rs
```

---

## Rust Data Models

```rust
pub struct ArticleMeta {
    pub id: String,
    pub title: String,
    pub source_name: String,
    pub date: String,                      // YYYY-MM-DD
    pub primary_topics: Vec<String>,
    pub secondary_topics: Vec<String>,
    pub skills: Vec<String>,
    pub level: Option<String>,
    pub teacher_pdf: Option<String>,
    pub private: bool,
    pub copyright_mode: Option<String>,
}

pub enum IpaMode { Absent, Mixed, Present }   // ← NOT None/Some/All

pub struct VocabEntry {
    pub id: String,
    pub term: String,
    pub ipa: Option<String>,
    pub category: String,
    pub entry_type: EntryType,
    pub definition_en: String,
    pub meaning_vi: String,
    pub source_sentence: String,
    pub my_sentence: Option<String>,
    pub priority: Priority,
    pub collocation_pattern: Option<String>,
    pub ielts_topics: Vec<String>,
    pub skill_use: Vec<String>,
    pub difficulty: Option<String>,
    pub card_types: Vec<CardType>,
    pub origin: Origin,
    pub review_status: ReviewStatus,
    pub anki_export: bool,
    pub tags: Vec<String>,
}

pub enum Priority       { P1, P2, P3 }
pub enum CardType       { Basic, Cloze, Production }
pub enum Origin         { Teacher, Codex, User, Manual }
pub enum ReviewStatus   { Draft, Reviewed, Approved, Rejected }
```

---

## Suggested Dependencies

```toml
[dependencies]
clap       = { version = "4", features = ["derive"] }
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
csv        = "1"
toml       = "0.8"
anyhow     = "1"
```

Session timer uses `std::time::Instant` and `std::fs` only — no external crate.
Typst invocation uses `std::process::Command`.

---

## Phase 1 Acceptance Criteria

```bash
# Both must succeed:
aet build data/articles/dengue-bangladesh         # IPA present → with IPA column
aet build data/articles/vietnam-two-child-policy  # No IPA → without IPA column
```

**Dengue Bangladesh checks:**
- `vocabulary.pdf` has IPA column (IpaMode::Mixed → 114 present, 1 absent)
- `anki-basic.tsv` contains only P1+approved+anki_export=yes rows
- `anki-production.tsv` back shows reference panel, NOT a model answer
- Vietnamese text renders correctly (UTF-8)
- IPA text renders correctly
- Session timer displayed

**Vietnam Two-Child Policy checks:**
- `vocabulary.pdf` has NO IPA column (IpaMode::Absent)
- Table column widths use the 4-column layout (25/28/28/19%)
- No empty IPA column or "—" placeholders
- All other checks same as above

**Both fixtures:**
- `aet validate` passes with 0 errors
- `anki-production.tsv` generated even if all `my_sentence` fields are empty
- Session timer accumulated correctly across validate + build calls in same day

---

## Recommended Anki Deck Settings (for README)

Document these settings in the README so the learner configures Anki correctly on first import. Without these, cards graduate too fast and the SRS algorithm doesn't work as intended for vocabulary acquisition.

**Source:** Ali Abdaal Anki Masterclass (2024) + Migaku SRS research.

```markdown
## Anki Setup for AET Decks

After importing any AET TSV file, apply these settings to the AET deck:

| Setting | Value | Reason |
|---|---|---|
| Steps (new cards) | 15 1440 8640 | 15min → 1day → 6days before graduating |
| Graduating interval | 15 days | Prevents premature long-term scheduling |
| Easy interval | 60 days | Reserve "Easy" for truly mastered items |
| Max reviews/day | 9999 | Never cap — let the queue clear naturally |
| New interval (lapse) | 70% | Partial reset on forgetting, not full reset |
| Leech threshold | 8 | Tag as leech after 8 failures — don't suspend |
| New card order | Random | Avoids memorizing position rather than meaning |

### Production deck setup
After importing `anki-production.tsv`:
1. Open `AET::Article::{article-id}` > Production sub-deck
2. Select all cards → Suspend
3. After a basic card reaches "young" status (answered correctly 2+ times),
   find its production counterpart by tag and unsuspend it manually.

Use stable kebab-case article/topic IDs in deck names, for example `AET::Article::vietnam-two-child-policy` and `AET::Topic::health-medicine`. Display titles may contain spaces inside card content, but deck identifiers should stay import- and AnkiConnect-friendly.
```

---

## Implementation Order

1. Create Rust workspace + Cargo.toml
2. `aet-core`: VocabEntry model + CSV parser
3. `aet-core`: ArticleMeta model + TOML parser
4. `aet-core`: Validator (IpaMode detection, controlled vocabulary checks)
5. `aet-anki`: basic TSV exporter
6. `aet-anki`: cloze TSV exporter (with fallback logic)
7. `aet-anki`: production TSV exporter (reference panel format — Decision D8)
8. `aet-cli`: `validate` command + session timer
9. `aet-cli`: `build` command
10. `aet-typst`: IpaMode-aware JSON serializer
11. `aet-typst`: Typst CLI invocation
12. Typst template: `vocab-pack.typ` (IPA-optional, Vietnamese, A4, category headings)
13. Sample datasets (both fixtures)
14. Tests
15. README (includes production card usage note — Decision D9)
