---
description: 执行系统性文献综述，遵循 PRISMA 2020 方法论
argument-hint: [研究主题] [可选:年份范围如2020-2024]
---

# Systematic Literature Review

Execute a systematic literature review following PRISMA 2020 methodology.

Canonical Task ID (from the globally installed `research-paper-workflow` skill):
- `B1` systematic review pipeline

## Topic

$ARGUMENTS

## Workflow

### Phase 0: Project Scaffolding

Create the project directory structure:

```
RESEARCH/[topic]/
├── protocol.md
├── search_strategy.md
├── search_log.md
├── search_results.csv
├── screening/
│   ├── title_abstract.md
│   ├── full_text.md
│   └── prisma_flow.md
├── notes/
├── extraction_table.md
├── quality_table.md
├── meta_analysis_plan.md        # optional (if quantitative pooling)
├── effect_size_table.md         # optional (if quantitative pooling)
├── meta_analysis_results.md     # optional (if quantitative pooling)
├── analysis/                    # optional (code + plots)
├── synthesis_matrix.md
├── synthesis.md
└── bibliography.bib
```

**Naming Convention:** Convert topic to lowercase, replace spaces with hyphens (e.g., "AI in Education" → `ai-in-education`).

### Phase 1: Research Question Scoping

Use the **question-refiner** skill to:
1. Ask clarifying questions about the research focus
2. Apply PICO (intervention) or PEO (non-intervention) framework
3. Generate a structured research question
4. Define inclusion/exclusion criteria

Output: Structured RQ and protocol draft → `RESEARCH/[topic]/protocol.md`

### Phase 1.5: Protocol Registration & Governance

**STOP & CONFIRM**: Before proceeding, ask the user:
> "Do you want to register this protocol on PROSPERO/OSF for transparency? (Y/N)"

If yes:
1. Generate protocol using **templates/protocol-template.md**
2. Include registration ID field for later completion
3. Document any planned deviations from standard PRISMA

Establish amendment tracking:
- All protocol changes must be logged in `RESEARCH/[topic]/protocol.md` with date and rationale
- Reference: PRISMA-P 2015 guidelines

Output: Updated protocol with registration section → `RESEARCH/[topic]/protocol.md`

### Phase 2: Search Strategy Development

Develop comprehensive search strategy:
1. Identify key concepts and synonyms
2. Build Boolean search queries
3. Select appropriate databases:
   - Semantic Scholar (primary - 200M+ papers)
   - arXiv (CS/AI/Physics/Math)
   - Google Scholar (broad coverage)
   - PubMed (biomedical - if relevant)

Output: Search strategy document → `RESEARCH/[topic]/search_strategy.md`

**STOP & CONFIRM**: Present the search strategy to the user:
> "Please review the search strategy above. Proceed with execution? (Y/N)"

Wait for explicit user approval before proceeding to Phase 3.

### Phase 3: Literature Search Execution

Use the **academic-searcher** skill to:
1. Execute Semantic Scholar API search
2. Execute arXiv API search (if relevant domain)
3. Supplement with Google Scholar web search
4. Remove duplicates (DOI-first, then title+year+author)
5. Document search results per database

**Document all searches** → `RESEARCH/[topic]/search_log.md`
**Save dedup-ready record table** → `RESEARCH/[topic]/search_results.csv`

Output: Raw search results with counts

**Iteration Check:** If total unique results < 20, consider:
- Broadening search terms
- Adding synonyms
- Expanding date range
- Searching additional databases
Then return to Phase 2 to refine strategy.

### Phase 3.5: Citation Snowballing & Grey Literature

Use the **citation-snowballer** skill to:
1. Identify seed papers (highly cited, seminal works from Phase 3)
2. Execute forward citation search (who cites these?)
3. Execute backward reference search (what do these cite?)
4. Remove duplicates with Phase 3 results

Grey literature sources:
- ProQuest Dissertations & Theses
- Conference proceedings (ACM DL, IEEE Xplore)
- Working paper repositories (SSRN, NBER, RePEc)
- Institutional repositories

Use the **fulltext-fetcher** skill to:
1. Attempt OA retrieval via Unpaywall/CORE/Semantic Scholar
2. Document retrieval status for each paper
3. Log "not retrieved" reasons (required for PRISMA)

Output:
- Snowball log → `RESEARCH/[topic]/snowball_log.md`
- Updated search log → `RESEARCH/[topic]/search_log.md`

### Phase 4: Screening

Use the **paper-screener** skill to:

