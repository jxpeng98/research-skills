# Skills Core Reference

Optional consolidated digest. Prefer the relevant `skills/*/*.md` card for execution; read only the matching section here when a compact overview helps. Cards and canonical contracts own current inputs and outputs.

---

## workflow-standard

**Purpose:** Keep tasks and outputs consistent across Codex, Claude Code, and Antigravity

**Source of truth:** `standards/research-workflow-contract.yaml`

**Rule:** Infer the applicable Task ID (`A1`...`M7`) without asking for known context. Formal tasks write contract outputs under `RESEARCH/[topic]/`; a direct answer may stay in chat without claiming formal task completion. Follow the entrypoint scope, tool-availability and approval rules.

**Cross-cutting quality substrate:**
- Central claims go in `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` using `references/evidence-ledger-contract.md`.
- Unsupported central claims become `gap_note` rows and `context/gap_notes.md`, not invented citations.
- Final writing, proofread, submission, rebuttal, and presentation-facing outputs should apply `references/citation-risk-policy.md`.
- High-risk stage transitions should write `context/stage_handoff.md` using `references/stage-handoff-contract.md`.
- Stage C design work should produce or consume `design/method-diagnostic-report.md` and `design/validity-threat-matrix.md`.
- Writing Harness Contract applies to Stage F writing even when using only this core reference: lock the Story Spine before prose, then write in section or paragraph-cluster chunks with a write -> review -> confirm checkpoint. Do not draft the whole artifact in one uninterrupted pass; stop for the next blocking boundary/grill question when there is mainline drift, missing support, generic or vague claims, or an unsettled evidence threshold.

---

## question-refiner

**Purpose:** Convert topic → structured research question

**Process:**
1. Ask clarifying questions (domain, scope, output type)
2. Apply framework: **PICO** (intervention) or **PEO** (non-intervention)
3. Generate structured RQ with sub-questions
4. Evaluate with FINER criteria (Feasible, Interesting, Novel, Ethical, Relevant)
5. Define inclusion/exclusion criteria + key search terms

**Output:** `RQSet` (Structured RQ, FINER assessment)

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
7. Produce method diagnostics for construct validity, internal validity, external validity, statistical conclusion validity, measurement validity, data leakage, missingness, confounding, and selection bias

**Templates:** `templates/study-design.md`, `templates/analysis-plan.md`, `templates/data-management-plan.md`, `templates/method-diagnostic-report.md`, `templates/validity-threat-matrix.md`

**Output:** `DesignSpec`, `AnalysisPlan`

---

## data-management-plan

**Purpose:** Create FAIR-compliant data management plans

**Process:**
1. Define data types, formats, and metadata standards
2. Specify storage, backup, and security procedures
3. Detail access, sharing, and reuse policies
4. Address ethical and legal compliance

**Output:** `DataManagementPlan`

---

## data-dictionary-builder

**Purpose:** Generate structured variable codebooks

**Process:**
1. Parse raw data or variable descriptions
2. Define variable types, scales, and permitted values
3. Document missing data codes and transformations
4. Format into a canonical schema

**Output:** `DataDictionary`

---

## variable-operationalizer

**Purpose:** Map abstract constructs to measurable variables

**Process:**
1. Define theoretical constructs
2. Identify measurement instruments or proxy variables
3. Specify validation and reliability criteria
4. Document scoring algorithms or indices

**Output:** `OperationalizationMap`

---

## prereg-writer

**Purpose:** Generate OSF/AsPredicted preregistration documents

**Process:**
1. Ingest DesignSpec and AnalysisPlan
2. Format according to target registry template (e.g., OSF standard, AsPredicted)
3. Ensure exact mapping of hypotheses to statistical tests
4. Lock analysis decisions to prevent p-hacking

**Output:** `PreregistrationDoc`

---

## proposal-writer

**Purpose:** Draft research proposals, opening reports, prospectuses, and study plans before results-focused manuscript writing

**Process:**
1. Ingest framing artifacts (RQ, gap analysis, theory, contribution)
2. Ingest design artifacts (study design, analysis plan, DMP, optional venue/program expectations)
3. Draft rationale, literature gap, theory, RQs/objectives, expected contribution, methods, analysis, ethics, feasibility, timeline, and risks
4. Mark missing citations, data access, ethics status, sample-size rationale, and institutional requirements as gap notes
5. Keep planned finding, interpretation, and implication separate; do not write planned work as completed results

