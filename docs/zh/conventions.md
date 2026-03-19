# 规范约定

这一页是对 `docs/conventions.md` 的中文整理版，目标是把维护者最常用的判断规则放到一页里。

## 术语

### Portable Skill 与 Internal Skill Spec

- `research-paper-workflow/` 是面向 Codex / Claude / Gemini 分发的 portable skill package
- `skills/` 是仓库内部的执行规格，不等于客户端安装包
- 如果改动影响客户端安装体验，先看 `research-paper-workflow/`
- 如果改动影响仓库内部执行行为，先看 `skills/`

### Functional Agent 与 Runtime Agent

- Functional agent 表示研究责任层，例如 literature、methods、writing、compliance
- Runtime agent 表示实际模型执行者，例如 `codex`、`claude`、`gemini`
- 这两层不要混写成一回事

## 系统层次

| 层 | 位置 | 职责 |
|---|---|---|
| Contract | `standards/research-workflow-contract.yaml` | Task ID、产物路径、质量门 |
| Capability Map | `standards/mcp-agent-capability-map.yaml` | MCP / skill / runtime 路由 |
| Functional Agents | `roles/` | 责任与质量 ownership |
| Internal Skill Specs | `skills/` | 可复用执行规格 |
| Pipelines | `pipelines/` | DAG 编排与依赖 |
| Workflows | `.agent/workflows/` | 客户端入口命令 |
| Bridges | `bridges/` | 运行时适配器 |
| Portable Skill Package | `research-paper-workflow/` | 面向客户端分发 |

## 依赖方向

默认把系统看成单向依赖图：

1. `standards/research-workflow-contract.yaml`
2. `standards/mcp-agent-capability-map.yaml`
3. `roles/` 与 `skills/`
4. `pipelines/` 与 `.agent/workflows/`
5. `bridges/`
6. `research-paper-workflow/` 作为分发面

落地含义：

- 不要在 `standards/` 之外重新定义 artifact path 或 quality gate
- 不要在 `pipelines/`、`bridges/`、workflow 文件里再造第二份 routing truth
- 不要让 portable client package 变成隐藏的内部真源

## 什么时候该新增顶层 skill

只有同时满足以下条件才考虑新增 internal top-level skill：

1. 有 typed inputs / outputs
2. 拥有稳定 artifact path
3. 值得被 pipeline / task 直接依赖
4. 具备独立失败模式或质量门价值

否则优先考虑：

- 放进 MCP/provider 层
- 作为现有 skill 内的子步骤
- 作为模板、脚本或 registry 逻辑

## 什么时候应该扩展已有 skill

优先扩展现有 skill 的信号：

- artifact path 不变
- 父 skill 已经拥有 review 与 failure mode
- pipeline 不需要直接感知这个新子能力
- 变化主要在输出结构、prompt 或 checklist

## 变更时的编辑顺序

跨多层修改时，优先按这个顺序：

1. `standards/`
2. `roles/` 与 `skills/`
3. `templates/`
4. `pipelines/` 与 `.agent/workflows/`
5. `bridges/`
6. `research-paper-workflow/`

## 继续阅读

- 需要具体修改路径：看 [扩展 Research Skills](/zh/advanced/extend-research-skills)
- 需要架构总览：看 [系统架构](/zh/architecture)
- 需要 CLI 与运行入口：看 [CLI 参考](/zh/reference/cli)
