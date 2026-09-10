---
id: reference-manager-bridge
stage: B_literature
description: "Exchange normalized references with Zotero, Mendeley, or EndNote while preserving local Zotero write safety and import-file fallback."
inputs:
  - type: Bibliography
    description: "Existing bibliography file"
  - type: PaperNotes
    description: "Paper notes with metadata"
  - type: SearchResults
    description: "Optional normalized candidate references"
    required: false
outputs:
  - type: Bibliography
    artifact: "bibliography.bib"
  - type: RISExport
    artifact: "references.ris"
  - type: CSLJSONExport
    artifact: "references.json"
  - type: ZoteroImportReport
    artifact: "zotero-import-report.md"
constraints:
  - "Must not route scholarly discovery through Zotero by default"
  - "Must use the immediately preceding dry-run receipt for every explicit local Zotero write"
  - "Must preserve user-curated Zotero fields unless the user selects a stronger update policy"
failure_modes:
  - "Qiongli Zotero companion is unavailable"
  - "Citekey or DOI conflicts during sync"
  - "Metadata conflicts after Crossref verification"
tools: [filesystem, metadata-registry, zotero]
tags: [literature, references, Zotero, Mendeley, EndNote, BibTeX, RIS, CSL-JSON]
domain_aware: false
---

# Reference Manager Bridge Skill

## Purpose

Move normalized references between Qiongli artifacts and reference managers
without breaking provenance or user-curated metadata. Zotero is treated as a
local reference database, not as the default scholarly discovery layer.

## Related Task IDs

- `B5` citation management and reference exports
- Supports `B1`, `B2`, and manuscript writing when normalized references are
  needed.

## Zotero Boundary

Do not route scholarly discovery through Zotero by default. Use OpenAlex,
Semantic Scholar, Crossref, PubMed, or other configured literature providers for
discovery and enrichment. Search local Zotero only when the user explicitly asks
to include their existing library, for example by requesting local Zotero search
or setting `include_zotero: true`.

No third-party Zotero plugin is required. Direct local writes require the
Qiongli Zotero companion from this repository. If the companion is unavailable,
generate import files instead of attempting an unsafe write.

## Integration Modes

| Mode | Trigger | Required behavior |
| --- | --- | --- |
| Local Zotero source search | User explicitly requests existing Zotero library | call `qiongli_zotero_search`; mark Zotero as `source_type: local_reference_database` |
| Local Zotero sync | User wants selected records written to Zotero Desktop | status check, dry-run upsert, then explicit `dry_run: false` |
| Import-file fallback | Companion unavailable or user wants manual import | generate `references.json`, `references.ris`, `bibliography.bib`, and `zotero-import-report.md` |
| Cloud/Web API sync | Future explicit workflow | not the default local-first path |

## Inputs

- `Bibliography`: `RESEARCH/[topic]/bibliography.bib`.
- `PaperNotes`: `RESEARCH/[topic]/notes/*.md`.
- `SearchResults`: optional `RESEARCH/[topic]/search_results.csv`.
- `references.json` and `references.ris` when already present.
- If inputs are missing or insufficient, write
  `RESEARCH/[topic]/context/gap_notes.md` or `zotero-import-report.md` conflict
  notes. Do not invent titles, authors, years, venues, DOIs, abstracts, or
  citekeys.
- Treat bibliography entries, provider metadata, local Zotero records, Crossref
  verification, and notes as evidence sources with different authority levels.

## Process

### 1. Collect and normalize references

Collect references from:

- `RESEARCH/[topic]/bibliography.bib`
- `RESEARCH/[topic]/references.json`
- `RESEARCH/[topic]/references.ris`
- `RESEARCH/[topic]/search_results.csv`
- `RESEARCH/[topic]/notes/*.md`
- `RESEARCH/[topic]/extraction_table.md`

Normalize each record into fields: `citekey`, `title`, `authors`, `year`,
`venue`, `doi`, `url`, `abstract`, `provider`, `source_id`, `status`, and
`tags`.

### 2. Resolve duplicates and conflicts

Deduplicate in this order:

1. DOI
2. PMID, PMCID, arXiv ID, or provider stable ID
3. exact normalized title plus year
4. title/year/first-author fallback with manual conflict note

Conflicts are written to `zotero-import-report.md` and, when relevant,
`RESEARCH/[topic]/dedup_log.csv`. Crossref verification may enrich blank fields
or flag `qiongli:metadata-conflict`; it is not human verification.