**Template:** `templates/research-proposal-template.md`

**Output:** `ResearchProposal` → `proposal/research_proposal.md`

---

## ethics-irb-helper

**Purpose:** Ethics/IRB documentation bundle (not legal advice)

**Process:**
1. Identify participants, sensitive data, and risk level
2. Draft consent + recruitment + withdrawal policy
3. Specify privacy/security (minimization, access control, retention)
4. Draft manuscript-ready ethics + data availability statements

**Template:** `templates/ethics-irb-pack.md`

**Output:** `EthicsPackage`

---

## academic-searcher

**Purpose:** Provider-backed literature search with reproducible diagnostics

**Process:**
1. Build `search_strategy.md` from RQ scope, concept blocks, translated
   provider queries, seed recall expectations, and dedup policy
2. Execute each query through `scholarly-search` or a compatible MCP/provider
   adapter; keep provider names, filters, timestamps, counts, and failures in
   `search_log.md`
3. Normalize retained rows into `search_results.csv` and append every merge,
   duplicate, or keep-separate decision to `dedup_log.csv`
4. Write `search_diagnostics.md` with provider coverage, concept coverage,
   known-item recall, dedup ratio, coverage gaps, and screening readiness
5. Block review-grade/systematic-review claims when diagnostics are missing,
   fewer than two productive providers are recorded, a known item remains
   missing, a required concept block has zero usable hits, or
   `weak screening readiness` is unresolved

**Outputs (contract-aligned):** `SearchQueryPlan`, `SearchResults`,
`SearchLog`, `DedupLog`, `SearchDiagnostics`

---

## paper-screener

**Purpose:** Diagnostics-aware title/abstract and full-text screening

**Process:**
1. Ingest `search_results.csv`, `search_diagnostics.md`, RQ scope, and
   inclusion/exclusion criteria
2. **Stage 1 (Title/Abstract):** record `INCLUDE`, `EXCLUDE`, or `UNCERTAIN`
   with reason codes and source anchors
3. **Stage 2 (Full-text):** verify eligibility only when retrieval status makes
   full text available; preserve `abstract_only` and `metadata_only` limits
4. Carry search diagnostic flags into screening notes instead of silently
   treating weak search coverage as eligible evidence
5. Generate PRISMA-ready counts that reconcile with search, dedup, retrieval,
   and screening logs

**Decisions:** INCLUDE / EXCLUDE (+ reason) / UNCERTAIN

**Outputs:** `ScreeningDecisionLog`, `FullTextScreening`, `PRISMAFlowData`

---

## paper-extractor

**Purpose:** Source-anchored extraction from included papers

**Extraction Framework:**
- Bibliographic: Title, Authors, Year, Venue, DOI
- Context: Problem, Gap, RQs/Objectives
- Theory: Framework, Key Concepts, Model
- Method: Design, Sample, Data Collection, Analysis, Validity
- Findings: Key Results, Effect Sizes, Themes
- Discussion: Interpretation, Implications
- Meta: Limitations, Future Research, Contributions

**Process:**
1. Use only included papers and their available source level
   (`full_text`, `abstract_only`, `metadata_only`, or `unavailable`)
2. Add `source_anchor` and `evidence_limit` to every extracted claim
3. Mark unavailable fields as `unsupported_gap`; do not infer methods,
   findings, samples, or limitations from metadata-only records
4. Keep per-paper notes and rollup rows synchronized

**Outputs:** `ExtractionTable`, `PaperNotes`

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

**Output:** `QualityTable`, `GRADESummary`

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

**Output:** `EvidenceTable`

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

**Output:** `GapAnalysis`

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

**Output:** `TheoreticalFramework`

---

## citation-snowballer

**Purpose:** Expand the corpus through citation-graph tracing

**Process:**
1. Select seed papers with explicit rationale and source anchors
2. Trace forward and backward citation edges through `citation-graph`,
   `scholarly-search`, or a compatible MCP/provider adapter
3. Record provider, seed ID, edge direction, retrieval time, hit counts,
   relevance rationale, and saturation status in `snowball_log.md`
4. Deduplicate additions against the current corpus and append merge/drop/keep
   decisions to `dedup_log.csv`
5. Feed accepted records back into `search_results.csv` with provenance

