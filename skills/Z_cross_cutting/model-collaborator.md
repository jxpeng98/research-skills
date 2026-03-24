---
id: model-collaborator
stage: Z_cross_cutting
version: "0.2.1"
description: "Coordinate execution and cross-review across model runtimes (Codex, Claude, Gemini) for multi-agent verification."
inputs:
  - type: TaskPacket
    description: "Task specification for multi-agent execution"
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

Multi-model collaboration for academic research code tasks.

## Purpose

Coordinate Codex and Gemini for code-related research tasks:
- 论文算法实现
- 代码复现
- 数据处理管道设计（Polars/R）

## When to Trigger

- 论文包含算法伪代码需要实现
- 需要复现论文中的实验代码
- 设计数据分析管道
- 验证代码实现与论文一致性

## Collaboration Modes

### 1. Parallel (并行分析)

两个模型同时分析，合并高置信度结论。

**适用:** 代码审查、Bug 分析、安全审计

```bash
python -m bridges.orchestrator parallel \
  --prompt "分析 train.py 中的数据泄露问题" \
  --cwd "/path/to/project"
```

### 2. Chain (链式验证)

一个模型生成，另一个验证改进。

**适用:** 算法实现、代码生成、论文复现

```bash
python -m bridges.orchestrator chain \
  --prompt "实现论文中的 Transformer attention 机制" \
  --cwd "/path/to/project" \
  --generator codex
```

### 3. Role-Based (分工协作)

按模型专长分配任务。

| 模型 | 专长 |
|-----|-----|
| Codex | 代码生成、Bug 修复、算法实现 |
| Gemini | 代码解释、架构分析、文档生成 |

```bash
python -m bridges.orchestrator role \
  --cwd "/path/to/project" \
  --codex-task "实现 Algorithm 1 from the paper" \
  --gemini-task "生成详细的 API 文档"
```

### 4. Single (单模型)

简单任务使用单一模型。

```bash
python -m bridges.orchestrator single \
  --model codex \
  --prompt "修复这个 IndexError" \
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

### Pattern A: 论文算法复现

1. 提取论文中的算法描述
2. 使用 **chain** 模式：Codex 实现，Gemini 验证与论文一致性

### Pattern B: 数据管道设计 (Polars)

1. 描述研究问题和数据需求
2. 使用 **role** 模式：
   - Codex: 生成 Polars 数据处理代码
   - Gemini: 解释设计决策，生成文档

### Pattern C: R 统计分析

1. 描述统计分析需求
2. 使用 **chain** 模式生成 R 代码并验证

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
- `/paper-read` - When paper contains code
- Research code generation tasks
