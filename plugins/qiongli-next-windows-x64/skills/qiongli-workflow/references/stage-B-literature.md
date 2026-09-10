# Stage B — Literature & Related Work (B1–B6)

This stage builds the *evidence base* for positioning: search → screening → extraction → mapping → related work narrative.

## Canonical outputs (contract paths)

- `B1` → `protocol.md`, `search_strategy.md`, `search_log.md`, `search_results.csv`, `dedup_log.csv`, `search_diagnostics.md`, `snowball_log.md`, `screening/`, `notes/`, `bibliography.bib`, `retrieval_manifest.csv`, `extraction_table.md`, `quality_table.md`, `synthesis_matrix.md`, `synthesis.md`
- `B1_5` → `literature/concept_extraction.md`
- `B2` → `notes/`, `bibliography.bib`, `retrieval_manifest.csv`, `literature/paper_reading_summary.md`, `literature/paper_reading_matrix.md`
- `B3` → `snowball_log.md`, `search_results.csv`, `dedup_log.csv`
- `B4` → `manuscript/manuscript.md` (related work section)
- `B5` → `bibliography.bib`, `references.ris`, `references.json`
- `B6` → `literature/literature_map.md`

## Quality gate focus

- `Q4` (reproducibility baseline) starts here: search queries, inclusion/exclusion, and logs must be reproducible.
- For `systematic-review`, PRISMA consistency is enforced later (`G2`), but data should be prepared in B.
- `B1` search quality is validated against `references/literature-search-quality-contract.md`; `search_diagnostics.md` is required before review-grade screening or systematic-review claims.

## Literature Provider Contract

Treat the literature stack as four coordinated layers, not one blob:

1. `scholarly-search`
   Owns `search_strategy.md`, `search_log.md`, `search_results.csv`
   Appends to `dedup_log.csv`
2. `citation-graph`
   Owns `snowball_log.md`
   Appends to `search_results.csv` and `dedup_log.csv`
3. `metadata-registry`
   Owns `bibliography.bib`
   Appends to `dedup_log.csv`
4. `fulltext-retrieval`
   Owns `screening/full_text.md` and `retrieval_manifest.csv`

If a workflow touches literature evidence, it should respect those ownership boundaries even when one runtime agent executes multiple steps.

Execution rule:
- literature discovery and retrieval should flow through MCP/provider adapters (`scholarly-search`, `citation-graph`, `metadata-registry`, `fulltext-retrieval`) rather than hard-coded direct web-tool calls inside skill prose
- manual spot checks are allowed, but they must be logged as supplemental evidence instead of becoming the default reproducible pipeline

Builtin baseline expectation:
- `citation-graph` should first try to derive seed identifiers from `search_results.csv`, `bibliography.bib`, and `notes/` before requiring an explicit `target_paper_id`
- `metadata-registry` should treat `bibliography.bib` as the canonical export, but it may derive normalized reference state from `references.json`, `references.ris`, `search_results.csv`, and `notes/`
- `fulltext-retrieval` should at least draft `retrieval_manifest.csv` and `screening/full_text.md` from local literature artifacts, even when actual PDF retrieval is delegated to an external resolver

---

## B1 — Systematic Review Pipeline (PRISMA-style)

Use `B1` when you want an end-to-end pipeline. If you only need one component (e.g., only snowballing), use the corresponding task (`B3`, `E1`, etc.).

### Definition of done (minimum)

- A protocol statement exists (`protocol.md`) with scope + eligibility criteria
- `search_strategy.md` contains reproducible database queries (exact strings + limits)
- `search_log.md` records dates/timestamps + counts per source, plus the provider or overlay used for each execution
- `search_results.csv` is a dedup-ready record table (one row per record)
- `dedup_log.csv` records merge/drop decisions and the basis for each dedup action
- `search_diagnostics.md` records mode-aware search quality checks, known-item recall, provider/query coverage, dedup ratio, coverage gaps, and next search actions
- `snowball_log.md` records citation-based expansions when used
- `screening/` contains title/abstract + full-text decisions and reasons
- `notes/` contains structured notes for included studies
- `retrieval_manifest.csv` tracks full-text provenance, version, and retrieval status
- `extraction_table.md` and `quality_table.md` cover all included studies
- `synthesis_matrix.md` supports transparent synthesis
- `synthesis.md` is consistent with extraction + quality tables
- `bibliography.bib` covers all included studies (and optionally key excluded background papers)

