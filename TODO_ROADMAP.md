# Optimization Roadmap TODO

> 基于 2026-02-27 三模型交叉验证审查（Claude + Codex + Gemini）生成。
> 审查结论：综合评分 **8/10**，修复 P0 后可达 **9+/10**。
> 关联文件：`BETA_TODO.md`（Beta 发布门槛，P0 已全部完成）

---

## 评分快照

| 维度 | 当前 | 目标 |
|------|------|------|
| 学术覆盖完整性 | 9.5/10 | 10/10 |
| 方法论严谨性 | 9/10 | 9.5/10 |
| 跨平台可执行性 | 6.5/10 | 9/10 |
| 工程成熟度 | 7/10 | 9/10 |
| 用户体验 | 8/10 | 9/10 |

---

## 当前可用性评估（2026-02-28）

### 路径 A：Claude Code 单模型 — 立即可用

13 个 slash command 全部可执行，skill 指令精确，30+ 模板覆盖全链条。
不依赖外部模型即可独立完成学术全流程，**可投入实际学术工作**。

| 任务 | 命令 | 可用性 |
|------|------|--------|
| 定义研究问题 | `/paper` → A1 | 完全可用 |
| 系统性文献综述 | `/lit-review` | 完全可用（S2/arXiv API 已集成） |
| 深度阅读论文 | `/paper-read` | 完全可用 |
| 识别研究 Gap | `/find-gap` | 完全可用 |
| 构建理论框架 | `/build-framework` | 完全可用 |
| 研究设计 | `/study-design` | 完全可用 |
| 伦理/IRB 打包 | `/ethics-check` | 完全可用 |
| 证据综合 | `/synthesize` | 完全可用 |
| 学术写作（单段落） | `/academic-write` | 完全可用 |
| 撰写完整论文 | `/paper-write` | 完全可用 |
| 投稿打包 | `/submission-prep` | 完全可用 |
| 处理审稿意见 | `/rebuttal` | 完全可用 |
| 研究代码生成 | `/code-build` | 完全可用 |

**关键优势**：
- 每个 workflow 有明确的 `STOP & CONFIRM` 人工确认点，防止失控
- Skill 行为约束精确（用哪个 API、输出什么格式、写到哪个路径）
- 模板提供结构化骨架，Claude 只需填充内容
- 产物路径严格遵循 `RESEARCH/[topic]/` 规范

### 路径 B：跨模型编排 — 部分可用，有限制

| 模式 | 状态 | 说明 |
|------|------|------|
| `doctor` | 可用 | 预检 CLI 和环境变量 |
| `parallel` | 可用 | 三模型并行分析 + 综合（本次审查即为实例） |
| `chain` | 可用 | 生成→验证的迭代模式 |
| `role` | 可用 | 按模型特长分工 |
| `task-run` | **受限** | 能跑，但 validator-gate 未实现（P0-3）、MCP 多数为空壳（P2-1）、profile runtime_options 不生效（P0-2） |
| `code-build` | 可用 | Standard tier 正常，Advanced tier 依赖 Codex 质量 |

**阻断项**（修复后可完全可用）：
- Profile 的 sandbox/permission 参数不会实际生效 → P0-2
- `task-run --mcp-strict` 因 MCP 未配置而阻断 → P2-1
- 产物验证缺失，无法自动确认输出完整性 → P0-3

### 路径 C：Codex 可移植包 — 可用但弱

`research-paper-workflow/SKILL.md` 可在 Codex 中独立使用，但仅为路由入口。
实际执行质量取决于 Codex 自身的学术写作能力，远不如 Claude Code 路径成熟。

### 实用建议

- **立即做学术研究** → 直接用路径 A（Claude Code 单模型），workflow 设计质量已达可产出真实学术产物的水平
- **咨询性多模型任务**（代码审查、方案验证） → 用 `parallel` / `chain` 模式，已可用
- **全自动多模型学术流水线** → 等待 P0 三项修复完成后使用 `task-run`

---

## P0 — 阻断级（立即修复）

### P0-1: 对齐 `skill_catalog.default_outputs` 与 `contract.artifacts` 路径 [C1]

