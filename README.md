# Academic Deep Research Skills

A systematic research skills system designed for Claude Code, providing tools for literature review, paper analysis, gap identification, and academic writing.

## Features

- 📚 **Systematic Literature Review** - PRISMA 2020 compliant methodology
- 📖 **Deep Paper Reading** - Structured notes + BibTeX
- 🔍 **Research Gap Identification** - 5 types of academic gap analysis
- 🧠 **Theoretical Framework Building** - Concept relationship mapping
- ✍️ **Academic Writing Assistance** - Standard-compliant formatting

## Quick Start

### Installation

Clone this repository into your project. Claude Code will automatically recognize commands in `.agent/workflows/`.

```bash
git clone <repository-url> research-skills
```

### Commands

| Command | Purpose | Example |
|---------|---------|---------|
| `/lit-review` | Systematic literature review | `/lit-review transformer architecture 2020-2024` |
| `/paper-read` | Deep paper analysis | `/paper-read https://arxiv.org/abs/2303.08774` |
| `/find-gap` | Identify research gaps | `/find-gap LLM in education` |
| `/build-framework` | Build theoretical framework | `/build-framework technology acceptance` |
| `/academic-write` | Academic writing assistance | `/academic-write introduction AI ethics` |

## Core Workflows

### 1. Systematic Literature Review `/lit-review`

Follows PRISMA 2020 methodology:

```
Research Question Definition (PICO/PEO)
       ↓
Protocol Registration (PROSPERO/OSF)
       ↓
Multi-database Search (Semantic Scholar, arXiv, OpenAlex, Google Scholar)
       ↓
Citation Snowballing + Grey Literature
       ↓
Title/Abstract Screening → Full-text Screening
       ↓
Data Extraction + Quality Assessment (RoB, GRADE)
       ↓
Synthesis Report + PRISMA Flow Diagram
```

### 2. Deep Paper Reading `/paper-read`

Extracts from papers:
- Research Questions (RQs)
- Theoretical Framework
- Methodology (design, sample, analysis)
- Key Findings
- Contributions & Limitations
- Future Work

Output: Markdown notes + BibTeX citation

### 3. Research Gap Identification `/find-gap`

Identifies five types of research gaps:
- **Theoretical Gap** - Incomplete or conflicting frameworks
- **Methodological Gap** - Research method limitations
- **Empirical Gap** - Missing contextual evidence
- **Knowledge Gap** - Understudied topics
- **Population Gap** - Unrepresented groups

### 4. Theoretical Framework Building `/build-framework`

- Existing theory review and comparison
- Concept relationship mapping (Mermaid diagrams)
- Hypothesis/proposition derivation

### 5. Academic Writing Assistance `/academic-write`

Supports all paper sections:
- Abstract, Introduction, Literature Review
- Methodology, Discussion, Conclusion

## Evidence Quality Rating (A-E)

| Grade | Evidence Type |
|-------|--------------|
| **A** | Systematic reviews, Meta-analyses, Large RCTs |
| **B** | Cohort studies, High-IF journal papers |
| **C** | Case studies, Expert opinion, Conference papers |
| **D** | Preprints, Working papers |
| **E** | Anecdotal, Theoretical speculation |

## Project Structure

```
research-skills/
├── .agent/workflows/     # User commands
│   ├── lit-review.md
│   ├── paper-read.md
│   ├── find-gap.md
│   ├── build-framework.md
│   └── academic-write.md
├── skills/               # Reusable skill modules
│   ├── question-refiner.md
│   ├── academic-searcher.md
│   ├── paper-screener.md
│   ├── paper-extractor.md
│   ├── gap-analyzer.md
│   ├── theory-mapper.md
│   ├── citation-formatter.md
│   ├── quality-assessor.md
│   ├── metadata-enricher.md
│   ├── citation-snowballer.md
│   ├── fulltext-fetcher.md
│   ├── prisma-checker.md
│   └── reference-manager-bridge.md
├── templates/            # Output templates
│   ├── prisma-flowchart.md
│   ├── prisma-checklist.md
│   ├── protocol-template.md
│   ├── extraction-table.md
│   ├── quality-table.md
│   ├── rob2-table.md
│   ├── grade-summary-of-findings.md
│   ├── synthesis-matrix.md
│   ├── search-log.md
│   └── paper-note.md
├── RESEARCH/             # Research output directory
├── CLAUDE.md             # Claude Code quick reference
├── glossary.md           # Research terminology
├── README.md             # This file (English)
└── README_CN.md          # Chinese version
```

## Supported APIs

| Source | Purpose | Coverage |
|--------|---------|----------|
| Semantic Scholar | Primary search | 200M+ papers |
| arXiv | CS/AI/Physics preprints | Full coverage |
| OpenAlex | Bibliometrics, author data | 250M+ works |
| Crossref | Metadata verification | 140M+ DOIs |
| Unpaywall | OA full-text access | DOI-based |
| CORE | Repository content | 200M+ OA articles |
| Google Scholar | Broad coverage | Supplementary |
| PubMed | Biomedical | Domain-specific |

## Reference Manager Integration

Supports export to:
- **Zotero** - BibTeX, CSL-JSON
- **Mendeley** - BibTeX, RIS
- **EndNote** - RIS, EndNote XML

## License

MIT
