# Skills Core Reference

Consolidated skill reference for token-efficient workflow execution. Use this file by default; only load full skill files (`skills/*/*.md`) for detailed output formats or error recovery.

---

## workflow-standard

**Purpose:** Keep tasks and outputs consistent across Codex, Claude Code, and Gemini

**Source of truth:** `standards/research-workflow-contract.yaml`

**Rule:** Always map user intent to a canonical Task ID (`A1`...`I8`) and write outputs to the contract path under `RESEARCH/[topic]/`.

---

## question-refiner

**Purpose:** Convert topic → structured research question

**Process:**
1. Ask clarifying questions (domain, scope, output type)
2. Apply framework: **PICO** (intervention) or **PEO** (non-intervention)
3. Generate structured RQ with sub-questions
4. Evaluate with FINER criteria (Feasible, Interesting, Novel, Ethical, Relevant)
5. Define inclusion/exclusion criteria + key search terms

**Output:** Structured RQ, FINER assessment, inclusion/exclusion criteria

---

## study-designer

**Purpose:** Turn RQ → executable empirical study design

**Process:**
1. Choose study type (experiment/quasi/observational/qual/mixed) based on claims + constraints
2. Define constructs → operationalization (IV/DV/measures or qualitative codes)
3. Specify sampling/recruitment + sample size strategy (power/MDE or saturation)
4. Draft data collection instruments and procedures
5. Pre-specify analysis plan (primary outcomes, models, missingness, robustness)
6. Plan validity/rigor + reproducibility (DMP + prereg optional)

**Templates:** `templates/study-design.md`, `templates/analysis-plan.md`, `templates/data-management-plan.md`

**Output:** `study_design.md`, `analysis_plan.md`, `data_management_plan.md` (+ optional instruments/prereg)

---

## ethics-irb-helper

**Purpose:** Ethics/IRB documentation bundle (not legal advice)

**Process:**
1. Identify participants, sensitive data, and risk level
2. Draft consent + recruitment + withdrawal policy
3. Specify privacy/security (minimization, access control, retention)
4. Draft manuscript-ready ethics + data availability statements

**Template:** `templates/ethics-irb-pack.md`

**Output:** `ethics_irb.md` (+ optional consent/recruitment scripts)

---

## academic-searcher

**Purpose:** Multi-database academic literature search

**Process:**
1. Build Boolean queries: `(concept1 OR synonym) AND (concept2 OR synonym)`
2. Search databases in priority:
   - Semantic Scholar API: `https://api.semanticscholar.org/graph/v1/paper/search`
   - arXiv API: `http://export.arxiv.org/api/query`
   - OpenAlex API: `https://api.openalex.org/works`
   - Web search for Google Scholar
3. Deduplicate by DOI → Title+Year+Author
4. Document results per database

**Fallback:** S2 → OpenAlex → Google Scholar

**Outputs (contract-aligned):** `search_strategy.md`, `search_log.md`, `search_results.csv`

---

## paper-screener

**Purpose:** Apply inclusion/exclusion criteria systematically

**Process:**
1. **Stage 1 (Title/Abstract):** Quick filter based on criteria
2. **Stage 2 (Full-text):** Detailed eligibility verification
3. Document decision + reason for each paper
4. Generate PRISMA flow data

**Decisions:** INCLUDE / EXCLUDE (+ reason) / UNCERTAIN

**Outputs (contract-aligned):** `screening/title_abstract.md`, `screening/full_text.md`, `screening/prisma_flow.md`

---

## paper-extractor

**Purpose:** Extract structured data from papers

**Extraction Framework:**
- Bibliographic: Title, Authors, Year, Venue, DOI
- Context: Problem, Gap, RQs/Objectives
- Theory: Framework, Key Concepts, Model
- Method: Design, Sample, Data Collection, Analysis, Validity
- Findings: Key Results, Effect Sizes, Themes
- Discussion: Interpretation, Implications
- Meta: Limitations, Future Research, Contributions

**Outputs (contract-aligned):** `notes/` (per-paper notes) + `extraction_table.md`

---

## quality-assessor

**Purpose:** Evidence quality evaluation

**A-E Rating:**
| Grade | Type | Examples |
|-------|------|----------|
| A | SR, Meta-analysis, RCT | Cochrane reviews |
| B | Cohort, Top venue papers | Nature/CHI |
| C | Case studies, Expert opinion | Conference papers |
| D | Preprints, Working papers | arXiv |
| E | Blog posts, Opinions | Non-academic |

**RoB Tool Selection:**
- RCTs → RoB 2
- Non-randomized → ROBINS-I
- Diagnostic → QUADAS-2
- Qualitative → CASP
- SRs → AMSTAR 2

**Output:** Quality grade + RoB assessment

---

## evidence-synthesizer

**Purpose:** PRISMA-ready evidence synthesis (narrative, qualitative, meta-analysis)

**Process:**
1. Decide synthesis type *per outcome* and justify (pool vs narrative vs qualitative)
2. Build/update synthesis matrix (themes × papers)
3. If meta-analysis is feasible:
   - Draft plan → `meta_analysis_plan.md`
   - Extract pooled-ready effect sizes → `effect_size_table.md`
   - Pool effects + assess heterogeneity (I²/τ²), sensitivity, missing-results bias
