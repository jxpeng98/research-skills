from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import yaml


STAGE_ORDER = [
    "A_framing",
    "B_literature",
    "C_design",
    "D_ethics",
    "E_synthesis",
    "F_writing",
    "G_compliance",
    "J_proofread",
    "H_submission",
    "I_code",
    "K_presentation",
    "Z_cross_cutting",
]


STAGE_META_EN: dict[str, dict[str, str]] = {
    "A_framing": {
        "label": "A. Framing",
        "focus": "topic framing, questions, theory, gap, venue",
        "intent": '"What exactly is my contribution?"',
        "intro": "Use Stage A when you are still defining the research question, contribution, theory anchor, or venue positioning.",
    },
    "B_literature": {
        "label": "B. Literature",
        "focus": "search, screen, extract, cite, map",
        "intent": '"What does the literature say, and how do I build a corpus?"',
        "intro": "Use Stage B when you are building or maintaining the literature base for a topic, especially systematic or reproducible reviews.",
    },
    "C_design": {
        "label": "C. Design",
        "focus": "design, variables, robustness, datasets",
        "intent": '"How should this study be designed and operationalized?"',
        "intro": "Use Stage C when the question is already clear and the next problem is design validity, data feasibility, and operationalization.",
    },
    "D_ethics": {
        "label": "D. Ethics",
        "focus": "IRB, privacy, governance",
        "intent": '"What ethics and data-protection materials do I need?"',
        "intro": "Use Stage D when the study touches human participants, sensitive data, governance, or data-release constraints.",
    },
    "E_synthesis": {
        "label": "E. Synthesis",
        "focus": "evidence synthesis, quality, bias",
        "intent": '"How do I combine and rate evidence?"',
        "intro": "Use Stage E when the evidence base already exists and the task is to combine, rate, or stress-test that evidence.",
    },
    "F_writing": {
        "label": "F. Writing",
        "focus": "manuscript building, tables, figures, results writing",
        "intent": '"How do I turn analysis into publishable text?"',
        "intro": "Use Stage F when the main question is turning evidence and analysis into sections, tables, figures, and readable claims.",
    },
    "G_compliance": {
        "label": "G. Compliance",
        "focus": "reporting checklists, tone, PRISMA",
        "intent": '"Is this compliant and submission-ready?"',
        "intro": "Use Stage G when the paper exists and now needs formal checklist coverage, tone cleanup, or reporting verification.",
    },
    "J_proofread": {
        "label": "J. Proofread",
        "focus": "AI detection, humanization, similarity, final polish",
        "intent": '"How do I de-AI and finalize the manuscript?"',
        "intro": "Use Stage J when the draft is substantively complete and needs AI-fingerprint review, human-voice rewriting, similarity screening, or final proofreading before submission.",
    },
    "H_submission": {
        "label": "H. Submission",
        "focus": "submission package, rebuttal, review simulation",
        "intent": '"How do I package, defend, and stress-test the paper?"',
        "intro": "Use Stage H when the manuscript is near submission or already under review.",
    },
    "I_code": {
        "label": "I. Code",
        "focus": "academic code, stats, reproducibility",
        "intent": '"How do I implement and verify research code?"',
        "intro": "Use Stage I for academic code, data workflows, statistical execution, and reproducibility. This lane is stricter than general engineering prompts.",
        "extra": "The core strict sequence is:\n\n1. `code-specification`\n2. `code-planning`\n3. `code-execution`\n4. `code-review`\n5. `reproducibility-auditor`\n\nThat sequence is what `code-build --focus full` is designed to reinforce.",
    },
    "K_presentation": {
        "label": "K. Presentation",
        "focus": "academic talks, slide planning, Slidev, Beamer",
        "intent": '"How do I turn the paper into a defensible talk?"',
        "intro": "Use Stage K when the paper already exists and the next task is to turn it into a talk, seminar deck, or conference presentation.",
    },
    "Z_cross_cutting": {
        "label": "Z. Cross-Cutting",
        "focus": "metadata, model collaboration, self-critique",
        "intent": '"How do I improve quality across stages?"',
        "intro": "Use Stage Z when the need cuts across stages rather than belonging to one paper section.",
    },
}


