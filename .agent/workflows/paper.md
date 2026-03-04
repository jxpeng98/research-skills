---
description: 研究论文写作工作流入口（选择论文类型 + 当前阶段/要写的部分）
argument-hint: [可选: RESEARCH下topic文件夹名] [可选: venue]
---

# Research Paper Workflow (Menu / Router)

Provide a systematic “choose-your-path” workflow for writing research papers. The user selects:
1) paper type, and 2) what they want to do right now (stage/section).

Use the canonical standard contract:
- `standards/research-workflow-contract.yaml`
- Task IDs (`A1` ... `I8`) and output paths in that file are authoritative.

## Context

$ARGUMENTS

## Step 0: Select Project Folder

Ask the user:
> "Which `RESEARCH/[topic]/` folder should we work in?"
> - Existing projects: [List folders under `RESEARCH/`]
> - Create new: `RESEARCH/[new-topic]/`

Normalize `[topic]` (lowercase, hyphens).

## Step 1: Choose Paper Type (Pick One)

Ask the user to select:
1. **Empirical (实证)** — data/experiments/interviews; emphasis on Methods + Results
2. **Systematic Review (系统综述)** — PRISMA workflow; synthesis/meta-analysis
3. **Methods (方法)** — propose a new method + validation
4. **Theory/Conceptual (理论)** — build framework/propositions; emphasis on argument + positioning

Store selection as `[paper_type]`.

## Step 1.5: Choose Task ID (Canonical)

Ask the user to choose a Task ID from the contract (for example `F3`, `G1`, `B1`).
If the user describes intent in natural language, map it to the closest Task ID and confirm once.

## Step 2: Choose What You Want to Do Now

Ask the user to pick a **stage** first. Then show only the relevant **sub-options** and ask them to pick one.

When possible, always include the matching Task ID in the option label (for example: `F3 Full draft`).

### A) Framing & Positioning (研究问题/贡献/定位)
Pick one:
1. **A1 Research question(s)** (PICO/PEO + inclusion/exclusion + keywords)
2. **A1_5 Hypothesis generation** (Translate RQ into testable hypotheses)
3. **A2 Contribution & novelty statement** (3–5 bullets + “so what”)
4. **A3 Theoretical framing** (concepts → relationships → propositions)
5. **A4 Research gap scan** (what’s missing + why it matters)
6. **A5 Target venue analysis** (Analyze fit and formatting for specific venues)

**Routing:**
- A1_5 → use `hypothesis-generator`
- A5 → use `venue-analyzer`
- A3 → `/build-framework`
- A4 → `/find-gap`
- A1/A2 → run a light framing pass in this session:
  - Ensure `RESEARCH/[topic]/framing/` exists
  - Use `question-refiner` + write:
    - `RESEARCH/[topic]/framing/research_question.md`
    - `RESEARCH/[topic]/framing/contribution_statement.md`

