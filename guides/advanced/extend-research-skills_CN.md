# 如何扩展 / 修改 research-skills（按阶段/任务/技能精细化迭代）

本指南回答两个问题：
1) 当前工作流整体是怎么跑的（整理流程）；2) 你想“改某一部分”时应该改哪里、怎么改、如何验证不破坏三端一致性。

> 核心原则：**标准合同先行（contract-first）**。所有三端一致性的“硬约束”都来自 `standards/`，其余内容按需细化（stage playbooks / skills / templates）。

---

## 1) 工作流总览（A–I）

### 1.1 运行时流程（Skill + MCP + Agents）

```text
User intent (自然语言)
  -> 选择 paper_type + task_id + topic
  -> 标准合同解析 (contract: outputs / gates)
  -> 能力映射解析 (capability-map: required_skills / agents / mcp)
  -> plan
  -> mcp-evidence
  -> primary-agent-draft
  -> review-agent-check
  -> validator-gate
  -> 写入 RESEARCH/[topic]/ 下的契约产物
```

单一真源（Single Source of Truth）：
- 合同（任务与输出）：`standards/research-workflow-contract.yaml`
- 编排（skills/mcp/agents 路由）：`standards/mcp-agent-capability-map.yaml`
- 阶段 DoD（更细的“怎么做才算完成”）：`research-paper-workflow/references/stage-*.md`

### 1.2 阶段地图（建议从这里开始）

| Stage | 目标 | Task IDs | 细化指南 |
|---|---|---|---|
| A | 选题定位/研究问题/贡献/理论/差距 | `A1–A5` | `research-paper-workflow/references/stage-A-framing.md` |
| B | 文献与相关工作（含系统综述流水线） | `B1–B6` | `research-paper-workflow/references/stage-B-literature.md` |
| C | 研究设计与分析计划 | `C1–C5` | `research-paper-workflow/references/stage-C-design.md` |
| D | 伦理/IRB/合规材料 | `D1–D3` | `research-paper-workflow/references/stage-D-ethics.md` |
| E | 证据综合/Meta | `E1–E5` | `research-paper-workflow/references/stage-E-synthesis.md` |
| F | 写作（大纲→段落→全文） | `F1–F6` | `research-paper-workflow/references/stage-F-writing.md` |
| G | 打磨与一致性/合规检查 | `G1–G4` | `research-paper-workflow/references/stage-G-compliance.md` |
| H | 投稿包与返修/审稿模拟 | `H1–H4` | `research-paper-workflow/references/stage-H-submission.md` |
| I | 研究代码支持（spec→plan→execute→review→audit） | `I1–I8` | `research-paper-workflow/references/stage-I-code.md` |

推荐最小覆盖（按 paper_type 的“至少做哪些任务”）见：
- `research-paper-workflow/references/coverage-matrix.md`

---

## 2) 你要改什么？先选“变更类型”

把改动分类，会直接决定你应该改哪个文件层：

1. **改“必须产出什么”**（产物路径/Task 输出集合/质量门）：改 `standards/research-workflow-contract.yaml`
2. **改“这个任务由谁来做/怎么协同”**（三端路由、MCP 依赖、required_skills）：改 `standards/mcp-agent-capability-map.yaml`
3. **改“怎么做才算完成”**（DoD、更细步骤、检查清单）：改 `research-paper-workflow/references/stage-*.md`
4. **改“具体执行规范/输出格式”**（技能说明、可复用结构）：改 `skills/*/*.md` 与/或 `templates/*`
5. **改“Claude Code 菜单/路由体验”**（命令入口/菜单项）：改 `.agent/workflows/*.md`
6. **改“编排器行为”**（task-run 注入、并发策略、外部 MCP 命令协议）：改 `bridges/*.py`

> 经验法则：**先改 contract（如果产物/路径会变化）→ 再改 capability-map（谁来做）→ 再补细 stage playbooks / skills / templates（怎么做）→ 最后修 workflows（交互层）**。

### 2.1 “我要把某个方向做专精” 时，应该从哪里改？

