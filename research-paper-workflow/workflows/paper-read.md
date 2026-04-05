---
description: 深度阅读并分析单篇学术论文，生成结构化笔记
---

# Deep Paper Reading & Analysis

Conduct in-depth reading and structured analysis of an academic paper.

Canonical Task ID (from the globally installed `research-paper-workflow` skill):
- `B2` targeted paper reading

## Paper

$ARGUMENTS

## Workflow

### Step 0: Project Context Selection

**Required**: Determine which research project this paper belongs to.

Ask the user:
> "Which research project folder should this paper be saved to?"
> - Existing projects: [List folders under `RESEARCH/`]
> - Create new: `RESEARCH/[new-topic]/`
> - Standalone: `RESEARCH/standalone/`

Set `[topic]` variable based on user selection.

Ensure the target directory structure exists:
```
RESEARCH/[topic]/
├── notes/
└── bibliography.bib
```

### Step 1: Paper Retrieval

Attempt to access the paper through:
1. Direct URL (if provided)
2. DOI lookup via Semantic Scholar API
3. arXiv API (for arXiv links)
4. Title search via Semantic Scholar

If full text unavailable, work with abstract and metadata.

### Step 2: Metadata Extraction

Extract bibliographic information:
- Title
- Authors (with affiliations if available)
- Publication venue (journal/conference)
- Year
- Volume/Issue/Pages
- DOI
- Keywords
- Abstract

### Step 3: Deep Reading Analysis

Use the **paper-extractor** skill to analyze:

#### Research Problem
- What problem does this paper address?
- Why is this problem important?
- What is the research gap being filled?

#### Research Questions/Objectives
- What are the explicit RQs or hypotheses?
- What are the research objectives?

#### Theoretical Framework
- What theories/frameworks guide the research?
- How are key concepts defined?
- What is the conceptual model (if any)?

#### Methodology
- **Research Design**: Qualitative/Quantitative/Mixed?
- **Sample/Data**: Who/what was studied? Sample size?
- **Data Collection**: How was data gathered?
- **Data Analysis**: What analytical methods were used?
- **Validity/Reliability**: How was rigor ensured?

#### Key Findings
- What are the main results?
- What patterns/themes emerged?
- What are the effect sizes/statistics (if quantitative)?

#### Contributions
- What new knowledge does this paper add?
- What are the theoretical contributions?
- What are the practical implications?

#### Limitations
- What limitations do the authors acknowledge?
- What limitations do you identify?

#### Future Research
- What do authors suggest for future work?
- What questions remain unanswered?

### Step 4: Critical Evaluation

Apply the **quality-assessor** skill:
- Assign A-E evidence rating
- Evaluate argument strength
- Assess methodological rigor
- Identify potential biases

### Step 5: Generate Outputs

**Paper Note** (Markdown):
Create structured note using the template → Save to `RESEARCH/[topic]/notes/[citekey].md`

Use the **metadata-enricher** skill to:
1. Normalize DOI and metadata via Crossref/OpenAlex
2. Generate consistent citekey

Use the **fulltext-fetcher** skill to:
1. Attempt OA retrieval if full text not available
2. Document retrieval status

**BibTeX Entry**:
Generate properly formatted BibTeX → Append to `RESEARCH/[topic]/bibliography.bib`

## Output Format

The paper note should follow this structure:

```markdown
# [Paper Title]

## Metadata
- **Authors**:
- **Year**:
- **Venue**:
- **DOI**:
- **Evidence Rating**: [ ] A [ ] B [ ] C [ ] D [ ] E

## Quick Summary
[2-3 sentence summary]

## Research Problem
[Problem statement and significance]

## Research Questions
1. RQ1: ...
2. RQ2: ...

## Theoretical Framework
[Theories and key concepts]

## Methodology
| Aspect | Description |
|--------|-------------|
| Design | |
| Sample | |
| Data Collection | |
| Analysis | |

## Key Findings
- Finding 1
- Finding 2
- ...

## Contributions
- Theoretical: 
- Practical: 

## Limitations
- 

## Future Research
- 

## My Notes
[Personal reflections, connections to other work, questions]

## BibTeX
```bibtex
@article{...}
```
```

Begin deep paper analysis now.
