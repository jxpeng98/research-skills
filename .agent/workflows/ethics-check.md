---
description: 生成伦理/IRB 文档包（consent、recruitment、data security、statement）
---

# Ethics / IRB Pack

Prepare ethics-ready documentation for a study. This is organizational support, not legal advice.

Canonical Task ID (from the globally installed `research-paper-workflow` skill):
- `D1` ethics pack

## Project

$ARGUMENTS

## Workflow

### Step 0: Select Project Folder

Ask the user:
> "Which `RESEARCH/[topic]/` folder should receive the ethics pack?"

### Step 1: Collect Ethics Inputs

Ask:
1. Participant population + any vulnerable groups
2. Data types (PII/sensitive, audio/video, identifiers)
3. Recruitment method + compensation
4. Consent procedure (online/in-person) + withdrawal policy
5. Data storage/encryption + retention schedule
6. Data sharing plan (open/restricted/none)

### Step 2: Generate Ethics Pack

Use **ethics-irb-helper** with `templates/ethics-irb-pack.md` to produce:
- `RESEARCH/[topic]/ethics_irb.md`
- (if applicable) `RESEARCH/[topic]/instruments/consent_form.md`
- (if applicable) `RESEARCH/[topic]/instruments/recruitment_script.md`
- Update `RESEARCH/[topic]/data_management_plan.md` if needed

Begin ethics pack drafting now.
