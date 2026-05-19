# AET

AET is a local-first Academic English / IELTS vocabulary pipeline. It turns reviewed article vocabulary into a Typst PDF and three Anki TSV exports while keeping private source material local.

## Current Phase

Phase 1 system work is in place: both fixture articles validate and build end to end.

Phase 1 learning proof still requires real learner output: at least 10 `my_sentence` entries across the fixtures before moving into Phase 2 work.

## Commands

Run from the repo root:

```powershell
cargo check --workspace --target-dir .codex-target
cargo test --workspace --target-dir .codex-target
cargo run -p aet-cli -- validate data/articles/vietnam-two-child-policy
cargo run -p aet-cli -- build data/articles/vietnam-two-child-policy
cargo run -p aet-cli -- validate data/articles/dengue-bangladesh
cargo run -p aet-cli -- build data/articles/dengue-bangladesh
cargo run -p aet-cli -- build-topic health-medicine
```

Typst must be installed and on `PATH` for PDF builds:

```powershell
typst --version
```

## Outputs

Article builds write to `dist/{article_id}/`:

```text
vocabulary.pdf
anki-basic.tsv
anki-cloze.tsv
anki-production.tsv
anki-import-guide.md
anki-templates/
```

Topic builds write to `dist/topics/{topic}/`.

## Learning Loop

1. Read the article for meaning.
2. Review the teacher vocabulary PDF.
3. Approve only 10-15 P1 items for Anki.
4. Import basic and cloze cards first.
5. Suspend production cards after import.
6. Unsuspend production cards only after the basic card matures.
7. Write `my_sentence` after genuine recall, not at import time.

## Anki Setup

Use stable kebab-case deck names, for example:

```text
AET::Article::vietnam-two-child-policy
AET::Article::dengue-bangladesh
AET::Topic::health-medicine
```

Recommended deck settings:

| Setting | Value |
|---|---|
| Steps | `15 1440 8640` |
| Graduating interval | `15 days` |
| Easy interval | `60 days` |
| New interval after lapse | `70%` |
| Leech threshold | `8` |
| New card order | `Random` |

## Privacy

Do not commit private article PDFs, teacher vocabulary PDFs, full article text, teacher notes, private corrections, or generated `dist/` output. Public export is a future phase and must define sanitization before implementation.