- **严重度**: Critical
- **来源**: Codex + Claude 共识
- **问题**: `mcp-agent-capability-map.yaml` 的 `skill_catalog.default_outputs` 使用
  `literature/search_strategy.md`、`extraction/effect_size_table.csv`、`manuscript/draft.md` 等路径，
  与 `research-workflow-contract.yaml` 的 `artifacts` 定义（`search_strategy.md`、
  `effect_size_table.md`、`manuscript/manuscript.md`）不一致。多 agent 运行时产物散落在不同位置。
- **涉及文件**:
  - `standards/mcp-agent-capability-map.yaml:64-200`（skill_catalog 段）
  - `standards/research-workflow-contract.yaml:5-31`（artifacts 段）
  - `scripts/validate_research_standard.py`（需增加路径等价性校验）
- **修复方案**:
  - [ ] 以 `research-workflow-contract.yaml` 为唯一真源，统一 `skill_catalog.default_outputs`
  - [ ] 在 `validate_research_standard.py` 中增加 `skill_catalog` 路径与 `contract.artifacts` 的交叉校验规则
  - [ ] 运行 `python3 scripts/validate_research_standard.py --strict` 确认零 warning
- **验收**: `--strict` 校验通过，`default_outputs` 路径与 `artifacts` 完全一致

---

### P0-2: 实现 Profile `runtime_options` 透传到 Bridge [C2]

- **严重度**: Critical
- **来源**: Codex
- **问题**: `agent-profiles.example.json` 定义了 `codex.sandbox`、`claude.permission_mode`、
  `gemini.sandbox` 等字段，但 bridge 实现完全忽略这些参数。Codex 固定在 read-only 模式，
  无法通过 profile 控制运行时权限。
- **涉及文件**:
  - `bridges/orchestrator.py:88`（profile 解析）
  - `bridges/orchestrator.py:1231`（runtime_options 组装）
  - `bridges/codex_bridge.py:42`（build_command）
  - `bridges/claude_bridge.py:31`（build_command）
  - `bridges/gemini_bridge.py:35`（build_command）
  - `standards/agent-profiles.example.json:10`
- **修复方案**:
  - [ ] 在 `BaseBridge.execute()` 或各子类 `build_command()` 中接收并应用 `sandbox`/`permission_mode` 参数
  - [ ] Orchestrator 将 profile 中的 runtime_options 透传至 bridge 的 `execute(**kwargs)`
  - [ ] 为 orchestrator CLI 增加 `--sandbox`/`--permission-mode` 全局覆盖参数
  - [ ] 在 `agent-profiles.example.json` 中添加字段说明注释
- **验收**: 通过不同 profile 运行 `task-run`，确认 Codex sandbox / Claude permission_mode 正确生效

---

### P0-3: 实现 `validator-gate`（产物后验） [H1]

- **严重度**: High（但属于标准完整性的阻断项）
- **来源**: Codex
- **问题**: `mcp-agent-capability-map.yaml` 定义的执行链为
  `plan → mcp-evidence → primary-agent-draft → review-agent-check → validator-gate`，
  但 `task-run` 实现仅有前四步，缺少最后一步的文件系统验证。
- **涉及文件**:
  - `standards/mcp-agent-capability-map.yaml:15-20`（fixed_execution_chain）
  - `bridges/orchestrator.py:1507-1616`（task_run 方法）
  - `bridges/mcp_connectors.py:47`（filesystem collect）
- **修复方案**:
  - [ ] 在 `task_run()` 方法末尾添加 `_validate_gate()` 步骤
  - [ ] 验证 `required_outputs` 中的文件是否已在 `RESEARCH/[topic]/` 下生成
  - [ ] 根据 `quality_gates` 列表执行对应的完整性检查（Q1-Q4）
  - [ ] 验证失败时在 `CollaborationResult` 中标记 `validation_status: "fail"` 并列出缺失项
  - [ ] 增加 `--skip-validation` CLI 参数以允许跳过（需输出警告）
- **验收**: `task-run` 运行结束后自动输出产物验证报告，缺失文件明确列出

---

## P1 — 高优（短期，1-2 周内）

### P1-1: 补全 `platform_mapping` 至 33/33 Task ID [M1]