### `search_results.csv` minimal schema (recommended)

Use a *stable* schema so later steps can be automated.

```csv
record_id,source,query_id,retrieved_at,title,authors,year,venue,doi,url,abstract
```

Notes:
- `record_id`: stable within a project (e.g., `S2-000001`, `OA-000123`)
- `query_id`: tie back to the exact query string in `search_strategy.md`
- `retrieved_at`: ISO timestamp with timezone

### `dedup_log.csv` minimal schema (recommended)

Keep one row per deduplication decision:

```csv
candidate_record_id,canonical_record_id,decision,match_basis,resolver,notes
```

Notes:
- `decision`: `merge` / `drop_duplicate` / `keep_separate`
- `match_basis`: DOI / title-year-author / provider-id / manual-review
- `resolver`: human, builtin provider, or external MCP name

### `retrieval_manifest.csv` minimal schema (recommended)

Keep one row per full-text retrieval attempt:

```csv
record_id,citekey,doi,retrieval_status,version_label,source_provider,retrieved_at,fulltext_path,access_url,license,notes
```

Notes:
- `retrieval_status`: align with the controlled `fulltext_status` vocabulary
- `version_label`: published / accepted / submitted / abstract-only
- `source_provider`: Zotero, Unpaywall, CORE, arXiv, PMC, publisher page, etc.

### Screening logs (recommended tables)

**`screening/title_abstract.md`**

```markdown
| record_id | decision (include/exclude/uncertain) | primary_reason | notes |
|---|---|---|---|
```

**`screening/full_text.md`**

```markdown
| record_id | decision (include/exclude) | exclusion_reason (if any) | fulltext_status | notes |
|---|---|---|---|---|
```

Where `fulltext_status` uses a controlled set:
- `retrieved_oa` / `retrieved_preprint` / `abstract_only` / `not_retrieved:<reason>`

### Common failure modes

- Search strategy cannot be reproduced (missing exact query strings / limits / dates)
- Dedup is undocumented (later PRISMA counts cannot reconcile)
- Search diagnostics are missing or still show `known_item_missing`, `provider_undercoverage`, `query_too_narrow`, or `weak_screening_readiness` when the workflow claims review-grade coverage
- Retrieval provenance is undocumented (later full-text decisions cannot be audited)
- Screening reasons are inconsistent or missing (PRISMA 2020 failure)
- Synthesis makes claims not supported by extracted evidence

### Search diagnostics gate

Write `search_diagnostics.md` from templates/search-diagnostics.md after provider execution and deduplication. The audit command is:

```bash
python3 scripts/audit_literature_search_quality.py RESEARCH/[topic] --task-id B1
```

Mode-aware expectations:
- `systematic_review` / review-grade: at least two productive providers, no unresolved known-item misses, no zero-hit required concept blocks, and no weak screening readiness flag
- `targeted_search`: single-provider coverage or zero-hit concept blocks are warnings, but the artifact must not be used later as evidence of exhaustive coverage

### Coverage and access semantics

Discovery coverage and full-text access coverage are separate:

- `discovery coverage`: how broad and reproducible the metadata search was across providers, query variants, years, venues, document types, citation snowballing, and known-item recall.
- `full-text access coverage`: how many sought reports have a controlled `retrieval_manifest.csv` status such as `retrieved_oa`, `retrieved_preprint`, `abstract_only`, or `not_retrieved:<reason>`.
- `native_fulltext_queries`: platform-native LLM search queries that the active agent may execute to discover PDF, PMC, arXiv, repository, author-manuscript, or publisher full-text candidates. These outputs stay `candidate_only` until retrieval status is recorded.
- `Zotero attachment verification`: local Zotero attachment metadata that can distinguish citation-only Zotero matches from records with a local or linked PDF attachment.
- `evidence_limit`: what the workflow is allowed to claim from a record: `full_text`, `abstract_only`, `metadata_only`, or `unavailable`.

