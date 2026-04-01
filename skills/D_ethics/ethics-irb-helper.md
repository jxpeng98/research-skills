---
id: ethics-irb-helper
stage: D_ethics
description: "Prepare IRB-ready materials: risk assessment, consent, recruitment, data governance."
inputs:
  - type: DesignSpec
    description: "Study design with participant/data types"
outputs:
  - type: EthicsPackage
    artifact: "ethics_irb.md"
constraints:
  - "Must classify risk level (exempt / expedited / full board)"
  - "Must generate consent document appropriate to risk level"
  - "Must address applicable data governance frameworks (GDPR, HIPAA, etc.)"
  - "Must document any deception, sensitive topics, or vulnerable populations"
failure_modes:
  - "Study type does not involve human subjects but review is still needed"
  - "Cross-jurisdiction research with conflicting ethics requirements"
  - "Secondary data re-analysis with unclear original consent scope"
tools: [filesystem]
tags: [ethics, IRB, consent, privacy, GDPR, compliance, human-subjects]
domain_aware: false
---

# Ethics & IRB Helper Skill

Prepare a complete ethics/IRB application package: risk classification, informed consent, recruitment materials, data governance, and required disclosure statements.

## Purpose

Prepare IRB-ready materials: risk assessment, consent, recruitment, data governance.

## Related Task IDs

- `D1` (ethics/IRB pack)

## Output (contract path)

- `RESEARCH/[topic]/ethics_irb.md`

## When to Use

- Before data collection involving human participants or sensitive data
- Before using secondary datasets (verify original consent covers your use case)
- When the venue or institution requires an ethics review statement
- When research touches vulnerable populations, deception, or sensitive topics

## Process

### Step 1: Determine Whether Ethics Review is Required

| Research Activity | Review Required? | Typical Level |
|------------------|-----------------|---------------|
| Primary data collection (surveys, interviews, experiments) | ✅ Yes | Expedited or Full |
| Observation of public behavior (no interaction) | ⚠️ Usually exempt | Exempt |
| Secondary analysis of de-identified public datasets | ⚠️ Depends on original consent | Exempt or Not required |
| Text/document analysis (no human subjects) | Usually no | N/A |
| Analysis of publicly available social media | ⚠️ Contested — check institutional policy | Varies |

> **When in doubt**: Err on the side of submitting. An "exempt" determination from the IRB is still a formal record that protects you.

### Step 2: Classify Risk Level

| Level | Criteria | Typical Studies |
|-------|----------|-----------------|
| **Exempt** | Minimal risk; normal educational/survey/interview settings; de-identified data | Anonymous surveys, public observation, secondary data |
| **Expedited** | No more than minimal risk; some identifiable data; standard procedures | Recorded interviews, identifiable surveys, low-risk behavioral tasks |
| **Full Board** | More than minimal risk; vulnerable populations; deception; sensitive topics | Clinical interventions, children, prisoners, deception studies |

**Risk factors that upgrade classification**:
- Minors (<18) or other vulnerable groups → typically Full Board
- Financial/legal jeopardy if data disclosed → at least Expedited
- Deception involved → Full Board (most institutions)
- Physiological measurements or interventions → Full Board
- Cross-cultural research with different vulnerability standards → consult both IRBs

### Step 3: Prepare Consent Document

Generate a consent form containing all required elements:

#### Informed Consent Required Elements

| Element | Content | Required For |
|---------|---------|-------------|
| **Study title** | Full title as submitted | All |
| **PI and contact** | Name, institution, email, phone | All |
| **Purpose** | Why the study is being done, in plain language | All |
| **Procedures** | What participation involves, step by step | All |
| **Duration** | Expected time commitment (per session and total) | All |
| **Risks** | Physical, psychological, social, economic, legal risks | All |
| **Benefits** | To participant (if any) and to society/science | All |
| **Alternatives** | Other options available to participant | Clinical |
| **Confidentiality** | How data stored, who has access, retention period | All |
| **Compensation** | Amount, form, and conditions | If applicable |
| **Voluntary participation** | Right to withdraw at any time without penalty | All |
| **Data sharing** | Whether data may be shared or reused | When applicable |
| **Future use** | Whether data may be used for future research | Best practice |
| **Contact for questions** | PI + independent contact (ombudsperson/IRB office) | All |
| **Signature/consent mechanism** | Physical signature, electronic consent, or verbal consent documentation | All |

**For online/survey research**, adapt:
- Use checkbox consent (not wet signature)
- Include data collection technology disclosure (cookies, IP logging)
- Specify right to delete data and deadline for withdrawal