STAGE_META_ZH: dict[str, dict[str, str]] = {
    "A_framing": {
        "label": "A. Framing",
        "focus": "选题、问题、理论、gap、期刊定位",
        "intent": "“我的研究问题和贡献到底是什么？”",
        "intro": "当你还在定义研究问题、理论锚点、贡献定位、目标期刊时，用 Stage A。",
    },
    "B_literature": {
        "label": "B. Literature",
        "focus": "检索、筛选、提取、引文、文献地图",
        "intent": "“文献怎么系统找、系统筛、系统整理？”",
        "intro": "当你要构建某个主题的文献基础，尤其是系统综述或可复现检索流程时，用 Stage B。",
    },
    "C_design": {
        "label": "C. Design",
        "focus": "研究设计、变量、稳健性、数据可得性",
        "intent": "“这个研究该怎么设计和 operationalize？”",
        "intro": "当问题已经较清楚，下一步变成“怎么设计研究、怎么找数据、怎么定义变量”时，用 Stage C。",
    },
    "D_ethics": {
        "label": "D. Ethics",
        "focus": "IRB、隐私、治理",
        "intent": "“伦理与数据合规材料要怎么准备？”",
        "intro": "当研究涉及 IRB、人类受试者、敏感数据或数据治理要求时，用 Stage D。",
    },
    "E_synthesis": {
        "label": "E. Synthesis",
        "focus": "证据综合、质量评估、发表偏倚",
        "intent": "“已有证据要怎么整合和评级？”",
        "intro": "当你已经有了证据材料，现在要做证据整合、质量评估或偏倚检查时，用 Stage E。",
    },
    "F_writing": {
        "label": "F. Writing",
        "focus": "结构、结果解释、表格、图、摘要",
        "intent": "“如何把分析结果写成论文？”",
        "intro": "当你的主要问题变成“怎么把分析和证据写成论文文本”时，用 Stage F。",
    },
    "G_compliance": {
        "label": "G. Compliance",
        "focus": "PRISMA、报告规范、学术语气",
        "intent": "“论文是否已经满足提交前规范？”",
        "intro": "当论文已经成形，需要做规范检查、PRISMA 核对和语气收敛时，用 Stage G。",
    },
    "J_proofread": {
        "label": "J. Proofread",
        "focus": "AI 痕迹检查、人声化改写、相似度、终稿校对",
        "intent": "“怎么在投稿前去 AI 痕迹并做终稿校对？”",
        "intro": "当稿件内容已经基本完成，下一步要做 AI 痕迹扫描、人声化改写、相似度检查和最终校对时，用 Stage J。",
    },
    "H_submission": {
        "label": "H. Submission",
        "focus": "投稿包、回复审稿、模拟评审",
        "intent": "“投稿前后怎么打包和应对审稿？”",
        "intro": "当稿件接近投稿，或者已经进入审稿往返阶段时，用 Stage H。",
    },
    "I_code": {
        "label": "I. Code",
        "focus": "学术代码、统计、可复现性",
        "intent": "“研究代码如何实现、审查、复现？”",
        "intro": "当你做的是学术代码、统计执行、数据流水线和可复现性收口时，用 Stage I。它比通用工程 prompt 更强调“低自由度、强审计”。",
        "extra": "当前严格主链是：\n\n1. `code-specification`\n2. `code-planning`\n3. `code-execution`\n4. `code-review`\n5. `reproducibility-auditor`\n\n这也是 `code-build --focus full` 想要强化的使用方式。",
    },
    "K_presentation": {
        "label": "K. Presentation",
        "focus": "学术报告、幻灯片规划、Slidev、Beamer",
        "intent": "“怎么把论文变成一个可讲、可答辩的学术报告？”",
        "intro": "当论文已经成形，下一步是把内容转成 seminar、conference talk 或 defense deck 时，用 Stage K。",
    },
    "Z_cross_cutting": {
        "label": "Z. Cross-Cutting",
        "focus": "元数据、多模型协作、自我批判",
        "intent": "“哪些能力是跨阶段通用的？”",
        "intro": "当问题并不属于某一个论文 stage，而是跨阶段通用时，用 Stage Z。",
    },
}


