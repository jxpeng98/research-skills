---
id: statement-generator
stage: D_ethics
description: "Generate standardized ethics, data availability, and conflict-of-interest statements for manuscript inclusion."
inputs:
  - type: EthicsPackage
    description: "Approved IRB and ethics materials"
  - type: DataManagementPlan
    description: "Data governance and sharing plan"
outputs:
  - type: Manuscript
    artifact: "manuscript/manuscript.md"
constraints:
  - "Must strictly adhere to ICMJE statement guidelines"
  - "Must clearly state IRB approval numbers if applicable"
  - "Data availability statements must include repository links or reasons for restriction"
failure_modes:
  - "Using generic 'data available upon request' statements when prohibited by the journal"
  - "Omitting secondary data source citations in the ethics statement"
tools: [filesystem]
tags: [ethics, statements, compliance, irb, data-availability]
domain_aware: true
---

# Statement Generator Skill

Generate boilerplate and standardized compliance statements required for journal submissions, translating complex ethics and data management packages into concise, journal-ready text.

## Purpose

Generate standardized ethics, data availability, and conflict-of-interest statements for manuscript inclusion.

## Related Task IDs

- `D2` (ethics-statements)

## Output (contract path)

- `RESEARCH/[topic]/manuscript/manuscript.md` (Appended to the statements section)

## When to Use

- When finalizing a manuscript for submission and ensuring compliance with journal statement requirements (e.g., PLOS, Nature, Elsevier).
- When translating an IRB approval into a concise 'Ethics Declarations' paragraph.

## Process

### Step 1: Input Analysis
- Review the `EthicsPackage` to extract approval statuses, institutional names, and waiver details.
- Review the `DataManagementPlan` to identify where data and code are hosted, or constraints preventing sharing.

### Step 2: Information Synthesis
Extract and explicitly address:
1. **Ethical Approval**: Did an IRB approve it? What is the protocol number? Was informed consent obtained?
2. **Data Availability**: Is data public? If so, what is the DOI/link? If not, under what conditions can it be accessed?
3. **Code Availability**: Are analysis scripts published?
4. **Conflicts of Interest**: Are there any financial or non-financial conflicts to declare?

### Step 3: Drafting Statements
Generate standardized paragraphs for:
- **Ethics Statement**
- **Data Availability Statement**
- **Code Availability Statement**
- **Competing Interests Statement**

### Step 4: Manuscript Insertion
Format the output as Markdown paragraphs, ready to be appended to the 'Declarations' or 'Methods' section of the manuscript.

## Quality Bar

- [ ] All required statements (Ethics, Data, Code, COI) are present.
- [ ] Contains specific institutional names and protocol numbers, avoiding placeholders if data exists.
- [ ] Data availability statement clearly delineates public vs. restricted data.
- [ ] Language is highly formal, objective, and aligns with ICMJE standards.

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Missing Protocol Numbers | States "Approved by IRB" without proof | Extract and include the exact numeric ID from the EthicsPackage |
| "Upon reasonable request" | Many journals outright ban this phrase | Specify the exact contact point, conditions, and repository link |
| Conflating Data and Code | Mentioning only data but ignoring scripts | Keep Data Availability and Code Availability explicitly separated |
