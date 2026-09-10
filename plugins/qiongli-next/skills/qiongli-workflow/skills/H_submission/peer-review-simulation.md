---
id: peer-review-simulation
stage: H_submission
description: "Review a manuscript through relevant referee lenses, grounding findings in evidence and distinguishing simulation from actual independent review."
inputs:
  - type: Manuscript
    description: "Draft manuscript for simulated review"
outputs:
  - type: PeerReviewSimulation
    artifact: "revision/peer_review_simulation.md"
constraints:
  - "Must disclose actual review participants; simulated lenses are not independent reviewers"
  - "Must aggregate and reconcile conflicting feedback"
  - "Must produce actionable items, not vague criticism"
failure_modes:
  - "Personas converge to similar critique (lack of diversity)"
  - "Missing domain expertise for specialized methods"
  - "Reviews are too positive (false reassurance)"
tools: [filesystem]
tags: [submission, peer-review, simulation, multi-persona, red-team]
domain_aware: false
---

# Peer Review Simulation Skill

Assess a manuscript from relevant referee perspectives, with source-bound findings and explicit review limits.

## Purpose

Provide a requested referee-style critique. Distinguish evidence-backed weaknesses
from speculative concerns; simulated recommendations are not journal decisions.

## Related Task IDs

- `H3` (peer review simulation)

## Output (contract path)

- `RESEARCH/[topic]/revision/peer_review_simulation.md`

## When to Use

- When the user requests a referee-style manuscript review or formal H3 task
- For a focused method/claim check, use the relevant lens without a full panel
- Explaining a published paper belongs to paper reading; revising against actual
  received comments belongs to rebuttal/revision work

## Inputs

- `Manuscript`: Draft manuscript for simulated review
- Reuse any supplied review scope, target venue and previous findings. A focused
  answer may stay in chat; formal H3 retains its report and applicable gates.
  Do not draft a new manuscript, launch other models or require a project merely
  to review supplied material.
- If a required input is missing or insufficient, write a gap note under `RESEARCH/[topic]/context/gap_notes.md` and ask for the missing artifact instead of inventing content.
- Treat literature, data, citations, and project files as evidence sources; keep unsupported assumptions visibly marked.

## Process

### Step 1: Configure the Review Panel

Select lenses that address the requested scope. A full H3 review covers methods,
positioning and internal consistency; a focused check uses only the relevant lens.
Start with the active model. Distinct personas in one conversation are a simulated
multi-perspective self-review, not independent reviewers.

If actual independent review is requested or required, follow
`skills/Z_cross_cutting/model-collaborator.md` using available authorized reviewers.
Record real participants and separation. If they are unavailable, report that
requirement as unresolved; do not satisfy it with renamed personas. Formal minimum
review counts and existing BLOCK findings remain binding.

#### Core Persona Set

| Persona | Focus | Mindset | Looks For |
|---------|-------|---------|-----------|
| **Methodologist** | Design, identification, statistical analysis | "Is the evidence credible?" | Endogeneity threats, measurement validity, power, robustness gaps, missing diagnostics |
| **Domain Expert** | Theory, positioning, literature coverage | "Does this advance the field?" | Missing citations, contribution clarity, theory-evidence fit, novelty |
| **Reviewer 2** (Skeptic) | Clarity, logic, consistency, reproducibility | "Can I follow this? Am I convinced?" | Unclear writing, logical gaps, missing details, inconsistencies between sections |

#### Extended Persona Set (add based on paper type)

| Persona | Add When | Looks For |
|---------|----------|-----------|
| **Qualitative Methodologist** | Qualitative or mixed-methods paper | Trustworthiness procedures, reflexivity, coding rigor, thick description, negative cases |
| **Statistics Specialist** | Complex quantitative methods (SEM, Bayesian, ML) | Model specification, assumption violations, estimation choice, reporting completeness |
| **Ethics Reviewer** | Human subjects, sensitive data, AI use | IRB documentation, consent adequacy, de-identification, AI disclosure |
| **Practitioner** | Applied research with industry implications | Practical relevance, implementation feasibility, translation of findings |
| **Associate Editor** | High-tier journal submission | Scope fit, novelty threshold, positioning clarity, desk-reject triggers |