@dataclass
class SkillDocEntry:
    id: str
    stage: str
    file: str
    canonical: bool
    deprecated: bool
    alias_of: str
    summary: str
    display_name: str
    when_to_use: str
    summary_zh: str
    display_name_zh: str
    when_to_use_zh: str
    outputs: list[str]


def load_skill_doc_entries(root: Path) -> list[SkillDocEntry]:
    payload = yaml.safe_load((root / "skills" / "registry.yaml").read_text(encoding="utf-8")) or {}
    skills = payload.get("skills", [])
    entries: list[SkillDocEntry] = []
    for item in skills:
        if not isinstance(item, dict):
            continue
        skill_id = str(item.get("id", "")).strip()
        if not skill_id:
            continue
        entries.append(
            SkillDocEntry(
                id=skill_id,
                stage=str(item.get("stage", "")).strip(),
                file=str(item.get("file", "")).strip(),
                canonical=bool(item.get("canonical", False)),
                deprecated=bool(item.get("deprecated", False)),
                alias_of=str(item.get("alias_of", "")).strip(),
                summary=str(item.get("summary", "")).strip(),
                display_name=str(item.get("display_name", "")).strip(),
                when_to_use=str(item.get("when_to_use", "")).strip(),
                summary_zh=str(item.get("summary_zh", "")).strip(),
                display_name_zh=str(item.get("display_name_zh", "")).strip(),
                when_to_use_zh=str(item.get("when_to_use_zh", "")).strip(),
                outputs=[str(output).strip() for output in item.get("outputs", []) if str(output).strip()],
            )
        )
    return entries


def load_domain_profile_ids(root: Path) -> list[str]:
    profile_dir = root / "skills" / "domain-profiles"
    profile_ids = [
        path.stem
        for path in sorted(profile_dir.glob("*.yaml"))
        if path.stem != "custom-template"
    ]
    return profile_ids


def _format_outputs(outputs: list[str]) -> str:
    return ", ".join(f"`{item}`" for item in outputs) if outputs else "-"


def _build_stage_overview_en(entries: list[SkillDocEntry]) -> str:
    counts = {stage: 0 for stage in STAGE_ORDER}
    for entry in entries:
        counts[entry.stage] = counts.get(entry.stage, 0) + 1
    lines = [
        "## Stage Overview",
        "",
        "| Stage | Focus | Skill count | Typical user intent |",
        "|---|---|---:|---|",
    ]
    for stage in STAGE_ORDER:
        meta = STAGE_META_EN[stage]
        lines.append(
            f"| `{stage}` | {meta['focus']} | {counts.get(stage, 0)} | {meta['intent']} |"
        )
    return "\n".join(lines)


def _build_stage_overview_zh(entries: list[SkillDocEntry]) -> str:
    counts = {stage: 0 for stage in STAGE_ORDER}
    for entry in entries:
        counts[entry.stage] = counts.get(entry.stage, 0) + 1
    lines = [
        "## Stage 总览",
        "",
        "| Stage | 关注点 | Skill 数量 | 使用者最常见的问题 |",
        "|---|---|---:|---|",
    ]
    for stage in STAGE_ORDER:
        meta = STAGE_META_ZH[stage]
        lines.append(
            f"| `{stage}` | {meta['focus']} | {counts.get(stage, 0)} | {meta['intent']} |"
        )
    return "\n".join(lines)


def _build_skills_by_stage_en(entries: list[SkillDocEntry]) -> str:
    lines = ["## Canonical Skills By Stage", ""]
    for stage in STAGE_ORDER:
        meta = STAGE_META_EN[stage]
        stage_entries = [entry for entry in entries if entry.stage == stage and entry.canonical and not entry.deprecated]
        lines.extend(
            [
                f"### {meta['label']}",
                "",
                meta["intro"],
                "",
            ]
        )
        extra = meta.get("extra", "").strip()
        if extra:
            lines.extend([extra, ""])
        lines.extend(
            [
                "| Skill | Display Name | When to use | Produces |",
                "|---|---|---|---|",
            ]
        )
        for entry in stage_entries:
            lines.append(
                f"| `{entry.id}` | {entry.display_name or entry.id} | {entry.when_to_use or entry.summary} | {_format_outputs(entry.outputs)} |"
            )
        lines.append("")
    return "\n".join(lines).rstrip()