这里的“方向专精”默认指：
- **不改** Task ID、阶段顺序、核心产物契约
- **只增强** 某个学科/方法方向的知识、检查清单、推荐库、数据库、写作规范、投稿偏好

这种情况下，**不要一上来就改 `standards/`**。当前架构把“核心工作流”和“领域知识”拆开了，推荐默认从 `skills/domain-profiles/` 进入。

| 你的目标 | 首改文件 | 什么时候再往上层改 |
|---|---|---|
| 调整推荐库、方法模板、诊断项、常见坑、数据库、期刊规范 | `skills/domain-profiles/<domain>.yaml` | 只有新增字段时才改 `schemas/domain-profile.schema.json` |
| 新增一个全新的方向 | 复制 `skills/domain-profiles/custom-template.yaml` 为新文件 | 如需在帮助文案/示例中显式展示，再补文档或 CLI 帮助 |
| 让更多 skill 也能感知该方向 | `skills/registry.yaml` + 对应 `skills/*/*.md` | 如 task 路由或 required_skills 变化，再改 `standards/mcp-agent-capability-map.yaml` |
| 修改代码 lane 如何使用方向配置 | `skills/I_code/code-builder.md`、`skills/I_code/stats-engine.md`、`skills/I_code/code-review.md` | 只有运行时注入逻辑变化时才需要看 `bridges/*.py` |
| 改某个方向的输出路径、Task 产物、质量门 | `standards/research-workflow-contract.yaml` | 这已经不是“轻量专精”，而是标准层变更 |

推荐修改顺序：
1. 先判断你是“改已有方向”，还是“新增一个方向”。
2. 先把方向知识沉到 `skills/domain-profiles/<domain>.yaml`，优先维护这些字段：
   - `libraries`
   - `method_templates`
   - `stats_diagnostics`
   - `reporting_guidelines` / `reporting_standards`
   - `common_pitfalls`
   - `visualization_templates`
   - `default_databases`
   - `methodology_priors`
   - `venue_norms`
3. 再判断这个方向是否只影响 **代码支持 I 阶段**。
   - 如果只影响代码构建、统计诊断、代码 review，通常改 domain profile 就够了。
   - 如果还要影响选题、检索、设计、写作、投稿，继续更新对应的 `domain_aware` skill。
4. 如果你新增了 domain profile 里的字段，再同步更新 `schemas/domain-profile.schema.json`。
5. 只有在新增了 skill、改变了 task 路由、或者改变了产物契约时，才升级到 `standards/` 层。

常见会一起受方向影响的 skill：
- `skills/A_framing/venue-analyzer.md`
- `skills/B_literature/concept-extractor.md`
- `skills/C_design/study-designer.md`
- `skills/C_design/robustness-planner.md`
- `skills/C_design/dataset-finder.md`
- `skills/C_design/variable-constructor.md`
- `skills/F_writing/manuscript-architect.md`
- `skills/G_compliance/reporting-checker.md`
- `skills/H_submission/submission-packager.md`
- `skills/I_code/code-builder.md`
- `skills/I_code/stats-engine.md`
- `skills/I_code/code-review.md`

不要这样改：
- 只是想补某个学科的方法 checklist，却直接改 `standards/research-workflow-contract.yaml`
- 只是想补某个方向的注意事项，却在多个 skill 文件里重复拷贝同一份 checklist，而不沉到 `domain-profiles/`
- 新增了 domain profile 字段，但没有同步更新 schema 或相关 skill 的消费逻辑
- 还没确认是否真的需要独立 artifact，就先新增顶层 skill 文件

方向专精后的验证建议：
```bash
python3 scripts/validate_research_standard.py --strict
python3 -m unittest tests.test_orchestrator_workflows -v
```

如本次修改影响了具体方向的运行行为，建议再做一次最小手工验证，例如：
- 用目标方向跑一遍 `code-build` / `task-run`
- 检查生成内容是否真的引用了新的方法 checklist、诊断项与 reporting 规范

---

## 3) 常见修改场景（操作清单）

### 3.1 “我想让某个 Task 更细/更专用，但不改输出路径”

