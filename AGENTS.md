# AET Agent Onboarding

## Source of Truth

- `DECISIONS.md` is the locked Phase 1 decision log.
- `WORKER_BRIEF.md` is the implementation contract.
- `AET_product_spec.md` is broader product context and may be older where it conflicts with the two files above.

Do not start Phase 2 work until Phase 1 validates and builds both article fixtures end to end.

## Phase 1 Scope

AET v0.1 is a deterministic local CLI:

- Read `article.toml` and `vocab.csv`.
- Validate the locked 19-column CSV schema.
- Export `anki-basic.tsv`, `anki-cloze.tsv`, `anki-production.tsv`.
- Export `vocabulary.pdf` through the Typst CLI.
- Print session timer output for every command.

No built-in AI calls, AnkiConnect sync, web app, dedupe command, database, or public site export belongs in Phase 1.

`build-topic` exists as an early cross-article export convenience. Do not expand it into Phase 3 master vocabulary, dedupe, or canonical topic-bank storage until Phase 1 learning proof is signed off.

## Commands

Run from the repo root on Windows PowerShell:

```powershell
cargo check --workspace --target-dir .codex-target
cargo test --workspace --target-dir .codex-target
cargo run -p aet-cli -- validate data/articles/vietnam-two-child-policy
cargo run -p aet-cli -- build data/articles/vietnam-two-child-policy
cargo run -p aet-cli -- validate data/articles/dengue-bangladesh
cargo run -p aet-cli -- build data/articles/dengue-bangladesh
cargo run -p aet-cli -- build-topic health-medicine
```

Typst must be installed and available on `PATH` for PDF builds:

```powershell
typst --version
```

## Data and Privacy Rules

- Do not commit private article PDFs, teacher PDFs, full article text, teacher notes, or private corrections.
- Keep `data/inbox/`, `data/private/`, and generated `dist/` output out of Git.
- Do not fabricate IPA. If a source has no IPA column, leave `ipa` empty.
- Do not mark AI-generated rows as `approved` unless the user explicitly reviewed them.

## Serena

Use Serena for symbol-level navigation before editing Rust code:

- Activate this repo as the `aet` project.
- Prefer symbol overview and focused symbol reads over opening large source files.
- `.serena/` is local tool state and must not be committed.

## Current Fixture Note

`data/articles/dengue-bangladesh` is generated from the teacher vocabulary reference PDF and exercises the IPA-present path. `data/articles/vietnam-two-child-policy` exercises the IPA-absent path.
