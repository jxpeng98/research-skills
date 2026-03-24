---
id: ethics-irb-helper
stage: D_ethics
version: "0.2.2"
description: "Prepare IRB-ready materials including risk assessment, informed consent, recruitment scripts, and data governance."
inputs:
  - type: DesignSpec
    description: "Study design with participant details"
outputs:
  - type: EthicsPackage
    artifact: "ethics_irb.md"
constraints:
  - "Must address all standard IRB categories (risk, consent, privacy, data security)"
  - "Must comply with relevant regulations (GDPR, HIPAA, Common Rule)"
failure_modes:
  - "Incomplete risk assessment for vulnerable populations"
  - "Missing data retention/destruction plan"
tools: [filesystem]
tags: [ethics, IRB, consent, privacy, GDPR, compliance]
domain_aware: false
---

# Ethics / IRB Helper Skill

Prepare an ethics-ready research plan and IRB-style documentation bundle (templates, checklists, and text drafts). This is organizational support, **not legal advice**.

## When to Use

- Any study involving human participants, sensitive data, or potentially harmful interventions.
- Before data collection begins.

## Inputs (Ask / Collect)

- Participant population (including vulnerable groups)
- Data types (audio/video, medical, identifiers, location, biometrics, etc.)
- Recruitment method + incentives
- Consent process (online/in-person), withdrawal policy
- Data storage/sharing plans (retention, anonymization)
- Risks/benefits and mitigation measures

## Process

1. **Risk classification**: minimal vs more-than-minimal; identify sensitive domains
2. **Consent & transparency**: what participants are told; comprehension checks if needed
3. **Privacy & security**: data minimization, access control, encryption, retention schedule
4. **Harm mitigation**: adverse events plan, debriefing, support resources
5. **Equity & fairness**: inclusion, compensation fairness, avoid coercion
6. **Data sharing**: de-identification plan; what will be public vs restricted
7. **Ethics statements**: manuscript-ready ethics and data availability statements

## Outputs (Create/Update)

Use `templates/ethics-irb-pack.md` and produce:
- `RESEARCH/[topic]/ethics_irb.md`
- `RESEARCH/[topic]/instruments/consent_form.md` (if applicable)
- `RESEARCH/[topic]/instruments/recruitment_script.md` (if applicable)
- Update `RESEARCH/[topic]/data_management_plan.md` (if needed)