def _build_skills_by_stage_zh(entries: list[SkillDocEntry]) -> str:
    lines = ["## 按 Stage 看 Canonical Skills", ""]
    for stage in STAGE_ORDER:
        meta = STAGE_META_ZH[stage]
        stage_entries = [entry for entry in entries if entry.stage == stage and entry.canonical and not entry.deprecated]
        lines.extend(
            [
                f"### {meta['label']}",
                "",
                meta["intro"],
                "",
            ]
        )
        extra = meta.get("extra", "").strip()
        if extra:
            lines.extend([extra, ""])
        lines.extend(
            [
                "| Skill | 中文名 | 适用场景 | 产出类型 |",
                "|---|---|---|---|",
            ]
        )
        for entry in stage_entries:
            lines.append(
                f"| `{entry.id}` | {entry.display_name_zh} | {entry.when_to_use_zh} | {_format_outputs(entry.outputs)} |"
            )
        lines.append("")
    return "\n".join(lines).rstrip()


def _build_domain_profiles_section_en(profile_ids: list[str]) -> str:
    profile_lines = "\n".join(f"- `{profile_id}`" for profile_id in profile_ids)
    return f"""## Domain Profiles

The base skill system stays generic. Domain specialization is injected at runtime through `skills/domain-profiles/*.yaml`.

Current shipped profiles include:

{profile_lines}

Use domain profiles when:

- the default framing or design logic is too generic
- the code lane needs domain-specific diagnostics
- reporting or venue expectations differ by field

For example, the Stage-I code lane can load field-specific method checks through `--domain`."""


def _build_domain_profiles_section_zh(profile_ids: list[str]) -> str:
    profile_lines = "\n".join(f"- `{profile_id}`" for profile_id in profile_ids)
    return f"""## Domain Profiles

底层 skill 系统默认保持通用，学科差异通过 `skills/domain-profiles/*.yaml` 在运行时注入。

当前仓库自带的 profile 包括：

{profile_lines}

适合使用 domain profile 的情况：

- 默认 framing / design 逻辑太泛
- Stage-I 代码链需要领域专属诊断
- 不同学科的 reporting / venue 规范差异明显

例如，Stage-I 代码链可以通过 `--domain` 加载更贴近学科的方法检查规则。"""


