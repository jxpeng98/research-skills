---
description: 准备学术报告（选择报告类型 → 内容规划 → 幻灯片设计 → 选择输出格式）
---

# Academic Presentation Workflow

// turbo-all

## Overview

从已完成的论文或研究成果出发，准备一个学术报告演示文稿。

支持三种输出格式：
- **Slidev + scholarly theme** — Markdown 驱动，Web 渲染，内置引文
- **LaTeX Beamer** — PDF 输出，数学公式支持
- **PowerPoint (PPTX)** — 可编辑格式

## Steps

### 1. Talk Planning（报告规划）

使用 `presentation-planner` skill：

1. 确认报告类型（conference talk / seminar / job talk / poster）
2. 确认时间限制和观众类型
3. 设计 story arc（三幕式 / claim-first / puzzle）
4. 创建 slide inventory（从论文各部分映射到"展示/删除/附录"）
5. 输出 slide blueprint（每页标题、内容、时间）

**输出**: `RESEARCH/[topic]/presentation/presentation_plan.md`

### 2. Slide Content Design（幻灯片内容设计）

使用 `slide-architect` skill：

1. 将每页 blueprint 转化为 assertion-evidence 格式
2. 每页确定一个 takeaway message（assertion title）
3. 选择 evidence 类型（图表 / bullet / 表格 / 引用）
4. 论文图表 → 幻灯片适配（放大字体、简化、增加标注）
5. 写 speaker notes
6. 规划 appendix slides（应对 Q&A）

**输出**: `RESEARCH/[topic]/presentation/slide_deck_spec.md`

### 3. Choose Output Format（选择输出格式）

根据场景选择：

| 场景 | 推荐格式 |
|------|---------|
| 需要 Web 访问 / 代码友好 / 现代外观 | **Slidev + scholarly** |
| 数学密集 / 期刊要求 LaTeX / LaTeX 用户 | **LaTeX Beamer** |
| 合作者需要可编辑 / 机构模板 | **PowerPoint** |

### 4a. Slidev Scholarly（如果选择 Slidev）

使用 `slidev-scholarly-builder` skill：

1. 初始化项目：`npx sch init my-talk --template academic`
2. 配置 frontmatter（theme、authors、preset）
3. 选择 visual preset（oxford-burgundy / yale-blue / ...）
4. 使用学术 layouts（cover / section / methodology / results / compare / references）
5. 使用 components（@citekey 引文 / Theorem / Block / Steps）
6. 导入 figures 到 `/public/figures/`
7. 配置 `references.bib`
8. 预览：`npx slidev` → 导出 PDF：`npx slidev export`

**输出**: `RESEARCH/[topic]/presentation/slides.md` + `references.bib`

### 4b. LaTeX Beamer（如果选择 Beamer）

使用 `beamer-builder` skill：

1. 选择 Beamer 主题（metropolis / Madrid / Berlin / ...）
2. 配置文档骨架（aspectratio、packages、metadata）
3. 构建各类 frame（content / figure / table / columns / theorem）
4. 添加 overlays / animations（\pause, \onslide, \only）
5. 配置 BibLaTeX 引文（\textcite, \parencite）
6. 编译：`latexmk -pdf slides.tex`

**输出**: `RESEARCH/[topic]/presentation/slides.tex` + `references.bib`

### 4c. PowerPoint（如果选择 PPTX）

手动或使用 `python-pptx` 构建：

1. 选择或导入机构模板
2. 按 slide_deck_spec 逐页创建
3. 导入适配后的图表
4. 添加 speaker notes
5. 检查字体、颜色一致性

**输出**: `RESEARCH/[topic]/presentation/slides.pptx`

### 5. Review and Rehearsal

1. 通读所有幻灯片，检查 assertion-evidence 一致性
2. 计时排练（目标：总时间 ≤ 分配时间的 85%）
3. 检查图表在投影仪上的可读性
4. 准备 Q&A 预案（appendix slides）
