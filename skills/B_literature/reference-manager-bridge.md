---
id: reference-manager-bridge
stage: B_literature
description: "Export and import references between the research system and Zotero, Mendeley, or EndNote in BibTeX, RIS, or CSL-JSON formats."
inputs:
  - type: Bibliography
    description: "Existing bibliography file"
  - type: PaperNotes
    description: "Paper notes with metadata"
outputs:
  - type: Bibliography
    artifact: "bibliography.bib"
  - type: RISExport
    artifact: "references.ris"
  - type: CSLJSONExport
    artifact: "references.json"
constraints:
  - "Must normalize metadata across all entries"
  - "Must generate consistent tags for organization"
failure_modes:
  - "Citekey conflicts during bidirectional sync"
  - "Metadata loss during format conversion"
tools: [filesystem, metadata-registry]
tags: [literature, references, Zotero, Mendeley, EndNote, BibTeX, RIS]
domain_aware: false
---

# Reference Manager Bridge Skill

Export and import references between the research system and reference management software.

## Purpose

Enable seamless integration with popular reference managers:
- Export in multiple formats (BibTeX, RIS, CSL-JSON)
- Generate Zotero/Mendeley compatible tags
- Maintain consistent citekeys across systems
- Support bidirectional sync workflows

## Supported Formats

| Format | Extension | Best For | Notes |
|--------|-----------|----------|-------|
| BibTeX | .bib | LaTeX users, Overleaf | Native format, full support |
| RIS | .ris | EndNote, Mendeley | Widely compatible |
| CSL-JSON | .json | Zotero, citation.js | Modern, structured |
| EndNote XML | .xml | EndNote | Full metadata |

## Supported Reference Managers

| Manager | Import Format | Export Format | Notes |
|---------|---------------|---------------|-------|
| Zotero | BibTeX, RIS, CSL-JSON | BibTeX, CSL-JSON | Best with CSL-JSON |
| Mendeley | BibTeX, RIS | BibTeX | Prefers BibTeX |
| EndNote | RIS, EndNote XML | RIS | Use RIS for compatibility |
| Papers | BibTeX, RIS | BibTeX | |
| JabRef | BibTeX | BibTeX | Native BibTeX |

## Process

### Step 1: Collect References

Gather all papers from the review:

```
Source files:
- RESEARCH/[topic]/bibliography.bib (existing BibTeX)
- RESEARCH/[topic]/references.json (existing CSL-JSON, optional)
- RESEARCH/[topic]/references.ris (existing RIS, optional)
- RESEARCH/[topic]/search_results.csv (search-derived candidate metadata)
- RESEARCH/[topic]/notes/*.md (paper notes with metadata)
- RESEARCH/[topic]/extraction_table.md (extracted metadata)

`bibliography.bib` remains the canonical export target in this repo, but it does not have to be the user's primary working format.
```

### Step 2: Normalize Metadata

Ensure consistent fields across all entries:

**Required Fields:**
| Field | BibTeX | RIS | CSL-JSON |
|-------|--------|-----|----------|
| Authors | author | AU | author |
| Title | title | TI | title |
| Year | year | PY | issued.date-parts |
| DOI | doi | DO | DOI |
| Type | @article/@inproceedings | TY | type |

**Recommended Fields:**
| Field | BibTeX | RIS | CSL-JSON |
|-------|--------|-----|----------|
| Journal | journal | JO/T2 | container-title |
| Volume | volume | VL | volume |
| Issue | number | IS | issue |
| Pages | pages | SP-EP | page |
| Abstract | abstract | AB | abstract |
| Keywords | keywords | KW | keyword |
| URL | url | UR | URL |

### Step 3: Generate Tags

Create consistent tags for reference manager organization:

**Tag Schema:**
| Tag Category | Format | Example |
|--------------|--------|---------|
| Project | `project:[topic]` | project:ai-ethics |
| Status | `status:[screening|included|excluded]` | status:included |
| Quality | `quality:[A-E]` | quality:B |
| Theme | `theme:[theme-name]` | theme:privacy |
| Read Status | `read:[unread|reading|complete]` | read:complete |
| Priority | `priority:[high|medium|low]` | priority:high |

### Step 4: Export Formats

#### BibTeX Export
```bibtex
@article{smith2024machine,
  author = {Smith, John and Jones, Jane},
  title = {Machine Learning for Healthcare},
  journal = {Nature Medicine},
  year = {2024},
  volume = {30},
  number = {1},
  pages = {100--115},
  doi = {10.1038/s41591-024-00001-1},
  abstract = {This paper presents...},
  keywords = {machine learning, healthcare, AI},
  note = {project:ai-health, status:included, quality:A}
}
```

#### RIS Export
```
TY  - JOUR
AU  - Smith, John
AU  - Jones, Jane
TI  - Machine Learning for Healthcare
JO  - Nature Medicine
PY  - 2024
VL  - 30
IS  - 1
SP  - 100
EP  - 115
DO  - 10.1038/s41591-024-00001-1
AB  - This paper presents...
KW  - machine learning
KW  - healthcare
KW  - project:ai-health
KW  - status:included
ER  -
```

