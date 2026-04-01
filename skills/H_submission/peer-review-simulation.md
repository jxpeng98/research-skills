---
id: peer-review-simulation
stage: H_submission
description: "Simulate parallel, independent cross-reviews using distinct reviewer personas (Methodologist, Domain Expert, Reviewer 2)."
inputs:
  - type: Manuscript
    description: "Draft manuscript for simulated review"
outputs:
  - type: PeerReviewSimulation
    artifact: "revision/peer_review_simulation.md"
constraints:
  - "Each persona must review independently with distinct focus areas"
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

Simulate rigorous, independent peer reviews using distinct reviewer personas — catching weaknesses before real reviewers do.

## Purpose

Simulate parallel, independent cross-reviews using distinct reviewer personas (Methodologist, Domain Expert, Reviewer 2).

## Related Task IDs

- `H3` (peer review simulation)

## Output (contract path)

- `RESEARCH/[topic]/revision/peer_review_simulation.md`

## When to Use

- Before submission (final red-team pass)
- Before sharing a preprint for informal feedback
- After major revisions (verify the revision addresses original weaknesses)

## Process

### Step 1: Configure the Review Panel

Select 3–5 personas that match the expected reviewer pool for the target venue:

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

### Step 2: Execute Each Review Independently

For each persona, produce a complete review following this structure:

```markdown
### Review by [Persona Name]

#### Overall Assessment
- Recommendation: [Accept / Minor Revision / Major Revision / Reject]
- Confidence: [High / Medium / Low] (how qualified this reviewer feels for this topic)

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

### Step 3: Apply Persona-Specific Critique Frameworks

#### Methodologist Checklist

| Question | What to Check | Red Flag |
|----------|--------------|----------|
| Is the design appropriate for the RQ? | Causal claim → experimental/quasi; descriptive → observational | Causal claims with cross-sectional data |
| Is the identification strategy sound? | Exogenous variation, instrument validity, parallel trends | "We control for X" as sole defense against endogeneity |
| Is the sample size adequate? | Power analysis or MDE reported | N < 50 without justification; no power analysis |
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
- Note when multiple personas independently identify the same weakness (higher severity signal)

#### Reconciliation Matrix

| Issue | Flagged By | Severity | Consensus | Priority Action |
|-------|-----------|----------|-----------|-----------------|
| Weak identification strategy | Methodologist (M1), Reviewer 2 (M3) | Fatal | Unanimous | Must fix: add instrument / design defense |
| Missing recent citations | Domain Expert (M2) | Major | Single reviewer | Should fix: update lit review |
| Abstract overclaims | Reviewer 2 (M1), Methodologist (m4) | Major | 2/3 agree | Must fix: calibrate language |

#### Decision Rules

| Consensus | Action |
|-----------|--------|
| 3/3 personas flag as fatal | **Do not submit** until fixed |
| 2/3 flag as major | **Fix before submission** |
| 1/3 flags as major | **Author judgment** — consider fixing if easy |
| Conflicting recommendations | **Analyze why** — different methodological norms may apply |

### Step 5: Produce the Final Simulation Report

The consolidation should result in:
1. **Overall submission readiness**: Ready / Needs Major Revision / Not Ready
2. **Top 3 risks**: Most likely reasons for rejection
3. **Prioritized action list**: Ordered by severity × feasibility
4. **Estimated revision effort**: Quick fixes (<1 day) vs major revisions (>1 week)

## Quality Bar

The simulation is **ready** when:

- [ ] At least 3 independent persona reviews completed
- [ ] Each review has summary + recommendation + major issues + minor issues
- [ ] Methods are assessed for identification/validity (not just surface)
- [ ] Literature coverage and positioning are assessed
- [ ] Writing clarity and internal consistency are assessed
- [ ] Consolidated reconciliation matrix produced
- [ ] Prioritized action list maps each issue to a fix location and effort
- [ ] Overall submission readiness recommendation given

## Minimal Output Format

```markdown
# Peer Review Simulation

## Overall Readiness: [Ready / Needs Revision / Not Ready]
## Top 3 Rejection Risks:
1. ...
2. ...
3. ...

## Individual Reviews

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
