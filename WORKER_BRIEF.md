# AET — Academic English Training
# Worker Agent Brief — v0.1 Prototype

Read this entire document before writing any code.
Read DECISIONS.md before writing any code.
Read /mnt/skills/user/rust-best-practices/SKILL.md before writing any Rust.
Read /mnt/skills/user/typst-author/SKILL.md before writing any Typst template.

---

## What You Are Building

A Rust CLI tool called `aet` that reads structured IELTS study data (one article folder)
and produces three output files: a vocabulary PDF, and two Anki TSV files.

This is a local developer tool. There is no web server, no database, no UI.
The user runs it from the terminal. An AI agent may also run it.

---

## Acceptance Criteria — v0.1 Is Done When

Given the test fixture at `data/articles/vietnam-two-child-policy/` (already populated):

1. `aet validate data/articles/vietnam-two-child-policy` exits 0 and prints a summary:
   ```
   ✓ article.toml — valid
   ✓ vocab.csv — 88 entries loaded
   ⚠ 88 entries missing my_sentence — add production practice sentences
   ```

2. `aet build data/articles/vietnam-two-child-policy` exits 0 and produces:
   ```
   dist/vietnam-two-child-policy/
     vocab.pdf          ← compiled by Typst
     anki-basic.tsv     ← tab-delimited, UTF-8
     anki-cloze.tsv     ← tab-delimited, UTF-8
   ```

3. `anki-basic.tsv` imports cleanly into Anki desktop without errors.

4. `anki-cloze.tsv` imports cleanly into Anki desktop with cloze deletions rendering correctly.

5. `vocab.pdf` renders all 88 entries grouped by category, matching the teacher's 4-column layout.

6. If `typst` is not in PATH, the build command prints:
   ```
   Error: typst not found in PATH.
   Install from: https://github.com/typst/typst/releases
   ```
   and exits with code 1.

7. All Vietnamese characters render correctly in both TSV and PDF outputs.

---

## Project Structure to Create

```
aet/
  Cargo.toml                    ← workspace root
  DECISIONS.md                  ← copy from bootstrap
  README.md                     ← short project description
  
  data/                         ← test fixture (copy from bootstrap)
    articles/
      vietnam-two-child-policy/
        article.toml
        vocab.csv
  
  templates/
    typst/
      vocab-pack.typ            ← Typst template (parameterised)
  
  dist/                         ← gitignored, build output
  
  crates/
    aet-core/
      Cargo.toml
      src/
        lib.rs
        models.rs               ← VocabEntry, ArticleMeta structs
        parser.rs               ← CSV + TOML parsing
        validator.rs            ← validation logic + warning collection
    
    aet-anki/
      Cargo.toml
      src/
        lib.rs
        basic.rs                ← generate basic card TSV rows
        cloze.rs                ← generate cloze card TSV rows
    
    aet-typst/
      Cargo.toml
      src/
        lib.rs
        renderer.rs             ← build .typ file, shell out to typst CLI
    
    aet-cli/
      Cargo.toml
      src/
        main.rs
        commands/
          validate.rs
          build.rs
```

---

## Data Models

### VocabEntry (from vocab.csv)

```rust
pub struct VocabEntry {
    pub id: String,                    // e.g. "pop-001"
    pub source_type: String,           // "article" | "topic"
    pub source_id: String,             // e.g. "vietnam-two-child-policy"
    pub term: String,
    pub category: String,              // teacher's grouping label
    pub entry_type: String,            // "noun_phrase" | "verb" | etc (CSV field: type)
    pub definition_en: String,
    pub meaning_vi: String,
    pub source_sentence: String,
    pub my_sentence: Option<String>,   // None if empty string in CSV
    pub collocation_pattern: String,
    pub ielts_topic: String,
    pub skill_use: Vec<String>,        // split on ";"
    pub difficulty: String,            // "B1" | "B2" | "C1" | "C2"
    pub card_types: Vec<String>,       // split on ";"
    pub status: String,                // "new" | "learning" | "mature"
    pub tags: Vec<String>,             // split on " "
}
```

### ArticleMeta (from article.toml)

```rust
pub struct ArticleMeta {
    pub id: String,
    pub title: String,
    pub source: String,
    pub author: String,
    pub date: String,
    pub level: String,
    pub topics: Vec<String>,
    pub skills: Vec<String>,
    pub subtitle: Option<String>,
}
```

### ValidationResult

```rust
pub struct ValidationResult {
    pub article_valid: bool,
    pub vocab_count: usize,
    pub missing_my_sentence: Vec<String>,  // list of entry IDs
    pub errors: Vec<String>,               // hard errors
    pub warnings: Vec<String>,             // soft warnings
}
```

---

## Anki TSV Output Specification

### anki-basic.tsv

Columns (tab-separated, no header row):
```
front <TAB> back <TAB> tags
```

Rules for generating each row from a VocabEntry:
- Only generate if `card_types` contains `"basic"`
- `front`: `What does "{term}" mean?`
- `back`: `{definition_en}\n\nVietnamese: {meaning_vi}\n\nExample: {source_sentence}`
  - If `my_sentence` is Some and non-empty, append: `\n\nYour sentence: {my_sentence}`
- `tags`: space-separated, all lowercase — combine `ielts_topic` + `difficulty` + `"aet"` + tags
  - Example: `population b2 aet demographics aging`

### anki-cloze.tsv

Columns (tab-separated, no header row):
```
text <TAB> back_extra <TAB> tags
```

