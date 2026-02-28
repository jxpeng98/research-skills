# Agent + Skill 协同增强指南

本指南用于在 `research-skills` 中系统增强某一能力（不仅限代码），并保持跨模型一致性。

## 1) 先定目标：增强哪一类能力

先绑定到标准任务 ID（`A1`~`I3`）：

- **选题与定位**：`A1`~`A4`
- **文献与综述**：`B1`~`B5`
- **研究设计/伦理**：`C1`~`D2`
- **证据综合**：`E1`~`E5`
- **写作与投稿**：`F1`~`H4`
- **代码与复现**：`I1`~`I8` (包含 CCG 强约束代码引擎)

只要确定目标任务，就能复用统一编排链：`plan -> mcp-evidence -> primary-agent-draft -> review-agent-check -> validator-gate`。

## 2) 协同分工原则（固定）

- **Skill**：方法与产物标准（做什么、产出什么）。
- **MCP**：证据与工具层（从哪里取证、怎么落盘）。
- **Agent**：推理与写作执行层（如何完成草稿与复核）。

建议始终保留“双 agent”结构：主执行 + 独立复核。

## 3) 如何增强某个能力（标准流程）

1. 选定目标任务（例如 `E3` 或 `I2`）。
2. 在 `standards/mcp-agent-capability-map.yaml` 中更新：
   - `required_mcp`
   - `required_skills`
   - `required_skill_cards`（由 `skill_catalog` 自动解析）
   - `primary_agent/review_agent/fallback_agent`
3. 若新增 skill：
   - 新建 `skills/<skill-name>.md`
   - 加入 `skill_registry`、`skill_catalog` 与 `task_skill_mapping`
4. 若新增 agent runtime：
   - 在 `bridges/` 增加 bridge
   - 在 `bridges/orchestrator.py` 的 runtime 路由中接入
   - 参考现有实现：`bridges/claude_bridge.py`
5. 运行校验：
   - `python3 scripts/validate_research_standard.py --strict`

## 3.1) 外部 MCP 接入约定（命令模式）

对 `filesystem` 以外的 MCP，`task-run` 使用环境变量注入外部命令：

- 变量命名：`RESEARCH_MCP_<PROVIDER>_CMD`
- 例子：`RESEARCH_MCP_SCHOLARLY_SEARCH_CMD`

执行约定：

1. 编排器向命令 `stdin` 传入 JSON：
   - `provider`
   - `task_packet`
2. 外部命令从 `stdout` 返回 JSON：
   - `status`: `ok|warning|error|not_configured`
   - `summary`: 简要结果
   - `provenance`: 来源列表（可选）
   - `data`: 结构化附加信息（可选）

未配置变量时，状态为 `not_configured`；可用 `task-run --mcp-strict` 强制阻断执行。

## 3.2) Skills 注入约定（标准化 skill cards）

`task-run` 会从 `skill_catalog` 自动注入 `required_skill_cards`，每张 card 至少包含：

- `skill`：技能名
- `category`：技能类别（如 `evidence-synthesis`、`research-code`）
- `focus`：执行重点
- `file`：技能规范路径（`skills/*.md`）
- `default_outputs`：建议产物路径

可用 `task-run --skills-strict` 在技能规范文件缺失时阻断执行。

## 3.3) Profile 注入约定（人格 / 审稿风格 / 工具权限）

避免全局固定配置，使用“按运行注入”的 profile 机制：

- profile 文件：`standards/agent-profiles.example.json`
- 并发模式：
  - `parallel --profile-file ... --profile ... --summarizer-profile ...`
- 任务模式：
  - `task-run --profile-file ... --profile ...`
  - `task-run --draft-profile ... --review-profile ... --triad-profile ...`

优先级（高 -> 低）：

1. 命令行显式传参（如 `--review-profile strict-review`）
2. `task_overrides`（按 Task ID 覆盖）
3. `--profile`（本次运行默认 profile）
4. 内置 `default` profile