### Step 2: Review the Selected Material

Check the source using the chosen lenses. For a full report, use the structure
below without inventing findings to fill its slots. For a focused review, return
only the relevant findings and limits. An ordinary check runs once plus targeted
fix verification; repeated unchanged reviews are not progress.

```markdown
### Review by [Persona Name]

#### Overall Assessment
- Recommendation: [Accept / Minor Revision / Major Revision / Reject]
- Review basis: [source scope, relevant expertise limits, actual participants]

#### Summary (3–5 sentences)
[Overall impression of the paper's contribution, strengths, and weaknesses]

#### Major Issues (must-fix)

**M1: [Issue title]**
- **Location**: [Section / page / paragraph]
- **Problem**: [What is wrong and why it matters]
- **Evidence**: [Quote or reference from the paper]
- **Suggested fix**: [Specific recommendation]
- **Severity**: [Fatal / Major]

**M2: ...**

#### Minor Issues (should-fix)

| # | Location | Issue | Suggestion |
|---|----------|-------|-----------|
| m1 | § 2.3, ¶ 2 | [specifics] | [fix] |
| m2 | ... | ... | ... |

#### Strengths (what works well)
1. ...
2. ...
```

### Step 3: Apply Relevant Critique Lenses

Use the questions appropriate to this design, evidence and venue. Examples are
not universal thresholds or a quota of flaws; judge the claim actually made.

#### Methodologist Checklist

| Question | What to Check | Red Flag |
|----------|--------------|----------|
| Is the design appropriate for the RQ? | Causal claim → experimental/quasi; descriptive → observational | Causal claims with cross-sectional data |
| Is the identification strategy sound? | Exogenous variation, instrument validity, parallel trends | "We control for X" as sole defense against endogeneity |
| Is the sample size adequate? | Power analysis or MDE reported | Missing justification for the precision or claims required by this design |
| Are measures valid and reliable? | Cronbach's α, factor loading, validated scales | New scales without validation |
| Are robustness checks sufficient? | Multiple specifications, sensitivity to outliers | Single model, no sensitivity analysis |
| Is missing data handled? | Listwise deletion justification, imputation, or sensitivity | >20% missing without discussion |
| Are results correctly reported? | CI, effect sizes, exact p-values | Only stars without CI |

#### Domain Expert Checklist

| Question | What to Check | Red Flag |
|----------|--------------|----------|
| Is the contribution clear and novel? | Can state novelty in 1 sentence by page 2 | Contribution buried on page 8 |
| Is the literature review comprehensive? | Key papers cited; recent (2–3 years) | Missing seminal or recent work |
| Is the paper well-positioned? | Clear gap statement; not just "nobody has studied X" | Gap is a truism, not a research problem |
| Does the theory support the hypotheses? | Mechanism articulated, not just correlation prediction | "Based on prior literature, we hypothesize..." |
| Is the discussion substantive? | Engages with theory, not just "implications for managers" | Discussion = restated results |
| Are findings compared to prior work? | Agreements and disagreements discussed with reasons | "Our results are consistent with Smith (2020)" without analysis |

#### Reviewer 2 (Skeptic) Checklist

| Question | What to Check | Red Flag |
|----------|--------------|----------|
| Is the abstract accurate? | Claims match results section | Abstract says "significant" but results are marginal |
| Is the writing clear? | Can follow the argument without re-reading | Dense paragraphs, jargon without definition |
| Are sections consistent? | RQ in intro → method → results → discussion alignment | RQ2 not addressed in results |
| Are tables/figures clear? | Self-explanatory with adequate notes | "Table 1. Results." |
| Is the paper the right length? | Within venue limits; no padding | 20% over limit; entire section that could be an appendix |
| Are claims calibrated? | Language matches evidence strength | "Proves" with correlational design |