- **严重度**: Medium
- **来源**: 三方共识
- **问题**: `research-workflow-contract.yaml:270-284` 的 `platform_mapping.claude_code` 仅覆盖
  14/33 个 Task ID。缺失: A1, A2, B3, B5, C2-C5, D2, E2-E5, F1, F4, F5, G1, G3, I2, I3。
- **涉及文件**:
  - `standards/research-workflow-contract.yaml:269-289`
  - `.agent/workflows/paper.md`（路由菜单）
- **修复方案**:
  - [ ] 为缺失的 19 个 Task ID 添加 Claude Code 命令映射
  - [ ] 评估是否需要新增 slash command（如 `/refine-question` 对应 A1）
  - [ ] 对共享 workflow 的 Task ID（如 C2-C5 共享 `/study-design`）使用参数区分
  - [ ] 同步更新 `research-paper-workflow/references/platform-routing.md`
- **验收**: `platform_mapping` 中 33 个 Task ID 均有对应命令

---

### P1-2: 实现批量任务断点恢复 [H5]

- **严重度**: High
- **来源**: Gemini + Claude 共识
- **问题**: `fulltext-fetcher` 和 `paper-screener` 在处理大规模文献（500+ 篇）时，
  API 限流或网络中断后无法恢复进度，需从头开始。
- **涉及文件**:
  - `skills/fulltext-fetcher.md`
  - `skills/paper-screener.md`
  - `skills/academic-searcher.md`
  - `.agent/workflows/lit-review.md`
- **修复方案**:
  - [ ] 定义进度持久化格式（如 `RESEARCH/[topic]/.progress/screening_checkpoint.json`）
  - [ ] 在 `paper-screener` 和 `fulltext-fetcher` 的 skill spec 中增加断点保存/恢复流程
  - [ ] 在 `lit-review.md` workflow 中增加 `--resume` 参数说明
  - [ ] 在 `skills-core.md` 中更新对应 skill 的 Process 描述
- **验收**: 中断后再次运行相同命令可跳过已完成的记录

---

### P1-3: 补全 `artifacts.optional_by_stage` 产物清单 [M2]

- **严重度**: Medium
- **来源**: Codex
- **问题**: `contract.yaml:11-31` 的 `optional_by_stage` 遗漏了多个 stage/task 定义的产物：
  - Stage G: `prisma_checklist.md` 已列在 `stages.G.outputs` 但未在 `optional_by_stage` 中
  - Stage B: `bibliography.bib` 仅在 `stages.B.outputs` 中
  - Task E4: `grade_sof.md` 仅在 `task_catalog.E4.outputs` 中
  - Task A3: `theoretical_framework.md` 仅在 `task_catalog.A3.outputs` 中
  - Task A4: `gap_analysis.md` 仅在 `task_catalog.A4.outputs` 中
  - Task C5: `preregistration.md` 仅在 `task_catalog.C5.outputs` 中
- **涉及文件**:
  - `standards/research-workflow-contract.yaml:11-31`
- **修复方案**:
  - [ ] 遍历 `stages` 和 `task_catalog` 中所有 `outputs`，确保每个文件均出现在 `required_core` 或 `optional_by_stage` 中
  - [ ] 在 `validate_research_standard.py` 中增加"产物完整性"交叉校验
- **验收**: 校验器无 warning，`optional_by_stage` 涵盖所有非 core 产物

---

### P1-4: 替换 regex YAML 解析为安全解析 [M3]

- **严重度**: Medium
- **来源**: Codex
- **问题**: `orchestrator.py:828` 附近使用 regex + 缩进假设提取 YAML 内容，
  注释、换行、内联列表等微小格式变动可能导致路由静默失败。
- **涉及文件**:
  - `bridges/orchestrator.py`（YAML 解析相关函数）
- **修复方案**:
  - [ ] 将 regex 解析替换为 `yaml.safe_load()`（Python 标准库 `pyyaml` 或内置 `tomllib` 替代）
  - [ ] 如果不想增加依赖，至少对关键路径（task_catalog, task_execution）使用 `json.loads()` 预处理
  - [ ] 增加解析失败的明确错误提示（指出具体行号和期望格式）
- **验收**: 手动修改 YAML 缩进/添加注释后，解析结果不变

---

