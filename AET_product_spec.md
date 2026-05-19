# AET — Academic English Training

## Product Specification

**Status:** Planning / Prototype Ready
**Version:** v0.2 (enhanced)
**Primary user:** Thanh Tu Do
**Product type:** Local-first academic English learning pipeline
**Core idea:** Capture teacher-provided and self-produced academic English learning materials once, organize them into reusable learning objects, and export them into multiple study formats such as PDF, Anki, and later web/iPad study pages.

---

### Changelog from v0.1

| # | Change | Reason |
|---|---|---|
| 1 | Added Section 3.3 — Fixture Comparison | Both teacher PDFs now fully read; key structural differences documented |
| 2 | Added Vietnam `article.toml` example | Only Dengue was shown before; both fixtures must be spec-complete |
| 3 | Clarified `category` vs `ielts_topics` | These are two different fields; the spec conflated them |
| 4 | Added `collocation_pattern` notation guide | Variable-slot patterns were undocumented (e.g. `boost [a fertility rate]`) |
| 5 | Added type mapping for Vietnam-style categories | Verb-Noun, Adj-Noun categories map to existing `type` values; now explicit |
| 6 | Added IPA-optional handling in Typst spec | Vietnam vocabulary has no IPA; template must not break or show empty column |
| 7 | Added teacher PDF → CSV as primary Codex task | Codex section only mentioned article highlights, not the teacher reference PDF |
| 8 | Enhanced Risk 6 with concrete CLI mitigation | "15–20 min" was aspirational with no enforcement mechanism |
| 9 | Added section 13.5 — Session Timer | Structural enforcement of daily time budget |
| 10 | Fixed section ordering | Original had non-linear jumps and duplicated passages |
| 11 | Added `teacher_pdf` field to `article.toml` | Need to link each article to its teacher vocabulary reference |
| 12 | Noted `secondary_topics` for Vietnam fixture | Previously undocumented fixture metadata |

---

# 1. Vision

AET is a local-first academic English learning system designed to support intensive IELTS / Academic English study.

The class produces a large amount of valuable material:

- Daily high-quality articles from sources such as NYT, Economist, FT, WSJ, New Scientist, Foreign Affairs, etc.
- Teacher vocabulary references (structured PDFs — the primary Codex ingestion source).
- Lecture notes.
- Grammar exercises.
- Facebook group exercises.
- Google Classroom submissions and corrections.
- Academic topic vocabulary banks.
- Writing prompts and essay structures.
- Future IELTS Cambridge practice.

Without a system, this knowledge becomes scattered across PDFs, Google Drive, Facebook comments, Classroom submissions, Anki, notebooks, and memory.

AET turns this into a repeatable workflow:

```text
Read article → extract vocabulary/collocations → review duplicates → approve learning items → export PDF + Anki → revise and produce language
```

The goal is not only to store vocabulary. The goal is to help the learner move from passive recognition to active academic production.

```text
seen in article
→ understood
→ remembered
→ used in my sentence
→ used in paragraph
→ used in essay / speaking answer
```

---

# 2. Product Principles

## 2.1 Study workflow first, software second

AET should make the daily learning process easier. It should not become a large software project that distracts from IELTS study. The prototype should be deliberately small and useful immediately.

## 2.2 Local-first by default

Private learning data should stay local:

- Full article PDFs.
- Teacher vocabulary reference PDFs.
- Personal highlights.
- Personal sentences.
- Teacher corrections.
- Draft CSV files.
- Anki exports.

Public publishing to `tado-site` comes later and must use a sanitized public export.

## 2.3 Deterministic core, AI-assisted workflow

AET itself should remain deterministic:

```text
CSV / TOML / Markdown → validated data → PDF / Anki / JSON
```

AI should not be embedded directly into the core for the prototype. Instead, Codex or another agent can operate around the system:

```text
Codex reads article + teacher PDF → prepares draft CSV → checks duplicates → writes report → user reviews → AET validates/builds
```

## 2.4 Human approval before learning export

AI-generated entries should never go straight into Anki without review. All draft content should have a `review_status`. No row with `review_status = draft` should ever reach an Anki export.

## 2.5 Production over collection

The system should not encourage collecting thousands of words without using them. Each important vocabulary item should eventually require:

- A source sentence (from the article or teacher example).
- A collocation pattern.
- A learner-written sentence (`my_sentence`).
- An Anki card.
- Optional use in writing/speaking practice.

## 2.6 One source, many outputs

AET should follow the same philosophy as a report generation pipeline:

```text
structured learning data → renderer/exporter → many output formats
```

Outputs may include:

- Typst PDF (vocabulary reference, practice worksheet, weekly review pack).
- Anki TSV.
- Web JSON.
- MDX for blog/web.
- Local web practice pages.
- Future EPUB or printable workbook.

---

# 3. Scope

## 3.1 AET v0.1 Scope

AET v0.1 should handle only the article vocabulary pipeline.

### Inputs

- `article.toml`
- `vocab.csv`

### Outputs

- `vocabulary.pdf` using Typst.
- `anki-basic.tsv`
- `anki-cloze.tsv`

### Mandatory behavior

- Validate required fields.
- Warn, but do not block, when `my_sentence` is missing.
- Support Vietnamese text.
- Support IPA pronunciation (present or absent — see Section 11.4).
- Support topic tags.
- Support priority tags.
- Support duplicate metadata.
- Match teacher vocabulary PDF layout first before designing a custom style.

## 3.2 Explicit Non-Goals for v0.1

Do not build these yet:

- Full local web app.
- `tado-site` integration.
- SQLite database.
- `.apkg` generation.
- Built-in AI API calls.
- Google Drive integration.
- Facebook / Google Classroom scraping.
- Teacher correction database.
- Grammar exercise system.
- Spaced repetition scheduler.
- User accounts or cloud sync.

## 3.3 Fixture Comparison

Both v0.1 test fixtures are now fully documented. The two teacher PDFs reveal important differences that the spec must handle.

