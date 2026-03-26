---
id: manuscript-architect
stage: F_writing
description: "Build coherent paper structure from outline to section-level drafts with claim-evidence integrity and analysis-depth checking."
inputs:
  - type: RQSet
    description: "Research questions"
  - type: EvidenceTable
    description: "Synthesized evidence"
  - type: DesignSpec
    description: "Study design (for empirical or qualitative papers)"
    required: false
outputs:
  - type: ManuscriptOutline
    artifact: "manuscript/outline.md"
  - type: Manuscript
    artifact: "manuscript/manuscript.md"
  - type: ClaimGraph
    artifact: "manuscript/claims_evidence_map.md"
  - type: FiguresTablesPlan
    artifact: "manuscript/figures_tables_plan.md"
constraints:
  - "Must maintain claim-evidence alignment across sections"
  - "Must follow venue-specific sectioning requirements"
  - "Every section must have an explicit analytical job"
failure_modes:
  - "Evidence gaps discovered during writing"
  - "Scope creep from unfocused claims"
  - "Discussion that merely restates results without interpretation"
tools: [filesystem]
tags: [writing, manuscript, outline, drafting, claim-evidence]
domain_aware: true
---

# Manuscript Architect Skill

Build and revise a full research paper manuscript across stages: outline → section drafts → coherence pass → compliance checks → submission-ready package.

## When to Use

- You want to draft a paper from an existing `RESEARCH/[topic]/` project folder (empirical study or systematic review).
- You want to draft a fully qualitative paper from interview, case, ethnographic, or document-based materials without defaulting back to a quantitative structure.
- You have partial materials (notes, tables, analysis plan, results) and want a coherent manuscript.

## Granularity Boundary

Keep these as embedded writing subflows inside `manuscript-architect` unless they own a separate contract artifact:

- story spine construction
- section-level drafting
- paragraph ordering and transitions
- results anchoring against tables/figures
- section-to-section coherence passes

## Analytical Depth Contract

These depth rules apply across empirical, qualitative, mixed-methods, theory, and methods papers:

- Every substantive paragraph must advance at least two of:
  - mechanism or process
  - tension, comparison, or contradiction
  - alternative explanation
  - boundary condition
  - implication
- Qualitative writing must interpret themes into analytic claims about mechanism, meaning, or scope. Do not stop at theme labels or illustrative quotes.
- If the evidence cannot support a deeper inference, narrow the claim instead of simulating depth with abstract wording.

## Inputs (Ask / Collect)

1. Paper type: **empirical** / **qualitative** / **systematic review** / **theory** / **methods** (default: empirical)
2. Target venue (optional): scope + formatting + double-blind requirements
3. Current artifacts available in `RESEARCH/[topic]/`
4. Citation style (APA/IEEE/BibTeX) and reference source (`bibliography.bib` preferred)

## Process

### Step 0: Set Manuscript Folder

Create/ensure:
```
RESEARCH/[topic]/manuscript/
├── outline.md
├── manuscript.md
├── claims_evidence_map.md
└── figures_tables_plan.md
```

### Step 1: Define Contribution & Story Spine

Before writing a word, articulate:

| Element | What to Write | Purpose |
|---------|--------------|---------|
| **Core contribution** | 1–2 sentence "elevator pitch" | Forces clarity of what's new |
| **Novelty points** | 3–5 bullets: what the paper adds that didn't exist before | Feeds Introduction and Discussion |
| **Main thesis** | What the reader should believe after reading | Ensures argumentative coherence |
| **Why the gap persisted** | Why this contribution wasn't made earlier | Elevates above "nobody has studied X" |

**Story spine template**:
```
"Despite [existing knowledge], we still don't understand [gap] because [why gap persisted].
This paper addresses this by [approach], finding that [key results].
This contributes to [theory/field] by [specific theoretical contribution],
and has practical implications for [audience] because [why it matters]."
```

### Step 2: Build Outline with Word Budget

Allocate word budget proportionally:

