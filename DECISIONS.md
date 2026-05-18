
# AET — Decision Log

**Version:** v3.0 (research-updated)
**Status:** Locked for Phase 1 implementation

---

## Schema Decisions

### S1 — Column count: 19 columns total
Locked. 10 user-input columns + 9 Codex-filled columns.

### S2 — Duplicate fields deferred to v0.2
`duplicate_status`, `duplicate_of`, `canonical_id` dropped from v0.1 schema.
These fields are meaningless without the `aet dedupe` command. Adding them now creates 3 columns that will always be empty and add daily friction.

### S3 — Both fixtures required for Phase 1 acceptance
Dengue Bangladesh (~115 entries, IPA present) and Vietnam Two-Child Policy (111 entries, no IPA) are both required. Neither is optional. The IPA-optional rendering logic cannot be tested on a single fixture.

### S4 — Anki export filter: P1 + approved + anki_export=yes by default
`--all-priorities` flag overrides the default.
This prevents AI-drafted entries from polluting the Anki deck automatically.

### S5 — Production cards included in v0.1
`anki-production.tsv` is always generated (even when `my_sentence` is empty).
The production card is a **prompt card**, not a model-answer card. See D8.

### S6 — Session timer: any aet command, accumulated via temp file
Clock starts on the first `aet` command of the day.
Temp file: `/tmp/aet-session-YYYY-MM-DD.json`
No external crate — `std::time::Instant + std::fs` only.
Resets automatically at date change.

---

## Rust Implementation Decisions

### R1 — IpaMode enum naming (Bug fix)
Do NOT use `None`/`Some`/`All` — these conflict with Rust's `Option` prelude.

**Correct:**
```rust
pub enum IpaMode {
    Absent,   // all entries have empty ipa → render table without IPA column
    Mixed,    // some present, some empty → render column, show "—" for missing
    Present,  // all entries have ipa → render full IPA column
}
```

### R2 — IpaMode derived at runtime, not read from TOML
`has_ipa` in `article.toml [stats]` is for human reference only.
`IpaMode` must always be computed from the loaded `VocabEntry` rows after parsing.
The validator detects IpaMode and passes it to both the Anki and Typst exporters.
If `has_ipa` in TOML disagrees with the computed IpaMode, emit a warning (not an error).

---

## Anki Format Decisions

### A1 — All TSV files use 4 columns: front | back | extra | tags
**Research basis:** Ali Abdaal's Anki Masterclass (2024): the `extra` field is the most underused Anki feature. The back should be atomic (clean answer only). Context — source sentence, collocation pattern, topic — belongs in `extra` as a reference panel shown below the answer.

**Format:**
```
front <TAB> back <TAB> extra <TAB> tags
```

**Back:** clean answer only — definition + Vietnamese meaning.
**Extra:** source sentence + collocation pattern + topic tags.

This applies to all three TSV files (basic, cloze, production).

### A2 — Recommended Anki deck settings documented in README
Settings from Ali Abdaal Anki Masterclass (Oct 2024):

| Setting | Value |
|---|---|
| Steps (new cards) | `15 1440 8640` |
| Graduating interval | 15 days |
| Easy interval | 60 days |
| Max reviews/day | 9999 |
| New interval (lapse) | 70% |
| Leech threshold | 8 (tag only, never suspend) |
| New card order | Random |

These settings are not enforced by the CLI — they are a README usage note only.

### A3 — Production deck: suspend on import, unsuspend after basic maturity
Import all three TSV files from day 1.
In Anki: immediately suspend all new cards in the production deck after import.
Unsuspend a production card for a given term only after its basic card has been answered correctly 2+ times (Anki "young" status, interval > 1 day).
This is a README instruction, not a CLI feature.

---

## Learning Design Decisions (Research-Backed)

### D7 — `my_sentence` is a post-review log, not an import field

**Decision:** Do not require or expect `my_sentence` at import time. Leave it empty.

**Research basis:**
Nation (2001) identifies three stages in vocabulary acquisition: *noticing → retrieving → generating*. Productive use (generation) cannot precede receptive consolidation. Schmitt (2000): "vocabulary acquisition is incremental — words are not learned from single exposures." A 2022 controlled study (146 EFL students, 12 weeks) confirmed that sentence-writing tasks outperform gap-filling and definition-choosing for productive mastery — but only after sufficient receptive exposure.

**Implication for AET:**
- At import: `my_sentence` is empty. Validator warns but does not block.
- During Anki review: after 3–5 successful basic card recalls, the production card fires as a prompt.
- After a successful production attempt in a review session: optionally write the sentence back into the CSV. This is the `my_sentence` field.
- `my_sentence` is a *record of internalized production*, not a preparation exercise.

**Workflow rule:** Never fabricate a `my_sentence` at import. A fake sentence written before internalization is indistinguishable from copying the source sentence — it provides no learning value and creates false confidence.

### D8 — Production card back: reference panel, not model answer

**Decision:** The back of a production card shows reference material only. It does not show a pre-written model answer.

**Research basis:**
Swain's Output Hypothesis (1985, 2000): productive gains require *pushed output* — the learner must make real effort to produce language, notice the gap between their output and proficient usage, and revise. Showing a model answer immediately after the front prompt removes the push. The learner reads the model instead of producing.

**Correct production card format:**