| Property | Dengue Bangladesh | Vietnam Two-Child Policy |
|---|---|---|
| Article source | NYT, Sept 25 2023 | NYT, June 4 2025 |
| IELTS topics | health-medicine, environment, urban-rural | family-relationships-teenagers, government-policy, business-economics |
| Teacher PDF entries | ~140 | 88 |
| Teacher categories | 8 (disease/health/climate/severity/people/symptoms/economy/reporting) | 7 (demographics/government/economy/verb-noun/adj-noun/reporting/phrases) |
| IPA present | Yes | No |
| Level | B2–C1 | B2–C1 (inferred) |
| Primary collocation types | collocations, fixed phrases, prepositional patterns | verb-noun, adjective-noun, fixed phrases, reporting verbs |

**Key implications:**

1. Both articles have far more entries than the P1 daily target of 10–20. Codex must aggressively filter P1 candidates before the user review step.
2. Vietnam has no IPA — the Typst template must render gracefully without an IPA column (see Section 11.4).
3. Category names are defined by the teacher per article, not by AET's IELTS topic taxonomy. These are two different systems (see Section 8.3).

---

# 4. Target Users

## 4.1 Primary user

A serious Academic English / IELTS learner attending an intensive teacher-led course.

Needs:

- Capture daily article vocabulary.
- Learn collocations and academic phrases.
- Reuse vocabulary in writing and speaking.
- Avoid losing teacher notes and corrections.
- Prepare for IELTS writing and speaking.
- Review from iPad, laptop, PDF, and Anki.

## 4.2 Secondary user (later)

Future public readers of `tado-site`.

Needs:

- Public academic English notes.
- Vocabulary explanations.
- Topic-based study posts.
- No copyrighted full articles.
- No private teacher notes.

## 4.3 AI / agent user

Codex or another coding/research agent.

Needs:

- Clear file structure.
- Stable schemas.
- Deterministic CLI commands.
- JSON or machine-readable validation output.
- Safe rules: draft only, do not overwrite approved data.

---

# 5. Content Types

AET will eventually handle multiple learning object types.

## 5.1 Article Vocabulary

Vocabulary, collocations, fixed phrases, academic reporting verbs, prepositional patterns, and example sentences extracted from a daily article — primarily sourced from the teacher vocabulary reference PDF.

Examples:

- `strain scarce resources`
- `overwhelmed hospital system`
- `declining fertility rate`
- `have little impact on`
- `preventive measures`

## 5.2 Topic Bank Vocabulary

Standalone vocabulary organized by academic topic. Initial topic taxonomy:

- `environment`
- `poverty-global-issues`
- `law-crimes`
- `business-economics`
- `health-medicine`
- `education`
- `family-relationships-teenagers`
- `urban-rural`
- `personality`
- `physical-appearance`
- `work-jobs`
- `government-policy`

`personality` and `physical-appearance` are intentionally separate topics so the topic bank does not grow around an ambiguous combined label.

## 5.3 Grammar Patterns

Writing grammar and sentence-structure learning units. Examples:

- Run-on sentence.
- Comma splice.
- Fragment.
- Stringy sentence.
- Choppy sentence.
- Misplaced modifier.
- Dangling modifier.
- Parallel structure.
- Subject-verb agreement.

## 5.4 Writing Prompts and Essay Structures

Future content type for:

- Argumentative essay.
- Discussion essay.
- Causes and effects essay.
- Causes and solutions essay.
- Effects and solutions essay.
- Advantages and disadvantages essay.

## 5.5 Teacher Corrections

Future high-value content type. For each correction:

- Original sentence.
- Corrected sentence.
- Error type.
- Teacher explanation.
- Learner rewrite.
- Related grammar rule.
- Related topic.

**Source:** Google Classroom submissions and teacher feedback. Requires a future Classroom export or manual copy workflow — not automated in v0.1. This can later generate a personal writing weakness report.

---

# 6. Data Architecture

## 6.1 Recommended Initial Folder Structure

```text
aet/
  Cargo.toml
  crates/
    aet-core/
    aet-anki/
    aet-typst/
    aet-cli/
  data/
    articles/
      vietnam-two-child-policy/
        article.toml
        vocab.csv
      dengue-bangladesh/
        article.toml
        vocab.csv
    vocab/
      health-medicine.csv
      business-economics.csv
      environment.csv
      population.csv
    inbox/
      2026-05-18-dengue-bangladesh/
        article.pdf
        teacher-vocab.pdf        ← teacher reference PDF (private)
        highlights.md
        draft_vocab.csv
        duplicate_report.md
  templates/
    typst/
      vocab-pack.typ
      practice-pack.typ
  dist/
    vietnam-two-child-policy/
      vocabulary.pdf
      anki-basic.tsv
      anki-cloze.tsv
```

## 6.2 Future Folder Structure

```text
aet/
  data/
    articles/
    topics/
    grammar/
    essays/
    corrections/
    ielts-tests/
    private/
    public-export/
```

## 6.3 Privacy Rules

These should never be committed to public GitHub:

```text
private/
inbox/
*.private.json
full-article.txt
teacher-notes.md
teacher-vocab.pdf
corrections/private/
```

Suggested `.gitignore`:

```gitignore
/data/private/
/data/inbox/
**/*.private.json
**/full-article.txt
**/teacher-notes.md
**/teacher-vocab.pdf
/dist/
```

---

# 7. Article Metadata Schema

File: `article.toml`

### 7.1 Dengue Bangladesh Example

```toml
id = "dengue-bangladesh"
title = "Deadly Dengue Fever Outbreak in Bangladesh Strains Scarce Resources"
source_name = "The New York Times"
author = "Saif Hasnat & Sameer Yasir"
date = "2023-09-25"
level = "B2-C1"
source_type = "article"
primary_topics = ["health-medicine", "environment", "urban-rural"]
secondary_topics = ["poverty-global-issues", "government-policy"]
skills = ["reading", "vocabulary", "writing-task2", "speaking-part3"]
teacher_pdf = "inbox/2023-09-25-dengue-bangladesh/teacher-vocab.pdf"
private = true
copyright_mode = "private-study-only"
```

### 7.2 Vietnam Two-Child Policy Example