| Section | Typical % of Total | For 8,000 words | For 12,000 words | Key Function |
|---------|-------------------|-----------------|------------------|-------------|
| Introduction | 10–15% | 800–1,200 | 1,200–1,800 | Establish gap → state contribution |
| Literature / Related Work | 15–20% | 1,200–1,600 | 1,800–2,400 | Position contribution; build theoretical foundation |
| Methods | 15–25% | 1,200–2,000 | 1,800–3,000 | Convince reader the evidence is credible |
| Results / Findings | 20–30% | 1,600–2,400 | 2,400–3,600 | Present evidence systematically |
| Discussion | 15–20% | 1,200–1,600 | 1,800–2,400 | Interpret → contribute to theory → implications |
| Limitations | 3–5% | 240–400 | 360–600 | Honest assessment of scope |
| Conclusion | 3–5% | 240–400 | 360–600 | Summarize contribution; forward-looking |

> **Venue calibration**: Check 3 recent papers in the target venue to verify proportional expectations. Theory journals have longer lit reviews; empirical journals have longer methods/results.

### Step 3: Draft Sections Iteratively

Draft in this order (not the publication order):

#### 3a: Introduction — The Opening Argument

| Paragraph | Job | Template |
|-----------|-----|----------|
| ¶1 | Hook / establish importance | "[Topic] is [significance statement]. [Evidence of importance]." |
| ¶2 | Existing knowledge | "Prior research has established [what we know], including [key findings]." |
| ¶3 | Gap statement | "However, [gap — what remains unknown/unresolved]. This gap persists because [reason]." |
| ¶4 | What this paper does | "This paper addresses this gap by [approach]. Specifically, we [brief design overview]." |
| ¶5 | Preview of contribution + roadmap | "Our findings contribute to [theory] by [contribution]. The paper proceeds as follows: [roadmap]." |

> **Anti-pattern**: "There is a gap in the literature on X" without explaining why the gap matters or persisted. Reviewers want to know why this gap should be filled.

#### 3b: Literature / Related Work

Do NOT write a serial summary of papers. Instead, organize by:

| Organization Style | When to Use | Structure |
|-------------------|------------|-----------|
| **Theme-based** | Established field with clear streams | § per literature stream; end each § positioning your work |
| **Tension-based** | Competing explanations exist | § per debate/tension; position your theory against both sides |
| **Mechanism-based** | Paper proposes a mechanism | § per mechanism candidate; show why yours is needed |
| **Process-based** | Qualitative / temporal phenomenon | § per phase or stage; show what's known about each |

Each literature subsection must end with a "gap statement" that connects to your contribution. Use the literature map from `literature-mapper` (B6) as the skeleton.

> **Anti-pattern**: "Smith (2020) found X. Jones (2021) found Y. Lee (2022) found Z." → This is a list, not a review. Synthesize across papers.

#### 3c: Methods — The Credibility Case

