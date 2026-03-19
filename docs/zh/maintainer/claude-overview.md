# CLAUDE 指南摘要

这一页把 `CLAUDE.md` 里的维护者要点压缩成一个更适合站内导航的版本。

## `CLAUDE.md` 在解决什么问题

`CLAUDE.md` 不只是项目介绍，它本质上是一份维护者/操作者手册，覆盖：

- 仓库快速定位
- 常用运行命令
- 架构预期
- 质量与证据语言
- `codex`、`claude`、`gemini` 之间的协作方式

## 维护者优先级

### 1. 契约真源优先

当行为发生变化时，优先按下面顺序修：

1. `standards/`
2. `roles/` 或 `skills/`
3. `templates/`
4. `pipelines/` 与 `.agent/workflows/`
5. `bridges/`
6. `research-paper-workflow/`

### 2. 把 workflows 当成入口 UX，而不是真源

斜杠命令只是易用入口。产物真相、路由真相、Task 真相仍然在 `standards/` 这一层。

### 3. 通过稳定命令驱动仓库

从 `CLAUDE.md` 抽出来的核心命令：

```bash
python3 -m bridges.orchestrator doctor --cwd .
python3 -m bridges.orchestrator task-run --task-id F3 --paper-type empirical --topic my-topic --cwd .
python3 scripts/validate_research_standard.py --strict
python3 -m unittest tests.test_orchestrator_workflows -v
```

### 4. 用协作模式思考，而不是只看单命令

维护者应当把执行模式理解成：

- 单 agent：调试、窄任务
- draft/review/fallback：标准 task 执行链
- triad：需要独立审查时
- role split / parallel fanout：任务可以拆分时

## 什么时候回看原始 `CLAUDE.md`

当你需要下面这些内容时，直接回看原始文件更合适：

- 原始命令示例
- 更长的 workflow 描述
- 完整术语块
- 多模型协作与 stage 的完整示例

多数日常维护场景下，这一页加上 [系统架构](/zh/architecture) 与 [规范约定](/zh/conventions) 就足够了。