## P2 — 中期（2-4 周内）

### P2-1: 为核心 MCP Provider 提供参考实现 [H2]

- **严重度**: High
- **来源**: Codex
- **问题**: 11 个 MCP provider 中仅 `filesystem` 有本地实现，其余全靠 env-var 桩。
  `scholarly-search` 和 `citation-graph` 是文献检索阶段的核心依赖，缺少实现使
  `task-run --mcp-strict` 在标准环境下直接失败。
- **涉及文件**:
  - `bridges/mcp_connectors.py:84-178`
  - `standards/mcp-agent-capability-map.yaml:22-33`
- **修复方案**:
  - [ ] 实现 `scholarly-search` 参考 provider（封装 Semantic Scholar + OpenAlex API）
  - [ ] 实现 `citation-graph` 参考 provider（封装 S2 `/citations` + `/references`）
  - [ ] 可选: 实现 `metadata-registry` provider（封装 Crossref API）
  - [ ] 在 `doctor` 命令中区分"有参考实现"和"需用户配置"的 provider
  - [ ] 更新 `mcp-agent-capability-map.yaml` 标注每个 provider 的实现状态
- **验收**: `doctor` 输出清晰标注各 provider 状态；`scholarly-search` 可独立运行并返回结构化结果

---

### P2-2: 更新 Bridge 适配最新 CLI 参数 [H3]

- **严重度**: High
- **来源**: Codex + Claude
- **问题**:
  - Gemini bridge 使用已废弃的 `--prompt` 参数
  - Codex bridge 的 `turn.completed` 检测依赖特定 JSON 格式，CLI 更新可能改变
  - 三个 bridge 的 session ID 提取使用硬编码字段名
- **涉及文件**:
  - `bridges/gemini_bridge.py:35, 67, 105`
  - `bridges/codex_bridge.py:42, 91`
  - `bridges/base_bridge.py:236-242`
- **修复方案**:
  - [ ] 检查 Gemini CLI 最新文档，替换废弃参数
  - [ ] 为 `is_completed()` 增加多格式兼容检测（正则回退）
  - [ ] Session ID 提取增加容错（缺失时返回 None 而非报错）
  - [ ] 在 `doctor` 命令中增加 CLI 版本检测和兼容性警告
- **验收**: 使用最新版 Codex/Gemini CLI 运行 `parallel` 模式无报错

---

### P2-3: 统一 Skills 工具名与 MCP Provider 映射 [H4]

- **严重度**: High
- **来源**: Codex
- **问题**: `skills/academic-searcher.md` 等 skill 文件引用 `search_web`、`read_url_content`
  等非标准工具名，与 `mcp-agent-capability-map.yaml` 中定义的 provider 名称
  （`scholarly-search`、`fulltext-retrieval`）无映射关系。跨平台执行时无法自动解析。
- **涉及文件**:
  - `skills/academic-searcher.md:68`
  - `skills/fulltext-fetcher.md`
  - `skills/citation-snowballer.md`
  - `standards/mcp-agent-capability-map.yaml:22-33`
- **修复方案**:
  - [ ] 在 `mcp-agent-capability-map.yaml` 中增加 `tool_aliases` 段，映射 skill 内工具名到 MCP provider
  - [ ] 或直接在 skill 文件中将工具调用名统一为 MCP provider 名称
  - [ ] 在 `skills-core.md` 中标注各 skill 依赖的 MCP provider
- **验收**: 每个 skill 引用的工具名均可在 `mcp_registry` 或 `tool_aliases` 中找到

---

### P2-4: 添加方法论选择说明（教育性提示） [M6]

- **严重度**: Medium
- **来源**: Gemini + Claude
- **问题**: 系统假设用户已了解 AMSTAR2、ROBINS-I、QUADAS-2 等工具，
  初级研究生在被提示使用特定工具时不知其含义和选择理由。
- **涉及文件**:
  - `skills/quality-assessor.md`
  - `skills/reporting-checker.md`
  - `skills-core.md:119-139`
- **修复方案**:
  - [ ] 在 `quality-assessor` 输出中增加"选择理由"字段（如"选择 RoB 2 因为纳入研究为 RCT"）
  - [ ] 在 `reporting-checker` 输出中增加 guideline 简介（如"STROBE: 适用于观察性研究的报告规范"）
  - [ ] 在 `skills-core.md` 的 RoB Tool Selection 表中增加"适用场景"列