**For interview research**, add:
- Recording consent (audio/video separately)
- Transcript review opportunity
- Pseudonym/anonymization procedure
- Data excerpt usage in publications

### Step 4: Prepare Recruitment Materials

| Material | Purpose | Key Requirements |
|----------|---------|-----------------|
| Recruitment script/email | Initial contact | IRB-approved language; no coercion |
| Flyer/posting | Broad recruitment | Must include PI contact; avoid guaranteeing benefits |
| Screening questionnaire | Eligibility determination | Keep minimal; destroy ineligible data |
| Social media post | Online recruitment | Platform-specific rules; screenshot for records |

**Anti-coercion checklist**:
- [ ] No academic credit as sole incentive (offer alternatives)
- [ ] No employer/supervisor in recruitment chain
- [ ] Compensation is not so high as to constitute undue inducement
- [ ] Power dynamics addressed (professor↔student, doctor↔patient)

### Step 5: Data Governance Plan

| Aspect | What to Document |
|--------|-----------------|
| **Storage** | Where (institution server, cloud, local), encryption standard |
| **Access control** | Who can access raw vs de-identified data |
| **De-identification** | Method (see `deidentification-planner` D3) |
| **Retention** | How long, per institutional/funder requirements |
| **Destruction** | When and how data will be destroyed |
| **Transfer** | Rules for sharing with collaborators, especially cross-border |
| **Breach protocol** | Steps if data is compromised |

**Regulatory frameworks to address** (if applicable):

| Framework | Applies When | Key Requirements |
|-----------|-------------|-----------------|
| GDPR | EU/EEA participants or researchers | Lawful basis, DPO, DPIA, data subject rights |
| HIPAA | US health data | De-identification standard (Safe Harbor or Expert Determination) |
| FERPA | US student records | Consent or directory exception |
| COPPA | US, children under 13 online | Parental consent, data minimization |
| TCPS2 | Canadian research | Tri-Council policy compliance |
| Local/institutional | Always | Check your institution's specific requirements |

### Step 6: Required Statements for Manuscript

Draft the following for inclusion in the manuscript (→ feeds into H1 submission package):

```
Ethics statement:
"This study was approved by [Institution] IRB (protocol #[number], approved [date]).
All participants provided [written/verbal/electronic] informed consent prior to participation."

Data availability statement:
"De-identified data are available at [repository/DOI] / upon reasonable request to the corresponding author / cannot be shared due to [IRB restrictions/participant privacy]."

AI disclosure statement (increasingly required):
"[AI tool] was used for [specific purpose]. All AI-assisted outputs were reviewed and verified by the authors. The authors take full responsibility for the content."
```

## Quality Bar

An ethics package is **ready** when:

- [ ] Risk level classified with justification
- [ ] Consent document contains ALL required elements for the risk level
- [ ] Recruitment materials are non-coercive and IRB-appropriate
- [ ] Data governance plan covers storage, access, retention, and destruction
- [ ] Applicable regulatory frameworks identified and addressed
- [ ] Manuscript ethics statement drafted
- [ ] If vulnerable populations: specific protections documented
- [ ] If deception: debriefing procedure included

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| "No human subjects" assumption for online data | Social media users may be identifiable | Check institutional policy on internet research |
| Consent form too technical | Participants don't understand risks | Write at 8th-grade reading level |
| Missing cross-border considerations | GDPR applies to EU participants regardless of researcher location | Address data transfer mechanisms (SCCs, adequacy decisions) |
| Forgetting to IRB-approve recruitment materials | All participant-facing materials need approval | Include flyers, emails, scripts in submission |
| Not planning for incidental findings | Sensitive data may reveal concerning information | Pre-specify what constitutes a reportable finding and the response protocol |

## Minimal Output Format

```markdown
# Ethics / IRB Package

## Risk Classification
- Level: [Exempt / Expedited / Full Board]
- Justification: ...
- Special considerations: [vulnerable populations / deception / sensitive data]

## Informed Consent Document
[Full consent form with all required elements]

## Recruitment Materials
- [Recruitment email/script]
- [Flyer/posting]

## Data Governance
| Aspect | Plan |
|--------|------|
| Storage | ... |
| Access | ... |
| De-identification | ... |
| Retention | ... |
| Destruction | ... |
| Regulatory frameworks | ... |

## Manuscript Statements
- Ethics statement: ...
- Data availability: ...
- AI disclosure: ...
```