**Stage 1: Title/Abstract Screening**
- Apply inclusion criteria
- Apply exclusion criteria
- Document decisions with reasons
- Output: `RESEARCH/[topic]/screening/title_abstract.md`

**Stage 2: Full-text Screening**
- Retrieve and review full texts
- Verify eligibility
- Document exclusion reasons
- Output: `RESEARCH/[topic]/screening/full_text.md`

Generate PRISMA 2020 flowchart → `RESEARCH/[topic]/screening/prisma_flow.md`

### Phase 5: Data Extraction

Use the **paper-extractor** skill to extract from each included paper:
- Bibliographic info (authors, year, journal, DOI)
- Research questions/objectives
- Theoretical framework
- Methodology (design, sample, data collection, analysis)
- Key findings
- Contributions
- Limitations
- Future work suggestions

Output: 
- Individual paper notes → `RESEARCH/[topic]/notes/`
- Extraction table → `RESEARCH/[topic]/extraction_table.md`

### Phase 6: Quality Assessment

Use the **quality-assessor** skill to:
1. **Select appropriate RoB tool** based on study design:
   - RCTs → RoB 2
   - Non-randomized interventions → ROBINS-I
   - Diagnostic studies → QUADAS-2
   - Qualitative studies → CASP Qualitative Checklist
   - Systematic reviews → AMSTAR 2
2. Apply design-specific risk of bias assessment
3. Apply A-E evidence rating to each paper
4. Evaluate methodological rigor

For systematic reviews with sufficient homogeneity, apply GRADE framework:
- Generate Summary of Findings (SoF) table using **templates/grade-summary-of-findings.md**
- Rate certainty of evidence per outcome

Output:
- Quality assessment table → `RESEARCH/[topic]/quality_table.md`
- Design-specific RoB tables → `RESEARCH/[topic]/rob_[design].md`
- GRADE SoF (if applicable) → `RESEARCH/[topic]/grade_sof.md`

### Phase 7: Evidence Synthesis (Narrative / Qualitative / Meta-analysis)

Use the **evidence-synthesizer** skill to:
1. Decide synthesis type *per outcome* (meta-analysis vs narrative vs qualitative) and document rationale
2. Build/update synthesis matrix → `RESEARCH/[topic]/synthesis_matrix.md`
3. If quantitative pooling is feasible:
   - Draft synthesis plan → `RESEARCH/[topic]/meta_analysis_plan.md` (use **templates/meta-analysis-plan.md**)
   - Extract pooled-ready effect sizes → `RESEARCH/[topic]/effect_size_table.md` (use **templates/effect-size-extraction-table.md**)
   - Run pooling (optional) using code templates under `templates/code/statistics/`
   - Write-up quantitative results → `RESEARCH/[topic]/meta_analysis_results.md` (use **templates/meta-analysis-report.md**)
4. Integrate all outcomes (including heterogeneity, sensitivity, missing-results bias, certainty) → `RESEARCH/[topic]/synthesis.md`

Output: Synthesis report → `RESEARCH/[topic]/synthesis.md`

### Phase 8: Bibliography Generation

Use the **citation-formatter** skill to:
1. Generate BibTeX entries for all included papers
2. Create formatted reference list

Use the **reference-manager-bridge** skill to:
1. Export in multiple formats (BibTeX, RIS, CSL-JSON)
2. Generate Zotero/Mendeley compatible tags

Output: `RESEARCH/[topic]/bibliography.bib`

### Phase 9: PRISMA Compliance Verification

Use the **prisma-checker** skill to:
1. Complete PRISMA 2020 checklist
2. Verify consistency across all counts and logs
3. Generate compliance report

Output: `RESEARCH/[topic]/prisma_checklist.md`

## PRISMA Compliance Checklist

Ensure the review includes:
- [ ] Protocol registration (PROSPERO/OSF) or justification for not registering
- [ ] Explicit research question (PICO/PEO)
- [ ] Documented search strategy with PRESS verification
- [ ] Search log with timestamps and exact queries
- [ ] Citation snowballing documentation
- [ ] Grey literature search documentation
- [ ] Clear inclusion/exclusion criteria
- [ ] PRISMA 2020 flow diagram with all counts
- [ ] Full-text retrieval log with "not retrieved" reasons
- [ ] Data extraction table
- [ ] Design-specific quality/RoB assessment
- [ ] GRADE Summary of Findings (if applicable)
- [ ] Synthesis matrix
- [ ] Transparent reporting of all steps
- [ ] Amendment log (if protocol was modified)

Begin the systematic literature review now.
