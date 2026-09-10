---
id: paper-extractor
stage: B_literature
description: "Extract source-anchored paper notes and rollup tables while preserving evidence limits for metadata, abstract-only, and full-text records."
inputs:
  - type: ScreeningDecisionLog
    description: "Papers that passed screening"
  - type: FullTextAccess
    description: "Full-text PDFs, URLs, or retrieval manifest rows"
outputs:
  - type: ExtractionTable
    artifact: "extraction_table.md"
  - type: PaperNotes
    artifact: "notes/"
constraints:
  - "Must generate one note per included paper"
  - "Must preserve source_anchor and evidence_limit for extracted claims"
  - "Must mark unsupported fields as unsupported_gap"
failure_modes:
  - "Only metadata or abstract is available"
  - "Source paper does not report required method or result details"
  - "Extraction fields are inconsistent across papers"
tools: [filesystem, extraction-store]
tags: [literature, extraction, source-anchor, evidence-limit, structured-notes]
domain_aware: false
---

# Paper Extractor Skill

## Purpose

Extract included papers into source-anchored notes and a rollup table. This skill
separates what the source directly reports from what the project infers, and it
preserves evidence limits when only metadata, abstracts, or partial full text are
available.

## Related Task IDs

- `B2` targeted key paper reading
- `B1` systematic review pipeline
- Supports `E1`-`E5` synthesis and `F` writing tasks.

## Outputs (contract paths)

- `RESEARCH/[topic]/notes/{citekey}.md`
- `RESEARCH/[topic]/extraction_table.md`
- Optional `RESEARCH/[topic]/effect_size_table.md` when pooling is planned.

## Inputs

- Included records from `RESEARCH/[topic]/screening/full_text.md`.
- Retrieval status from `RESEARCH/[topic]/retrieval_manifest.csv`.
- Bibliography metadata from `RESEARCH/[topic]/bibliography.bib`,
  `references.json`, or `search_results.csv`.
- If inputs are missing or insufficient, write
  `RESEARCH/[topic]/context/gap_notes.md` and ask for the missing full text,
  metadata, or screening decision instead of inventing extraction content.
- Treat full text, abstracts, metadata, tables, figures, and notes as evidence
  sources with different limits.

## Process

### 1. Assign evidence limit before extracting

Every note and rollup row must include `evidence_limit`.

Allowed values:

- `full_text`
- `abstract_only`
- `metadata_only`
- `unavailable`

Extraction from `abstract_only` or `metadata_only` records must not infer sample
size, methods, findings, limitations, effect sizes, datasets, or theoretical
claims not visible in the available source.

### 2. Extract with source anchors

Every project-level extraction claim must include `source_anchor`.

Valid anchors:

- citekey plus page, section, table, figure, appendix, or quote ID
- abstract sentence or metadata field
- DOI/provider metadata field
- retrieval manifest row

Use `unsupported_gap` when the desired field is not available in the source.

### 3. Write one paper note per included study

Use `RESEARCH/[topic]/notes/{citekey}.md`.

Minimum note fields:

```markdown
---
citekey:
evidence_limit: full_text | abstract_only | metadata_only | unavailable
source_anchor:
---

# Paper Note

## Bibliographic Metadata
## Research Context
## Theory And Constructs
## Method Or Identification
## Dataset Or Source
## Findings
## Limitations
## Project Relevance
## Unsupported Gaps
```

Each substantive field should distinguish:

- `finding`: what the paper reports
- `interpretation`: what the authors or project infer
- `implication`: what this means for the current project

### 4. Write extraction rollup

`RESEARCH/[topic]/extraction_table.md` must preserve:

- citekey
- evidence_limit
- source_anchor
- theory/framework
- method or identification
- dataset/source
- sample or unit of analysis when available
- main finding
- effect size or qualitative theme when available
- limitations
- unsupported_gap fields

## Output Contract

- `PaperNotes`: write one note per included paper under
  `RESEARCH/[topic]/notes/`.
- `ExtractionTable`: write `RESEARCH/[topic]/extraction_table.md`.
- `EffectSizeInputs`: write `RESEARCH/[topic]/effect_size_table.md` only when
  pooling is planned and the source reports extractable effect data.
- Separate finding, interpretation, and implication in notes and rollups.
- Do not invent citations, source anchors, datasets, sample sizes, methods,
  results, effect sizes, limitations, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose
  or review artifacts.

### Evidence Ledger and Source Integrity

- Update `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` when extracted
  evidence supports a central scholarly claim.
- Follow `references/evidence-ledger-contract.md`: supported claims need source
  pointers; unsupported central claims become `gap_note` rows and
  `RESEARCH/[topic]/context/gap_notes.md` entries.
- Every rollup claim needs `source_anchor` and `evidence_limit`.

## Quality Bar

- [ ] Every included paper has one note under `notes/`.
- [ ] Every note and rollup row has `evidence_limit`.
- [ ] Every substantive extracted claim has `source_anchor`.
- [ ] Abstract-only and metadata-only records do not contain inferred full-text
      details.
- [ ] Missing fields are marked `unsupported_gap`.
- [ ] Rollup fields are consistent enough for synthesis.

## Common Pitfalls

| Pitfall | Problem | Fix |
| --- | --- | --- |
| Treating abstracts as full text | Extraction overclaims source support | Mark `abstract_only` and narrow fields |
| Missing source anchors | Later synthesis cannot verify claims | Add page, section, table, quote, or metadata field |
| Filling blanks from memory | Hallucinated methods or results | Use `unsupported_gap` |
| Mixed rollup schemas | Synthesis cannot compare papers | Use consistent columns |
| Author claim vs project inference blurred | Writing overstates evidence | Separate finding, interpretation, implication |

## When to Use

- Use after screening and retrieval when included papers need structured notes or
  an extraction table.
- Do not use for quality appraisal or synthesis; use `quality-assessor` and
  `evidence-synthesizer`.
