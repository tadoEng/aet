# AET — Academic English Training
# Decision Log

This file records every design decision made before the prototype was built.
Worker agents must read this before making any structural changes.
Never override a decision here without updating this file and noting the reason.

---

## Project Identity

| Field | Value | Rationale |
|---|---|---|
| Project name | AET (Academic English Training) | Broad enough to grow beyond IELTS into the full curriculum |
| Binary name | `aet` | Short, typeable, professional |
| Repo name | `aet` | Matches binary |
| Workspace type | Cargo workspace with multiple crates | Separation of concerns; each exporter is independent |

---

## Scope — v0.1 Prototype

### In scope
- Parse `article.toml` + `vocab.csv` for one article
- Validate data and warn on missing `my_sentence` fields
- Export `vocab.pdf` via Typst
- Export `anki-basic.tsv`
- Export `anki-cloze.tsv`

### Explicitly out of scope for v0.1
- `public.json` / `private.json` (web export) — v0.2
- Grammar content type — v0.2
- Essay/writing prompt content type — v0.3
- Speaking card content type — v0.3
- Topic bank (shared vocab across articles) — v0.2
- `aet query` command — v0.2
- `aet serve` (local web) — v0.3
- tado-site / web integration — v0.3
- `.apkg` Anki package — v0.2 (TSV import is sufficient for v0.1)
- AnkiConnect sync — v0.3

---

## Data Decisions

### D1 — Vocabulary Entry ID Scheme
**Decision:** topic-prefix + zero-padded sequential integer
**Format:** `{topic_code}-{NNN}`
**Examples:** `pop-001`, `eco-007`, `law-042`

**Topic codes (initial set):**
| Code | Topic |
|---|---|
| `pop` | population / demographics |
| `eco` | economy / business |
| `env` | environment |
| `law` | law / crime |
| `hlt` | health / medicine |
| `edu` | education |
| `fam` | family / relationships / teenagers |
| `urb` | urban / rural |
| `per` | personality / physical appearance |
| `wrk` | work / jobs |
| `pol` | government / politics / policy |
| `soc` | society / social issues |
| `gen` | general / cross-topic |

**Rationale:** IDs must survive across articles. When "demographic shift" appears in a South Korea article next month, it maps to the same `pop-001` master record. Article-slug IDs would create duplicates. Term-slug IDs break on multi-word terms with special characters in filenames.

**Rule:** Once an ID is assigned and an Anki card is created from it, the ID is frozen. IDs are never reused even if an entry is deleted.

### D2 — Missing `my_sentence` Behavior
**Decision:** Warn but still export
**Implementation:** Validator collects all entries with empty `my_sentence`, prints a grouped warning after the build summary. Does not block export. The warning includes a count: `⚠ 72 entries missing my_sentence — add production practice sentences for best results.`

### D3 — CSV Encoding and Format
**Decision:** UTF-8, comma-delimited, quoted fields where commas appear inside values
**Rationale:** Vietnamese characters require UTF-8. Fields like `definition_en` and `source_sentence` frequently contain commas and must be quoted. The parser must handle RFC 4180 CSV correctly.

### D4 — PDF Layout
**Decision:** Match teacher's layout exactly for v0.1
**Spec:** 4-column table per category: Term | Definition | Example | Vietnamese
**Page size:** A4
**Header:** Article title + source line, matching teacher's PDF style
**Category headings:** Bold, full-width, above each table group
**Font:** Match teacher's PDF — clean serif for headers, sans-serif for table body
**v0.2:** Develop AET's own visual design

### D5 — Anki TSV Format
**Decision:** Tab-delimited, UTF-8, no BOM, two card types exported as separate files
- `anki-basic.tsv` — columns: `front`, `back`, `tags`
- `anki-cloze.tsv` — columns: `text`, `back_extra`, `tags`
**Deck naming convention:** `AET::Population::Vietnam Two-Child Policy` (derived from topic + article title)
**Tags format:** space-separated, all lowercase, hyphens for spaces: `population policy b2 vietnam`

### D6 — Source of Truth
**Decision:** `vocab.csv` is the single source of truth for vocabulary data. PDF and Anki are outputs, never inputs. If you need to correct data, edit the CSV and rebuild.

---

## Technical Decisions

### T1 — Rust Edition
**Decision:** Rust 2021 edition, stable toolchain

### T2 — Key Crates (approved for use)
| Crate | Purpose |
|---|---|
| `csv` | CSV parsing |
| `serde` + `serde_derive` | Struct deserialization |
| `toml` | article.toml parsing |
| `clap` | CLI argument parsing |
| `anyhow` | Error handling |
| `thiserror` | Custom error types in library crates |
| `std::process::Command` | Invoke Typst compiler |

**Do not add crates without updating this log.**

### T3 — Typst Integration Strategy
**Decision:** AET generates a `.typ` file, then shells out to the `typst` CLI to compile to PDF.
**Rationale:** Embedding Typst as a library is complex and unstable. Shell-out is simple, testable, and keeps the Typst template editable independently.
**Requirement:** Worker agent must check that `typst` is available in PATH and emit a clear error if not: `Error: typst not found. Install from https://typst.app`

### T4 — Error Philosophy
**Decision:** Errors in data (bad CSV, missing required field) are reported as structured warnings/errors to stdout, never panics. The process exits with code 1 on hard errors, 0 with warnings on soft errors (like missing `my_sentence`).

---

## File Path Conventions

```
aet/
  data/
    articles/
      {article-slug}/
        article.toml      ← required
        vocab.csv         ← required
        notes.md          ← optional, private
  templates/
    typst/
      vocab-pack.typ      ← Typst template (parameterised)
  dist/
    {article-slug}/
      vocab.pdf
      anki-basic.tsv
      anki-cloze.tsv
```

All paths are relative to the project root. The CLI always runs from the project root.

---

## What Is Intentionally Deferred

- SQLite migration (from CSV) — not until the query command is needed
- Web/JSON export — v0.2
- Grammar content type — v0.2
- Topic bank (shared vocab) — v0.2
- `.apkg` export — v0.2
- tado-site integration — v0.3
- AI-assisted sentence feedback — future

---
*Last updated: 2026-05-18 — all v0.1 decisions locked*
