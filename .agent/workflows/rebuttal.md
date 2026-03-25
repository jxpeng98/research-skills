---
description: 把审稿意见转成可执行 revision 计划 + response matrix + response letter
---

# Rebuttal / Revision Response

Turn reviews into a structured revision and response package.

Canonical Task ID (from the globally installed `research-paper-workflow` skill):
- `H2` rebuttal package

## Project

$ARGUMENTS

## Workflow

### Step 0: Select Project Folder

Ask the user:
> "Which `RESEARCH/[topic]/` folder should store revision artifacts?"

Ensure structure exists:
```
RESEARCH/[topic]/revision/
├── response_matrix.md
├── response_letter.md
└── revision_plan.md          # optional
```

### Step 1: Collect Reviews

Ask the user to provide:
- Editor letter (if any)
- Reviewer comments (R1/R2/…)
- Any constraints (no new data? timeline? must keep word count?)

### Step 2: Generate Rebuttal Package

Use **rebuttal-assistant** to produce:
- `RESEARCH/[topic]/revision/response_matrix.md` (use `templates/rebuttal-response-matrix.md`)
- `RESEARCH/[topic]/revision/response_letter.md` (use `templates/rebuttal-letter.md`)
- Optional `RESEARCH/[topic]/revision/revision_plan.md`

Begin rebuttal drafting now.