def render_skill_reference_en(root: Path) -> str:
    entries = load_skill_doc_entries(root)
    profile_ids = load_domain_profile_ids(root)
    sections = [
        "# Skills Guide",
        "",
        "> Auto-generated from `skills/registry.yaml` by `python3 scripts/generate_skill_docs.py`. Do not edit this file by hand.",
        "",
        "This page is the user-facing map of what the `skills/` layer contains.",
        "",
        "It is meant to answer questions such as:",
        "",
        "- Which part of the system handles my current research problem?",
        "- What does each stage actually contain?",
        "- Which skills are canonical and auto-routed?",
        "- Which markdown cards are supplemental helpers or mirrors rather than primary routed skills?",
        "",
        "::: tip Canonical Source",
        "The canonical routed skill list lives in `skills/registry.yaml`. The tables below summarize that registry for human readers.",
        "User-facing surfaces may read `display_name`, `when_to_use`, `summary_zh`, `display_name_zh`, and `when_to_use_zh` directly from that registry.",
        ":::",
        "",
        "## How Users Should Read The Skills Layer",
        "",
        "- A **workflow command** such as `/paper` or `/code-build` is the entry UX.",
        "- A **Task ID** such as `B2`, `F3`, or `I6` is the contract-level unit of work.",
        "- A **skill** is the reusable execution behavior that the orchestrator injects behind the scenes through `required_skills` and `required_skill_cards`.",
        "",
        "In other words, most users should not manually choose raw markdown skill files one by one.",
        "You usually choose:",
        "",
        "1. a workflow entrypoint, or",
        "2. a Task ID via `task-plan` / `task-run`.",
        "",
        "Then the system decides which skills to load.",
        "",
        "If you need exact runtime flags, use [CLI Reference](/reference/cli).",
        "If you need to understand how agents and skills interact at runtime, use [Agent + Skill Collaboration](/advanced/agent-skill-collaboration).",
        "If you need to modify the system, use [Extend Research Skills](/advanced/extend-research-skills).",
        "If you want scenario-driven routes such as \"systematic review\", \"methods paper\", or \"rebuttal prep\", use [Task Recipes](/guide/task-recipes).",
        "",
        "## Important Boundaries",
        "",
        "- The current internal skill registry covers stages `A` through `K` except there is no routed top-level `L` or beyond; `J_proofread`, `K_presentation`, and `Z_cross_cutting` are first-class registry stages.",
        "- Some markdown files under `skills/` are **supplemental cards** or **mirror copies** for the Stage-I code lane. They are documented below, but they are not all separate routed skills.",
        "",
        _build_stage_overview_en(entries),
        "",
        _build_skills_by_stage_en(entries),
        "",
        "## Supplemental Cards And Mirror Files",
        "",
        "Not every markdown file under `skills/` is a primary routed skill.",
        "",
        "### Supplemental Manual Cards",
        "",
        "These are useful reference or helper cards, but they are not all first-class entries in the current registry:",
        "",
        "| File | Role |",
        "|---|---|",
        "| `skills/C_design/data-dictionary-builder.md` | builds a structured data dictionary |",
        "| `skills/C_design/data-management-plan.md` | writes FAIR-style data management plans |",
        "| `skills/C_design/prereg-writer.md` | drafts preregistration materials |",
        "| `skills/C_design/variable-operationalizer.md` | maps constructs to measurable variables |",
        "| `skills/H_submission/credit-taxonomy-helper.md` | prepares CRediT contribution statements |",
        "| `skills/I_code/release-packager.md` | packages code/data/environment for archival release |",
        "",
        "### Stage-I Mirror Directories",
        "",
        "These mirror the canonical Stage-I cards so prompts can stay close to the execution lane:",
        "",
        "- `skills/I_code/build/`",
        "- `skills/I_code/planning/`",
        "- `skills/I_code/run/`",
        "- `skills/I_code/qa/`",
        "",
        "Treat the canonical top-level files under `skills/I_code/` as the main reference unless you are editing the implementation details of the Stage-I lane itself.",
        "",
        "### Cross-Cutting Alias",
        "",
        "`skills/Z_cross_cutting/tone-normalizer.md` is a cross-cutting alias to the canonical compliance-oriented tone normalization behavior at `skills/G_compliance/tone-normalizer.md`.",
        "",
        _build_domain_profiles_section_en(profile_ids),
        "",
        "## Which Page Should You Use Next?",
        "",
        "- Need command syntax: [CLI Reference](/reference/cli)",
        "- Need to understand layer boundaries: [Conventions](/conventions)",
        "- Need to understand runtime cooperation: [Agent + Skill Collaboration](/advanced/agent-skill-collaboration)",
        "- Need to change or add skills: [Extend Research Skills](/advanced/extend-research-skills)",
    ]
    return "\n".join(sections).rstrip() + "\n"


