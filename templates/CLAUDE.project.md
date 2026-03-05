# Academic Deep Research Skills

This project is configured to use the **Academic Deep Research Skills** system with Claude Code.

> **Note:** The core skill pack is installed globally in your AI environment (e.g., `~/.claude/skills/research-paper-workflow/`). You do not need to install the full skill source code into this project directory.

## Quick Commands

Type the following commands in this project to trigger research workflows:

```bash
/paper [topic] [venue]                # Choose-your-path paper workflow (menu)
/lit-review [topic] [year range]      # Systematic literature review (PRISMA)
/paper-read [URL or DOI]              # Deep paper analysis
/find-gap [research area]             # Identify research gaps
/build-framework [theory/concept]     # Build theoretical framework
/academic-write [section] [topic]     # Academic writing assistance
/synthesize [topic] [outcome_id]      # Evidence synthesis / meta-analysis
/paper-write [topic] [type] [venue]   # Full manuscript drafting (outline → draft)
/study-design [topic]                 # Empirical study design (protocol + analysis plan)
/ethics-check [topic]                 # Ethics / IRB pack (human data)
/submission-prep [topic] [venue]      # Submission package (checklists + cover letter)
/rebuttal [topic]                     # Rebuttal / response to reviewers
/code-build [method] --domain ...     # Build academic research code
```

## Output Structure

All workflows will automatically organize your work under the `RESEARCH/[topic]/` directory:

```
RESEARCH/[topic]/
├── protocol.md              # Research protocol
├── search_log.md            # Reproducible search records
├── screening/               # PRISMA tracking
├── notes/                   # Individual paper notes
├── extraction_table.md      # Data extraction table
├── synthesis.md             # Final synthesis report
└── bibliography.bib         # BibTeX references

Optional (empirical / publication):
├── study_design.md
├── ethics_irb.md
├── manuscript/
└── submission/
```

## Advanced Usage

For full documentation, canonical Task IDs (`A1`–`I8`), and stage playbooks, refer to the globally installed `research-paper-workflow` skill.
