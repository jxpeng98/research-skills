# Stage B — Literature & Related Work (B1–B6)

This stage builds the *evidence base* for positioning: search → screening → extraction → mapping → related work narrative.

## Canonical outputs (contract paths)

- `B1` → `protocol.md`, `search_strategy.md`, `search_log.md`, `search_results.csv`, `dedup_log.csv`, `snowball_log.md`, `screening/`, `notes/`, `bibliography.bib`, `retrieval_manifest.csv`, `extraction_table.md`, `quality_table.md`, `synthesis_matrix.md`, `synthesis.md`
- `B1_5` → `literature/concept_extraction.md`
- `B2` → `notes/`, `bibliography.bib`, `retrieval_manifest.csv`
- `B3` → `snowball_log.md`, `search_results.csv`, `dedup_log.csv`
- `B4` → `manuscript/manuscript.md` (related work section)
- `B5` → `bibliography.bib`, `references.ris`, `references.json`
- `B6` → `literature/literature_map.md`

## Quality gate focus

- `Q4` (reproducibility baseline) starts here: search queries, inclusion/exclusion, and logs must be reproducible.
- For `systematic-review`, PRISMA consistency is enforced later (`G2`), but data should be prepared in B.

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

Builtin baseline expectation:
- `citation-graph` should first try to derive seed identifiers from `search_results.csv`, `bibliography.bib`, and `notes/` before requiring an explicit `target_paper_id`
- `metadata-registry` should treat `bibliography.bib` as the canonical export, but it may derive normalized reference state from `references.json`, `references.ris`, `search_results.csv`, and `notes/`

---

## B1 — Systematic Review Pipeline (PRISMA-style)

Use `B1` when you want an end-to-end pipeline. If you only need one component (e.g., only snowballing), use the corresponding task (`B3`, `E1`, etc.).

### Definition of done (minimum)

- A protocol statement exists (`protocol.md`) with scope + eligibility criteria
- `search_strategy.md` contains reproducible database queries (exact strings + limits)
- `search_log.md` records dates/timestamps + counts per source
- `search_results.csv` is a dedup-ready record table (one row per record)
- `dedup_log.csv` records merge/drop decisions and the basis for each dedup action
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
- Retrieval provenance is undocumented (later full-text decisions cannot be audited)
- Screening reasons are inconsistent or missing (PRISMA 2020 failure)
- Synthesis makes claims not supported by extracted evidence

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

Recommended note filename convention:
- `notes/{citekey}.md` (citekey derived from first author + year + keyword)

---

## B3 — Citation Snowballing

**Definition of done**
- `snowball_log.md` lists seeds + forward/backward expansions + dedup decisions
- `search_results.csv` is updated or append-ready for new snowballed candidates
- `dedup_log.csv` records which snowballed candidates were merged, dropped, or kept separate
- The corpus expands with a documented rationale (not “add everything”)

Suggested `snowball_log.md` table:

```markdown
| seed_citekey | direction (forward/backward) | candidate | decision | reason |
|---|---|---|---|---|
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
  - clusters/themes
  - representative papers per cluster
  - open problems per cluster

Write into: `literature/literature_map.md`.