def render_skill_reference_zh(root: Path) -> str:
    entries = load_skill_doc_entries(root)
    profile_ids = load_domain_profile_ids(root)
    sections = [
        "# Skills 指南",
        "",
        "> 本页由 `python3 scripts/generate_skill_docs.py` 基于 `skills/registry.yaml` 自动生成，请不要手工编辑。",
        "",
        "这一页是面向使用者的 `skills/` 全景说明。",
        "",
        "它主要回答这些问题：",
        "",
        "- 当前研究问题应该落在哪个 stage？",
        "- 每个 stage 里到底包含哪些能力？",
        "- 哪些 skill 是 canonical、会被系统自动注入？",
        "- 哪些 markdown 文件只是补充卡片或 Stage-I 镜像目录，不应该和主 skill 混在一起理解？",
        "",
        "::: tip Canonical Source",
        "系统自动路由的 canonical skill 列表以 `skills/registry.yaml` 为准；这一页是在它基础上的用户版说明。",
        "用户界面会直接读取其中的 `display_name`、`when_to_use`、`summary_zh`、`display_name_zh` 和 `when_to_use_zh`。",
        ":::",
        "",
        "## 使用者应该怎样理解 `skills/`",
        "",
        "- **workflow 命令**，例如 `/paper`、`/lit-review`、`/code-build`，是用户入口。",
        "- **Task ID**，例如 `B2`、`F3`、`I6`，是 contract 层的标准工作单元。",
        "- **skill** 是 orchestrator 在后台通过 `required_skills` 和 `required_skill_cards` 注入的可复用执行规格。",
        "",
        "所以，大多数使用者并不需要手工挑选 `skills/*.md` 去逐个执行。",
        "你通常只需要选择：",
        "",
        "1. 一个 workflow 入口，或",
        "2. 一个 Task ID（通过 `task-plan` / `task-run`）。",
        "",
        "然后系统会自动决定应该加载哪些 skill。",
        "",
        "如果你需要看精确命令参数，去 [CLI 参考](/zh/reference/cli)。",
        "如果你需要理解运行时 Agent 与 Skill 如何协同，去 [Agent + Skill 协同](/zh/advanced/agent-skill-collaboration)。",
        "如果你要修改系统本身，去 [扩展 Research Skills](/zh/advanced/extend-research-skills)。",
        "如果你更关心“系统综述 / qualitative paper / methods paper / 审稿回复”这种真实场景怎么选路径，请看 [任务场景](/zh/guide/task-recipes)。",
        "",
        "## 先记住几个边界",
        "",
        "- 当前 internal skill registry 已覆盖 `A` 到 `K` 的实际 routed stages，其中 `J_proofread`、`K_presentation` 和 `Z_cross_cutting` 都是一级 stage。",
        "- `skills/` 下面有一部分文件是**补充卡片**，还有一部分是 Stage-I 代码链路的**镜像目录**；它们都很有用，但不等于“独立的 canonical routed skill”。",
        "",
        _build_stage_overview_zh(entries),
        "",
        _build_skills_by_stage_zh(entries),
        "",
        "## 补充卡片与镜像目录",
        "",
        "`skills/` 下的每个 markdown 文件都值得参考，但它们不全都是 primary routed skills。",
        "",
        "### 补充型卡片",
        "",
        "下面这些卡片很有价值，但当前不都属于 registry 里的一级 routed skills：",
        "",
        "| 文件 | 作用 |",
        "|---|---|",
        "| `skills/C_design/data-dictionary-builder.md` | 生成结构化 data dictionary |",
        "| `skills/C_design/data-management-plan.md` | 生成 FAIR 风格数据管理计划 |",
        "| `skills/C_design/prereg-writer.md` | 生成预注册材料 |",
        "| `skills/C_design/variable-operationalizer.md` | 把抽象构念映射为可测量变量 |",
        "| `skills/H_submission/credit-taxonomy-helper.md` | 生成 CRediT 作者贡献说明 |",
        "| `skills/I_code/release-packager.md` | 为 Zenodo / GitHub / Dataverse 整理可发布复现包 |",
        "",
        "### Stage-I 镜像目录",
        "",
        "下面这些目录主要是为了让 Stage-I 代码链路的 prompt 与执行位置靠得更近：",
        "",
        "- `skills/I_code/build/`",
        "- `skills/I_code/planning/`",
        "- `skills/I_code/run/`",
        "- `skills/I_code/qa/`",
        "",
        "除非你正在修改 Stage-I 代码链路本身，否则阅读时优先以 `skills/I_code/*.md` 顶层 canonical 文件为主。",
        "",
        "### Cross-Cutting 别名",
        "",
        "`skills/Z_cross_cutting/tone-normalizer.md` 是一个跨阶段别名；真正的 canonical tone normalization 行为仍以 `skills/G_compliance/tone-normalizer.md` 为主。",
        "",
        _build_domain_profiles_section_zh(profile_ids),
        "",
        "## 接下来该看哪一页？",
        "",
        "- 想看命令和参数：去 [CLI 参考](/zh/reference/cli)",
        "- 想理解层次边界：去 [规范约定](/zh/conventions)",
        "- 想理解运行时协同：去 [Agent + Skill 协同](/zh/advanced/agent-skill-collaboration)",
        "- 想新增或改写 skill：去 [扩展 Research Skills](/zh/advanced/extend-research-skills)",
    ]
    return "\n".join(sections).rstrip() + "\n"


def generate_skill_reference_docs(root: Path) -> dict[str, str]:
    root = root.resolve()
    return {
        "docs/reference/skills.md": render_skill_reference_en(root),
        "docs/zh/reference/skills.md": render_skill_reference_zh(root),
    }