- **验收**: 用户首次看到 RoB 工具名时能从输出中理解其含义

---

### P2-5: 补充 CRediT 作者贡献声明模板 [M5]

- **严重度**: Medium
- **来源**: Gemini
- **问题**: 多数顶刊（Nature, Science, PNAS, Lancet 等）要求基于 CRediT 14 项角色的
  作者贡献声明，当前 `submission-packager` 流程中缺少此模板。
- **涉及文件**:
  - `templates/`（新建 `credit-author-statement.md`）
  - `skills/submission-packager.md`
  - `skills-core.md:302-313`
- **修复方案**:
  - [ ] 创建 `templates/credit-author-statement.md`，覆盖 14 项 CRediT 角色
  - [ ] 在 `submission-packager` 流程中增加 CRediT 声明生成步骤
  - [ ] 在 `skills-core.md` 的 `submission-packager` 段落中引用新模板
- **验收**: `/submission-prep` 执行后输出包含 CRediT 声明

---

### P2-6: 合并 Markdown/YAML 双合同为单一数据源 [M4]

- **严重度**: Medium
- **来源**: Codex
- **问题**: Codex 可移植包 `research-paper-workflow/references/workflow-contract.md` 是
  `research-workflow-contract.yaml` 的 markdown 副本，但 `validate_research_standard.py`
  仅校验 Task ID 表完整性，不校验路径/值等价性。随时间推移两份文件会漂移。
- **涉及文件**:
  - `research-paper-workflow/references/workflow-contract.md`
  - `standards/research-workflow-contract.yaml`
  - `scripts/validate_research_standard.py:693`
- **修复方案**:
  - [ ] 方案 A: 编写脚本从 YAML 自动生成 markdown（单向同步）
  - [ ] 方案 B: 在 `validate_research_standard.py` 中增加 YAML↔MD 等价性校验
  - [ ] 在 CI 中加入此校验步骤
- **验收**: 修改 YAML 后如果未同步 MD，CI 报错

---

### P2-7: `task-run` 增加 `--domain` 并注入学科 profile（Econ/Finance 专精基础设施） [D1]

- **严重度**: High
- **来源**: 用户需求（Economics/Finance empirical 专精）+ Codex
- **问题**:
  - 已存在 `skills/domain-profiles/*.yaml`（含 economics/finance）与 domain-aware 的 skill 规范（如 `code-review` / `stats-engine`），但目前仅 `code-build` 暴露 `--domain`；
  - `task-run` 无法携带 domain，导致 C/G/I 阶段无法稳定注入学科 checklist/诊断/常见坑，专精只能停留在“提示词层面”。
- **涉及文件**:
  - `bridges/orchestrator.py`（`task-run` CLI 参数 + task_packet 字段 + prompt 注入）
  - `skills/domain-profiles/economics.yaml`
  - `skills/domain-profiles/finance.yaml`
  - Collaboration guide: `guides/advanced/agent-skill-collaboration.md`
  - Multi-client install guide: `guides/basic/install-multi-client.md`
  - Full reference: `README.md` / `README_CN.md`
  - `tests/test_orchestrator_workflows.py`
- **修复方案**:
  - [ ] 为 `task-run`/`task-plan` 增加 `--domain`（支持 `economics|finance|auto`，并兼容别名 `econ`）
  - [ ] 将 `domain` 写入 task packet（例如 `packet["domain"]`）并在 `draft/review/triad` prompt 中显式展示
  - [ ] 在 `_collect_skill_context()` 注入匹配的 domain profile card（libraries/method_templates/stats_diagnostics/common_pitfalls）
  - [ ] 对 C/E/I/G 相关任务（至少 `C3/C3_5/E3/E3_5/G1/G3/I1–I8`）将 domain 的诊断/坑作为“必须逐条检查”的 reviewer 条目
  - [ ] 更新文档示例：`task-run --domain economics` / `task-run --domain finance`