4. Integrate results into `synthesis.md` (+ optional `meta_analysis_results.md`)

**Templates:** `templates/meta-analysis-plan.md`, `templates/effect-size-extraction-table.md`, `templates/meta-analysis-report.md`

---

## gap-analyzer

**Purpose:** Identify research gaps from literature

**5 Gap Types:**
1. **Theoretical:** Conflicting/missing theories, undefined concepts
2. **Methodological:** Dominant methods with limitations, underutilized approaches
3. **Empirical:** Understudied contexts, settings, time periods
4. **Knowledge:** Unanswered questions, unexplored relationships
5. **Population:** Underrepresented demographics, missing stakeholders

**Process:** For each gap → Describe → Evidence → FINER prioritize → Suggest RQ

**Output:** Gap analysis table with priority (H/M/L)

---

## theory-mapper

**Purpose:** Map theoretical frameworks and relationships

**Process:**
1. Identify theories: Name, Origin, Core Proposition, Assumptions
2. Map constructs: Definition, Dimensions, Related Constructs
3. Map relationships: Direction, Strength, Type (causal/mediating/moderating)
4. Generate Mermaid diagrams for visualization
5. Create theory comparison matrix
6. Synthesize integrated framework

**Output:** Theory profiles, construct map, visual model, comparison matrix

---

## citation-snowballer

**Purpose:** Trace citations forward and backward

**Process:**
1. Select seed papers (high citations, seminal, recent reviews)
2. **Forward:** Who cites these? (S2 `/citations` endpoint)
3. **Backward:** What do these cite? (S2 `/references` endpoint)
4. Deduplicate against existing corpus
5. Score relevance and add high-value papers

**APIs:** Semantic Scholar, OpenAlex, Crossref

**Output:** Snowballing log with new unique papers

---

## fulltext-fetcher

**Purpose:** Retrieve OA full-text PDFs

**Resolution Priority:**
1. arXiv / PubMed Central (direct)
2. Unpaywall API: `https://api.unpaywall.org/v2/{doi}`
3. Semantic Scholar `openAccessPdf`
4. CORE API

**Status Codes:** RETRIEVED_OA | RETRIEVED_PREPRINT | ABSTRACT_ONLY | NOT_RETRIEVED (reason)

**Output:** Retrieval log with PRISMA-compliant "not retrieved" reasons

---

## metadata-enricher

**Purpose:** Normalize and complete paper metadata

**Process:**
1. Normalize DOI: `10.xxxx/example` (canonical)
2. Query Crossref/OpenAlex for missing fields
3. Generate citekey: `lastname[year]keyword`
4. Create dedup keys for matching

**APIs:** Crossref (bibliographic), OpenAlex (OA + bibliometrics)

**Output:** Enriched metadata + citekey + dedup keys

---

## citation-formatter

**Purpose:** Format citations in academic styles

**Styles:** APA 7th, MLA 9th, Chicago, IEEE, Harvard, BibTeX

**BibTeX Types:** @article, @inproceedings, @book, @incollection, @misc

**Citekey Format:** `lastname[year]keyword` (e.g., `smith2024machine`)

**Output:** Formatted citations, BibTeX entries, reference list

---

## reference-manager-bridge

**Purpose:** Export/import with Zotero, Mendeley, EndNote

**Formats:**
- BibTeX (.bib) - Zotero, Mendeley, JabRef
- RIS (.ris) - EndNote, Mendeley
- CSL-JSON (.json) - Zotero

**Tag Schema:** `project:[topic]`, `status:included|excluded`, `quality:A-E`

**Output:** Export files in multiple formats + import instructions

---

## manuscript-architect

**Purpose:** Draft and revise a full research paper (outline → draft → integrity passes)

**Process:**
1. Create manuscript workspace (`manuscript/outline.md`, `manuscript/manuscript.md`)
2. Draft sections iteratively (Intro → Related work → Methods → Results → Discussion → Limitations → Conclusion)
3. Run claim–evidence integrity pass + figures/tables pass
4. Prepare for readiness checks (reporting/PRISMA) and submission packaging

**Templates:** `templates/manuscript-outline.md`, `templates/manuscript-skeleton.md`, `templates/claim-evidence-map.md`, `templates/figures-tables-plan.md`

**Output:** `manuscript/` drafts + traceability maps

---

## reporting-checker

**Purpose:** Reporting guideline completeness check (empirical studies)

**Process:**
1. Identify study design and select guideline (CONSORT/STROBE/COREQ/SRQR/TRIPOD)
2. Map checklist items to manuscript sections
3. Generate prioritized missing-items fix list

**Template:** `templates/reporting-checklist.md`

**Output:** `reporting_checklist.md` (+ action items)

---

## submission-packager

**Purpose:** Submission-ready packaging (cover letter + statements + final checklist)

**Process:**
1. Confirm target venue constraints + anonymization needs
2. Run reporting checks (and PRISMA if SR)
3. Draft submission auxiliary materials + assemble submission checklist