**Outputs:** `SnowballLog`, `SearchResults`, `DedupLog`

---

## fulltext-fetcher

**Purpose:** Retrieve and log full-text availability through resolver tools

**Process:**
1. Route lookup through `fulltext-retrieval` or a compatible resolver boundary
2. Write one `retrieval_manifest.csv` row for every sought record, including
   resolver, locator, status, timestamp, and failure reason
3. Update screening full-text status without changing inclusion decisions
4. Keep inaccessible, abstract-only, and user-access-required cases visible for
   PRISMA and extraction evidence limits

**Status Codes:** `RETRIEVED_OA` | `RETRIEVED_PREPRINT` | `ABSTRACT_ONLY` |
`NOT_RETRIEVED` | `NEEDS_USER_ACCESS`

**Outputs:** `RetrievalManifest`, `FullTextStatus`

---

## metadata-enricher

**Purpose:** Normalize and complete paper metadata

**Process:**
1. Normalize DOI: `10.xxxx/example` (canonical)
2. Route provider checks through the metadata registry boundary and preserve
   provider provenance
3. Generate citekey: `lastname[year]keyword`
4. Create dedup keys for matching

**Output:** `Bibliography`

---

## citation-formatter

**Purpose:** Normalize bibliography metadata and citekeys for export

**Styles:** APA 7th, MLA 9th, Chicago, IEEE, Harvard, BibTeX

**BibTeX Types:** @article, @inproceedings, @book, @incollection, @misc

**Citekey Format:** `lastname[year]keyword` (e.g., `smith2024machine`)

**Process:** normalize DOI values, resolve duplicate citekeys, flag missing
required fields, and write export-ready `bibliography.bib`

**Output:** `Bibliography`

---

## reference-manager-bridge

**Purpose:** Exchange references with Zotero and import-file formats safely

**Formats:**
- BibTeX (.bib) - Zotero, Mendeley, JabRef
- RIS (.ris) - EndNote, Mendeley
- CSL-JSON (.json) - Zotero

**Tag Schema:** `project:[topic]`, `status:included|excluded`, `quality:A-E`

**Process:**
1. Do not route scholarly discovery through Zotero by default
2. Use local Zotero sync only when explicitly requested, after status check and
   dry-run; write only after explicit `dry_run: false`
3. Fall back to `.bib`, `.ris`, or CSL-JSON import files when local sync is not
   available
4. Preserve provider source, local match status, user-curated fields, and import
   action in `zotero-import-report.md`

**Outputs:** `Bibliography`, `RISExport`, `CSLJSONExport`, `ZoteroImportReport`

---

## manuscript-architect

**Purpose:** Draft and revise a full research paper (outline → draft → integrity passes)

**Process:**
1. Create manuscript workspace (`manuscript/outline.md`, `manuscript/manuscript.md`)
2. Establish the Story Spine: central claim, argumentative mainline, section jobs, non-goals, and evidence threshold
3. Draft sections iteratively (Intro → Related work → Methods → Results → Discussion → Limitations → Conclusion)
4. For each section or paragraph-cluster, run write -> review -> confirm and check for mainline drift, missing support, generic or vague claims, and logic jumps
5. Run claim–evidence integrity pass + figures/tables pass
6. Prepare for readiness checks (reporting/PRISMA) and submission packaging

**Templates:** `templates/manuscript-outline.md`, `templates/manuscript-skeleton.md`, `templates/claim-evidence-map.md`, `templates/figures-tables-plan.md`

**Output:** `ManuscriptOutline`, `Manuscript`, `ClaimGraph`

---

## table-generator

**Purpose:** Generate publication-ready tables from statistical output

**Process:**
1. Parse raw statistical output (e.g., regression logs, summary stats)
2. Format into standard academic styles (APA, IEEE, domain-specific)
3. Generate Markdown, LaTeX, or HTML table code
4. Ensure proper alignment, decimal precision, and significance starring

**Output:** `FormattedTables`

---

## figure-specifier

**Purpose:** Specify publication-quality figures with reproducible code

**Process:**
1. Analyze data structure and intended message
2. Recommend appropriate visualization types (e.g., forest plots, scatter bounds)
3. Specify aesthetics (color palettes, accessibility, typography)
4. Generate reproducible plotting code (ggplot2, matplotlib, seaborn)

