# 📚 外部 MCP 集成：Zotero (本地文献库)

`research-skills` 生态系统内置了一个语义搜索引擎 provider，它调用公开的 Semantic Scholar API。这非常适合用来发现全球范围内的学术论文。

但是，为了进行极其严谨的系统集成综述，你可能希望 AI 仅查询你**仔细筛选过的本地 Zotero 文献库**。通过挂载外部的 Zotero MCP，AI 会将其证据提取范围完全限制在你收集并批注过的论文内。

## 工作原理 (降级链路)

默认情况下，当 Claude 或 Codex 运行到 `scholarly-search`（文献查询）阶段时，它会将请求路由至内置的 `MCPConnector`。

1. **检查环境变量**：它首先去寻找 `RESEARCH_MCP_SCHOLARLY_SEARCH_CMD`。
2. **退回原生实现**：如果未设置，它将执行系统内置的 Python 脚本 (`scripts/mcp_scholarly_search.py`)。

通过设置这个环境变量，你就能**覆盖**默认的 API 行为，将这个能力直接映射到你的本地实例。

## 第一步：安装一个 Zotero MCP 服务器

你需要一个 MCP 服务器来充当 Model Context Protocol 与本地 Zotero 数据库之间的桥梁。社区里有几个可选项。

*(以使用典型的社区版 Zotero MCP 为例)*:
```bash
# 确保已安装 Node.js
npm install -g @zcaceres/zotero-mcp-server
```

*注意：请根据具体 Zotero MCP 的官方文档提供所需的 Zotero API Key 和 User ID。*

## 第二步：配置你的环境

在项目根目录创建一个 `.env` 文件（或将其直接添加到你的全局 shell 环境变量中，比如 `~/.zshrc`）。

```env
# 将 scholarly-search 能力指向你的本地 Zotero MCP Node 脚本
RESEARCH_MCP_SCHOLARLY_SEARCH_CMD="npx -y @zcaceres/zotero-mcp-server"
```

*这里的命令必须与你在终端里手动拉起该 MCP 服务时输入的命令完全一致。我们的编排器 (Orchestrator) 会拉起这个命令作为子进程，将 JSON 数据传入它的 stdin，并从 stdout 读出 JSON 响应。*

## 第三步：验证连接是否成功

利用 `doctor` 命令来验证编排器是否成功探测到你的自定义 hook：

```bash
python3 -m bridges.orchestrator doctor --cwd .
```

在 **Check Details** 输出中，寻找 `scholarly-search` 这一项。它目前应该显示你自定义的 `npx` 命令，而不是默认的 Python 实现路径。

## 切回公网搜索

如果你后来又想“撒大网”并通过默认的 Semantic Scholar 集成检索全网，只需撤销这个环境变量即可：

```bash
unset RESEARCH_MCP_SCHOLARLY_SEARCH_CMD
```
