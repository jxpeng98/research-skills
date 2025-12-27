# Paper Extractor Skill

Extract structured information from academic papers for analysis and synthesis.

## Purpose

Systematically extract key information from papers including:
- Bibliographic metadata
- Research design elements
- Findings and contributions
- Methodological details

## Extraction Framework

### 1. Bibliographic Information

| Field | Example |
|-------|---------|
| Title | |
| Authors | (Last, First; Last, First; ...) |
| Year | |
| Venue | Journal/Conference name |
| Volume/Issue | |
| Pages | |
| DOI | |
| URL | |

### 2. Research Context

| Field | Extraction Prompt |
|-------|------------------|
| Research Problem | What problem does this paper address? |
| Motivation | Why is this problem important? |
| Research Gap | What gap in knowledge does this fill? |
| Objectives | What are the stated objectives? |
| Research Questions | What RQs or hypotheses are presented? |

### 3. Theoretical Elements

| Field | Extraction Prompt |
|-------|------------------|
| Theoretical Framework | What theories guide the research? |
| Key Concepts | What are the main constructs/concepts? |
| Conceptual Model | Is there a conceptual/research model? |
| Hypotheses | What hypotheses are tested? |

### 4. Methodology

| Field | Extraction Prompt |
|-------|------------------|
| Research Design | Qualitative/Quantitative/Mixed? Experimental? |
| Population | Who/what is studied? |
| Sampling | How was sample selected? Size? |
| Data Collection | What instruments/methods? |
| Data Sources | Primary/Secondary? What sources? |
| Variables | IVs, DVs, Controls, Moderators, Mediators? |
| Analysis Methods | What analytical techniques? |
| Validity/Reliability | What measures of rigor? |

### 5. Findings

| Field | Extraction Prompt |
|-------|------------------|
| Key Findings | What are the main results? |
| Hypothesis Support | Which hypotheses supported/rejected? |
| Effect Sizes | What are the reported effect sizes? |
| Statistical Results | Key statistics (p-values, R², etc.) |
| Themes | (For qualitative) What themes emerged? |

### 6. Discussion Elements

| Field | Extraction Prompt |
|-------|------------------|
| Interpretation | How do authors interpret findings? |
| Theory Engagement | How do findings relate to theory? |
| Literature Dialogue | How do findings compare to prior work? |
| Implications | Theoretical and practical implications? |

### 7. Contributions & Limitations

| Field | Extraction Prompt |
|-------|------------------|
| Theoretical Contribution | What new theoretical knowledge? |
| Methodological Contribution | Any methodological innovations? |
| Practical Contribution | What practical applications? |
| Limitations | What limitations acknowledged? |
| Future Research | What future work suggested? |

## Extraction Output Template

```markdown
# Paper Extraction: [Short Citation]

## Bibliographic Information
| Field | Value |
|-------|-------|
| Title | |
| Authors | |
| Year | |
| Venue | |
| DOI | |

## Research Context
**Problem Statement:**
[Text]

**Research Gap:**
[Text]

**Research Questions:**
1. RQ1: 
2. RQ2:

## Theoretical Framework
**Theories Used:**
- Theory 1: [Brief description]
- Theory 2: [Brief description]

**Key Concepts:**
| Concept | Definition |
|---------|------------|
| | |

## Methodology
| Aspect | Description |
|--------|-------------|
| Design | |
| Population | |
| Sample | N = |
| Data Collection | |
| Analysis | |
| Validity | |

## Key Findings
1. Finding 1: [Description + Evidence]
2. Finding 2: [Description + Evidence]
3. Finding 3: [Description + Evidence]

## Statistical Results (if quantitative)
| Hypothesis | Result | Effect Size | p-value |
|------------|--------|-------------|---------|
| H1 | Supported/Rejected | | |

## Contributions
**Theoretical:**
- 

**Practical:**
- 

## Limitations
1. 
2. 

## Future Research Suggestions
1. 
2. 

## Relevance to My Research
[Personal notes on how this relates to your work]

## Key Quotes
> "[Important quote 1]" (p. X)

> "[Important quote 2]" (p. Y)
```

## Batch Extraction Table

For multiple papers, create summary extraction table:

```markdown
| ID | Authors | Year | RQs | Theory | Method | Sample | Key Findings | Quality |
|----|---------|------|-----|--------|--------|--------|--------------|---------|
| 1 | | | | | | | | A-E |
| 2 | | | | | | | | A-E |
```

## Usage

This skill is called by:
- `/lit-review` - During extraction phase
- `/paper-read` - For deep reading