优先改这里（从上到下）：
1. `research-paper-workflow/references/stage-<X>-*.md`：补充 DoD、检查表、常见失败模式
2. `skills/<A-I_stage>/<skill>.md`：补充低自由度的输出结构（表格、字段、判断规则）
3. `templates/<template>.md`：把重复性结构沉到模板里（更稳定、可复用）

**不要改**：
- Task ID、stage 标签、契约输出路径（除非你准备做 3.2 的“路径变更”）

### 3.2 “我要新增/调整一个产物文件（输出路径变化）”

必须做全链一致性更新：
1. `standards/research-workflow-contract.yaml`
   - 更新对应 `task_catalog.<ID>.outputs`
   - 如属于阶段核心产物，更新 `stages.<stage>.outputs` 和 `artifacts.required_core`
2. `standards/mcp-agent-capability-map.yaml`
   - 如涉及某 skill 的默认产物，更新 `skill_catalog.<skill>.default_outputs`
3. 交互层（按需要）
   - `.agent/workflows/*.md`：确保写入/引用了新路径
   - `research-paper-workflow/references/workflow-contract.md`：更新 task 表格（保持可移植 skill 一致）
4. 产物结构（按需要）
   - `templates/`：补模板
   - `skills/`：补输出格式与“完成标准”

### 3.3 “我要新增一个 Task ID（扩展标准）”

这是“标准层”变更，建议谨慎（会影响所有平台的一致性）。

操作步骤：
1. `standards/research-workflow-contract.yaml`
   - 在 `task_catalog` 添加新 Task（stage/outputs）
2. `standards/mcp-agent-capability-map.yaml`
   - 在 `task_skill_mapping` 加 required_skills
   - 在 `task_execution` 加 required_mcp + agent 路由 + quality_gates
3. `research-paper-workflow/references/workflow-contract.md`
   - 在 task 表格中加入新 Task
4. `.agent/workflows/paper.md`
   - 菜单里加入口（用户能选到）
5. 更新本地一致性校验（否则 CI 会失败）：
   - `scripts/validate_research_standard.py`：加入新 Task ID 到期望集合
6. 可选：新增/更新 stage playbook（解释 DoD）

### 3.4 “我要新增一个 Skill（更精细模块）”

1. 新增技能规范文件：`skills/<A-I_stage>/<skill-name>.md`
2. 注册到编排表：`standards/mcp-agent-capability-map.yaml`
   - `skill_registry` 增加 `<skill-name>`
   - `skill_catalog` 增加条目（file/category/focus/default_outputs）
   - 在相关任务的 `task_skill_mapping.<task_id>.required_skills` 引入该 skill
3. 如该 skill 需要稳定结构：把结构下沉到 `templates/`，在 skill 中引用模板路径

### 3.5 “我要接入新的外部 MCP（例如 scholar/search/stats 的本地工具）”

1. 在 `standards/mcp-agent-capability-map.yaml` 的 `mcp_registry` 增加 provider 名称
2. 在相关任务的 `task_execution.<task_id>.required_mcp` 里引用它
3. 运行时通过环境变量注入命令（命令从 stdin 收 JSON，stdout 回 JSON）：
   - 变量名：`RESEARCH_MCP_<PROVIDER>_CMD`
   - 约定实现见：`bridges/mcp_connectors.py`

---

## 4) 变更后的验证（本地/CI 同一套）

在每次修改后至少跑一次：

```bash
python3 scripts/validate_research_standard.py --strict
python3 -m unittest tests.test_orchestrator_workflows -v
```

可选（本地三端 CLI 都装好、且 key 配置好时）：

```bash
./scripts/run_beta_smoke.sh
python3 -m bridges.orchestrator doctor --cwd .
```

---

## 5) 推荐的提交粒度（避免“改一处崩一片”）

- **第一步**：只改 `standards/`（contract / capability-map），跑 validator
- **第二步**：补 `references/stage-*.md`（DoD）、`skills/*/`（执行规范）、`templates/`（结构化模板）
- **第三步**：修 `.agent/workflows/`（交互层路由/菜单）
- **第四步**：跑全量校验并更新 release notes（如要发 beta）