### Step 4: Consolidate and Reconcile

After all reviews are complete, create a unified action plan:

#### Deduplication
- Group similar issues from different personas
- When two personas flag the same problem differently, use the more specific version
- Preserve each actual reviewer/source; severity follows evidence and consequence, not repeated votes

#### Reconciliation Matrix

| Issue | Source / reviewer / lens | Evidence and location | Severity | Action / unresolved disagreement |
|---|---|---|---|---|

Judge disagreements against the same source and claim. A substantiated blocker
remains blocking even if only one reviewer raises it; agreement alone establishes
neither severity nor correctness. Keep prior issue IDs across revisions.

### Step 5: Produce the Requested Report

Record actual review scope, self-review versus independent participants, supported
findings and gaps. Rank actual risks without a minimum count. State readiness
only against checks actually completed; unavailable evidence or independent
review requirements remain unresolved. Do not predict a journal decision.
Stop at the review; manuscript revision or a rebuttal is a separate requested action.

## Output Contract

- `PeerReviewSimulation`: write `RESEARCH/[topic]/revision/peer_review_simulation.md`.
- Separate finding, interpretation, and implication in the final artifact.
- Do not invent citations, data, sample sizes, statistical results, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose or review artifacts.

### Evidence Ledger and Source Integrity

- Update `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` when producing, revising, or validating central scholarly claims.
- Follow `references/evidence-ledger-contract.md`: supported claims need source pointers; unsupported central claims become `gap_note` rows and `RESEARCH/[topic]/context/gap_notes.md` entries.
- For final writing, proofread, submission, rebuttal, citation, or presentation-facing outputs, apply `references/citation-risk-policy.md` and write or update `RESEARCH/[topic]/proofread/citation-risk-report.md` when citation risk is material.

## Quality Bar

The simulation is **ready** when:

- [ ] Requested lenses completed; actual independent-review requirements satisfied or explicitly unresolved
- [ ] Findings are grounded in source locations; a clean review may have zero issues
- [ ] Full H3 methods are assessed for relevant identification/validity requirements
- [ ] Full H3 literature coverage and positioning are assessed within available evidence
- [ ] Full H3 clarity and internal consistency are assessed
- [ ] Consolidated reconciliation matrix produced
- [ ] Prioritized action list maps each issue to a fix location and effort
- [ ] Readiness limits and unavailable checks are explicit; no journal outcome is promised

## Minimal Output Format

```markdown
# Peer Review Simulation

## Overall Readiness: [Ready / Needs Revision / Not Ready]
## Supported Risks and Evidence Gaps
[Only actual findings; zero findings is valid within the stated scope]

## Review Scope and Participants
[Selected lenses; actual reviewers or simulated self-review; unavailable requirements]

## Findings by Relevant Lens

### Methodologist
- Recommendation: [Accept / Minor / Major / Reject]
- Major: [list]
- Minor: [list]

### Domain Expert
- Recommendation: ...

### Reviewer 2
- Recommendation: ...

## Reconciliation Matrix

| Issue | Flagged By | Severity | Action | Location | Effort |
|-------|-----------|----------|--------|----------|--------|

## Prioritized Action List

### Fatal (block submission)
1. ...

### Major (fix before submission)
1. ...

### Minor (fix if time allows)
1. ...
```

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| 为凑问题而批评 | 制造不存在的缺陷 | 只报告有来源和影响的问题，允许零发现 |
| 把 persona 当独立审稿人 | 虚报独立复核 | 记录实际参与者；单会话多个视角仍是自审 |
| 只关注写作 | 忽视方法论和数据问题 | 包含 Methodologist persona |
| 缺少 actionable feedback | 指出问题但不建议如何修 | 每条 concern 附带 suggested fix |
| 自动转入修稿 | 超出用户请求 | 给出修改建议；用户要求修稿时再执行 |