No search provider, native LLM search tool, or local Zotero library proves absolute completeness. There is no absolute completeness proof for Stage B. Review-grade claims require a reproducible search log, deduplication, known-item recall checks, citation snowballing where appropriate, Zotero attachment verification where available, and retrieval status for every included or sought report.

Provider JSON output can be materialized with `scripts/materialize_literature_search_bundle.py`, which writes `search_strategy.md`, `search_results.csv`, `search_log.md`, `dedup_log.csv`, and `search_diagnostics.md`.

---

## B1_5 — Concept / Keyword Extraction

Purpose: expand beyond the initial keywords to reduce confirmation bias.

**Definition of done**
- A concept list grouped into 2–5 “concept buckets”
- Synonyms, controlled vocabulary candidates (if relevant), and “near misses”
- A revised seed query that can be dropped into `search_strategy.md`

**Suggested structure: `literature/concept_extraction.md`**

```markdown
# Concept & Keyword Extraction

## Concept buckets
1. Concept A: synonyms...
2. Concept B: synonyms...

## Controlled vocabulary (if applicable)
- MeSH / ACM CCS / PACS / JEL: ...

## Revised query (draft)
(...) AND (...)
```

---

## B2 — Targeted Key Paper Reading

Use when you have 3–10 seed papers to bootstrap the project.

**Definition of done**
- `notes/` contains structured notes for each seed paper
- `bibliography.bib` has citekeys that match note filenames
- `retrieval_manifest.csv` records access status, source provider, version read, and retrieval limits
- `literature/paper_reading_matrix.md` compares seed papers by theory, method/identification, dataset/source, main finding, limitation, project relevance, source anchors, and evidence limit
- `literature/paper_reading_summary.md` organizes targeted reading into grounded themes, method/data patterns, stable single-paper or multi-paper findings, contradictions, gaps, writing-ready citation points, and uncertainty registers

Recommended note filename convention:
- `notes/{citekey}.md` (citekey derived from first author + year + keyword)

### Truthfulness boundary

B2 organizes targeted reading evidence; it does not create systematic-review-grade synthesis unless the upstream corpus and screening artifacts support that claim. Do not invent citations, page numbers, sample sizes, methods, results, effect sizes, datasets, author claims, or implications. If only metadata or an abstract is available, mark `evidence_limit: abstract_only` or `evidence_limit: metadata_only`, keep unsupported fields empty, and write missing claims as `unsupported_gap` entries.

Every project-level summary claim should carry:
- `source_anchor`: citekey plus section, page, table, quote ID, abstract, or metadata field
- `evidence_limit`: `full_text`, `abstract_only`, `metadata_only`, or `unavailable`
- `inference_strength`: `direct_evidence`, `reasonable_inference`, or `unsupported_gap`

---

## B3 — Citation Snowballing

**Definition of done**
- `snowball_log.md` lists seeds + forward/backward expansions + dedup decisions
- each round records `seed_selection_reason` and `saturation_status`
- `search_results.csv` is updated or append-ready for new snowballed candidates
- `dedup_log.csv` records which snowballed candidates were merged, dropped, or kept separate
- The corpus expands with a documented rationale (not “add everything”)

Suggested `snowball_log.md` table:

```markdown
| seed_citekey | seed_selection_reason | direction (forward/backward) | candidate | decision | reason | saturation_status |
|---|---|---|---|---|---|---|
```

---

## B4 — Related Work Writing

Related work should be *taxonomy/argument*-based, not chronological.

**Definition of done**
- A taxonomy with 3–6 clusters
- Positioning paragraph: “we differ because…”
- Claims are supported by citations that actually match the statement

Write into: `manuscript/manuscript.md` (related work section).

---

## B5 — Citation Management & Reference Exports

**Definition of done**
- `bibliography.bib` is clean (unique citekeys, required fields present)
- `references.ris` and `references.json` exist for tool interoperability

Basic integrity checks:
- No duplicate citekeys
- DOIs normalized (lowercase, no `https://doi.org/`)
- Venue/year fields present for all included studies

---

## B6 — Literature Mapping

Goal: map the field so your paper can claim novelty without hand-waving.

**Definition of done**
- A taxonomy/map with:
  - included studies
  - concept streams
  - representative papers per stream
  - evidence gaps and open problems per stream

Write into: `literature/literature_map.md`.
