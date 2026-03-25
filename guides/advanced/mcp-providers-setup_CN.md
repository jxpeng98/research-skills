# 🔌 可选 MCP Provider 接入指南

运行 `rsk upgrade` 后，你可能会看到类似以下的提示信息：

```
⚠  MCP screening-tracker: RESEARCH_MCP_SCREENING_TRACKER_CMD not configured
⚠  MCP extraction-store: RESEARCH_MCP_EXTRACTION_STORE_CMD not configured
...
```

**这些⚠仅为提示，不影响框架核心功能。** 这些 MCP（模型上下文协议工具）里有一部分已经是仓库内置 reference provider，另一部分仍是可选的外部能力增强接口。本文档说明它们各是什么、从哪里获取、以及如何接入。

如果你的目标是更严格、可复现的 academic literature search，请在读完本页后继续看 [严格 Academic Literature Search](./rigorous-literature-search_CN.md)。本页讲 provider 的接线方式，后者讲的是这些检索层应该如何组合。

---

## 工作原理

每个 MCP 对应一个环境变量（`RESEARCH_MCP_<NAME>_CMD`）。  
系统在执行任务时，会通过这个命令启动一个子进程，传入 JSON 数据包，并读取 JSON 格式的响应。

**查找逻辑（优先级从高到低）：**
1. 环境变量 → 使用你指定的外部命令
2. 内置 Python 脚本（`scripts/mcp_<name>.py`）→ 自动使用内置实现
3. 均无 → 显示 ⚠ 警告，任务仍可运行（无外部工具辅助）

---

## 各 MCP Provider 说明与接入方式

### 1. `metadata-registry` — 文献元数据标准化

**作用：** 补全和标准化论文的 DOI、期刊、年份、作者信息。  
**使用场景：** B 阶段（文献处理）、C1 任务等。

这个 provider 现在已经有仓库内置的本地 reference 实现，可用于 identifier 规范化、本地记录合并和 citekey 生成。它可以直接读取 `bibliography.bib`、`references.json`、`references.ris`、`search_results.csv` 和 `notes/*.md`。如果你需要更权威的 enrichment，优先建议在 builtin provider 之上叠加 OpenAlex 这类外部实现。

当前 enrichment merge policy 不是简单的“后写覆盖前写”，而是带 source-aware 优先级：
- `OpenAlex` 优先补 title、author list、venue、publisher 和 OA link
- `Crossref` 优先补 volume / issue / pages 这类书目信息，以及 DOI landing-page 规范化
- 本地已经确定的 `doi` 和 `citekey` 默认保持 sticky，除非原值缺失

**推荐工具：**