profile 可定义：

- `persona`
- `analysis_style` / `draft_style` / `review_style` / `summary_style` / `triad_style`
- `runtime_options`（按 agent 注入工具权限，如 Codex sandbox、Claude permission mode、Gemini sandbox）
  - 推荐设置：`non_interactive: true`、`timeout_seconds`
  - 可选严格认证：`require_api_key: true`（缺失 key 时直接快速失败，避免卡在登录流程）

## 4) 按能力类型给出推荐协同模板

### A. 代码能力（`I1`~`I8`）

- **CCG 强约束执行 (I5-I8)**：借鉴 `ccg-workflow`，将代码阶段严格拆分为约束集提取(I5)->无决策规划(I6)->主端执行(I7)->侧端验收(I8)。
- 推荐 skills：`code-specification`, `code-planning`, `code-execution`, `code-review`
- 推荐 MCP：`code-runtime`, `filesystem`
- agent 组合：主执行 `codex` (执行I7)，复核 `gemini` (验收I8)

### B. 系统综述能力（`B1`）

- 推荐 skills：`academic-searcher`, `paper-screener`, `paper-extractor`, `prisma-checker`, `evidence-synthesizer`
- 推荐 MCP：`scholarly-search`, `screening-tracker`, `extraction-store`, `fulltext-retrieval`
- agent 组合：主执行 `claude`，复核 `codex`

### C. 证据综合与 Meta（`E1/E2/E3`）

- 推荐 skills：`evidence-synthesizer`, `quality-assessor`, `code-builder`
- 推荐 MCP：`stats-engine`, `extraction-store`
- agent 组合：主执行 `codex`，复核 `claude`

### D. 写作与一致性（`F3/G3`）

- 推荐 skills：`manuscript-architect`, `citation-formatter`, `reporting-checker`, `quality-assessor`
- 推荐 MCP：`metadata-registry`, `reporting-guidelines`
- agent 组合：主执行 `claude`，复核 `codex`

### E. 投稿与返修（`H1`~`H4`）

- **多角色专家互审 (H3-H4)**：在正式投稿前，通过平行调用模拟 Methodologist、Domain Expert 等苛刻审稿人进行交叉审查（H3），并执行 Desktop-reject 致命缺陷排查（H4）。
- 推荐 skills：`submission-packager`, `rebuttal-assistant`, `peer-review-simulation`, `fatal-flaw-detector`
- 推荐 MCP：`submission-kit`, `metadata-registry`, `reporting-guidelines`
- agent 组合：主执行 `claude`，复核 `gemini/codex`

## 5) 运行入口（统一）

建议先做预检：

```bash
python -m bridges.orchestrator doctor --cwd ./project
```

使用 `task-run` 按任务执行并自动注入 `required_skills + required_skill_cards`：

```bash
python -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd ./project \
  --context "Target venue style and strict claim-evidence alignment" \
  --mcp-strict \
  --skills-strict \
  --triad
```

`--triad` 会在主执行 + 复核之后，自动调用第三个 runtime agent 做独立审查，从而在 `A`~`H` 非代码阶段也保持三端协同。

并发分析模式（不限定 Task ID）：

```bash
python -m bridges.orchestrator parallel \
  --prompt "审查当前研究方案的风险、证据缺口与改进顺序" \
  --cwd ./project \
  --summarizer claude
```

该模式默认三端并发（Codex/Claude/Gemini），并在并发后执行总结分析；若三端不可用，会自动降级为双端或单端。

## 6) 引入外部 agent 还是自建 agent？

推荐混合策略：

- **外部 agent/runtime**：负责通用能力上限（代码、推理、长文本）。
- **本地映射与约束**：负责研究场景一致性与可控性（Task ID、质量门、产物路径、技能约束）。

也就是：把“能力”交给外部，把“标准”留在本地。
