# Stage F — Manuscript Writing (F1–F6)

This stage turns artifacts into a publishable narrative: outline → draft → claims/evidence → figures/tables → abstract/title.

## Canonical outputs (contract paths)

- `F1` → `manuscript/outline.md`
- `F2` → `manuscript/manuscript.md` (section-level drafting)
- `F3` → `manuscript/manuscript.md`, `manuscript/results_interpretation.md`, `manuscript/effect_interpretation.md`
- `F4` → `manuscript/claims_evidence_map.md`
- `F5` → `manuscript/figures_tables_plan.md`, `manuscript/tables/`, `manuscript/figures/`
- `F6` → `manuscript/meta_optimization.md`

## Quality gate focus

- `Q1` (question-to-method alignment): intro/method/results must answer the same RQ(s).
- `Q2` (claim-evidence traceability): enforced via `F4` and `G3`.

## Cross-task depth rules

- Every substantive paragraph must move beyond description into at least two of: mechanism, comparison/tension, alternative explanation, boundary condition, implication.
- Qualitative findings should be interpreted as process, mechanism, or meaning claims, not left as theme labels plus quotes.
- If evidence only supports a narrow descriptive claim, write the narrow claim plainly; do not fake depth with abstract language.
- Related work and discussion should be organized around arguments, tensions, or explanatory structures rather than paper-by-paper listing.

---

## F1 — Manuscript Outline

**Definition of done**
- Section headings match venue norms
- Each section has bullet “promises” (what it will deliver)
- Results section mirrors analysis plan outputs

Write into: `manuscript/outline.md`.

---

## F2 — Single Section / Component Drafting

Use when you want to draft one component precisely (e.g., “intro gap paragraph”).

**Definition of done**
- The component has a clear rhetorical role (setup / gap / contribution / method / evidence / limitation)
- The component makes at least one analytical move beyond summary (mechanism, contrast, boundary, or implication)
- Citations are present where claims of prior work are made
- No new claims that contradict earlier artifacts

Write into: `manuscript/manuscript.md` (or a section placeholder within it).

---

## F3 — Full Draft

**Definition of done (minimum)**
- All required sections exist (title/abstract/intro/related work/method/results/discussion/limitations/conclusion)
- Methods contain enough detail for replication (given the artifact set)
- Results are consistent with analysis plan and reported with uncertainty
- Related work and discussion interpret tensions, mechanisms, and alternative explanations instead of paraphrasing sources or results
- Limitations discuss validity threats (not only “small sample”)
- The narrative states where claims stop: boundary conditions, contradictory cases, and inferential limits are explicit
- Result narration is externalized enough to support reuse in `manuscript/results_interpretation.md`
- Effect magnitude is translated into practical terms in `manuscript/effect_interpretation.md` when applicable

Write into:
- `manuscript/manuscript.md`
- `manuscript/results_interpretation.md`
- `manuscript/effect_interpretation.md`

---

## F4 — Claim–Evidence Map

This is the anti-overclaim tool: every major claim must trace to evidence (data, analysis, or citations).

**Definition of done**
- All major claims in abstract/introduction/discussion appear in the map
- Each claim has at least one evidence pointer
- Claims are typed (novelty / mechanism / empirical effect / robustness / synthesis)

Suggested table: `manuscript/claims_evidence_map.md`

```markdown
| claim_id | claim | claim_type | evidence | citation_keys | status (ok/weak/missing) | fix |
|---|---|---|---|---|---|---|
```

---

## F5 — Figures & Tables Plan

Plan visuals early so the narrative has “anchors”.

**Definition of done**
- List of planned figures/tables with:
  - purpose (what question it answers)
  - data source
  - caption claim (what it will show)
- Mapping from results sections → figures/tables

Write into: `manuscript/figures_tables_plan.md`.

Recommended pairing:
- `table-generator` for `manuscript/tables/`
- `figure-specifier` for `manuscript/figures/`

---

## F6 — Abstract & Title Optimization (Indexing / SEO / Reviewer Scanning)

**Definition of done**
- Title reflects: construct + setting + method + contribution (as appropriate)
- Abstract includes: problem, gap, method, main result(s), implication
- Keywords reflect both author terms and common index terms (without stuffing)

Write into: `manuscript/meta_optimization.md`.