```toml
id = "vietnam-two-child-policy"
title = "Facing a Demographic Shift, Vietnam Abolishes Two-Child Policy"
source_name = "The New York Times"
author = "Damien Cave"
date = "2025-06-04"
level = "B2-C1"
source_type = "article"
primary_topics = ["family-relationships-teenagers", "government-policy", "business-economics"]
secondary_topics = ["urban-rural", "poverty-global-issues"]
skills = ["reading", "vocabulary", "writing-task2", "speaking-part3"]
teacher_pdf = "inbox/2025-06-04-vietnam-two-child-policy/teacher-vocab.pdf"
private = true
copyright_mode = "private-study-only"
```

### 7.3 Required and Optional Fields

Required fields:

- `id`
- `title`
- `source_name`
- `date`
- `primary_topics`
- `skills`

Optional fields:

- `author`
- `level`
- `secondary_topics`
- `copyright_mode`
- `teacher_pdf` — path to the teacher vocabulary reference PDF (private, not exported)

---

# 8. Vocabulary CSV Schema

File: `vocab.csv`

## 8.1 Columns

```csv
id,term,ipa,category,type,definition_en,meaning_vi,source_sentence,my_sentence,collocation_pattern,ielts_topics,skill_use,difficulty,priority,card_types,origin,review_status,duplicate_status,duplicate_of,canonical_id,anki_export,tags
```

## 8.2 Field Definitions

| Field | Required | Description |
|---|---|---|
| `id` | Yes | Stable unique ID. Example: `health-001`, `pop-001` |
| `term` | Yes | Word, phrase, collocation, or fixed expression |
| `ipa` | No | IPA pronunciation. May be empty — see Section 8.5 |
| `category` | Yes | Teacher-style thematic heading from the vocabulary reference (see Section 8.3) |
| `type` | Yes | One of the values listed in Section 8.4 |
| `definition_en` | Yes | English definition |
| `meaning_vi` | Yes | Vietnamese meaning |
| `source_sentence` | Yes | Sentence from article or teacher example |
| `my_sentence` | No | Learner's own sentence. Warn if empty. |
| `collocation_pattern` | No | Pattern notation — see Section 8.5 |
| `ielts_topics` | Yes | Semicolon-separated canonical IELTS topic tags (see Section 8.3) |
| `skill_use` | Yes | Semicolon-separated skills |
| `difficulty` | No | `B1`, `B2`, `C1`, `C2` |
| `priority` | Yes | `P1`, `P2`, `P3` |
| `card_types` | Yes | `basic`, `cloze`, `reverse`, `production` |
| `origin` | Yes | `teacher`, `user`, `codex`, `manual` |
| `review_status` | Yes | `draft`, `reviewed`, `approved`, `rejected` |
| `duplicate_status` | No | `new`, `canonical`, `exact_duplicate`, `near_duplicate`, `variant`, `review_required` |
| `duplicate_of` | No | ID of duplicate item if applicable |
| `canonical_id` | No | Master vocabulary ID |
| `anki_export` | Yes | `yes` or `no` |
| `tags` | No | Free-form semicolon-separated tags |

## 8.3 Two Taxonomy Systems — `category` vs `ielts_topics`

These are two distinct fields that serve different purposes. Do not confuse them.

### `category` — Teacher-defined thematic heading

This is the section heading from the teacher vocabulary reference PDF. It is article-specific and defined by the teacher, not by AET. It controls how entries are grouped in the Typst PDF output.

Examples from Dengue Bangladesh:

```text
Outbreak, Disease & Public Health
Healthcare System & Hospitals
Climate, Environment & Geography
Scale & Severity
People, Refugees & Vulnerable Groups
Symptoms & Medical Effects
Economy, Supply & Response
Reporting Verbs & Academic Language
```

Examples from Vietnam Two-Child Policy:

```text
Demographic & Population Vocabulary
Government, Policy & Law Vocabulary
Economy & Society Vocabulary
Verb–Noun Collocations
Adjective–Noun Collocations
Reporting & Discussion Vocabulary
Useful Phrases & Expressions
```

**Rule:** Copy the teacher's category label exactly. AET does not normalize or map these. They exist only within one article's PDF output.

### `ielts_topics` — AET canonical topic tags

These are standardized tags from the AET topic taxonomy (Section 5.2). They exist independently of any article and are used for cross-article querying, deduplication, and topic bank building.

Example:

```csv
ielts_topics = "health-medicine;poverty-global-issues"
```

The same vocabulary item tagged `ielts_topics = "family-relationships-teenagers;government-policy"` will appear in the AET topic bank for that topic regardless of which article it came from.

## 8.4 Type Values and Mapping

The `type` field uses a controlled vocabulary. The teacher PDFs both use category names that map to these types:

| `type` value | Meaning | Teacher category examples |
|---|---|---|
| `word` | Single standalone word | any category |
| `noun_phrase` | Noun group functioning as a unit | Demographic & Population, Healthcare |
| `verb_phrase` | Verb + complement as a unit | Economy & Society |
| `collocation` | Strong word partnership (verb-noun or adj-noun) | Verb–Noun Collocations, Adjective–Noun Collocations |
| `fixed_phrase` | Multi-word expression used as a whole | Useful Phrases & Expressions |
| `prepositional_pattern` | Phrase built around a preposition | throughout all categories |
| `reporting_verb` | Verb used to attribute or introduce speech | Reporting Verbs & Academic Language, Reporting & Discussion |
| `sentence_frame` | A sentence-level pattern for imitation | Useful Phrases & Expressions |

When Codex converts a teacher PDF with a "Verb–Noun Collocations" category, it should set `type = collocation` for those entries.

## 8.5 Collocation Pattern Notation

The `collocation_pattern` field documents how a term is used structurally. Use square brackets `[ ]` for variable slots.

| Pattern style | Example |
|---|---|
| Fixed collocation | `strain scarce resources` |
| Verb + variable object | `boost [a fertility rate]` |
| Variable subject + verb | `[the outbreak] strained scarce resources` |
| Adjective + noun | `declining [fertility rate]` |
| Prepositional | `have little impact on [something]` |
| Verb + person + to-infinitive | `persuade [someone] to [do something]` |

