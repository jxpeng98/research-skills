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

## 0. 先选 `partial` 还是 `full`

现在推荐的首装路径是一键 bootstrap，不需要你先手动装 Python。

| Profile | 适用场景 | 结果 |
|---|---|---|
| `partial` | 你想先只安装全局 skills | 资产可用，但 orchestrator 还没准备好 |
| `full` | 你想直接拿到可运行的运行时、shell CLI 和 doctor 预检 | bootstrap 会复用现有 `python3 >= 3.12`，或自动安装 `mise` + `python@3.12` |

如果你不传 `--profile`，bootstrap 会先解释两种模式，再提示你选择。

## 1. 用一键 bootstrap 安装

Linux / macOS：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --project-dir "$PWD" --target all
```

Windows PowerShell：

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.ps1 -OutFile .\bootstrap_research_skill.ps1
powershell -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -ProjectDir "$PWD" -Target all
```

如果你想跳过交互：

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --profile partial --project-dir "$PWD" --target all
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --profile full --project-dir "$PWD" --target all
```

```powershell
# Windows PowerShell
powershell -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -Profile partial -ProjectDir "$PWD" -Target all
powershell -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -Profile full -ProjectDir "$PWD" -Target all
```

## 2. 先选入口

你通常只需要在下面三种入口里选一种：

| 入口 | 适用场景 | 命令 / 位置 |
|---|---|---|
| Slash 命令 | 你想直接用 `/paper`、`/lit-review` 等命令 | `rsk upgrade` 后自动通过 symlink 注册到 `~/.claude/commands/` 和 `~/.gemini/workflows/` |
| 安装 / 升级 CLI | 你想安装或刷新全局 skill，并在需要时显式初始化项目资产 | `research-skills` / `rsk` / `rsw` |
| Orchestrator CLI | 你想显式按 Task ID 执行与校验 | `python3 -m bridges.orchestrator ...` |

## 3. 先做环境检查

如果你的机器有 Python，建议先运行：

```bash
python3 -m bridges.orchestrator doctor --cwd .
python3 scripts/validate_research_standard.py --strict
```

说明：
- `doctor` 侧重运行时环境、CLI、API key、MCP wiring
- validator 侧重仓库内部 contract / schema 一致性

如果你想手动准备 Python，也可以直接执行：

```bash
mise install python@3.12
mise use -g python@3.12
python3 --version
```

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
