---
id: citation-formatter
stage: B_literature
version: "0.1.0"
description: "Format citations and references in APA, MLA, Chicago, IEEE, Harvard, Vancouver, or BibTeX with consistent citekey generation."
inputs:
  - type: PaperNotes
    description: "Extracted paper metadata"
outputs:
  - type: Bibliography
    artifact: "bibliography.bib"
constraints:
  - "Must generate valid BibTeX entries"
  - "Must follow target style guide precisely"
failure_modes:
  - "Incomplete metadata for proper citation"
  - "Style ambiguity for edge cases"
tools: [filesystem, metadata-registry]
tags: [literature, citations, BibTeX, APA, formatting]
domain_aware: false
---

# Citation Formatter Skill

Format citations and references according to academic style guidelines.

## Purpose

Generate properly formatted:
- In-text citations
- Reference list entries
- BibTeX entries
- Convert between citation formats

## Supported Styles

| Style | Disciplines | In-text Format |
|-------|-------------|----------------|
| APA 7th | Psychology, Social Sciences | (Author, Year) |
| MLA 9th | Humanities, Literature | (Author Page) |
| Chicago Author-Date | Sciences, Social Sciences | (Author Year) |
| Chicago Notes | History, Arts | Footnotes |
| IEEE | Engineering, CS | [1], [2], [3] |
| Harvard | Business, Sciences | (Author Year) |
| Vancouver | Medicine, Health | (1), (2), (3) |
| BibTeX | LaTeX documents | Various |

## Reference Type Templates

### Journal Article

**APA 7th:**
```
Author, A. A., & Author, B. B. (Year). Title of article. Title of Periodical, Volume(Issue), Page–Page. https://doi.org/xxxxx
```

**BibTeX:**
```bibtex
@article{citekey,
  author = {Last, First and Last, First},
  title = {Title of Article},
  journal = {Journal Name},
  year = {2024},
  volume = {10},
  number = {2},
  pages = {100--120},
  doi = {10.xxxx/xxxxx}
}
```

### Conference Paper

**APA 7th:**
```
Author, A. A. (Year). Title of paper. In E. E. Editor (Ed.), Proceedings of the Conference Name (pp. xx–xx). Publisher. https://doi.org/xxxxx
```

**BibTeX:**
```bibtex
@inproceedings{citekey,
  author = {Last, First and Last, First},
  title = {Title of Paper},
  booktitle = {Proceedings of Conference Name},
  year = {2024},
  pages = {100--110},
  publisher = {Publisher},
  doi = {10.xxxx/xxxxx}
}
```

### Book

**APA 7th:**
```
Author, A. A. (Year). Title of work: Capital letter also for subtitle. Publisher. https://doi.org/xxxxx
```

**BibTeX:**
```bibtex
@book{citekey,
  author = {Last, First},
  title = {Title of Book},
  year = {2024},
  publisher = {Publisher Name},
  address = {City},
  isbn = {xxx-x-xxx-xxxxx-x}
}
```

### Book Chapter

**APA 7th:**
```
Author, A. A. (Year). Title of chapter. In E. E. Editor (Ed.), Title of book (pp. xx–xx). Publisher. https://doi.org/xxxxx
```

**BibTeX:**
```bibtex
@incollection{citekey,
  author = {Last, First},
  title = {Title of Chapter},
  booktitle = {Title of Book},
  editor = {Editor Last, First},
  year = {2024},
  pages = {100--120},
  publisher = {Publisher}
}
```

### Preprint/arXiv

**APA 7th:**
```
Author, A. A. (Year). Title of preprint. arXiv. https://arxiv.org/abs/xxxx.xxxxx
```

**BibTeX:**
```bibtex
@misc{citekey,
  author = {Last, First and Last, First},
  title = {Title of Preprint},
  year = {2024},
  eprint = {2401.12345},
  archivePrefix = {arXiv},
  primaryClass = {cs.CL}
}
```

### Website/Online Source

**APA 7th:**
```
Author, A. A. (Year, Month Day). Title of page. Site Name. https://www.example.com/page
```

**BibTeX:**
```bibtex
@online{citekey,
  author = {Last, First},
  title = {Title of Page},
  year = {2024},
  url = {https://www.example.com},
  urldate = {2024-12-27}
}
```

## BibTeX Entry Generator

Input paper metadata and generate proper BibTeX:

```markdown
## Input

- Title: [Paper Title]
- Authors: [Author 1, Author 2, ...]
- Year: [Year]
- Journal/Conference: [Venue]
- Volume: [Vol]
- Issue: [Issue]
- Pages: [Start-End]
- DOI: [DOI]
- URL: [URL]
- Type: [article/inproceedings/book/incollection/misc]

## Output

```bibtex
@article{author2024keyword,
  author = {Author, First and Author, Second},
  title = {Paper Title},
  journal = {Journal Name},
  year = {2024},
  volume = {10},
  number = {2},
  pages = {100--120},
  doi = {10.xxxx/xxxxx}
}
```
```

## Citekey Generation Rules

Standard format: `lastname[year]keyword`

Examples:
- Single author: `smith2024machine`
- Two authors: `smithjones2024deep`
- 3+ authors: `smith2024learning` (use first author)

## In-Text Citation Formats

### APA 7th

| Situation | Format |
|-----------|--------|
| One author | (Smith, 2024) |
| Two authors | (Smith & Jones, 2024) |
| Three+ authors | (Smith et al., 2024) |
| Direct quote | (Smith, 2024, p. 15) |
| Narrative | Smith (2024) argued that... |
| Multiple sources | (Jones, 2023; Smith, 2024) |

### Narrative vs Parenthetical

**Parenthetical:**
> Research shows that AI improves learning outcomes (Smith, 2024).

**Narrative:**
> Smith (2024) found that AI improves learning outcomes.

## Bibliography Generation

Generate complete bibliography from paper notes:

```markdown
## Bibliography Generator

### Input
Provide list of citekeys or paper notes directory.

### Output Format
- [ ] APA 7th
- [ ] BibTeX
- [ ] Both

### Sorting
- [ ] Alphabetical by author
- [ ] Chronological
- [ ] Citation order
```

## Usage

This skill is called by:
- `/lit-review` - Generate bibliography
- `/paper-read` - Create BibTeX entry
- `/academic-write` - Format in-text citations