When IPA is absent (Vietnam fixture), leave the `ipa` field empty. Do not fabricate IPA.

## 8.6 Example Rows

**Dengue Bangladesh (with IPA):**

```csv
health-001,strain scarce resources,/streɪn skeəs rɪˈzɔː.sɪz/,Healthcare System & Hospitals,collocation,to put further pressure on supplies or services that are already limited,gây áp lực lên các nguồn lực vốn đã khan hiếm,The outbreak strained scarce resources in hospitals.,Climate change can strain scarce public health resources in vulnerable countries.,strain scarce resources,health-medicine;poverty-global-issues,writing-task2;reading,C1,P1,basic;cloze;production,teacher,approved,canonical,,health-001,yes,public-health;crisis;resources
```

**Vietnam Two-Child Policy (no IPA):**

```csv
pop-001,declining fertility rate,,Demographic & Population Vocabulary,collocation,a fertility rate that is steadily decreasing,tỷ suất sinh đang giảm,The government is alarmed by the country's declining fertility rate.,Vietnam's declining fertility rate has prompted the abolition of the two-child policy.,declining [fertility rate],family-relationships-teenagers;government-policy,writing-task2;speaking-part3,B2,P1,basic;cloze;production,teacher,approved,canonical,,pop-001,yes,demographics;population;policy
```

---

# 9. Duplicate and Category Procedure

## 9.1 Why Duplicate Filtering Matters

Daily article study will create repeated vocabulary. Examples:

- `scarce resources`
- `limited resources`
- `resource shortage`
- `strain scarce resources`

If every repeated phrase goes into Anki as a new card, the deck becomes noisy and inefficient. AET must separate:

```text
new learning item
existing item with new example
near duplicate
collocation variant
same word but different meaning
```

## 9.2 Duplicate Classification

| Type | Meaning | Action |
|---|---|---|
| Exact duplicate | Same term, same meaning | Do not export a new Anki card |
| Near duplicate | Similar term, similar meaning | Link to canonical item; review manually |
| Collocation variant | Same base word, different useful pattern | Usually keep as separate item if useful for production |
| Same word, different meaning | Same surface form, different sense | Keep separate |
| Topic overlap | Same term belongs to multiple topics | Keep one canonical item, add topic tags |

## 9.3 Pre-Anki Export Rules

Only export to Anki if:

- `review_status = approved`
- `anki_export = yes`
- `priority = P1` by default
- `source_sentence` is not empty
- `duplicate_status` is not `exact_duplicate`
- `duplicate_status` is not `rejected`

Optional export policy:

```text
P1 → export all selected card types
P2 → export only basic/cloze if manually selected
P3 → archive only, no Anki
```

## 9.4 Master Vocabulary and Article Usage

Long-term design:

```text
data/vocab/*.csv          → canonical vocabulary records
data/articles/*/vocab.csv → article-specific usage and examples
```

For v0.1, one `vocab.csv` per article is acceptable. For v0.2+, introduce master vocabulary files and article references.

---

# 10. Anki Export Specification

## 10.1 Basic Card

Tests passive meaning.

Front:
```text
What does "strain scarce resources" mean?
```
Back:
```text
to put further pressure on supplies or services that are already limited
Vietnamese: gây áp lực lên các nguồn lực vốn đã khan hiếm
Example: The outbreak strained scarce resources in hospitals.
```

## 10.2 Cloze Card

Tests vocabulary in context.

Front:
```text
The outbreak {{c1::strained scarce resources}} in hospitals.
```
Back:
```text
strain scarce resources = to put pressure on already limited resources
```

## 10.3 Production Card

Tests active production. Implements Swain's Output Hypothesis (1985): the learner must attempt genuine output *before* seeing the reference. Showing a pre-written model answer removes the productive effort and eliminates the learning value.

**Research basis:** A 2022 controlled study (146 EFL students, 12 weeks) confirmed sentence-writing tasks outperform gap-filling and definition-choosing for productive mastery — but only when the learner genuinely attempts production, not when they read a provided answer.

**Card design:**

Front:
```text
Use "strain scarce resources" in an IELTS-style sentence about health or government policy.
```

Back (reference panel — NO model answer):
```text
Pattern:    strain scarce [resources / services / funds]
Definition: to put further pressure on supplies already at their limit
Source:     "The outbreak strained scarce resources in hospitals across Dhaka."
Topic:      health-medicine · government-policy
```

If `my_sentence` exists (written by learner in a previous review session):
```text
Your previous sentence:
[my_sentence]
```

**How to use this card:**
Before flipping, attempt the sentence mentally or write it on paper. Only then check the reference panel. The back is a check — not an answer to read.

**When production cards activate:**
Do not review production cards immediately after import. In Anki, suspend all production cards after import. Unsuspend a production card for a given term only after its basic card has been answered correctly 2+ times. This implements the receptive-before-productive principle (Nation 2001): recognition must precede production.

## 10.4 TSV Format

`anki-basic.tsv`:
```text
front<TAB>back<TAB>tags
```

`anki-cloze.tsv`:
```text
text<TAB>extra<TAB>tags
```

## 10.5 Recommended Anki Deck Settings

Apply these to the AET deck after first import. Without them, cards graduate too fast and the SRS algorithm does not work correctly for vocabulary acquisition.

**Source:** Ali Abdaal Anki Masterclass (2024) + Migaku SRS research.

| Setting | Value | Reason |
|---|---|---|
| Steps (new cards) | `15 1440 8640` | 15min → 1day → 6days before graduating |
| Graduating interval | 15 days | Prevents premature long-term scheduling |
| Easy interval | 60 days | Reserve "Easy" for truly mastered items |
| Max reviews/day | 9999 | Never cap — let the queue clear naturally |
| New interval (lapse) | 70% | Partial reset on forgetting, not full reset |
| Leech threshold | 8 | Tag as leech after 8 failures — don't suspend |
| New card order | Random | Avoids memorizing position rather than meaning |