- **验收**:
  - `task-run --domain economics` 的输出中能看到 domain profile 的 checklist/diagnostics 被注入到 skill cards 与 review 阶段；
  - 未指定 domain 时行为与当前一致（向后兼容）；
  - 单元测试覆盖：domain 透传 + 兼容别名 + prompt 中出现 domain 字段。

---

### P2-8: 安装/升级支持“部件化（parts）控制”，避免强覆盖写入 [U1]

- **严重度**: Medium
- **来源**: 用户体验（安装/升级/part 功能控制）
- **问题**:
  - 当前安装脚本仅支持按 client target（codex/claude/gemini）划分，但无法细粒度控制“写入哪些部件”（例如只装 skills，不拷贝 `.agent/workflows`；不覆盖 `CLAUDE.md`；只生成 `.gemini/` quickstart 等）。
  - 对已有项目，用户希望“最小侵入式升级”（更可控、更可回滚）。
- **涉及文件**:
  - `scripts/install_research_skill.sh`
  - `research_skills/cli.py`（`rs upgrade` 透传 flags）
  - `guides/basic/install-multi-client.md`
  - `guides/basic/upgrade-research-skills.md`
- **修复方案**:
  - [ ] 安装脚本新增 `--parts`（逗号分隔）或 `--no-<part>`：例如 `skill,workflows,claude_md,gemini_quickstart,profiles`
  - [ ] `rs upgrade` 增加同名参数，并透传到安装脚本（保持 `--dry-run` 可预览）
  - [ ] `--overwrite` 语义细化到部件级：只覆盖被选择的 parts
  - [ ] 文档补齐：推荐的“最小升级”命令与常见组合
- **验收**:
  - 可实现“只更新三端 skills，不触碰项目内文件”的升级；
  - `--dry-run` 清晰展示将写入哪些路径；
  - 默认行为与当前一致（向后兼容）。

---

### P2-9: `rs` 增加 `doctor` 与 `init` 子命令（降低上手与排障成本） [U2]

- **严重度**: Medium
- **来源**: 用户体验（安装/升级闭环）
- **问题**:
  - `doctor` 目前需要用户手动运行 `python3 -m bridges.orchestrator doctor`；
  - `research-skills.toml` 需要手工创建，用户容易忘记上游配置与项目路径约定。
- **涉及文件**:
  - `research_skills/cli.py`
  - `guides/advanced/cli-reference.md`
  - `guides/basic/upgrade-research-skills.md`
- **修复方案**:
  - [ ] `rs doctor --project-dir <path>`：等价执行 orchestrator doctor（并输出结构化建议）
  - [ ] `rs init --repo <owner/repo> --project-dir <path>`：写入 `research-skills.toml`（若已存在则保持幂等/提示）
  - [ ] 可选：`rs init` 同时落盘 `.env.example` 或提示关键 env vars（只提示不写 secrets）
- **验收**:
  - 新用户只用 `rs init` + `rs upgrade --doctor` 就能完成闭环；
  - 排障路径统一收敛到 `rs doctor`。

---

## P3 — 长期（4+ 周）

### P3-1: 添加集成冒烟测试 [L1]

- **严重度**: Low
- **来源**: Codex + Claude
- **问题**: 当前仅有 MockOrchestrator 级别的单元测试，未验证真实 CLI 命令构建、
  输出解析、文件写入等端到端行为。
- **涉及文件**:
  - `tests/test_orchestrator_workflows.py`
  - `scripts/run_beta_smoke.sh`
- **修复方案**:
  - [ ] 增加集成测试文件 `tests/test_integration_smoke.py`
  - [ ] 至少覆盖: `doctor`（无需 API Key）、bridge `build_command()` 输出格式、`parse_output()` 对真实 CLI 输出的解析
  - [ ] 可选: 使用录制/回放模式（fixture JSON）模拟 CLI 输出
  - [ ] 在 CI 中区分 unit test（快速）和 integration test（需 credentials）
- **验收**: `python3 -m pytest tests/test_integration_smoke.py` 通过

---

### P3-2: 改进置信度评估机制 [L2]

- **严重度**: Low
- **来源**: Codex
- **问题**: `orchestrator.py:733` 的置信度计算基于 agent 数量和综合成功与否，
  未考虑 agent 间一致性、quality gate 通过情况或产物检查结果。
