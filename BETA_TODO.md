# Beta Release TODO（research-skills）

## 当前判断（2026-02-27）

- **Beta 就绪度：约 90%（P0 已完成）**
- **距离可发布 Beta：可发布（建议先按发布清单执行一轮）**
- 依据：P0 阻断项、关键路径测试、CI、发布说明与回滚方案已落地。

## Beta 发布门槛（Definition of Done）

- 三端编排（Codex/Claude/Gemini）在非理想环境下可稳定降级，不会卡死。
- `task-run`/`parallel` 的关键路径有自动化回归测试。
- 仓库具备 CI（至少校验、静态检查、最小 smoke test）。
- 文档覆盖“安装-配置-运行-排障-升级”的最小闭环。
- 发布版本有明确变更记录与回滚方案。

## P0（阻断项，Beta 前必须完成）

- [x] **运行超时与中断控制**（已完成：2026-02-27）
  - 为 `bridges/base_bridge.py` 增加每次调用的硬超时（例如 120s/300s 可配置）。
  - 超时后返回结构化错误，不允许 orchestrator 无限等待。
  - 支持 `timeout_seconds` 运行参数（可通过 profile 的 `runtime_options` 下发到各 agent）。
- [x] **CLI 可用性预检**（已完成：2026-02-27）
  - 增加 `python3 -m bridges.orchestrator doctor`（或独立脚本），检查：
    - `codex/claude/gemini` 是否在 PATH
    - 必需 API Key 是否存在
    - MCP 环境变量命令是否可执行
- [x] **非交互兼容策略**（已完成：2026-02-27）
  - 明确各 runtime 的“无交互参数”与失败回退策略，避免并发场景下阻塞。
  - 基础桥接层默认启用非交互环境变量（`CI=1`、`TERM=dumb`）与硬超时控制。
  - 支持在 profile `runtime_options` 中覆盖 `non_interactive`、`timeout_seconds`、`require_api_key`。
- [x] **关键路径自动化测试**（已完成：2026-02-27）
  - 至少覆盖：`parallel`、`task-run`、`profile-file` 解析、unknown profile 报错。
  - 使用 mock bridge，保证测试无需真实外部 CLI。
  - 测试文件：`tests/test_orchestrator_workflows.py`
  - 执行命令：`python3 -m unittest tests.test_orchestrator_workflows -v`
- [x] **CI 流水线**（已完成：2026-02-27）
  - 增加 GitHub Actions（或等效 CI）：`py_compile` + `validate_research_standard.py --strict` + 单元测试。
  - 工作流文件：`.github/workflows/ci.yml`

## P1（强烈建议，Beta 后期/RC 前完成）

- [ ] **Profile JSON 结构校验增强**
  - 增加专用 schema（字段类型、允许值、runtime_options 白名单）。
- [ ] **错误码与排障文档**
  - 统一错误码（如 `E_PROFILE_NOT_FOUND`、`E_RUNTIME_TIMEOUT`）并在 README 给出解决路径。
- [ ] **运行日志规范**
  - 输出标准化日志（task_id、agent、stage、fallback 路径、耗时、失败原因）。
- [ ] **最小示例工程**
  - 提供 `examples/`：一个 empirical 与一个 systematic-review 的端到端示例。

## P2（可选增强）

- [ ] 发布打包与版本策略（tag 规范、release note 模板）。
- [ ] 指标观测（成功率、平均耗时、fallback 触发率）。
- [ ] 更细粒度权限模板（按 task/stage 自动切换 profile）。

## 建议执行顺序（按风险）

1. 运行超时与中断控制  
2. CLI 预检（doctor）  
3. 关键路径自动化测试  
4. CI 流水线  
5. 文档与错误码补齐  

## 验收清单（可直接打勾）

- [x] `python3 scripts/validate_research_standard.py --strict` 通过  
- [x] 新增测试全部通过  
- [x] 并发 + 降级 + profile 覆盖 smoke test 通过  
- [x] README / README_CN / CLAUDE.md 发布说明同步  
- [x] 形成 `v0.x-beta` 发布说明与回滚说明  