#### CSL-JSON Export
```json
{
  "id": "smith2024machine",
  "type": "article-journal",
  "author": [
    {"family": "Smith", "given": "John"},
    {"family": "Jones", "given": "Jane"}
  ],
  "title": "Machine Learning for Healthcare",
  "container-title": "Nature Medicine",
  "issued": {"date-parts": [[2024]]},
  "volume": "30",
  "issue": "1",
  "page": "100-115",
  "DOI": "10.1038/s41591-024-00001-1",
  "abstract": "This paper presents...",
  "keyword": "machine learning, healthcare, project:ai-health, status:included"
}
```

### Step 5: Zotero-Specific Features

**Collection Mapping:**
```
Zotero Collection Structure:
└── Research Projects
    └── [Topic Name]
        ├── Included
        ├── Excluded
        └── To Screen
```

**Zotero Tags:**
- Use `#` prefix for auto-color: `#included`, `#excluded`
- Use descriptive tags: `Theme: Privacy`, `Quality: A`

**Zotero Notes:**
- Can import paper notes as child notes
- Link to `RESEARCH/[topic]/notes/[citekey].md`

### Step 6: Mendeley-Specific Features

**Folder Mapping:**
```
Mendeley Folders:
└── [Topic Name]
    ├── Screening
    ├── Included
    └── Excluded
```

**Mendeley Tags:**
- Flat tag structure (no hierarchy)
- Use consistent prefixes: `project:`, `status:`, `quality:`

## Output Format

```markdown
# Reference Manager Export Report

## Review: [Topic]
## Date: [Date]

---

## Export Summary

| Format | File | Entries | Size |
|--------|------|---------|------|
| BibTeX | bibliography.bib | 45 | 128 KB |
| RIS | references.ris | 45 | 95 KB |
| CSL-JSON | references.json | 45 | 156 KB |

---

## Generated Files

### BibTeX
Path: `RESEARCH/[topic]/bibliography.bib`
Compatible with: Zotero, Mendeley, JabRef, Overleaf

### RIS
Path: `RESEARCH/[topic]/references.ris`
Compatible with: EndNote, Mendeley, Zotero

### CSL-JSON
Path: `RESEARCH/[topic]/references.json`
Compatible with: Zotero, citation.js

---

## Tag Summary

| Tag | Count |
|-----|-------|
| project:[topic] | 45 |
| status:included | 30 |
| status:excluded | 15 |
| quality:A | 5 |
| quality:B | 15 |
| quality:C | 10 |

---

## Import Instructions

### Zotero
1. File → Import → Select `bibliography.bib` or `references.json`
2. Choose destination collection
3. Tags will be automatically imported

### Mendeley
1. File → Import → Select `bibliography.bib`
2. Move to appropriate folder
3. Tags appear under document details

### EndNote
1. File → Import → Select `references.ris`
2. Choose import filter: "Reference Manager (RIS)"
3. Select destination group

---

*Export completed: [Date]*
*Total references: [N]*
```

## Bidirectional Sync

For ongoing projects, maintain sync:

**From Reference Manager → Research System:**
1. Export updated BibTeX from Zotero/Mendeley
2. Parse and update `bibliography.bib`
3. Sync citekeys with `notes/` files

**From Research System → Reference Manager:**
1. Generate export files
2. Import to reference manager
3. Merge duplicates (by DOI)

## Usage

This skill is called by:
- `/lit-review` Phase 8 - Bibliography generation
- `/paper-read` - Adding new papers to library
- Manual export requests

## Quality Bar

- [ ] 导入/导出后文献条数一致（无丢失）
- [ ] Citekey 在双向同步后保持稳定
- [ ] 格式转换（BibTeX ↔ RIS ↔ CSL-JSON）无信息丢失
- [ ] 附件（PDF）关联关系在同步后保持完整
- [ ] 冲突条目已标记并手动解决

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Citekey 重写 | Zotero 自动生成 citekey 覆盖 | 使用 Better BibTeX 插件锁定 |
| 编码丢失 | 特殊字符在 RIS 中丢失 | 优先使用 BibTeX 或 CSL-JSON |
| 附件断链 | PDF 路径在同步后失效 | 使用相对路径或 linked file 模式 |
| 条目格式分类错误 | Article 被识别为 InProceedings | 手动检查 entry type 映射 |
| 无增量同步 | 全量覆盖导致手动编辑丢失 | 使用 merge 模式而非 overwrite |

## When to Use

- 需要将研究系统的 bibliography 导入 Zotero/Mendeley/EndNote 时
- 需要从外部工具导入文献到研究系统时
- 团队协作需要共享统一的文献库时
- 切换写作工具（LaTeX ↔ Word）需要转换引文格式时