```
FRONT:
Use "strain scarce resources" in an IELTS-style sentence about health-medicine.

BACK:
[Attempt your sentence before flipping]

EXTRA (reference panel):
Pattern:    strain scarce [resources/services/funds]
Definition: to put further pressure on supplies already at their limit
Source:     "The outbreak strained scarce resources in hospitals across Dhaka."
Topic:      health-medicine · government-policy
```

The learner attempts the sentence mentally or on paper *before* flipping. The extra panel is a check, not a model.

If `my_sentence` exists in the CSV (post-review log), it is shown in extra as:
```
Your previous sentence: {my_sentence}
```
This is a record of the learner's own past production — not an answer to copy.

### D9 — Production card activation: after receptive maturity, not at first import

**Decision:** Production cards are generated and exported from day 1. However, the learner should prioritize basic and cloze cards first, and only begin working production cards after a term has been successfully recalled 3+ times on basic/cloze cards.

**Research basis:**
Tofugu SRS guide and Migaku (2026): study recognition cards (L2→L1 or term→definition) first to build the receptive base. Add production cards (L1→L2 or prompt→production) after the word feels familiar. Recommended ratio: start at 70% recognition / 30% production, shifting toward more production as familiarity grows.

**Implementation in AET v0.1:**
AET cannot track Anki review history. Generate all three TSV files from day 1. Import all three, but suspend production cards in Anki immediately after import. Unsuspend production cards for a term only after its basic card reaches "young" status (answered correctly 2+ times, interval > 1 day). Document this as a README usage note, not a CLI feature.

### D10 — Collocations and chunks take priority over single words at P1

**Decision:** When Codex produces the P1 shortlist (max 15 items), collocations and multi-word chunks must be prioritized over single words, even high-frequency ones.

**Research basis:**
- IELTS Lexical Resource (25% of Writing and Speaking scores) explicitly rewards collocation awareness at Band 7+. Band 7 descriptor: "uses less common lexical items with some awareness of style and collocation."
- Cambridge research (2025): collocational sophistication — specifically, collocations containing lower-frequency lemmas — is a significant positive predictor of expert LR ratings.
- LanguageCrush SRS guide: "When we speak a language fluently, it's because our brain has stored many 2–4 word chunks, NOT single words."
- 73% of IELTS candidates stuck at 6.5 fail due to unnatural word combinations, not vocabulary size (Cambridge Assessment research).

**Codex P1 shortlist rule:**
Prefer: verb-noun collocations → adjective-noun collocations → fixed academic phrases → reporting verbs → prepositional patterns → single words (only if highly transferable and AWL-listed).

Never P1: article-specific proper nouns, technical terms unlikely to reappear in IELTS, or words already in the learner's productive vocabulary.

### D11 — Daily P1 target: 10–15 items (revised down from 10–20)

**Decision:** Lower the recommended P1 daily target from "10–20" to "10–15."

**Research basis:**
SRS review debt compounds quickly. Migaku (2026): adding 10 new cards today creates ~40 reviews due in one week and ~70 in one month as cards return at mixed intervals. Each P1 item generates up to 3 Anki cards (basic + cloze + production). 15 items = up to 45 new cards per article. At 5–10 new cards per day (sustainable SRS rate), this is already a 5–9 day queue per article.

**Revised daily workflow:**
- Import P1 items: 10–15 per article maximum.
- Anki daily new card limit: 10 (adjust upward only when review queue is consistently clear by midday).
- If reading two articles per week: import only the top 10 P1 items from each.

### D12 — AWL alignment check in Codex P1 shortlist

**Decision:** When Codex generates the P1 shortlist, it should flag which items appear in Coxhead's Academic Word List (AWL) sublists 1–6. AWL items are cross-disciplinary and highly transferable to IELTS.

**Research basis:**
Coxhead (2000): AWL covers ~10% of all academic text tokens. AWL sublists 1–6 cover the vast majority of academic vocabulary encountered in IELTS materials. Learners who master AWL sublists 1–6 have knowledge of ~90% of words in academic texts (GSL + AWL combined).

**Implementation:** Codex marks each P1 candidate with `awl: sublist_N` or `awl: not_listed` in `suggested_p1_items.md` only. This does not change the `vocab.csv` schema.

### D13 — Edit cards during review; sync back weekly

**Decision:** Learners should edit Anki cards in-app during review sessions (press E). These edits should be synced back to `vocab.csv` during the weekly review workflow.

**Research basis:**
Ali Abdaal Anki Masterclass (2024): "Edit your flash cards as you go along. It takes 3 seconds but your future self will benefit enormously." Living cards that evolve with the learner's understanding outperform static cards imported once and never touched.

**AET v0.2 implication:** `aet sync` command — reads Anki note edits via AnkiConnect and writes them back to `vocab.csv`. Deferred to Phase 2. For v0.1: manual copy-back during weekly review is acceptable.

---

## Deferred Decisions (v0.2+)

| Decision | Deferred to |
|---|---|
| `aet sync` — AnkiConnect edit sync | v0.2 / Phase 2 |
| AnkiConnect direct import | v0.3 |
| `.apkg` generation | v0.3 |
| `aet serve` local web | Phase 5 |
| `tado-site` export | Phase 6 |
| Duplicate command + duplicate fields | Phase 2 |
| Master vocabulary across articles | Phase 3 |
| Grammar corrections schema | Phase 4 |
| FSRS vs SM-2 recommendation | v0.2 README note only |
