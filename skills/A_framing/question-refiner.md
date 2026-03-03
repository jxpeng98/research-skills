# Question Refiner Skill

Transform vague research topics into structured, answerable research questions using academic frameworks.

## Purpose

Convert raw research ideas into well-structured research questions that are:
- Specific and focused
- Answerable within scope
- Grounded in appropriate frameworks (PICO/PEO)
- Evaluated by FINER criteria

## Process

### Step 1: Topic Exploration

Ask clarifying questions:

1. **Domain Clarification**
   - What is the broad research area?
   - What discipline(s) does this span?
   - What is the specific phenomenon of interest?

2. **Scope Definition**
   - What aspects are you most interested in?
   - What is the geographic scope?
   - What is the temporal scope?
   - What population/context is relevant?

3. **Output Requirements**
   - What type of output do you need? (Review, empirical study, theoretical paper)
   - What is the target venue/audience?
   - What is the expected depth of analysis?

### Step 2: Framework Application

#### For Intervention/Effect Studies - Use PICO

- **P**opulation: Who is being studied?
- **I**ntervention: What intervention or exposure?
- **C**omparison: What is the comparison/control?
- **O**utcome: What outcomes are measured?

Example:
- P: University students
- I: AI tutoring systems
- C: Traditional tutoring
- O: Learning outcomes, engagement

→ RQ: "How do AI tutoring systems compare to traditional tutoring in improving learning outcomes and engagement among university students?"

#### For Non-Intervention Studies - Use PEO

- **P**opulation: Who is being studied?
- **E**xposure: What phenomenon/exposure?
- **O**utcome: What is observed/measured?

Example:
- P: Software developers
- E: Remote work adoption
- O: Productivity, well-being

→ RQ: "How does remote work adoption affect productivity and well-being among software developers?"

### Step 3: Question Refinement

Transform into specific question types:

| Type | Starter | Use When |
|------|---------|----------|
| Descriptive | "What is...?" | Exploring phenomenon |
| Comparative | "How does X compare to Y...?" | Comparing groups/conditions |
| Relational | "What is the relationship between...?" | Examining associations |
| Causal | "Does X cause Y...?" | Testing causation |
| Exploratory | "How do participants experience...?" | Understanding perspectives |

### Step 4: FINER Evaluation

Evaluate the research question:

| Criterion | Question | Assessment |
|-----------|----------|------------|
| **F**easible | Can it be done with available resources? | ✓/✗ |
| **I**nteresting | Is it worth investigating? | ✓/✗ |
| **N**ovel | Does it add new knowledge? | ✓/✗ |
| **E**thical | Can it be done ethically? | ✓/✗ |
| **R**elevant | Does it matter to stakeholders? | ✓/✗ |

### Step 5: Output

Generate structured output:

```markdown
## Research Question

**Main RQ**: [Refined research question]

**Sub-questions**:
1. [Sub-RQ 1]
2. [Sub-RQ 2]
3. [Sub-RQ 3]

## Framework Used
[PICO/PEO breakdown]

## FINER Assessment
| Criterion | Assessment | Notes |
|-----------|------------|-------|
| Feasible | ✓ | ... |
| Interesting | ✓ | ... |
| Novel | ✓ | ... |
| Ethical | ✓ | ... |
| Relevant | ✓ | ... |

## Inclusion Criteria
- Criterion 1
- Criterion 2

## Exclusion Criteria
- Criterion 1
- Criterion 2

## Key Search Terms
- Term 1
- Term 2 (synonym: ...)
```

## Usage

This skill is called by:
- `/lit-review` - At the scoping phase
- `/find-gap` - To clarify research area
- `/build-framework` - To define scope
