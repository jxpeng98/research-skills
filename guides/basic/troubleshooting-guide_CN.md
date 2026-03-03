# 🚨 故障排除与统一错误码指南

本文档列出了在使用 Research Skills Orchestrator CLI 时可能遇到的所有标准 `ERR-RS-*` 错误码，以及它们的根本原因和标准解决方案。

> **专业提示：** 如果遇到任何异常行为，你的第一步永远应该是运行交互式诊断工具：
> `python3 -m bridges.orchestrator doctor --cwd .`

---

## 环境与认证 (ENV)

当 Orchestrator 无法在环境中找到所需的 CLI 二进制文件（如 `claude` 或 `codex`）或相应的 API 密钥时，会发生这些错误。

### `[ERR-RS-ENV-001]` 缺少 API 密钥或无效。
- **原因**：请求了一个活跃的 AI agent，但环境中缺少其认证密钥。
- **修复**：检查你的 `.env` 文件或直接在终端中导出密钥。
  - Claude: `export ANTHROPIC_API_KEY="sk-ant-..."`
  - Codex: `export OPENAI_API_KEY="sk-proj-..."`
  - Gemini: `export GOOGLE_API_KEY="AIza..."`

### `[ERR-RS-ENV-002]` 未安装所需的 CLI 工具或未将其添加到 PATH 中。
- **原因**：尚未安装包含目标模型的 Node.js 二进制包装器。
- **修复**：全局安装底层的 CLI 工具：
  - `npm install -g @anthropic-ai/claude-code`

---

## 配置与标准 (CFG)

当 YAML 契约或 JSON 配置映射表包含无效设置，或者你请求了无法映射的任务/配置文件时，会发生这些错误。

### `[ERR-RS-CFG-001]` 指定了未知或无效的 agent profile。
- **原因**：你传递了一个在 `standards/agent-profiles.example.json` 中不存在的 `--profile`（例如 `--profile fast-writer`）。
- **修复**：检查拼写。使用 `task-run --help` 或查看 JSON 文件以确认可用的 profiles（例如 `default`, `academic-strict`, `bilingual-collaborator`）。

### `[ERR-RS-CFG-002]` 无法读取或解析标准的 YAML 契约文件。
- **原因**：`standards/` 目录缺少关键的标准文件（如 `research-workflow-contract.yaml`），或者 YAML 语法被破坏。
- **修复**：从版本控制中恢复 `.yaml` 文件。运行 `python3 scripts/validate_research_standard.py --strict` 来精确定位 YAML 语法错误。

### `[ERR-RS-CFG-003]` 请求了未知的 Task ID 或无效的阶段逻辑。
- **原因**：你尝试运行 `rs task-run --task-id X99`，但 `X99` 在平台映射表中不存在。
- **修复**：参考 `standards/research-workflow-contract.yaml` 查看所有有效的任务 ID（A1-I8）。

---

## 执行与编排 (EXE)

这些错误发生在实际生成和 agent 运行时阶段。

### `[ERR-RS-EXE-001]` 所有请求的并行 agent 均执行失败。
- **原因**：你请求了 `parallel`（并行）执行，但所有 agent 都立即崩溃或返回了安全约束违规。
- **修复**：检查 `.agent/logs` 中的 agent 日志。这通常发生由于 `<cwd>` 目录完全为空导致 agent 缺乏上下文，或者是因为各种 API 的速率限制同时耗尽。

### `[ERR-RS-EXE-002]` 子进程执行超时。
- **原因**：某个任务超过了硬编码的最大等待时间（通常为几分钟）。
- **修复**：单次 prompt 的工作范围过大。请将你的 `task.md` 拆分为更小的子任务。

---

## 模型上下文协议 (MCP)

这些错误专门与 AI 用来与你的文件系统、搜索引擎或代码库交互的工具 (MCP) 有关。

### `[ERR-RS-MCP-001]` 未配置所需的 MCP provider。
- **原因**：某项任务（如 `scholarly-search`）明确要求使用你尚未在环境中配置的 MCP 工具。
- **修复**：查看 `.env.example`。设置适当的环境变量（例如 `RESEARCH_MCP_METADATA_REGISTRY_CMD="..."`）。或者，去掉 `--mcp-strict` 参数运行。

### `[ERR-RS-MCP-002]` MCP provider 命令执行失败或返回错误。
- **原因**：你配置的 MCP 服务器崩溃、返回了非 JSON 格式的输出，或者超时。
- **修复**：如果使用 Zotero MCP 或其他社区 MCP，请确保对应的 node 服务器正在运行且你本地的 API 密钥有效。使用 `doctor` 命令独立测试该 MCP。