| Subsection | Must Include | Common Failures |
|-----------|-------------|-----------------|
| **Design overview** | Design type + rationale (link to `study-designer` C1) | Design mentioned but not justified |
| **Setting / context** | Where, when, why this setting | Context omitted (reviewer can't assess fit) |
| **Sample / participants** | N, characteristics, recruitment, inclusion/exclusion | No sample size rationale |
| **Data collection** | Instruments, procedures, timing | Interview guide not in supplement |
| **Measures / coding** | Variable definitions, reliability, validation | Measures described but validity not discussed |
| **Analysis procedure** | Estimator, assumptions, steps | "We used regression" without specification |
| **Ethical considerations** | IRB, consent, de-identification | Often forgotten |

#### 3d: Results / Findings — The Evidence Presentation

**For quantitative papers**:

| Component | What to Include |
|-----------|----------------|
| Descriptive statistics | Sample table; correlations |
| Assumption checks | Normality, multicollinearity, outliers |
| Primary analysis | Main model(s) with estimates, CI, p-values |
| Secondary analysis | Subgroups, interactions, mediation |
| Robustness checks | Alternative specifications (main text or appendix) |
| Null results | Report with precision framing — don't bury |

**For qualitative papers**:

| Component | What to Include |
|-----------|----------------|
| Theme overview table | Theme, definition, prevalence, exemplar |
| Theme presentation | Analytical narrative + evidence (quotes, episodes) per theme |
| Cross-case patterns | How themes manifest across cases |
| Deviant / negative cases | Cases that contradict dominant patterns |
| Process model (if applicable) | Visual representation of how themes relate |

> Use `analysis-interpreter` for depth guidance. Every finding should reach Level 2+ on the interpretive depth ladder.

#### 3e: Discussion — The Intellectual Contribution

| Subsection | Job | Common Failure |
|-----------|-----|----------------|
| **Summary of findings** | 1–2 paragraphs recapping key results | Becomes a full results re-presentation |
| **Theoretical contribution** | What does this change or extend in theory? | "Our results are consistent with theory" (so what?) |
| **Comparison to prior work** | Where you agree/disagree with prior findings | Agreement stated without analysis of WHY |
| **Mechanism discussion** | What explains the pattern? | Mechanism asserted without evidence |
| **Boundary conditions** | When does this NOT hold? | Omitted entirely |
| **Practical implications** | What should practitioners do differently? | Overclaimed or too vague |
| **Limitations** | Explicit; linked to specific design choices | All in one paragraph; perfunctory |
| **Future directions** | Each flows from a specific limitation | "Future research should study X" without grounding |

> **Anti-pattern**: Discussion = restated Results + "Companies should...". The Discussion must INTERPRET, not just REPORT + ADVISE.

#### 3f: Abstract, Title, Keywords (Last)

Write these LAST — they must reflect the final, not the planned, paper. Use `meta-optimizer` (F6) for optimization.

### Step 4: Claim–Evidence Integrity Pass

For every non-trivial claim in the manuscript:

| Check | What to Verify | Action if Failing |
|-------|---------------|-------------------|
| **Has evidence** | Claim linked to data/citation | Add evidence or hedge/remove claim |
| **Evidence is sufficient** | Not a single anecdote for a general claim | Add cases or narrow scope |
| **Claim matches evidence strength** | Causal language ↔ causal design | Calibrate language to design |
| **RQ-Methods-Results alignment** | Each RQ addressed in methods AND results | Flag any unaddressed RQ |
| **Speculation labeled** | Hunches clearly distinguished from findings | Add "we speculate that..." framing |

Use `templates/claim-evidence-map.md` to produce:

```markdown
| Claim | Evidence Type | Location | Strength | Status |
|-------|--------------|----------|----------|--------|
| "X increases Y" | Regression β = 0.31, p < .001 | Table 2, § 4.1 | Strong | ✅ |
| "This suggests M drives the relationship" | Mediation analysis + informant accounts | § 4.3, Table 4 | Moderate | ✅ |
| "Could apply to healthcare" | Speculation | § 5.4 | Weak | ⚠️ Label as speculation |
```

### Step 5: Figures/Tables Pass

Use `figure-specifier` (F5) and `table-generator` (F5):

| Check | What to Verify |
|-------|---------------|
| Every figure/table is referenced in text | Look for "Figure 1" and "Table 1" references |
| Every figure/table has a purpose | No decorative visualizations |
| Captions are self-explanatory | Reader can understand without body text |
| Numbering is sequential | Table 1, 2, 3... Figure 1, 2, 3... |
| Data in figures is not duplicated in tables | Choose one format per data point |

### Step 6: Compliance / Readiness Pass

Run the right checker:
- Empirical: **reporting-checker** (CONSORT/STROBE/COREQ/etc.)
- Qualitative: **reporting-checker** (SRQR / COREQ / venue-specific transparency)
- Systematic review: **prisma-checker**

## Quality Bar

The manuscript is **ready for review** when:

- [ ] Story spine articulated — contribution clear in first 2 pages
- [ ] Literature organized by theme/tension/mechanism, not by author
- [ ] Methods detailed enough for replication (quant) or audit (qual)
- [ ] Every result has: estimate + CI + effect size (quant) or analytical claim + evidence + deviant case (qual)
- [ ] Discussion interprets (doesn't merely restate) and contributes to theory
- [ ] Claim-evidence map shows no unsupported claims
- [ ] Figures and tables are self-explanatory with adequate captions
- [ ] Reporting checklist completed for the study type
- [ ] Abstract accurately reflects final results

## Outputs

- Outline → `RESEARCH/[topic]/manuscript/outline.md`
- Draft manuscript → `RESEARCH/[topic]/manuscript/manuscript.md`
- Claim-evidence map → `RESEARCH/[topic]/manuscript/claims_evidence_map.md`
- Figures/tables plan → `RESEARCH/[topic]/manuscript/figures_tables_plan.md`

## Notes

- For citation formatting, pair with **citation-formatter**.
- For figure/table design, pair with **figure-specifier** and **table-generator**.
- For final compliance, pair with **reporting-checker** or **prisma-checker**.
