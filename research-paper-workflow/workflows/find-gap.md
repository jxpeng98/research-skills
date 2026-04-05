---
description: 系统性识别研究领域中的学术空白和研究机会
---

# Research Gap Identification

Systematically identify research gaps and opportunities in a given research area.

Canonical Task ID (from the globally installed `research-paper-workflow` skill):
- `A4` research gap scan

## Research Area

$ARGUMENTS

## Workflow

### Step 1: Research Question Clarification

Use the **question-refiner** skill to:
1. Clarify the scope of the research area
2. Identify sub-domains of interest
3. Define temporal boundaries (recent 5 years?)
4. Specify disciplinary perspective

### Step 2: Literature Landscape Mapping

Use the **academic-searcher** skill to:
1. Search Semantic Scholar for recent publications
2. Identify seminal/highly-cited papers
3. Find recent review articles in the area
4. Map key authors and research groups

### Step 3: Gap Analysis

Use the **gap-analyzer** skill to identify 5 types of gaps:

#### 1. Theoretical Gaps
- Are there competing theories that haven't been reconciled?
- Are there concepts lacking clear definition?
- Are there phenomena without theoretical explanation?
- Is there missing integration between theories?

**Analysis Questions:**
- What theoretical frameworks are used?
- Where do theories conflict?
- What theoretical extensions are needed?

#### 2. Methodological Gaps
- What methods dominate the field?
- What methods are underutilized?
- Are there validity/reliability concerns?
- Are there calls for methodological innovation?

**Analysis Questions:**
- What research designs are common?
- What analysis techniques are used?
- What methodological limitations are frequently mentioned?

#### 3. Empirical Gaps
- What contexts haven't been studied?
- What time periods need investigation?
- What geographic regions are underrepresented?
- What sectors/industries lack research?

**Analysis Questions:**
- Where was the research conducted?
- What settings were examined?
- What contexts remain unexplored?

#### 4. Knowledge Gaps
- What topics are understudied?
- What relationships haven't been examined?
- What variables are overlooked?
- What questions remain unanswered?

**Analysis Questions:**
- What do researchers say is unknown?
- What future research is suggested?
- What controversies exist?

#### 5. Population Gaps
- What demographic groups are underrepresented?
- What organizational types lack study?
- What cultural contexts are missing?
- What stakeholder perspectives are absent?

**Analysis Questions:**
- Who has been studied?
- Who is missing from the research?
- Whose voices are not heard?

### Step 4: Gap Prioritization

Evaluate each identified gap using FINER criteria:

| Gap | Feasible | Interesting | Novel | Ethical | Relevant | Priority |
|-----|----------|-------------|-------|---------|----------|----------|
| Gap 1 | ✓/✗ | ✓/✗ | ✓/✗ | ✓/✗ | ✓/✗ | H/M/L |
| Gap 2 | ... | ... | ... | ... | ... | ... |

### Step 5: Research Opportunity Synthesis

For each high-priority gap, develop:
1. Potential research question
2. Suggested methodology
3. Expected contribution
4. Feasibility assessment
5. Relevant literature foundation

### Step 6: Generate Output

Create gap analysis report:

```markdown
# Research Gap Analysis: [Topic]

## Executive Summary
[2-3 paragraph overview of key gaps]

## Literature Landscape
[Overview of current research state]

## Identified Gaps

### Theoretical Gaps
| Gap | Description | Evidence | Priority |
|-----|-------------|----------|----------|

### Methodological Gaps
| Gap | Description | Evidence | Priority |
|-----|-------------|----------|----------|

### Empirical Gaps
| Gap | Description | Evidence | Priority |
|-----|-------------|----------|----------|

### Knowledge Gaps
| Gap | Description | Evidence | Priority |
|-----|-------------|----------|----------|

### Population Gaps
| Gap | Description | Evidence | Priority |
|-----|-------------|----------|----------|

## Research Opportunities
### Opportunity 1: [Title]
- **Gap Addressed**: 
- **Proposed RQ**: 
- **Methodology**: 
- **Expected Contribution**: 
- **Key References**: 

## Conclusion
[Summary and recommendations]

## References
```

Output: `RESEARCH/[topic]/gap_analysis.md`

Begin research gap identification now.