**Templates:** `templates/cover-letter.md`, `templates/submission-checklist.md`, `templates/title-page.md`, `templates/highlights.md`, `templates/suggested-reviewers.md`, `templates/author-contributions-credit.md`, `templates/funding-statement.md`, `templates/coi-statement.md`, `templates/data-availability.md`, `templates/ai-disclosure.md`, `templates/supplementary-inventory.md`

**Output:** `submission/cover_letter.md`, `submission/submission_checklist.md`, `submission/title_page.md`, `submission/highlights.md`, `submission/suggested_reviewers.md`, `submission/author_contributions_credit.md`, `submission/funding_statement.md`, `submission/coi_statement.md`, `submission/data_availability.md`, `submission/ai_disclosure.md`, `submission/supplementary_inventory.md`

---

## rebuttal-assistant

**Purpose:** Reviewer response workflow (response matrix + response letter)

**Process:**
1. Atomize reviewer requests into a trackable table
2. Decide action per item (accept/partial/disagree) with evidence
3. Draft response matrix + letter with precise change locations

**Templates:** `templates/rebuttal-response-matrix.md`, `templates/rebuttal-letter.md`

**Output:** `revision/response_matrix.md`, `revision/response_letter.md`

---

## prisma-checker

**Purpose:** Verify PRISMA 2020 compliance

**Checks:**
1. Artifact completeness (12 required files)
2. Count consistency across documents
3. PRISMA checklist (40 items)
4. Best practices assessment

**Consistency Rules:**
- Pre-dedup total = Sum of all database results
- Screened = Included + Excluded
- Sought = Retrieved + Not Retrieved
- Included = Extracted = Assessed = Synthesized

**Output:** Compliance report with action items

---

## model-collaborator

**Purpose:** Multi-model collaboration for research code tasks

**Modes:**
1. **parallel**: Both models analyze, merge high-confidence conclusions
2. **chain**: One generates, other verifies (Codex → Gemini or reverse)
3. **role**: Task division (Codex: code gen, Gemini: explanation)
4. **single**: Single model execution

**Invocation:**
```bash
python -m bridges.orchestrator [mode] --prompt "..." --cwd "/path"
```

**Model Strengths:**
- Codex: 算法实现, Bug 修复, 代码生成
- Gemini: 代码解释, 架构分析, 文档生成

**Output:** Standardized JSON with confidence score

---

## code-builder

**Purpose:** Build academic research code (multi-discipline, multi-language)

**Domain Profiles:** `skills/domain-profiles/*.yaml` define discipline-specific libraries, method templates, diagnostics, and pitfalls. Available: finance, economics, psychology, biomedical, education, cs-ai, political-science, epidemiology, ecology-environmental. Add new domains via `domain-profiles/custom-template.yaml`.

**Languages:** Python, R, Stata, MATLAB, Julia (inferred from domain profile if not specified)

**Strategies:**
1. **Standard (Tier 1):** Use domain-profile recommended library + method checklist
2. **Advanced (Tier 2):** Methodological Decomposition (JAX/PyTorch/Custom MLE)

**Invocation:**
```bash
python -m bridges.orchestrator code-build \
  --method "GARCH" --domain finance --tier standard --lang python
```

**Output:** Implementation + validation test + domain checklist pass/fail + diagnostic plots

## stats-engine (enhanced)

**Purpose:** Statistical modeling with domain-specific method selection

**Key features (expanded):**
- **Method selection decision tree**: maps research goal × data structure → recommended model family
- **Domain diagnostics**: auto-loads from domain profile (e.g., PH test for biomedical, parallel trends for econ)
- **Cross-domain pitfall table**: common errors and fixes

**Output:** `analysis/stats_report.md` with method rationale, diagnostics, robustness, caveats

## code-review (enhanced)

**Purpose:** Domain-aware independent code review

**Key features (expanded):**
- **9 domain-specific review checklists** (economics, finance, psychology, biomedical, education, CS/AI, political-science, epidemiology, ecology)
- Each checklist targets the most common field-specific coding errors

**Output:** `code/code_review.md` with domain-specific checklist pass/fail

---

## self-critique

**Purpose:** Iterative red teaming and Socratic critique of outputs

**Process:**
1. Act as a harsh "Reviewer 2" or Socratic Questioner.
2. Ask stage-specific critique questions (e.g., claiming causality vs correlation, omitted variables, confirmation bias).
3. Challenge the Generator to defend or revise their work.
4. Ensure logical flow, empathy in rebuttals, and rigorous claims.

**Output:** Self-critique log and revised (improved) output

---

## API Quick Reference

| API | Base URL | Rate Limit |
|-----|----------|------------|
| Semantic Scholar | `api.semanticscholar.org/graph/v1` | 100/5min |
| arXiv | `export.arxiv.org/api` | Reasonable use |
| OpenAlex | `api.openalex.org` | 10/sec |
| Crossref | `api.crossref.org` | 50/sec (polite) |
| Unpaywall | `api.unpaywall.org/v2` | 100k/day |
| CORE | `api.core.ac.uk/v3` | Varies |