**Production deck setup after import:**
1. Open `AET::Article::{article-id}` -> Production sub-deck.
2. Select all cards → Suspend.
3. After a basic card reaches "young" status (2+ correct recalls, interval > 1 day), unsuspend its matching production card by tag.

## 10.6 Deck Naming Convention

```text
AET::Article::vietnam-two-child-policy
AET::Article::dengue-bangladesh
AET::Topic::health-medicine
AET::Topic::family-relationships-teenagers
AET::Topic::government-policy
```

Use stable kebab-case IDs in deck names. Display titles can still appear in card content, but deck identifiers should remain friendly to Anki import workflows and AnkiConnect.

---

# 11. Typst PDF Specification

## 11.1 v0.1 Goal

Match the teacher vocabulary reference layout first. Structure:

- Header: `Academic English — Thầy Hà` or configurable `source_label`.
- Title.
- Article source metadata.
- Scope note.
- Thematic categories (from `category` field, grouped in order of first appearance).
- Table with columns: Term | IPA | Definition | Example | Vietnamese.

## 11.2 Required Features

- A4 page size.
- Vietnamese Unicode support.
- Category headings.
- Multi-page table support.
- Page numbers.
- Clean wrapping for long terms and examples.
- Optional total entry count footer.

## 11.3 Future PDF Types

- Vocabulary reference.
- Reading notes.
- Practice worksheet.
- Cloze worksheet.
- Weekly review pack.
- Topic bank pack.
- Grammar correction pack.

## 11.4 IPA-Optional Handling

The two teacher PDFs differ in IPA usage:

- **Dengue Bangladesh** — IPA present for all entries.
- **Vietnam Two-Child Policy** — No IPA column at all.

The Typst template must handle both gracefully:

```text
If ALL entries in the article have ipa = "" → render table without IPA column
If SOME entries have ipa and some do not → render IPA column, show "—" for missing entries
If ALL entries have ipa → render full IPA column
```

This must be a template-level decision, not a per-row decision. The `aet build` command should detect IPA presence at the article level and pass a boolean flag to the Typst renderer.

---

# 12. CLI Specification

Binary name:
```bash
aet
```

## 12.1 Commands for v0.1

### Validate

```bash
aet validate data/articles/dengue-bangladesh
```

Checks:

- Required files exist.
- Required CSV columns exist.
- Required fields are filled.
- IDs are unique.
- UTF-8 is valid.
- Topic tags are from the allowed taxonomy.
- Priority values are valid.
- Card types are valid.
- Warn on missing `my_sentence`.
- Warn on duplicate candidate terms.
- Detect IPA presence and output `ipa_mode: all | some | none` for use by the Typst renderer.

### Build

```bash
aet build data/articles/dengue-bangladesh
```

Produces:

```text
dist/dengue-bangladesh/vocabulary.pdf
dist/dengue-bangladesh/anki-basic.tsv
dist/dengue-bangladesh/anki-cloze.tsv
```

### Build Specific Output

```bash
aet build data/articles/dengue-bangladesh --only anki
aet build data/articles/dengue-bangladesh --only pdf
```

### Dedupe

```bash
aet dedupe data/articles/dengue-bangladesh/vocab.csv --against data/vocab
```

v0.1 deterministic checks only:

- Lowercase term match.
- Slug match.
- Same canonical ID.
- Same collocation pattern.

Codex handles semantic duplicate review outside AET.

## 12.2 Future Commands

```bash
aet query --topic health-medicine --priority P1
aet study-set --topics health-medicine,environment --output dist/week-01
aet export web data/articles/dengue-bangladesh
aet serve data/articles/dengue-bangladesh
aet report progress
```

---

# 13. Workflow

## 13.1 Daily Manual Workflow

```text
1.  Teacher uploads article.
2.  Teacher uploads vocabulary reference PDF.
3.  Save both PDFs into local inbox.
4.  Read article on iPad.
5.  Highlight useful words, collocations, fixed phrases, and strong sentences.
6.  Export highlights to Markdown or text.
7.  Ask Codex to convert teacher PDF → draft CSV (primary source).
8.  Ask Codex to merge highlights into any items missing from draft CSV.
9.  Ask Codex to compare draft CSV against existing vocabulary.
10. Review duplicate report.
11. Approve P1 items (target: 10–15 items — see Decision D11).
12. Leave my_sentence EMPTY — do not write sentences at import time.
13. Run aet validate.
14. Run aet build.
15. Import all three Anki TSVs (basic, cloze, production).
16. In Anki: suspend all production cards immediately after import.
17. Review basic and cloze cards daily.
18. After a basic card reaches "young" status (2+ correct recalls): unsuspend its production card.
19. When a production card fires: attempt the sentence mentally or on paper BEFORE flipping.
20. After a successful production attempt: optionally write the sentence back into vocab.csv as my_sentence.
```

**Note on step 7:** The teacher vocabulary reference PDF is the primary and most reliable input. It is already structured (Term / Definition / Example / Vietnamese). Codex should convert this first, not the article text. Highlights from step 5-6 supplement the teacher list, not replace it.

**Note on steps 12 and 18–20:** This implements the receptive-before-productive principle (Nation 2001, Schmitt 2000). Vocabulary acquisition is incremental — words are not learned from single exposures. Writing a sentence at import time (before any review cycles) produces a fake sentence that paraphrases the source without genuine internalization. `my_sentence` is a *post-review log*, written after the word has been genuinely recalled 3–5 times across Anki sessions.

**Why production cards are still included from day 1:** The card is generated at import and sits suspended. It becomes active only when the basic card matures. This means the production card fires at exactly the right moment — after recognition is established, before production is routine. A 2022 study confirms sentence-writing tasks produce significantly higher productive mastery than passive recognition tasks (p < 0.05), but only when the learner genuinely attempts production rather than reading a provided answer.

**Note on collocations vs single words (IELTS research):** Collocational control is a significant part of IELTS Lexical Resource, especially at B2-C1, where examiners reward natural combinations rather than isolated advanced words. The P1 shortlist should always prioritize verb-noun and adjective-noun collocations over isolated words, even AWL-listed single words. A smaller set of mastered collocations is usually more useful for IELTS Writing Band 7+ than a large list of isolated vocabulary.