**Output:** `FigureSpecs`, `PlottingCode`

---

## reporting-checker

**Purpose:** Reporting guideline completeness check (empirical studies)

**Process:**
1. Identify study design and select guideline (CONSORT/STROBE/COREQ/SRQR/TRIPOD)
2. Map checklist items to manuscript sections
3. Generate prioritized missing-items fix list

**Template:** `templates/reporting-checklist.md`

**Output:** `ReportingChecklist`

---

## submission-packager

**Purpose:** Submission-ready packaging (cover letter + statements + final checklist)

**Process:**
1. Confirm target venue constraints + anonymization needs
2. Run reporting checks (and PRISMA if SR)
3. Draft submission auxiliary materials + assemble submission checklist

**Templates:** `templates/cover-letter.md`, `templates/submission-checklist.md`, `templates/title-page.md`, `templates/highlights.md`, `templates/suggested-reviewers.md`, `templates/author-contributions-credit.md`, `templates/funding-statement.md`, `templates/coi-statement.md`, `templates/data-availability.md`, `templates/ai-disclosure.md`, `templates/supplementary-inventory.md`

**Output:** `SubmissionPackage`

---

## credit-taxonomy-helper

**Purpose:** Generate CRediT author contribution statements

**Process:**
1. Map team members to 14 CRediT roles
2. Resolve degrees of contribution (lead, equal, supporting)
3. Apply venue-specific formatting
4. Verify all authors are accounted for

**Output:** `CRediTStatement`

---

## rebuttal-assistant

**Purpose:** Reviewer response workflow (response matrix + response letter)

**Process:**
1. Atomize reviewer requests into a trackable table
2. Decide action per item (accept/partial/disagree) with evidence
3. Draft response matrix + letter with precise change locations

**Templates:** `templates/rebuttal-response-matrix.md`, `templates/rebuttal-letter.md`

**Output:** `ResponseToReviewers`, `ResponseLetter`

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

**Output:** `PRISMAChecklist`

---

## model-collaborator

**Purpose:** Evidence-based independent review across research tasks.

Read `skills/Z_cross_cutting/model-collaborator.md`. Use the configured model and
visible Host capabilities; Full MCP supplies bounded handoffs, not model processes.
Sequential roles in one conversation are self-review. Record sources, participants,
disagreements and unresolved gates in `logs/model_collab_trace.md`; agreement or
confidence never substitutes for evidence or artifact approval.

---

## code-builder

**Purpose:** Build academic research code (multi-discipline, multi-language)

**Domain Profiles:** `skills/domain-profiles/*.yaml` define discipline-specific libraries, method templates, diagnostics, and pitfalls. Available: finance, economics, psychology, biomedical, education, cs-ai, political-science, epidemiology, ecology-environmental. Add new domains via `domain-profiles/custom-template.yaml`.

**Runtime Subject Refinement:** The default installed package is adaptive core.
Use `standards/subject-refinement-contract.yaml` to distinguish core-only work,
borrowed method lenses, suggested subjects, confirmed subjects, and locked
subjects. Borrowed lenses load the narrow audited method pack without changing
`active_subject`.

**Languages:** Python, R, Stata, MATLAB, Julia (inferred from domain profile if not specified)

**Strategies:**
1. **Standard (Tier 1):** Use domain-profile recommended library + method checklist
2. **Advanced (Tier 2):** Methodological Decomposition (JAX/PyTorch/Custom MLE)

**Invocation:** Read `workflows/code-build.md` and execute the selected Stage I
task in the active Host. Use visible Full MCP for a registered project handoff;
legacy Python controller commands are not native 2.x dependencies.

**Output:** `AnalysisCode`, `StatsReport`

---

## release-packager

**Purpose:** Package code, data, and environment for reproducible releases

**Process:**
1. Audit dependencies and generate `requirements.txt` or `env.yaml`
2. Validate directory structure and data paths
3. Generate comprehensive `README.md`
4. Assemble release bundle with appropriate licensing

**Output:** `ReleasePackage`

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

**Purpose:** Check concrete claim, evidence and method defects at the requested scope

