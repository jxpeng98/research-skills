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
  - Gemini: `export GEMINI_API_KEY="..."`（headless 首选）
  - Gemini / Vertex AI: `export GOOGLE_GENAI_USE_VERTEXAI=true`，再配合 `GOOGLE_API_KEY` 或 `GOOGLE_APPLICATION_CREDENTIALS`，以及 `GOOGLE_CLOUD_PROJECT`、`GOOGLE_CLOUD_LOCATION`
  - Gemini / Google 登录订阅：先在桌面会话里启动 `python3 scripts/gemini_session_broker.py --host 127.0.0.1 --port 8767`，再导出 `RESEARCH_GEMINI_BROKER_URL=http://127.0.0.1:8767`
- **transport 控制**：可以设置 `RESEARCH_GEMINI_TRANSPORT=auto|broker|direct` 来全局选择 Gemini 执行路径，或在 agent profile 的 `runtime_options.gemini.transport` 里按 profile 覆盖。
- **补充说明**：在 `parallel`、`task-run`、`team-run` 这类非交互协作链路里，不要把 `direct` 路径上的“浏览器已登录”当成稳定前提。Gemini CLI 的缓存 OAuth 登录态在非交互 Python 子进程里可能不会被可靠复用；因此直连子进程模式应优先使用 `GEMINI_API_KEY` 或 Vertex 环境变量。
- **broker 说明**：当 `RESEARCH_GEMINI_TRANSPORT=auto` 时，编排器会先探测 broker，再决定是否回退到直连 Gemini CLI。当前内置 broker 默认仍走 Gemini CLI backend；如果你需要更稳定的 Google 登录常驻态，可以通过 `RESEARCH_GEMINI_BROKER_BACKEND_CMD` 挂接自定义 backend。

### `[ERR-RS-ENV-002]` 未安装所需的 CLI 工具或未将其添加到 PATH 中。
- **原因**：尚未安装包含目标模型的 Node.js 二进制包装器。
- **修复**：全局安装底层的 CLI 工具：
  - `npm install -g @anthropic-ai/claude-code`

### `[ERR-RS-ENV-003]` `curl: (60) SSL certificate problem: certificate is not yet valid`。
- **原因**：安装命令已经连到了 GitHub，但在下载脚本前 TLS 校验失败。常见原因是系统时间不对、CA 证书包过旧，或者公司代理拦截了 HTTPS。
- **修复**：
  - 先检查系统时间：`date -u` 和 `timedatectl status`。
  - 刷新 CA 证书：
    - Debian/Ubuntu：`sudo apt-get update && sudo apt-get install --reinstall ca-certificates curl`
    - RHEL/CentOS/Fedora：`sudo dnf reinstall ca-certificates curl`，然后执行 `sudo update-ca-trust`
  - 如果在公司代理后面，需要把代理的根证书导入系统信任库。
  - 重试时请确认脚本名和变量展开都写对：
    - `curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --project-dir "$PWD" --target all`
  - 除非你明确接受安全风险，否则不要用 `curl -k` 绕过证书校验。

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
- **原因**：你尝试运行 `rsk task-run --task-id X99`，但 `X99` 在平台映射表中不存在。
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