## 13.2 What to Highlight on iPad

Highlight:

- Strong academic verbs.
- Noun phrases.
- Verb-noun collocations.
- Adjective-noun collocations.
- Reporting verbs.
- Prepositional patterns.
- Sentences useful for imitation.
- Phrases useful for Writing Task 2.
- Phrases useful for Speaking Part 3.

Do not highlight every unknown word. Prioritize reusable academic language.

## 13.3 Priority Rules

```text
P1 = must actively learn and export to Anki
P2 = useful, store and maybe review later
P3 = archive only
```

Recommended daily target:

```text
P1: 10–20 items per article
P2: unlimited archive
P3: article-specific or low-priority terms
```

Both current fixtures (Dengue: ~140 items, Vietnam: 88 items) significantly exceed the P1 target. Codex must propose a shortlist. The user makes the final P1 decision.

## 13.4 Weekly Review Workflow

Once per week:

```text
1. Query all P1 items from the week.
2. Find items missing my_sentence.
3. Write or improve personal sentences.
4. Export weekly review PDF.
5. Export combined Anki TSV.
6. Review duplicate clusters.
7. Promote useful P2 items to P1 if needed.
```

## 13.5 Session Timer

To enforce the daily time budget (see Risk 6), the CLI should track and display elapsed time:

```bash
$ aet build data/articles/dengue-bangladesh

[aet] Building dengue-bangladesh...
[aet] Validated 140 entries (15 P1, 80 P2, 45 P3).
[aet] Output: dist/dengue-bangladesh/vocabulary.pdf
[aet] Output: dist/dengue-bangladesh/anki-basic.tsv
[aet] Output: dist/dengue-bangladesh/anki-cloze.tsv
[aet] Done in 2.3s.
[aet] Session time since aet validate: 18 min. ✓ Within 20-min budget.
```

If total session time exceeds 20 minutes:

```bash
[aet] ⚠ Session time: 24 min. Consider stopping here and continuing tomorrow.
```

This is a display-only warning. It does not block the build.

---

# 14. Codex / AI Integration Strategy

## 14.1 Key Decision

Do not integrate AI directly into AET v0.1. Instead:

```text
AET = deterministic tool
Codex = external worker agent
User = final reviewer
```

This avoids building a complex AI subsystem too early.

## 14.2 Codex Responsibilities

Codex can help with:

**Primary task — teacher PDF conversion:**

- Parse the teacher vocabulary reference PDF structure (Term / IPA / Definition / Example / Vietnamese).
- Convert each row directly into an AET `vocab.csv` entry.
- Set `origin = teacher` for all converted entries.
- Infer `type` from the teacher's category name (see Section 8.4 mapping).
- Fill `ielts_topics` based on the article's primary topics.
- Leave `ipa` empty if the teacher PDF has no IPA column.

**Secondary task — highlights and supplement:**

- Extract candidate vocabulary from article PDF or highlights that the teacher PDF did not include.
- Set `origin = codex` for all extracted entries.
- Suggest definitions, Vietnamese meanings, IPA, collocations, IELTS topics, and priority levels.

**Review and quality tasks:**

- Check all draft entries against existing CSV files for duplicates.
- Produce a duplicate report.
- Generate a P1 shortlist recommendation.
- Generate draft `my_sentence` examples for review.
- Generate cloze card candidates.
- Prepare weekly study sets.

## 14.3 Codex Must Not

Codex must not:

- Overwrite approved master vocabulary without explicit instruction.
- Delete existing entries.
- Mark AI-generated rows as `approved` automatically.
- Push directly to Anki.
- Commit private article content to public GitHub.
- Publish full copyrighted articles to `tado-site`.
- Fabricate IPA — if the teacher PDF has no IPA, leave `ipa` empty.

## 14.4 Recommended Codex Task Prompt

```text
You are working in the AET Academic English Training repo.

Task:
You have two inputs in data/inbox/<article-id>/:
  1. teacher-vocab.pdf — the teacher vocabulary reference PDF (primary source).
  2. highlights.md — learner highlights from the article (secondary source).

Step 1: Convert teacher-vocab.pdf to AET vocab.csv format.
  - For each row in the teacher PDF, create one CSV entry.
  - Set origin = teacher.
  - Set review_status = draft.
  - Infer type from the teacher category heading (see type mapping in spec).
  - If the teacher PDF has no IPA column, leave ipa empty for all rows.
  - Fill ielts_topics from the article.toml primary_topics list.

Step 2: Add any terms from highlights.md not already covered.
  - Set origin = codex.
  - Set review_status = draft.

Step 3: Compare all draft terms against data/vocab/**/*.csv and data/articles/**/*.csv.

Produce:
  1. data/inbox/<article-id>/draft_vocab.csv
  2. data/inbox/<article-id>/duplicate_report.md
  3. data/inbox/<article-id>/suggested_p1_items.md (10–20 items maximum)

Rules:
  - Do not overwrite approved data.
  - Do not delete anything.
  - Mark all AI-generated rows as origin=codex and review_status=draft.
  - Mark uncertain duplicates as duplicate_status=review_required.
  - Only suggest P1 for highly reusable academic vocabulary or collocations.
  - Prefer collocations and sentence patterns over isolated rare words.
  - Keep full article text private.
  - Do not fabricate IPA.
```

## 14.5 Codex Review Output Format

`duplicate_report.md` should include:

```markdown
# Duplicate Report

## Exact duplicates
| Draft term | Existing ID | Recommendation |
|---|---|---|

## Near duplicates
| Draft term | Similar existing term | Existing ID | Recommendation |
|---|---|---|---|

## Collocation variants worth keeping
| Draft term | Base term | Reason |
|---|---|---|

## Review required
| Draft term | Issue |
|---|---|
```

`suggested_p1_items.md` should include:

```markdown
# Suggested P1 Items (max 20)

| Rank | Term | Type | Reason for P1 |
|---|---|---|---|
```

---

# 15. Daily Automation Plan

## 15.1 Manual v0.1 Daily Loop