| 工具 | 类型 | 地址 |
|------|------|------|
| 内置本地 reference provider | 仓库内置 | `scripts/mcp_metadata_registry.py` |
| OpenAlex MCP | 开源 Python | [github.com/b-vitamins/openalex-mcp](https://github.com/b-vitamins/openalex-mcp) |

```bash
# 示例：保留 builtin metadata-registry，再叠加 OpenAlex enrichment
pip install openalex-mcp
export RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD="python3 -m openalex_mcp"
```

只有当你想完全替换 builtin metadata-registry 时，才去设置 `RESEARCH_MCP_METADATA_REGISTRY_CMD`。

---

### 2. `fulltext-retrieval` — 论文全文获取

**作用：** 解析并获取论文 PDF 全文、追踪版本来源。  
**使用场景：** B1（系统综述）、B2（全文提取）任务。

这个 provider 现在也有仓库内置的 retrieval-planning stub。内置模式不会真正下载 PDF，但会根据本地文献产物草拟 `retrieval_manifest.csv` 和 `screening/full_text.md`，保留已有 manifest 行、检查本地路径是否存在，并标出需要 OA/manual follow-up 的条目。

**推荐工具：**

| 工具 | 类型 | 地址 |
|------|------|------|
| 内置 retrieval-planning stub | 仓库内置 | `scripts/mcp_fulltext_retrieval.py` |
| Zotero MCP Server | Node.js，接入本地 Zotero 库 | [github.com/zcaceres/zotero-mcp](https://github.com/zcaceres/zotero-mcp) |
| Unpaywall API wrapper | 可获取开放获取全文 | 自定义脚本接入 `api.unpaywall.org` |

```bash
# 如果你想要真正下载全文，再接入 Zotero MCP（需要 Zotero 桌面客户端运行中）
npm install -g @zcaceres/zotero-mcp-server
export RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD="npx -y @zcaceres/zotero-mcp-server"
```

只有当你想把 builtin planning stub 升级成“真实全文解析 provider”时，才需要设置这个环境变量。

> **提示：** 详细的 Zotero 接入流程见 [`mcp-zotero-integration_CN.md`](./mcp-zotero-integration_CN.md)。

---

### 3. `screening-tracker` — 文献筛选状态追踪

**作用：** 跟踪 PRISMA 流程中每篇文献的纳入/排除决策状态。  
**使用场景：** B1 系统综述任务。

**推荐工具：**

| 工具 | 类型 | 地址 |
|------|------|------|
| Rayyan MCP（实验性）| 接入 Rayyan 平台 API | 见 [rayyan.ai](https://www.rayyan.ai) 开发者文档 |
| 自定义 SQLite 追踪器 | 本地文件 | 参考下方"自建 stub"章节 |

Rayyan 是目前最主流的系统综述文献筛选平台（支持多人协作、盲筛）。如无现成 MCP，可先用轻量 stub 脚本代替（见文末）。

---

### 4. `extraction-store` — 结构化数据提取存储

**作用：** 将从论文中提取的研究特征、效应量、结局指标等存入结构化数据库。  
**使用场景：** B2/B6/E 系列（数据提取与分析）任务。

**推荐工具：**

| 工具 | 类型 | 地址 |
|------|------|------|
| Covidence MCP（规划中）| 系统综述管理平台 | [covidence.org](https://www.covidence.org) |
| 本地 CSV/SQLite stub | 轻量实现 | 参考下方"自建 stub"章节 |

目前没有成熟的开源 extraction-store MCP，建议先用 stub（见文末）让框架静默跳过该模块。

---

### 5. `stats-engine` — 统计分析引擎

**作用：** 执行元分析、混合效应模型、贝叶斯推断等统计计算。  
**使用场景：** C3、E3_5、I 系列（统计与代码）任务。

**推荐工具：**

| 工具 | 类型 | 地址 |
|------|------|------|
| R MCP Server | 使用 `metafor`/`brms` 等 R 包 | [github.com/holtzy/R-MCP](https://github.com/holtzy/R-MCP)（参考） |
| Python 统计 MCP | 使用 `statsmodels`/`pymc` | 参考下方"自建 stub"章节 |

```bash
# 如果你有 R 环境，可接入 R MCP
export RESEARCH_MCP_STATS_ENGINE_CMD="Rscript /path/to/stats_mcp.R"

# 或使用 Python
export RESEARCH_MCP_STATS_ENGINE_CMD="python3 /path/to/stats_mcp.py"
```

---

### 6. `code-runtime` — 代码执行运行时

**作用：** 在沙箱中安全地执行生成的研究代码（Python/R），并返回输出结果。  
**使用场景：** I 系列（研究代码）任务。

**推荐工具：**

| 工具 | 类型 | 地址 |
|------|------|------|
| Jupyter MCP Server | 通过 Jupyter HTTP 内核执行 | [github.com/datalayer/jupyter-mcp-server](https://github.com/datalayer/jupyter-mcp-server) |
| Modal.com MCP | 云端无服务器代码执行 | [modal.com/docs](https://modal.com/docs) |

```bash
pip install jupyter-mcp-server
export RESEARCH_MCP_CODE_RUNTIME_CMD="python3 -m jupyter_mcp_server"
```

---

### 7. `reporting-guidelines` — 报告规范查询

**作用：** 查询并验证 CONSORT、PRISMA、STROBE、CHEERS 等报告规范条目的完整性。  
**使用场景：** G/H 系列（合规与投稿）任务。

**推荐工具：**

| 工具 | 类型 | 地址 |
|------|------|------|
| EQUATOR MCP（规划中）| 接入 EQUATOR Network 数据库 | [equator-network.org](https://www.equator-network.org) |
| 本地规范 YAML stub | 内嵌规范条目 | 参考下方"自建 stub"章节 |

目前尚无官方发布的 EQUATOR MCP，但框架内的 `skills/G_compliance/reporting-checker.md` 技能卡已内嵌核心规范条目，**不配置此 MCP 不影响合规检查功能**。

---

### 8. `submission-kit` — 投稿材料包管理

**作用：** 管理期刊投稿所需的封面信、作者 CRediT 声明、补充材料清单等文件。  
**使用场景：** H1（投稿准备）任务。

**推荐工具：**

| 工具 | 类型 | 地址 |
|------|------|------|
| Open Journal Systems MCP | 接入 OJS API | 参考 [pkp.sfu.ca/ojs](https://pkp.sfu.ca/ojs/) |
| Overleaf MCP | 接入 Overleaf 项目 | 实验性，见 [overleaf.com/devs](https://www.overleaf.com/devs) |

通常这个 MCP 对于大多数用户意义不大，`submission-packager` 技能卡已可生成完整的投稿材料包。

---

## 统一配置方式

### 方式 A：`.env` 文件（推荐，仅对当前项目生效）

在你的项目根目录创建 `.env`（复制自 `.env.example`）：

```env
# 按需取消注释并填写命令

# RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
# RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD="python3 -m openalex_mcp"
# RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD="npx -y @zcaceres/zotero-mcp-server"
# RESEARCH_MCP_SCREENING_TRACKER_CMD="python3 /path/to/screening_stub.py"
# RESEARCH_MCP_EXTRACTION_STORE_CMD="python3 /path/to/extraction_stub.py"
# RESEARCH_MCP_STATS_ENGINE_CMD="python3 /path/to/stats_mcp.py"
# RESEARCH_MCP_CODE_RUNTIME_CMD="python3 -m jupyter_mcp_server"
# RESEARCH_MCP_REPORTING_GUIDELINES_CMD="python3 /path/to/reporting_stub.py"
# RESEARCH_MCP_SUBMISSION_KIT_CMD="python3 /path/to/submission_stub.py"
```

### 方式 B：Shell 环境变量（全局生效）

```bash
# 添加到 ~/.zshrc 或 ~/.bashrc
export RESEARCH_MCP_STATS_ENGINE_CMD="python3 /path/to/stats_mcp.py"
```

---

## 自建 Stub（让 ⚠ 警告消失的最简方案）

如果你暂时不需要某个 MCP 的功能，但又想消除警告，可以创建一个什么都不做的 stub 脚本。

将下面的文件保存为 `scripts/mcp_<name>.py`（名称对应 MCP，用下划线替换连字符），系统会**自动发现并使用它**，无需配置环境变量：

```python
# scripts/mcp_screening_tracker.py  （示例：screening-tracker 的 stub）
import json, sys

payload = json.loads(sys.stdin.read())
print(json.dumps({
    "status": "ok",
    "summary": "screening-tracker stub: no external tracker configured.",
    "provenance": [],
    "data": {}
}))
```

以下 stub 文件名对应关系供参考：

| MCP 名称 | stub 文件名 |
|---|---|
| `screening-tracker` | `scripts/mcp_screening_tracker.py` |
| `extraction-store` | `scripts/mcp_extraction_store.py` |
| `stats-engine` | `scripts/mcp_stats_engine.py` |
| `code-runtime` | `scripts/mcp_code_runtime.py` |
| `reporting-guidelines` | `scripts/mcp_reporting_guidelines.py` |
| `submission-kit` | `scripts/mcp_submission_kit.py` |
| `metadata-registry` | `scripts/mcp_metadata_registry.py` |
| `fulltext-retrieval` | `scripts/mcp_fulltext_retrieval.py` |

---

## 验证配置

配置完成后，运行诊断命令检查状态：

```bash
python3 -m bridges.orchestrator doctor --cwd .
```

`[OK]` 代表已成功接入，`[WARNING]` 代表仍未配置（但不影响运行）。
