---
id: fulltext-fetcher
stage: B_literature
description: "Plan and record full-text retrieval through the fulltext-retrieval provider boundary with PRISMA-ready provenance."
inputs:
  - type: ScreeningDecisionLog
    description: "Papers requiring full-text retrieval"
  - type: SearchResults
    description: "Optional records with DOI, URL, provider IDs, or OA metadata"
    required: false
outputs:
  - type: FullTextStatus
    artifact: "screening/full_text.md"
  - type: RetrievalManifest
    artifact: "retrieval_manifest.csv"
constraints:
  - "Must route retrieval planning or resolution through fulltext-retrieval"
  - "Must record version, source provider, license, and not-retrieved reasons"
  - "Must avoid illegal or paywall-bypassing access instructions"
failure_modes:
  - "External resolver is unavailable"
  - "Only abstract or metadata is available"
  - "OA candidate link is broken or not a readable PDF"
tools: [filesystem, fulltext-retrieval]
tags: [literature, fulltext, open-access, retrieval, PRISMA, Zotero]
domain_aware: false
---

# Full-text Fetcher Skill

## Purpose

Plan and record full-text retrieval for B-stage review work. This skill owns
`retrieval_manifest.csv` and full-text status fields in `screening/full_text.md`.
It does not decide study eligibility; eligibility remains a `paper-screener`
decision.

## Related Task IDs

- `B1` systematic review pipeline
- `B2` targeted key paper reading

## Provider Ownership Boundary

`fulltext-retrieval` owns retrieval planning and resolver handoff.

The built-in provider is a planning stub: it can draft manifests, identify
locator gaps, and mark OA/manual follow-up candidates. Actual downloads usually
come from an external resolver such as Zotero, Unpaywall, CORE, arXiv, PMC, or a
publisher-hosted OA page.

Search providers can identify full-text candidates, but they do not prove that
the full text was retrieved or read. Treat `open_access_pdf_url` and
`access_url` as retrieval candidates until `retrieval_manifest.csv` records
`retrieved_oa`, `retrieved_preprint`, or a controlled `not_retrieved:*` status.

Do not overwrite `search_strategy.md`, `search_results.csv`, or
`bibliography.bib`. If retrieval evidence changes eligibility, update
`screening/full_text.md` and let `paper-screener` reconcile the decision.

## Inputs

- `ScreeningDecisionLog`: records needing full-text retrieval.
- Optional `SearchResults`: DOI, URL, arXiv ID, PMID/PMCID, provider IDs, and OA
  metadata.
- If inputs are missing or insufficient, write
  `RESEARCH/[topic]/context/gap_notes.md` or a manifest row with
  `not_retrieved:missing_locator`; do not invent locators.
- Treat metadata, abstracts, resolver responses, and local files as evidence
  sources with different evidence limits.

## Process

### 1. Build retrieval candidates

For each record, collect locators in priority order:

1. DOI
2. arXiv ID
3. PMID or PMCID
4. provider paper ID
5. access URL
6. title plus first author plus year

No locator means the row remains in `retrieval_manifest.csv` with
`not_retrieved:missing_locator`.

### 2. Resolve through `fulltext-retrieval`

Ask the provider layer to plan or resolve retrieval. Preserve the provider
result instead of rewriting it as a screening decision.

Allowed `source_provider` values include:

- `Zotero`
- `Unpaywall`
- `CORE`
- `arXiv`
- `PMC`
- `publisher_page`
- `manual_supplemental`
- `builtin_stub`

### 3. Write `retrieval_manifest.csv`

Minimum schema:

```csv
record_id,citekey,doi,retrieval_status,version_label,source_provider,retrieved_at,fulltext_path,access_url,license,notes
```

Controlled `retrieval_status` values:

- `retrieved_oa`
- `retrieved_preprint`
- `abstract_only`
- `not_retrieved:paywall`
- `not_retrieved:embargo`
- `not_retrieved:broken_link`
- `not_retrieved:not_found`
- `not_retrieved:access_restricted`
- `not_retrieved:needs_provider`
- `not_retrieved:missing_locator`
- `not_retrieved:oa_candidate`

Controlled `version_label` values:

- `published`
- `accepted`
- `submitted`
- `abstract_only`
- `metadata_only`
- `unknown`

### 4. Update full-text screening status

Mirror retrieval status into `RESEARCH/[topic]/screening/full_text.md` without
changing include/exclude decisions. For every non-retrieved report, preserve the
reason needed for PRISMA "reports not retrieved" counts.

### 5. Verify retrieved files when present

When a resolver returns a local file or URL, record whether it is readable, but
do not bypass paywalls or advise illegal access. Broken links, non-PDF error
pages, and unreadable files become `not_retrieved:broken_link` or
`not_retrieved:not_found` rows with notes.

## Output Contract

- `RetrievalManifest`: write `RESEARCH/[topic]/retrieval_manifest.csv`.
- `FullTextStatus`: update `RESEARCH/[topic]/screening/full_text.md`.
- Separate finding, interpretation, and implication in retrieval summaries.
- Do not invent citations, URLs, PDF paths, access rights, licenses, sample
  sizes, results, or full-text evidence.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose
  or review artifacts.

### Evidence Ledger and Source Integrity

- Update `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` only when
  full-text availability supports a central scholarly claim.
- Follow `references/evidence-ledger-contract.md`: supported claims need source
  pointers; unsupported central claims become `gap_note` rows and
  `RESEARCH/[topic]/context/gap_notes.md` entries.
- Preserve resolver name, source provider, version label, access URL, license,
  and retrieval timestamp.

## Quality Bar

- [ ] Every sought report has one `retrieval_manifest.csv` row.
- [ ] Every row uses a controlled `retrieval_status`.
- [ ] Every retrieved row has source provider, version label, timestamp, and
      access URL or local path.
- [ ] Every non-retrieved row has a specific `not_retrieved:*` reason.
- [ ] Built-in stub rows are not presented as completed downloads.
- [ ] No illegal or paywall-bypassing access instruction is present.

## Common Pitfalls

| Pitfall | Problem | Fix |
| --- | --- | --- |
| Treating stub output as PDF retrieval | Review overstates evidence access | Keep `not_retrieved:needs_provider` or `oa_candidate` |
| Losing version labels | Extraction may cite a preprint as final article | Record `published`, `accepted`, or `submitted` |
| Silent paywall failures | PRISMA counts cannot reconcile | Use `not_retrieved:paywall` |
| Eligibility edits in retrieval step | Screening decisions become unauditable | Update status only; let `paper-screener` decide |
| Missing license/source | Reuse rights are unclear | Preserve provider provenance |

## When to Use

- Use when B1 or B2 needs full-text retrieval status, PRISMA retrieval counts,
  or resolver handoff.
- Do not use for reference-manager export or Zotero library writes; use
  `reference-manager-bridge`.