```bash
# 1. Create article folder
mkdir -p data/inbox/2026-05-18-dengue-bangladesh

# 2. Put article.pdf, teacher-vocab.pdf, and highlights.md there

# 3. Ask Codex to convert teacher PDF and draft CSV
#    (use prompt from Section 14.4)

# 4. Review and move approved CSV
mkdir -p data/articles/dengue-bangladesh
cp data/inbox/2026-05-18-dengue-bangladesh/article.toml data/articles/dengue-bangladesh/
cp data/inbox/2026-05-18-dengue-bangladesh/reviewed_vocab.csv data/articles/dengue-bangladesh/vocab.csv

# 5. Validate
aet validate data/articles/dengue-bangladesh

# 6. Build
aet build data/articles/dengue-bangladesh
```

## 15.2 Semi-Automated v0.2 Daily Loop

```bash
aet new article dengue-bangladesh --date 2026-05-18
# Creates article folder and template files
aet validate data/articles/dengue-bangladesh
aet build data/articles/dengue-bangladesh
```

Codex still handles extraction and duplicate review.

## 15.3 Future Automation

Future command:

```bash
aet daily ingest --inbox data/inbox/2026-05-18-dengue-bangladesh
```

Possible behavior:

- Detect article PDF and teacher-vocab PDF.
- Detect highlights.
- Create article folder.
- Create empty `article.toml`.
- Run deterministic duplicate check.
- Build outputs after approval.

## 15.4 Daily Checklist

```text
[ ] Article saved locally
[ ] Teacher vocabulary PDF saved locally
[ ] Read article once for meaning
[ ] Highlight useful language on iPad
[ ] Codex draft CSV created from teacher PDF
[ ] Duplicate report reviewed
[ ] P1 list approved (10–20 items)
[ ] my_sentence written for top P1 entries
[ ] aet validate passed
[ ] PDF generated
[ ] Anki TSV generated
[ ] Anki import completed
[ ] Session time checked (< 20 min)
```

---

# 16. Web / iPad Roadmap

## 16.1 Public Blog Mode

Eventually export sanitized public content to `tado-site`.

Public content may include:

- Article summary in learner's own words.
- Topic explanation.
- Vocabulary list.
- Collocation examples.
- Writing ideas.
- Speaking questions.

Public content must not include:

- Full copyrighted article text.
- Teacher private notes.
- Teacher vocabulary PDF (private and may be copyrighted).
- Google Classroom corrections.
- Full paid-source screenshots.

Deferred Phase 6 rule: "sanitized" must be explicitly defined before public export is implemented. Until then, public export must treat teacher-provided definitions, teacher examples, full article sentences, private corrections, and paid-source passages as non-public by default. Public wording should be learner-authored or newly written from approved concepts, not copied verbatim from private/source material.

## 16.2 Private Local Web Mode

Future local app:

```bash
aet serve data/articles/dengue-bangladesh
```

Features:

- Reading notes.
- Vocabulary table.
- Flashcards.
- Cloze practice.
- Hide/show Vietnamese.
- Hide/show IPA (useful when no IPA is present).
- Mark learned.
- Local progress tracking.
- Export selected cards to Anki.

## 16.3 iPad Access Options

Phase 1:
```text
Run local server on laptop → access from iPad on same Wi-Fi
```

Phase 2:
```text
Use Tailscale to access local study server remotely
```

Phase 3:
```text
Offline PWA bundle with local progress storage
```

---

# 17. Roadmap

## Phase 0 — Planning and Dataset Preparation

Goals:

- Define schema.
- Prepare first two article datasets.
- Write worker brief.
- Confirm output expectations.

Fixtures:

- Vietnam two-child policy article. ✓ (88 vocab entries, no IPA)
- Dengue Bangladesh article. ✓ (~140 vocab entries, with IPA)

## Phase 1 — AET v0.1 Article Vocabulary Pipeline

Features:

- Rust workspace.
- `aet-core` data models.
- CSV/TOML parser.
- Validator (including `ipa_mode` detection).
- Anki TSV exporter.
- Typst PDF exporter (IPA-optional).
- CLI commands: `validate`, `build`.
- Session timer display in CLI.

Acceptance criteria:

```bash
# Given article.toml + vocab.csv, aet build creates:
vocabulary.pdf
anki-basic.tsv
anki-cloze.tsv

# Passes for BOTH fixtures:
aet build data/articles/dengue-bangladesh    # with IPA
aet build data/articles/vietnam-two-child-policy  # without IPA
```

Additional criteria:

- Vietnamese text renders correctly.
- IPA field is supported and optional.
- PDF renders without IPA column when no IPA present.
- Missing `my_sentence` generates warning only.
- Duplicate term warnings are shown.
- Session timer displayed in CLI output.

## Phase 2 — Codex-Assisted Draft Workflow

Features:

- Codex worker prompt (teacher PDF conversion as primary task).
- `inbox/` workflow with `teacher-vocab.pdf` slot.
- Duplicate report format.
- Draft CSV review process.
- P1 shortlist generation.
- More robust validation.

AET remains deterministic.

## Phase 3 — Topic Bank and Master Vocabulary

Features:

- Master vocabulary files by topic.
- Article-to-vocab references.
- Canonical ID system.
- Better duplicate handling.
- Query command.

Note: the Phase 1 CLI may offer `aet build-topic <topic>` as a read-only export over article fixtures. That does not start Phase 3 unless it introduces master vocabulary storage, canonical cross-article records, or dedupe/query workflows.

## Phase 4 — Grammar and Corrections

Features:

- Grammar pattern schema.
- Teacher correction schema (sourced from Google Classroom exports).
- Grammar Anki cards.
- Error frequency report.
- Writing weakness dashboard.

## Phase 5 — Local Web Study Mode

Features:

- `aet serve`.
- Local web UI.
- iPad-friendly layout.
- Flashcard and cloze practice.
- Local progress storage.

## Phase 6 — tado-site Public Export

Features:

- Public JSON export.
- MDX export.
- Astro content collection integration.
- Sanitized public academic English posts.

## Phase 7 — Advanced Automation

Features:

- Weekly study pack generation.
- Progress reports.
- AnkiConnect sync.
- Optional `.apkg` export.
- Optional cloud/local model integration.

---

# 18. Worker Agent Implementation Notes

## 18.1 Implementation Order

1. Create Rust workspace.
2. Define data models in `aet-core` (Article, VocabEntry).
3. Parse `article.toml`.
4. Parse `vocab.csv`.
5. Implement validator (including `ipa_mode` detection, topic tag validation).
6. Implement Anki basic TSV export.
7. Implement Anki cloze TSV export.
8. Implement Typst data rendering (IPA-optional template).
9. Implement PDF generation via Typst CLI.
10. Add sample datasets (both fixtures).
11. Add tests.
12. Add README.

## 18.2 Rust Crate Layout

```text
crates/
  aet-core/
    src/models/article.rs
    src/models/vocab.rs
    src/parser/csv.rs
    src/parser/toml.rs
    src/validator.rs
  aet-anki/
    src/basic.rs
    src/cloze.rs
    src/lib.rs
  aet-typst/
    src/lib.rs
    src/vocab_pack.rs
  aet-cli/
    src/main.rs
    src/commands/validate.rs
    src/commands/build.rs
    src/session_timer.rs
```

## 18.3 Suggested Rust Dependencies

- `clap` for CLI.
- `serde` for data models.
- `serde_json` for optional machine-readable output.
- `csv` for CSV parsing.
- `toml` for TOML parsing.
- `anyhow` or `miette` for errors.
- `camino` for UTF-8 paths if desired.
- `duct` or `std::process::Command` for Typst CLI invocation.
- `std::time::Instant` for session timer (no external crate needed).

## 18.4 Quality Requirements

- No silent data loss.
- Clear validation messages.
- Preserve UTF-8 Vietnamese.
- Preserve IPA characters when present.
- Do not reorder CSV rows unless explicitly requested.
- Do not overwrite existing output unless building to `dist/`.
- Machine-readable output should be available later with `--json`.

---

# 19. Risks and Mitigations

## Risk 1 — Scope creep

**Problem:** The curriculum is large, so the project can grow too quickly.

**Mitigation:** Keep v0.1 limited to article vocabulary → PDF + Anki. Do not touch Phases 2–7 until Phase 1 acceptance criteria pass.

## Risk 2 — Collecting too much vocabulary

**Problem:** Too many cards make Anki painful. Both current fixtures (88 and ~140 items) far exceed the P1 daily target.

**Mitigation:** Use priority levels. Export P1 by default. Codex must produce a max-20 P1 shortlist. The user must actively choose to elevate items above that list.

## Risk 3 — AI-generated low-quality entries

**Problem:** AI may create wrong definitions, unnatural examples, or bad Vietnamese translations. IPA is especially risky — Codex may fabricate plausible-looking but incorrect phonetics.

**Mitigation:** Use `origin`, `review_status`, and manual approval. Codex must not fabricate IPA. When the teacher PDF has no IPA column, all `ipa` fields must be left empty.

## Risk 4 — Duplicate explosion

**Problem:** Repeated vocabulary across articles creates noisy decks.

**Mitigation:** Use canonical IDs, duplicate reports, and pre-Anki export filters. The cross-fixture example: `strain scarce resources` (Dengue) and `declining fertility rate` (Vietnam) may share `ielts_topics` but are distinct items. Codex handles semantic review outside AET.

## Risk 5 — Copyright and privacy

**Problem:** Full article text and teacher notes should not be published publicly. Teacher vocabulary PDFs may also be private or under copyright.

**Mitigation:** Private local mode by default. Teacher vocabulary PDF is listed in `.gitignore`. Public export must be sanitized and must not include teacher PDF content verbatim.

## Risk 6 — Software distracts from learning

**Problem:** Building AET becomes more interesting than studying English.

**Mitigation — structural, not aspirational:**

1. **Daily time cap:** `aet build` displays session elapsed time and warns if it exceeds 20 minutes (Section 13.5). The warning is visible at every build.
2. **v0.1 hard freeze:** Do not touch Phase 2 code until both fixture articles have been processed end-to-end through the Phase 1 pipeline.
3. **Personal rule:** If a session involves more lines of Rust than lines of `my_sentence`, it was the wrong kind of session.
4. **Definition of Done gating:** Phase 1 is not done until at least 10 `my_sentence` entries exist across the two fixtures. Code quality does not substitute for learning output.

---

# 20. Definition of Done for Prototype

AET v0.1 is done when:

```text
Given one article folder containing article.toml and vocab.csv,
running `aet build <article-folder>` produces:
  1. vocabulary.pdf
  2. anki-basic.tsv
  3. anki-cloze.tsv
with all outputs generated from the same source data.
```

Additional acceptance criteria:

- Works with the Vietnam article dataset (no IPA — PDF renders correctly without IPA column).
- Works with the Dengue Bangladesh article dataset (with IPA).
- Supports Vietnamese text.
- Supports IPA text when present, gracefully omits when absent.
- Warns but does not block missing `my_sentence`.
- Filters Anki export using `priority`, `review_status`, `anki_export`, and duplicate fields.
- Generates a PDF close to the teacher vocabulary reference style.
- Session timer displayed in CLI output.
- At least 10 `my_sentence` entries exist across both fixture articles.

---

# 21. Final Product Direction

AET should become a coworker for academic learning tasks. It should help with:

```text
capture
organize
deduplicate
review
export
revise
reuse
```

But it should not replace the learner's role. The learner should still:

```text
read deeply
highlight intentionally
write original sentences
revise teacher corrections
practice speaking and writing
make final approval decisions
```

The best version of AET is not an AI chatbot. It is a clean local-first learning pipeline that lets AI agents help around the edges while preserving a deterministic, trustworthy core.

```text
AET core     = reliable system of record
Codex        = external preparation assistant (primary: teacher PDF conversion)
Anki         = memory system
Typst        = printable output (IPA-optional)
Local web    = iPad study interface
Tado-site    = future public publishing layer
```

This makes the system useful immediately for IELTS study and valuable long-term as a serious Rust + Typst + agent-friendly portfolio project.
