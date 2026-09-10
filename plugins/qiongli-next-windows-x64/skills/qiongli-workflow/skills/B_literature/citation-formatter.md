---
id: citation-formatter
stage: B_literature
description: "Normalize bibliography metadata, citekeys, DOI fields, and export-ready citation files without inventing missing reference data."
inputs:
  - type: PaperNotes
    description: "Extracted paper metadata"
  - type: Bibliography
    description: "Existing bibliography file"
    required: false
outputs:
  - type: Bibliography
    artifact: "bibliography.bib"
constraints:
  - "Must keep bibliography.bib as the canonical export target"
  - "Must normalize DOI, venue, year, author, and citekey fields"
  - "Must flag missing required metadata instead of inventing it"
failure_modes:
  - "Duplicate citekeys for same author-year"
  - "Incomplete metadata prevents valid BibTeX or CSL output"
  - "Style requirements conflict with available metadata"
tools: [filesystem, metadata-registry]
tags: [literature, citations, BibTeX, APA, metadata-integrity]
domain_aware: false
---

# Citation Formatter Skill

## Purpose

Prepare citation metadata for writing and export. This skill normalizes
`bibliography.bib`, citekeys, DOI fields, and required reference metadata. It is
not a full citation style manual and does not replace `reference-manager-bridge`
for Zotero, RIS, or CSL-JSON exchange.

## Related Task IDs

- `B5` citation management and reference exports
- Supports `B1`, `B2`, `F` writing, and submission preparation.

## Inputs

- Paper notes: `RESEARCH/[topic]/notes/*.md`.
- Existing bibliography: `RESEARCH/[topic]/bibliography.bib`.
- Optional search or extraction metadata:
  `RESEARCH/[topic]/search_results.csv`,
  `RESEARCH/[topic]/extraction_table.md`,
  `RESEARCH/[topic]/references.json`.
- If inputs are missing or insufficient, write
  `RESEARCH/[topic]/context/gap_notes.md` or bibliography comments identifying
  missing required metadata. Do not invent authors, titles, venues, years, DOIs,
  page ranges, issue numbers, or publishers.
- Treat notes, provider metadata, DOI registry data, and user-curated
  bibliography entries as evidence sources with different authority.

## Process

### 1. Select canonical bibliography target

Use `RESEARCH/[topic]/bibliography.bib` as the canonical export target inside
Qiongli. Other formats are produced by `reference-manager-bridge`.

### 2. Normalize required fields

Normalize these fields before style-specific formatting:

| Field | Rule |
| --- | --- |
| citekey | `firstauthorYEARkeyword`, with suffix `a`, `b`, `c` for duplicates |
| DOI | lowercase DOI only; remove `https://doi.org/` and trailing punctuation |
| authors | preserve order; use `and` separator for BibTeX |
| year | four-digit year or `missing_year` conflict note |
| venue | use `journal` for articles and `booktitle` for proceedings |
| title | preserve original title; protect required capitalization only when needed |

### 3. Resolve duplicate citekeys

Duplicate handling:

1. Same DOI or same normalized title/year: merge metadata and keep one citekey.
2. Same citekey but different DOI/title: append suffix and write conflict note.
3. Missing DOI and ambiguous title/year: keep separate and mark
   `metadata_conflict`.

### 4. Flag missing required metadata

Every entry must report missing required metadata. Minimum required fields:

- `author`
- `title`
- `year`
- `journal` or `booktitle` for article/proceeding entries
- `doi` or `url` when available from source metadata

Use comments or a report section rather than fabricated values.

### 5. Apply target style lightly

For writing-facing checks, verify the selected style family only at the level
needed for consistency:

- author-date vs numeric citation mode
- bibliography sort order
- whether DOI URLs or bare DOI values are expected in final prose
- whether preprints need archive identifiers

Detailed journal-specific formatting belongs in submission or venue guidance.

## Output Contract

- `Bibliography`: write `RESEARCH/[topic]/bibliography.bib`.
- Optional metadata issue notes may be written to
  `RESEARCH/[topic]/bibliography_metadata_issues.md`.
- Separate finding, interpretation, and implication when citation metadata is
  used in narrative review notes.
- Do not invent citations, authors, titles, venues, years, DOIs, page ranges,
  publishers, datasets, sample sizes, statistical results, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose
  or review artifacts.

### Evidence Ledger and Source Integrity

- Update `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` when a reference
  metadata decision supports a central scholarly claim.
- Follow `references/evidence-ledger-contract.md`: supported claims need source
  pointers; unsupported central claims become `gap_note` rows and
  `RESEARCH/[topic]/context/gap_notes.md` entries.
- Preserve source anchors for DOI registry, provider metadata, note files, or
  user-curated bibliography entries.

## Quality Bar

- [ ] `bibliography.bib` has unique citekeys.
- [ ] DOI values are normalized and duplicate DOI entries are merged.
- [ ] Missing required metadata is flagged, not invented.
- [ ] Author, title, year, venue, DOI/URL, and entry type are consistent.
- [ ] Style-specific decisions are recorded without turning this skill into a
      full style manual.

## Common Pitfalls

| Pitfall | Problem | Fix |
| --- | --- | --- |
| Inventing missing fields | Bibliography looks clean but is false | Flag missing required metadata |
| DOI URL inconsistency | Dedup and exports fail | Normalize DOI values |
| Duplicate citekeys | LaTeX/Pandoc collisions | Merge or suffix with conflict notes |
| Over-formatting too early | Submission target may change | Keep canonical metadata clean |
| Treating style examples as source data | Citation content becomes fabricated | Use only source-backed metadata |

## When to Use

- Use when B5 needs citekey cleanup, DOI normalization, duplicate bibliography
  handling, or export-ready `bibliography.bib`.
- Do not use for local Zotero writes or import files; use
  `reference-manager-bridge`.
