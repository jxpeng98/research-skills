---
id: model-collaborator
stage: Z_cross_cutting
description: "Coordinate independent multi-agent execution, disagreement tracking, and synthesis across literature, writing, review, and code tasks."
inputs:
  - type: TaskPacket
    description: "Task specification for multi-agent research execution and cross-review"
outputs:
  - type: CollaborationTrace
    artifact: "logs/model_collab_trace.md"
constraints:
  - "Must ensure independent execution before synthesis"
  - "Must document agent disagreements explicitly"
failure_modes:
  - "Agent unavailable for scheduled collaboration"
  - "Output format incompatible across agents"
tools: [filesystem]
tags: [cross-cutting, multi-agent, collaboration, Codex, Claude, Gemini]
domain_aware: false
---

# Model Collaborator Skill

Multi-model collaboration for academic research tasks, not only code work.

## Purpose

Coordinate Codex, Claude, and Gemini for full-lifecycle research tasks:
- 双盲文献筛选与冲突仲裁 (Double-blind screening consensus)
- 多代理同行评审模拟 (Multi-agent peer review simulation)
- 质性编码一致性校验 (Qualitative coding cross-check)
- 统计、代码与复现链路复核 (Code / stats / reproducibility verification)

## When to Trigger

- 需要模拟不同专长的审稿人或合作者，对同一研究产物做独立复核
- 需要对文献筛选、数据提取、质性编码或 rebuttal 回复达成一致（majority rules / adjudication）
- 需要验证统计分析、代码实现或图表解释是否与论文主张一致
- 需要让不同模型从方法、领域、审稿人等视角提出相互独立的对抗性意见

## Process

### 1. Parallel (并行分析)

两个模型同时分析，合并高置信度结论。

**适用:** 文献筛选冲突仲裁、peer review 模拟、结果解释交叉复核、代码审查

```bash
python -m bridges.orchestrator parallel \
  --prompt "Review the discussion section for overclaiming, missing caveats, and unsupported causal language." \
  --cwd "/path/to/project"
```

### 2. Chain (链式验证)

一个模型生成，另一个验证改进。

**适用:** rebuttal 回复打磨、定性编码复核、统计解释验证、论文复现

```bash
python -m bridges.orchestrator chain \
  --prompt "Draft a response to reviewer concern #3, then verify whether the response fully addresses the methodological objection." \
  --cwd "/path/to/project" \
  --generator codex
```

### 3. Role-Based (分工协作)

按模型专长分配任务。

| 模型 | 专长 |
|-----|-----|
| Codex | 结构化执行、代码/统计验证、流程落地 |
| Claude | 长文本评审、逻辑校准、叙事修订 |
| Gemini | 对照审查、替代视角、摘要综合 |

```bash
python -m bridges.orchestrator role \
  --cwd "/path/to/project" \
  --codex-task "Check whether the reported model specification matches the analysis code and tables." \
  --gemini-task "Review the same results section for interpretation drift and missing caveats."
```

### 4. Single (单模型)

简单任务使用单一模型。

```bash
python -m bridges.orchestrator single \
  --model codex \
  --prompt "Audit whether the current discussion section overstates causal claims." \
  --cwd "/path/to/project"
```

## Output Format

标准化 JSON 输出：

```json
{
  "mode": "chain",
  "task_description": "...",
  "confidence": 0.85,
  "merged_analysis": "...",
  "recommendations": [...],
  "codex": {
    "success": true,
    "session_id": "...",
    "content": "..."
  },
  "gemini": {
    "success": true,
    "session_id": "...",
    "content": "..."
  }
}
```

## Academic Research Patterns

### Pattern A: 多代理同行评审 (Peer Review Simulation)

1. 定义不同审稿人 Persona (方法论专家、领域专家、Reviewer 2)
2. 使用 **parallel** 模式：各模型独立生成评审意见
3. 使用 **chain/merge** 模式：由主模型汇总成 `revision/peer_review_simulation.md`

### Pattern B: 双盲文献筛选 (Double-blind Screening)

1. 提供 `SearchQueryPlan` 和检索结果集
2. 使用 **parallel** 模式：Claude 和 Gemini 独立执行摘要筛选
3. 如果结果有冲突，交由 orchestrator 或第三个模型解决 (majority rules)

### Pattern C: 统计 / 代码实现与跨模型复核

1. 提取论文中的算法描述
2. 使用 **chain** 模式：Codex 生成代码，Claude/Gemini 进行统计和逻辑有效性验证

### Pattern D: 混合方法协作与编码一致性

1. 研究问题包含定性和定量分析
2. 使用 **role** 模式：
   - Claude: 主导定性主题分析 (Thematic Analysis)
   - Codex: 生成相应的定量分析脚本 (Polars/R)
   - Gemini: 综合双边结论写入 `synthesis.md`

### Pattern E: Rebuttal 与投稿前交叉复核

1. 提供 reviewer comments、response draft 和修订后的 manuscript sections
2. 使用 **parallel** 或 **role** 模式：
   - Claude: 检查回复措辞与防御性
   - Gemini: 检查是否真的逐点回应 reviewer concern
   - Codex: 检查修订后的表格、分析和 supplement 是否与回复声明一致

## Prerequisites

```bash
# Install CLIs
npm install -g @openai/codex
npm install -g @google/gemini-cli

# Set API keys
export OPENAI_API_KEY="..."
export GOOGLE_API_KEY="..."
```

## Usage

This skill is called by:
- `/paper` - For multi-pass critique on specific Task IDs
- `/lit-review` - For resolving screening conflicts
- `/proofread` - For multi-pass de-AI processes
- `/rebuttal` - For peer review simulation (`H3`)
- `/code-build` - For cross-model code review (`I8`)

## Quality Bar

- [ ] 各 agent 的独立产出已记录
- [ ] 分歧点已显式标注并解决
- [ ] Collaboration trace 包含 handoff 日志
- [ ] 最终合并结论的依据已记录
- [ ] 任务拆分粒度使每个 agent 可独立完成其部分

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Agent 间信息泄漏 | 交叉审查前看到对方输出 | 强制独立执行后再 merge |
| 任务拆分不当 | 一个 agent 负载过重 | 按 skill boundary 而非任意拆分 |
| Merge 无规则 | 不知道以谁的结论为准 | 预定义 merge 策略（majority/primary） |
| 只比较最终结果 | 忽视中间推理差异 | 记录 reasoning trace 而非只比较 output |
| 未记录 handoff | 后续无法追踪协作路径 | collaboration trace 必须包含每步 |

## When to Use

- 需要 Codex、Claude 和 Gemini 分工协作或交叉复核时
- 文献筛选、peer review、质性编码或 rebuttal 需要独立意见后再汇总时
- 需要 primary-review agent 配对验证写作、分析、统计或代码时
- `parallel` / `task-run --triad` / `team-run` 需要显式记录 disagreement 和 synthesis trace 时
