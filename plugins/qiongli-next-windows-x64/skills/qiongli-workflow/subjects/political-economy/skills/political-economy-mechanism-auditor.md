---
id: political-economy-mechanism-auditor
stage: H_submission
description: "Audit political economy drafts for actor, institution, incentive, distributional conflict, and policy mechanism alignment."
inputs:
  - type: DesignSpec
    description: "Research question, institutional setting, actor set, method, and target outcome"
  - type: LiteratureMap
    description: "Political economy debate, rival mechanisms, and boundary conditions"
  - type: Manuscript
    description: "Draft argument, theory, design, findings, and discussion when available"
outputs:
  - type: PoliticalEconomyMechanismAudit
    artifact: "analysis/political_economy_mechanism_audit.md"
constraints:
  - "Must identify political actors, institutions, incentives, distributional conflict, and policy or economic outcome"
  - "Must distinguish political mechanism, correlation, causal claim, and interpretation"
  - "Must calibrate claim strength to direct mechanism evidence"
failure_modes:
  - "Policy outcome is treated as proof of political cause"
  - "Institutional setting is described but not used in the mechanism"
  - "Distributional conflict is implied but winners, losers, and incentives are not specified"
tools: [filesystem]
tags: [political-economy, political-mechanism, institution, policy]
domain_aware: true
---

# Political Economy Mechanism Auditor Skill

Audit whether a political economy manuscript connects actors, institutions, incentives, distributional conflict, and policy or economic outcomes clearly enough for reviewer scrutiny.

## Purpose

Prevent a political economy project from becoming either generic economics without politics or generic political science without a disciplined economic outcome and mechanism.

## When to Use

- Before finalizing the theory, research design, introduction, or discussion.
- When the paper claims a political mechanism behind a policy or economic outcome.
- When the strongest reviewer risk is weak actor logic, vague institutions, or unsupported distributional conflict.

## Inputs

- `DesignSpec`: research question, actor set, institution, outcome, method, evidence, and target venue.
- `LiteratureMap`: political economy debate, rival mechanisms, and adjacent literatures.
- `Manuscript`: introduction, theory, methods, findings, and discussion when available.
- `DomainProfile`: load `skills/domain-profiles/political-economy.yaml` when available.

If the actor set, institution, or outcome is missing, write a blocked-check note under `RESEARCH/[topic]/context/gap_notes.md` and do not invent the mechanism.

## Process

1. State the political economy claim in one sentence.
2. Identify actors, incentives, institution, distributional conflict, and policy or economic outcome.
3. Map the political mechanism from actor incentive to institutional action to outcome.
4. Check whether evidence supports the mechanism rather than only the outcome.
5. Compare the claim against at least two rival political or economic mechanisms.
6. Produce a pass / revise / blocked verdict.

## Output Contract

Write `RESEARCH/[topic]/analysis/political_economy_mechanism_audit.md`:

```markdown
# Political Economy Mechanism Audit

## Claim Under Review
- Focal claim:
- Claim strength:
- Target venue or literature:

## Actor-Institution-Outcome Map
| Element | Current statement | Evidence | Gap |
|---|---|---|---|
| Actors | | | |
| Incentives | | | |
| Institution | | | |
| Distributional conflict | | | |
| Policy or economic outcome | | | |

## Mechanism Evidence
| Mechanism step | Evidence source | Rival explanation | Status |
|---|---|---|---|

## Reviewer Risk
- Most likely objection:
- Required narrowing:
- Required evidence:

## Verdict
- Decision: pass / revise / blocked
- Required changes:
```

## Quality Bar

- The audit names the political mechanism rather than only describing a policy outcome.
- The institution, actor incentives, and distributional conflict are explicit.
- Causal claims are separated from descriptive or interpretive claims.
- Rival mechanisms are visible and tied to specific evidence gaps.
- The verdict narrows overclaims before submission or final writing.
