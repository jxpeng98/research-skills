---
id: geoeconomic-statecraft-auditor
stage: H_submission
description: "Audit geoeconomics drafts for economic statecraft logic, sanctions or supply chain evidence, and strategic competition claim calibration."
inputs:
  - type: DesignSpec
    description: "Research question, economic instrument, sender, target, response channel, method, and evidence"
  - type: LiteratureMap
    description: "Geoeconomics, international security, IPE, and policy literatures"
  - type: Manuscript
    description: "Draft argument, design, evidence, implications, and discussion when available"
outputs:
  - type: GeoeconomicStatecraftAudit
    artifact: "analysis/geoeconomic_statecraft_audit.md"
constraints:
  - "Must identify instrument, sender, target, intermediary, affected market, and response channel"
  - "Must distinguish statecraft mechanism from national-security rhetoric"
  - "Must evaluate timing, exposure, substitution, and strategic competition claim strength"
failure_modes:
  - "Sanctions or export controls are described without sender-target-response logic"
  - "Supply chain risk is asserted without exposure construction"
  - "Strategic competition claims exceed the economic evidence"
tools: [filesystem]
tags: [geoeconomics, statecraft, sanctions, supply-chain, strategic-competition]
domain_aware: true
---

# Geoeconomic Statecraft Auditor Skill

Audit whether a geoeconomics manuscript treats economic tools as strategic instruments with clear actors, targets, mechanisms, responses, and evidence boundaries.

## Purpose

Prevent a geoeconomics project from turning policy rhetoric about strategic competition into unsupported empirical or causal claims.

## When to Use

- Before finalizing the argument, research design, policy implications, or submission package.
- When the paper studies sanctions, export controls, industrial policy, financial statecraft, investment screening, or supply chain security.
- When reviewer risk centers on timing, exposure, substitution, target response, or unsupported strategic inference.

## Inputs

- `DesignSpec`: economic instrument, sender, target, intermediaries, market or firm response, method, and evidence.
- `LiteratureMap`: geoeconomics, international security, IPE, policy, and market-response literatures.
- `Manuscript`: introduction, theory, methods, findings, and discussion when available.
- `DomainProfile`: load `skills/domain-profiles/geoeconomics.yaml` when available.

If the instrument, target, or response channel is missing, write a blocked-check note under `RESEARCH/[topic]/context/gap_notes.md` and do not invent the statecraft mechanism.

## Process

1. State the geoeconomic statecraft claim in one sentence.
2. Identify sender, target, instrument, intermediaries, affected market, and response channel.
3. Classify the strategic objective as coercion, deterrence, resilience, signaling, industrial policy, or mixed.
4. Check timing, exposure construction, sanctions or policy bundling, supply chain substitution, and countermeasure risks.
5. Separate policy rhetoric from direct evidence of strategic competition and market response.
6. Produce a pass / revise / blocked verdict.

## Output Contract

Write `RESEARCH/[topic]/analysis/geoeconomic_statecraft_audit.md`:

```markdown
# Geoeconomic Statecraft Audit

## Claim Under Review
- Focal claim:
- Strategic objective:
- Claim strength:

## Instrument-Sender-Target Map
| Element | Current statement | Evidence | Gap |
|---|---|---|---|
| Sender | | | |
| Instrument | | | |
| Target | | | |
| Intermediaries | | | |
| Market or firm response | | | |

## Timing And Exposure
| Risk | Current status | Required check |
|---|---|---|
| Anticipation | | |
| Policy bundling | | |
| Exposure construction | | |
| Evasion or substitution | | |

## Reviewer Risk
- Most likely objection:
- Required narrowing:
- Required evidence:

## Verdict
- Decision: pass / revise / blocked
- Required changes:
```

## Quality Bar

- The audit identifies economic statecraft instruments and response channels explicitly.
- Sanctions, supply chain, or policy evidence is tied to timing and exposure.
- Strategic competition claims are separated from rhetoric and calibrated to evidence.
- Evasion, substitution, and countermeasure risks are addressed.
- The verdict narrows security overclaims before submission or final writing.