**Process:**
1. Identify the requested artifact and applicable checks; use only relevant stage lenses.
2. Review an ordinary answer once. Fix concrete defects and recheck affected work; no findings is a valid result, and no question quota applies.
3. Formal orchestrated runs retain configured limits and the minimum review contract: standard 2 passes, deep 3 when revision rounds are available. An early PASS receives the required stability review; it does not require inventing revisions.
4. Reuse issue IDs and keep unresolved blockers. Stop a blocked branch when progress requires unavailable evidence or a decision; reaching a limit does not turn BLOCK into PASS.
5. Keep `review/self_critique_log.md` for formal runs. A direct answer can report its check in chat. Do not start another model or persona merely because this card was loaded.

**Output:** Self-critique log and revised (improved) output

## boundary-interviewer

**Purpose:** Clarify scholarly boundaries one question at a time before high-risk Qiongli work proceeds; also run the Academic Grill Loop for academic idea discovery when a vague topic needs to become a defensible paper idea.

**Process:**
1. Inspect the request and existing research artifacts before asking. If they settle the relevant boundary, continue without a new interview; a missing boundary file alone does not block a direct answer.
2. Map the task to an academic boundary dimension: phenomenon, construct, contribution, claim strength, evidence threshold, method validity, rival explanation, generalizability, ethics/governance, venue/reviewer, research code, or submission/revision.
3. For brainstorms or Stage A work, use the Academic Grill Loop: ask one scholarly question at a time that tests whether the topic can become one answerable paper.
4. Write `AcademicIdeaFunnel` -> `context/idea_funnel.md` before Stage A outputs when the idea is still unsettled; include Candidate Idea Triage, recommended idea, core claim, research question, candidate gap, contribution type, evidence plan, weakest assumption, reviewer risk, `next_stage_recommendation`, and `boundary_review_handoff`.
5. Provide a recommended answer with rationale, evidence threshold, reviewer consequence, and confidence.
6. Record claim strength, evidence threshold, rival explanations, validity or trustworthiness risk, generalizability limit, and downstream sync targets.
7. Sync downstream-relevant decisions into `context/decision_log.md` or `context/stage_handoff.md`.
8. After the user answers, continue the active task within the locked boundary and require later skills to narrow, not broaden, unless a `revisit_trigger` records the justification.

**Output:** `AcademicIdeaFunnel` -> `context/idea_funnel.md`; `BoundaryReview` -> `context/boundary_review.md`

---

## presentation-planner

**Purpose:** Design the story arc, content budget, and audience calibration for an academic talk

**Process:**
1. Classify talk type (conference / seminar / job talk / poster / lightning)
2. Map manuscript sections → slide inventory (show / cut / appendix)
3. Choose story arc (3-act / claim-first / puzzle)
4. Calibrate for audience expertise level
5. Create slide blueprint with per-slide time budget
6. Plan appendix slides for Q&A
7. Select output backend (Slidev / Beamer / PPTX)

**Output:** `PresentationPlan`

---

## slide-architect

**Purpose:** Design backend-agnostic slide content specs using assertion-evidence format

**Process:**
1. Convert each blueprint item into assertion (title) + evidence (body)
2. Map slide types to Slidev / Beamer / PPTX layouts
3. Adapt paper figures for projection (≥20pt text, high contrast)
4. Write speaker notes for every content slide
5. Plan progressive reveal / animations

**Output:** `SlideDeckSpec`

---

## slidev-scholarly-builder

**Purpose:** Generate Slidev deck with `slidev-theme-scholarly` layouts and BibTeX citations

**Process:**
1. Scaffold project: `npx sch init my-talk --template academic`
2. Configure frontmatter (theme, authors, preset)
3. Build slides using scholarly layouts (cover, section, methodology, results, compare, references, etc.)
4. Use components (@citekey, Theorem, Block, Steps, Keywords)
5. Set up `references.bib` for auto-citations
6. Export: `npx slidev export`

**Output:** `SlidevDeck`, `BibTeXFile`

---

## beamer-builder

**Purpose:** Generate LaTeX Beamer presentation with theme selection and BibLaTeX

**Process:**
1. Select Beamer theme (metropolis / Madrid / Berlin / etc.)
2. Build document skeleton with packages and metadata
3. Construct frame types (content, figure, table, columns, theorem)
4. Add overlays/animations (\pause, \onslide, \only)
5. Configure BibLaTeX citations (\textcite, \parencite)
6. Compile: `latexmk -pdf slides.tex`

**Output:** `BeamerDeck`, `BibTeXFile`

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