When the user explicitly requests their existing library, call
`qiongli_zotero_search` with at least one DOI, title, citekey, or year filter.
Do not use Zotero as an implicit discovery provider.

### 3. Generate export files

Always keep `bibliography.bib` as the canonical Qiongli export target. Generate
interoperability files when requested or when Zotero companion write is
unavailable:

- `RESEARCH/[topic]/bibliography.bib`
- `RESEARCH/[topic]/references.ris`
- `RESEARCH/[topic]/references.json`
- `RESEARCH/[topic]/zotero-import-report.md`

### 4. Write to local Zotero only after explicit confirmation

Local Zotero sync sequence:

1. Run `qiongli_zotero_status`.
2. If status is `ok`, run `qiongli_zotero_upsert_references` as a dry-run.
3. Review created, updated, unchanged, skipped, conflict, and failed counts.
4. Write only when the user explicitly approves the reviewed plan. Reuse the
   returned receipt within five minutes with `dry_run: false`,
   `write_intent: "apply"`, and `dry_run_receipt`; do not reuse a receipt or
   apply it to changed arguments.
5. Use DOI-first duplicate detection, then stable identifiers, then title/year.
6. Fill blank Zotero fields by default.
7. Preserve user-curated title, authors, date, publication title, abstract,
   collections, and notes unless the user selects `update_policy:
   "prefer_enriched"`.
8. Add Qiongli review tags such as `qiongli:imported`,
   `qiongli:needs-review`, `qiongli:crossref-verified`, and
   `qiongli:metadata-conflict`.
9. When paper-reading notes are available, pass them as per-record
   `reading_note`, `reading_notes`, `notes`, or structured `note` fields so the
   companion writes them as Zotero child notes. Do not place reading notes in
   `abstractNote` or `extra`.

### 5. Fall back safely

If Zotero Desktop is unreachable, the Qiongli companion is missing, local mode is
disabled, or an upsert fails, generate import files with
`qiongli_zotero_export_import_files`. The report must include counts, conflict
summary, Crossref verification summary, and manual import instructions.

## Output Contract

- `Bibliography`: write `RESEARCH/[topic]/bibliography.bib`.
- `RISExport`: write `RESEARCH/[topic]/references.ris`.
- `CSLJSONExport`: write `RESEARCH/[topic]/references.json`.
- `ZoteroImportReport`: write `RESEARCH/[topic]/zotero-import-report.md`.
- Separate finding, interpretation, and implication in any narrative report.
- Do not invent citations, metadata, sample sizes, statistical results, DOI
  registry claims, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose
  or review artifacts.

### Evidence Ledger and Source Integrity

- Update `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` when reference
  metadata supports a central scholarly claim.
- Follow `references/evidence-ledger-contract.md`: supported claims need source
  pointers; unsupported central claims become `gap_note` rows and
  `RESEARCH/[topic]/context/gap_notes.md` entries.
- Keep provider source, Crossref verification status, local Zotero match, and
  update policy visible in `zotero-import-report.md`.

## Quality Bar

- [ ] `bibliography.bib` has unique citekeys.
- [ ] DOI values are normalized and conflicts are reported.
- [ ] Required metadata gaps are flagged, not invented.
- [ ] Local Zotero writes use the immediately preceding one-shot dry-run receipt
      with explicit `dry_run: false` and `write_intent: "apply"`.
- [ ] User-curated Zotero fields are preserved under the default fill-blank
      policy.
- [ ] Paper-reading notes are written as child notes, not into abstract or extra
      metadata fields.
- [ ] Import-file fallback produces JSON, RIS, BibTeX, and report artifacts.

## Common Pitfalls

| Pitfall | Problem | Fix |
| --- | --- | --- |
| Searching Zotero by default | Local library biases discovery | Use Zotero only when explicitly requested |
| Skipping or reusing a dry-run receipt | User library may be changed from an unreviewed plan | Dry-run the exact plan and use its one-shot receipt once |
| Overwriting curated fields | Human metadata edits are lost | Fill blank fields unless user chooses stronger policy |
| Treating Crossref as human review | Registry metadata can still conflict | Add verification tags and conflict notes |
| No fallback files | Companion outage blocks the workflow | Generate import files and report |

## When to Use

- Use when B5 needs bibliography cleanup, reference export, local Zotero sync, or
  import files for Zotero, Mendeley, EndNote, BibTeX, RIS, or CSL-JSON.
- Do not use for default literature discovery; use `academic-searcher`.