### B) Literature & Related Work (找文献/读文献/组织相关工作)
Pick one:
1. **B1 Systematic review pipeline** (PRISMA end-to-end)
2. **B1_5 Concept/keyword extraction** (Refine base search terms)
3. **B2 Targeted key papers** (read 3–10 seed papers with structured notes)
4. **B3 Citation snowballing** (forward/backward; expand corpus)
5. **B4 Related work writing** (theme/taxonomy-based, not chronological)
6. **B5 References & citation formatting** (BibTeX/APA/IEEE + manager export)
7. **B6 Literature mapping** (Visual or narrative taxonomy of the field's state of the art)

**Routing:**
- B1_5 → use `concept-extractor`
- B6 → use `literature-mapper`
- B1 → `/lit-review`
- B2 → `/paper-read` (repeat until enough seeds)
- B3 → run `citation-snowballer` skill + log into `RESEARCH/[topic]/snowball_log.md`
- B4 → `/academic-write related-work [topic]`
- B5 → use `citation-formatter` (+ `reference-manager-bridge` if needed)

### C) Study Design & Analysis Plan (实证：设计/工具/分析计划)
Pick one:
1. **C1 Study design doc** (design choice, sampling, measures, procedures)
2. **C1_5 Rival hypothesis design** (Proactive design of methods explicitly to rule out competing theories)
3. **C2 Instruments** (survey / interview guide skeleton)
4. **C3 Analysis plan** (estimands, models, missingness, robustness)
5. **C3_5 Robustness check plan** (Pre-specifying tests for heteroskedasticity, endogeneity, or sensitivity)
6. **C4 Data management plan** (privacy, storage, sharing, retention)
7. **C5 Preregistration draft** (optional)

**Routing:**
- C1_5 → use `rival-hypothesis-designer`
- C3_5 → use `robustness-planner`
- C1–C5 → `/study-design` (tell the workflow which artifacts you want to prioritize)

### D) Ethics / IRB (伦理与合规材料)
Pick one:
1. **D1 IRB/ethics pack** (risk, consent, recruitment, privacy/security)
2. **D2 Manuscript-ready statements** (ethics + data/code availability)
3. **D3 Participant de-identification plan** (Specific methodology for anonymizing and securing sensitive data)

**Routing:**
- D3 → use `deidentification-planner`
- D1 → `/ethics-check`
- D2 → `/academic-write methodology [topic]` (focus on statements only)

### E) Evidence Synthesis (系统综述：叙述/定性/Meta)
Pick one:
1. **E1 Synthesis strategy per outcome** (pool vs narrative vs qualitative)
2. **E2 Effect size extraction table** (pool-ready yi/SE)
3. **E3 Run meta-analysis** (optional code + write-up)
4. **E3_5 Publication bias assessment** (Funnel plots and fail-safe N)
5. **E4 GRADE / certainty summary** (optional)
6. **E5 Integrate into final synthesis** (`synthesis.md`)

**Routing:**
- E3_5 → use `publication-bias-checker`
- E1–E5 → `/synthesize` (tell it which outcome(s) / which deliverable you want first)

### F) Manuscript Writing (写作：从大纲到单段落)
Pick one:
1. **F1 Outline only** → create `RESEARCH/[topic]/manuscript/outline.md`
2. **F2 Single section** → `/academic-write [section] [topic]`
3. **F3 Full draft** → `/paper-write`
4. **F4 Claim–evidence map** → create `RESEARCH/[topic]/manuscript/claims_evidence_map.md`
5. **F5 Figures/tables plan** → create `RESEARCH/[topic]/manuscript/figures_tables_plan.md`
6. **F6 Abstract & Title Optimization** → optimize for indexing, SEO, and engagement

**Routing:**
- F6 → use `meta-optimizer`
- F1 → use `manuscript-architect` + `templates/manuscript-outline.md`
- F4 → use `manuscript-architect` + `templates/claim-evidence-map.md`
- F5 → use `manuscript-architect` + `templates/figures-tables-plan.md`
- F2 → `/academic-write`:
  - Sections: `title`, `keywords`, `abstract`, `introduction`, `related-work`, `methodology`, `results`, `discussion`, `limitations`, `conclusion`, `appendix`
  - If the user wants a smaller unit, ask which **component** to draft:
    - `introduction`: hook / gap paragraph / contributions paragraph / roadmap paragraph
    - `related-work`: taxonomy/themes / positioning paragraph / comparison paragraph
    - `methodology`: design / participants-data / measures / procedure / analysis / ethics statements
    - `results`: main results / robustness-sensitivity / subgroup-heterogeneity
    - `discussion`: interpretation / comparison-to-prior / implications / boundary conditions
    - `limitations`: internal / construct / external / statistical conclusion validity
  - Then draft only that component (1–3 paragraphs) and list missing inputs.
- F3 → `/paper-write`

### G) Polishing & Compliance (打磨与一致性检查)
Pick one:
1. **G1 Reporting completeness** (CONSORT/STROBE/COREQ/SRQR/TRIPOD)
2. **G2 PRISMA compliance** (systematic review)
3. **G3 Cross-section consistency** (RQs↔Methods↔Results, abstract alignment)
4. **G4 Tone & Style Normalization** (Rewriting for concise, objective academic tone)

**Routing:**
- G4 → use `tone-normalizer`
- G1 → run `reporting-checker` skill → `RESEARCH/[topic]/reporting_checklist.md`
- G2 → run `prisma-checker` skill → `RESEARCH/[topic]/prisma_checklist.md`
- G3 → use `manuscript-architect` integrity pass + update claim–evidence map

### H) Submission & Revision (投稿与返修)
Pick one:
1. **H1 Submission package** (cover letter + checklist + auxiliary materials: title page, COI, funding, data availability, AI disclosure, highlights, reviewers)
2. **H2 Rebuttal / revision response**
3. **H2_5 Reviewer Empathy Check** (Tone neutralization for responses)
4. **H3 Multi-Agent Peer Review** (Simulate methodologist, expert, and Reviewer 2 personas)
5. **H4 Fatal Flaw Analysis** (Constructive desk-reject analysis identifying critical flaws)

**Routing:**
- H3 → use `peer-review-simulation`
- H4 → use `fatal-flaw-detector`
- H2_5 → use `reviewer-empathy-checker`
- H1 → `/submission-prep`
- H2 → `/rebuttal`

### I) Research Code Support (可选：实现/复现/分析脚本)
Pick one:
1. **I1 Implement a method** (standard libs or from equations)
2. **I2 Reproduce a paper’s algorithm/experiment**
3. **I3 Build a data analysis pipeline**
4. **I4 Code Reproducibility Audit** (Checking random seeds, magic numbers, and containerization instructions)
5. **I5 Code Specification** (Generate strict constraints before coding, OPSX style)
6. **I6 Zero-Decision Planning** (Transform specs into execution plans)
7. **I7 Parallel Execution & Optimization** (Execute plan and run profilers/vectorization)
8. **I8 Cross-Model Code Review** (Secondary model reviews code logic, security, stats validity)

**Routing:**
- I5 → use `code-specification`
- I6 → use `code-planning`
- I7 → use `code-execution` (Codex execution)
- I8 → use `code-review` (Gemini review)
- I4 → use `reproducibility-auditor`
- I1–I3 → `/code-build` (use `model-collaborator` when verification is needed)

## Step 3: Execute the Chosen Path (In This Session)

After the user chooses, proceed immediately by following the corresponding workflow:
- If they pick “single section”: run the `/academic-write` workflow for that section.
- If they pick “full draft”: run `/paper-write`.
- Otherwise: run the selected workflow/skill and write outputs into `RESEARCH/[topic]/`.

Before finishing, verify:
1. The produced files match the output paths for the selected Task ID.
2. If a quality gate applies (`Q1`-`Q4`), record pass/fail briefly in the response.

Begin routing now.