Rules:
- Only generate if `card_types` contains `"cloze"`
- `text`: Take `source_sentence`, find the term in it (case-insensitive), wrap with `{{c1::term}}`
  - If term not found verbatim in source_sentence, use collocation_pattern instead
  - If neither found, skip this entry and emit a warning: `⚠ Could not generate cloze for {id}: term not found in source_sentence`
- `back_extra`: `{definition_en} | Vietnamese: {meaning_vi}`
- `tags`: same rule as basic cards

### Reverse cards
- v0.1: Do NOT generate reverse cards even if card_types contains "reverse"
- Emit one notice at build end: `ℹ Reverse cards skipped in v0.1 — will be added in v0.2`

### File encoding
- UTF-8, no BOM
- Line endings: LF (Unix)
- Tab character: \t (not spaces)

---

## Typst Template Specification

### Layout (match teacher's PDF exactly)

Page: A4, margins 20mm all sides

**Header block** (top of first page only):
```
[Bold, large] Academic English — Thầy Hà        [right-aligned small] Academic English — Thầy Hà
[Bold, medium] Academic Vocabulary & Collocations
Source: "{article title}"
{source_name} — {author}, {date}
```

**Category sections** (repeat for each unique category value in order of first appearance):
```
[Bold heading, full width, light background]  {N}. {Category Name}
```

Followed by a 4-column table:
| Term | Definition | Example | Vietnamese |
|---|---|---|---|
| Bold | Regular | Italic | Regular |

Column width ratios: 20% | 30% | 30% | 20%

Row alternating background: white / very light grey (#f8f8f8)
Table border: thin grey lines

**Footer** (every page):
```
Page N of M          [centered]          Academic English — Thầy Hà
```

**Last page footer addition:**
```
Total entries: {N} — organised into {M} thematic categories
```

### Template parameterisation

The `.typ` file must accept data injected by the Rust renderer, not hardcoded.
Use Typst's `#let` variables at the top of the file for all dynamic values.
The Rust renderer generates a complete `.typ` file with data embedded, then calls:
```
typst compile vocab-pack.typ dist/{article-id}/vocab.pdf
```

### Font requirements
- Headers: any clean serif (New Computer Modern or Liberation Serif)
- Body: any clean sans-serif (Noto Sans or Liberation Sans)
- Vietnamese characters: must render correctly — use Noto Sans which includes Vietnamese glyphs
- Do not use fonts that require external download at compile time

---

## CLI Interface

### Commands

```
aet validate <article-path>
aet build <article-path>
aet build --only anki <article-path>
aet build --only pdf <article-path>
```

### Output style

Use plain stdout. No colours required for v0.1 (but allowed).
Prefix lines with: `✓` (success), `⚠` (warning), `✗` (error), `ℹ` (info)
Machine-readable JSON output: `aet --json validate <article-path>` — optional stretch goal

### Exit codes
- 0: success (warnings are OK, they do not cause non-zero exit)
- 1: hard error (missing required file, parse failure, typst not found)

---

## Crate Dependencies

Only these crates are approved for v0.1. Do not add others without justification.

```toml
# aet-core
csv = "1"
serde = { version = "1", features = ["derive"] }
toml = "0.8"
anyhow = "1"
thiserror = "1"

# aet-cli
clap = { version = "4", features = ["derive"] }
anyhow = "1"

# aet-anki — no extra deps beyond aet-core
# aet-typst — no extra deps beyond std::process::Command + anyhow
```

---

## What NOT to Build in v0.1

- Do not build public.json / private.json export
- Do not build .apkg Anki package (TSV only)
- Do not build a web server or HTTP anything
- Do not build a grammar content type parser
- Do not build a topic bank / shared vocab system
- Do not build the query command
- Do not add a database (no SQLite)
- Do not add async (no tokio)
- Do not add logging frameworks (eprintln! is fine)
- Do not write reverse card generation (stub it with a notice)

---

## Test Fixture

The test fixture is already populated at:
```
data/articles/vietnam-two-child-policy/
  article.toml    ← 88 vocab entries, 2 writing prompts, 3 speaking questions
  vocab.csv       ← all 88 entries from teacher's PDF
```

Use this as the primary test case. All acceptance criteria must pass against this fixture.

Write at least one integration test that:
1. Parses the fixture vocab.csv
2. Asserts entry count == 88
3. Asserts pop-001 has term == "demographic shift"
4. Asserts entries with empty my_sentence produce warnings, not errors

---

## Definition of Done Checklist

Before marking v0.1 complete, verify:

- [ ] `cargo build --release` succeeds with zero warnings
- [ ] `cargo test` passes
- [ ] `aet validate data/articles/vietnam-two-child-policy` exits 0
- [ ] `aet build data/articles/vietnam-two-child-policy` exits 0
- [ ] `dist/vietnam-two-child-policy/vocab.pdf` exists and renders all 88 entries
- [ ] `dist/vietnam-two-child-policy/anki-basic.tsv` imports into Anki without errors
- [ ] `dist/vietnam-two-child-policy/anki-cloze.tsv` imports into Anki without errors
- [ ] Vietnamese characters are correct in all outputs
- [ ] Missing `typst` in PATH produces a clear error message
- [ ] No `unwrap()` calls in library code (use `?` and `anyhow`)
- [ ] README.md explains how to install typst, run validate, run build

---

*Brief version: 1.0 — locked for v0.1*
*Do not modify this brief during implementation — open a new brief for v0.2*
