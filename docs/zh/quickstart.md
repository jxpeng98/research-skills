# 快速开始

这一页是 `docs/quickstart.md` 的中文整理版，面向“先跑起来，再决定是否看维护者文档”的使用者。

如果你确实想看清 `skills/` 每一部分都包含什么内容，请配合 [Skills 指南](/zh/reference/skills) 一起使用。
如果你更关心“系统综述怎么走、qualitative paper 怎么走、methods paper 怎么走、审稿回复怎么走”，请直接看 [任务场景](/zh/guide/task-recipes)。

::: warning 完整功能依赖
如果你要使用完整功能集，请确保已经安装并配置：

- `python3`
- `codex`
- `claude`
- `gemini`
- `OPENAI_API_KEY`、`ANTHROPIC_API_KEY`、`GOOGLE_API_KEY`

如果缺少这些依赖，你仍然可以安装 workflow 资产并使用 shell `rsk check|upgrade|align`，但 `doctor`、validator、tests 与完整 orchestrator 执行链会受限。
:::

## 1. 全局一键安装

目前推荐的首装路径是一键 bootstrap。你不需要手动预装 Python，也不需要往你的各个科研文件夹里复制配置文件。

Linux / macOS：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --target all
```

Windows PowerShell：

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.ps1 -OutFile .\bootstrap_research_skill.ps1
powershell -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -Target all
```

Bootstrap 会把 `research-paper-workflow` 下载并安装到你电脑上各类 AI 客户端（Codex, Claude, Gemini）的专属全局配置目录下，并自动创建相应的 Slash Command 软链接。

## 2. 极简开局（零配置）

有了全局化命令注册，现在的开启流程完全可以做到肌肉记忆：

1. **新建一个空白文件夹：** `mkdir my-new-paper && cd my-new-paper`
2. **唤出你惯用的 AI：** 敲击 `claude` 或 `gemini`
3. **直接下发指令：** `输入 /paper` 或 `/lit-review` 等命令

模型会自动寻址并调用全局后台存放的技能体系。

## 3. 进阶调用方式

| 入口 | 适用场景 | 说明 |
|---|---|---|
| Slash 命令 | 你想直接用 `/paper`、`/lit-review` 等命令 | 基于全局软链接，开箱即可在任何目录触发 |
| Orchestrator CLI | 你想结合自己的自动化脚本，或执行环境预检 | `python3 -m bridges.orchestrator task-plan|task-run|doctor` |
| 安装 / 升级 CLI | 你想安装、刷新全局 skill 或卸载软链接 | `research-skills`、`rsk`、`rsw` |

## 4. 先确定 paper type

典型 paper type 与 pipeline 对应关系：

| paper type | pipeline | 场景 |
|---|---|---|
| `systematic-review` | `systematic-review-prisma` | PRISMA 风格系统综述 |
| `empirical` | `empirical-study` | 标准实证研究 |
| `qualitative` | `qualitative-study` | 访谈、案例、民族志或过程导向 qualitative paper |
| `empirical` | `rct-prereg` | 含预注册的 RCT |
| `theory` | `theory-paper` | 理论或概念型论文 |
| `methods` | `code-first-methods` | 代码与方法并重的 methods paper |

## 5. 先 plan 再 run

推荐先看任务的依赖和路由：

```bash
python3 -m bridges.orchestrator task-plan \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd .
```

你会看到：

- contract 产物
- 前置任务
- functional owner
- handoff 轨迹
- runtime plan（draft / review / fallback）

## 6. 再执行 canonical task

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --triad
```

常用参数：

- `--mcp-strict`
- `--skills-strict`
- `--triad`
- `--profile`
- `--draft-profile`
- `--review-profile`
- `--triad-profile`
- `--focus-output` 与 `--output-budget`：把本次运行收敛到更小的 active outputs，减少辅助文件扩散
- `--research-depth deep` 配合 `--max-rounds`：强制更窄、更有对抗性的证据扩展与修订流程

## 7. 什么时候切到维护者文档

你只是“使用系统”时，看这一页和 [入门](/zh/guide/) 就够了。

只有在下面这些场景才需要切换：

- 想理解系统分层：看 [系统架构](/zh/architecture)
- 想判断某个改动该落哪层：看 [规范约定](/zh/conventions)
- 想改具体行为：看 [扩展 Research Skills](/zh/advanced/extend-research-skills)