- **涉及文件**:
  - `bridges/orchestrator.py:733-798`
- **修复方案**:
  - [ ] 引入基于 quality gate 结果的置信度调整
  - [ ] 当多 agent 结论一致时提升置信度，分歧时降低
  - [ ] 将 `validator-gate`（P0-3）结果纳入置信度计算
- **验收**: 置信度分数能反映实际产物质量，而非仅反映执行成功率

---

### P3-3: 修复进程终止竞态 [L3]

- **严重度**: Low
- **来源**: Codex
- **问题**: `BaseBridge._run_command()` 在检测到 `turn.completed` 后立即终止进程，
  可能丢失缓冲区中尚未刷新的后续输出行。
- **涉及文件**:
  - `bridges/base_bridge.py:236-253`
- **修复方案**:
  - [ ] 检测到 `turn.completed` 后增加 drain 阶段（继续读取直到 EOF 或短超时）
  - [ ] 仅在 drain 完成后再调用 `process.terminate()`
  - [ ] 增加单元测试验证缓冲区完整性
- **验收**: 大输出场景下不再出现截断

---

### P3-4: 指标观测系统 [增强]

- **严重度**: Low
- **来源**: BETA_TODO.md P2 遗留
- **问题**: 无法量化系统运行质量（成功率、平均耗时、fallback 触发率）。
- **修复方案**:
  - [ ] 在 `CollaborationResult` 中增加 `timing` 字段（各 agent 耗时）
  - [ ] 增加 `--metrics` CLI 参数输出 JSON 格式指标摘要
  - [ ] 可选: 支持追加到本地 `metrics.jsonl` 文件用于趋势分析
- **验收**: `task-run --metrics` 输出包含各阶段耗时和 fallback 触发信息

---

### P3-5: Power Analysis 独立模板 [增强]

- **严重度**: Low
- **来源**: Gemini
- **问题**: 复杂功效分析（如 Monte Carlo 模拟用于混合效应模型）需要独立文档，
  当前仅在 `study-design.md` 中简要提及。
- **修复方案**:
  - [ ] 创建 `templates/power-analysis.md`（覆盖 analytic/simulation 两种路径）
  - [ ] 在 `study-designer` skill 中引用新模板
  - [ ] 可选: 在 `templates/code/statistics/` 中增加 R/Python 功效模拟脚本模板
- **验收**: `/study-design` 执行后可选输出独立的 power analysis 文档

---

### P3-6: Econ/Finance Empirical “方法包（Method Packs）+ 自动校验器” [D2]

- **严重度**: Medium
- **来源**: 用户需求（advanced 目标落地）
- **问题**:
  - 仅有 domain profile checklist 仍偏“规范文本”，难以稳定交付高级实证（staggered DID/RD/IV/event-study/Fama-MacBeth 等）的可复核产物；
  - 缺少“可执行合同 + 固定输出（tables/figures/logs）+ 自动诊断”会导致审稿关键点遗漏与复现不稳定。
- **涉及文件**:
  - `templates/code/economics/`（扩充：staggered DID、RD、IV、synthetic DID/SC、DoubleML）
  - `templates/code/finance/`（扩充：event study、FMB/双排序、HAC 因子回归、回测偏误检查、GARCH/DCC）
  - `skills/domain-profiles/economics.yaml` / `skills/domain-profiles/finance.yaml`
  - `skills/code-builder.md` / `skills/code-execution.md` / `skills/code-review.md`
  - `scripts/`（新增 `validators/` 或等效脚本集合）
- **修复方案**:
  - [ ] 定义 method pack 最小 schema（inputs schema、steps、required diagnostics、required outputs、known pitfalls）
  - [ ] 每个 method pack 提供至少 1 个端到端模板（Python/R/Stata 至少选其一作为主线）+ 合成数据单测
  - [ ] 新增自动校验器脚本：输出“PASS/BLOCK + 缺失项 + 修复建议”（用于 `I8` 与 `I4`）
  - [ ] 与 P2-7 联动：当 `--domain` 指定为 economics/finance 时，优先选择对应 method pack 作为代码骨架
- **验收**:
  - 对 2 个经济学方法 + 2 个金融方法实现可运行示例（含 tables/figures 输出与诊断报告）；
  - `I8` review 产物包含 method-pack checklist 的逐条结论；
  - `I4` audit 可一键复现并通过校验器。

---

### P3-7: 将 Q1–Q4 从“标签”升级为“可执行质量门”（semantic gates） [Q1]

- **严重度**: Medium
- **来源**: 用户需求（更完美、更可审计）
- **问题**: 当前 quality gates（Q1–Q4）在合同中是标准概念，但缺少可执行的自动检查，`task-run` 难以做到“完成即验收”。
- **涉及文件**:
  - `scripts/validate_project_artifacts.py`
  - `bridges/orchestrator.py`（validator-gate / 汇总呈现）
  - `standards/research-workflow-contract.yaml`（quality gate 定义）
- **修复方案**:
  - [ ] Q1：检查 `study_design.md`/`analysis_plan.md` 是否存在 “RQ/Hypothesis→data→model→outcome” 映射表（最小字段集）
  - [ ] Q2：检查 `manuscript/claims_evidence_map.md` 是否存在且无 `missing/weak` 未处理项（允许阈值配置）
  - [ ] Q3：检查 `reporting_checklist.md`/`prisma_checklist.md` 是否完成定位（至少包含“缺失项→修复任务”清单）
  - [ ] Q4：检查 `code/reproducibility_audit.md` 是否包含依赖、seed、run commands、数据来源与输出路径
- **验收**: `task-run` 结束时输出 gate 结果（PASS/WARN/BLOCK）并给出具体修复路径。

---

### P3-8: 运行时“步骤/部件”控制（task-run step toggles） [U3]

- **严重度**: Low
- **来源**: 用户体验（part 功能控制的运行时版本）
- **问题**: 用户希望在不同阶段以最小成本复跑（例如只跑 plan/evidence，不跑 draft；或跳过 triad；或只做 validator）。
- **涉及文件**:
  - `bridges/orchestrator.py`
  - `guides/advanced/cli-reference.md`
- **修复方案**:
  - [ ] 增加 `--only plan|evidence|draft|review|validate` 与 `--skip review|triad|validate`（保持默认全链）
  - [ ] 输出中明确提示“哪些步骤被跳过”以及风险提示
- **验收**: 支持快速迭代（复跑不必全链），同时保持输出结构一致与可追踪。

---

## 依赖关系

```
P0-1 ──┐
P0-2 ──┼── P1-1（mapping 需要路径一致后才有意义）
P0-3 ──┘
         ├── P2-1（MCP 参考实现依赖 validator-gate 验证）
         ├── P2-3（工具名映射依赖 MCP provider 确定后）
         └── P3-2（置信度改进依赖 validator-gate 结果）

P1-4 ──── P2-6（YAML 安全解析是双合同校验的前提）

P1-2（独立，可随时开始）
P2-4, P2-5（独立，可随时开始）
P3-1, P3-3, P3-4, P3-5（独立，可随时开始）

P2-7 ──── P3-6（方法包与自动校验器依赖 domain 注入）
P2-8（独立，但建议优先于大规模升级推广）
P2-9（独立，提升新用户转化与排障效率）
P3-7（建议在 P0-3 validator-gate 基础上推进）
P3-8（独立，偏体验增强）
```

---

## 验收总检查清单

- [ ] `python3 scripts/validate_research_standard.py --strict` 零 warning
- [ ] `python3 -m unittest tests.test_orchestrator_workflows -v` 全部通过
- [ ] `skill_catalog.default_outputs` 路径与 `contract.artifacts` 100% 对齐
- [ ] `platform_mapping` 覆盖 33/33 Task ID
- [ ] `validator-gate` 在 `task-run` 中自动执行
- [ ] Profile `runtime_options` 可透传至 bridge 并生效
- [ ] `doctor` 命令输出各 MCP provider 实现状态
- [ ] CI 流水线包含路径一致性校验 + 双合同等价性校验
- [ ] `task-run` 支持 `--domain` 且能注入 economics/finance profile
- [ ] 安装/升级支持 `--parts` 部件化控制（可做到“只更新 skills 不写项目文件”）
- [ ] `rs doctor` / `rs init` 可用，并在文档中给出推荐闭环
