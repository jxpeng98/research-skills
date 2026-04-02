# 🔌 可选 MCP Provider 接入指南

运行 `rsk upgrade` 后，你可能会看到类似以下的提示信息：

```
⚠  MCP screening-tracker: RESEARCH_MCP_SCREENING_TRACKER_CMD not configured
⚠  MCP extraction-store: RESEARCH_MCP_EXTRACTION_STORE_CMD not configured
...
```

**这些⚠仅为提示，不影响框架核心功能。** 这些 MCP（模型上下文协议工具）里有一部分已经是仓库内置 reference provider，另一部分仍是可选的外部能力增强接口。本文档说明它们各是什么、从哪里获取、以及如何接入。

如果你的目标是更严格、可复现的 academic literature search，请在读完本页后继续看 [严格 Academic Literature Search](/zh/advanced/rigorous-literature-search)。本页讲 provider 的接线方式，后者讲的是这些检索层应该如何组合。

---

## 工作原理

每个 MCP 对应一个环境变量（`RESEARCH_MCP_<NAME>_CMD`）。  
系统在执行任务时，会通过这个命令启动一个子进程，传入 JSON 数据包，并读取 JSON 格式的响应。

这个仓库里实际有三种接入模式：

1. **完整外部替换**
   设置 `RESEARCH_MCP_<NAME>_CMD`，让某个外部命令完整接管这个 provider slot。
2. **builtin baseline + 外部 overlay**
   对部分 literature MCP，推荐保留 builtin provider，再在其上叠加外部 enrichment / resolver：
   - `RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD`
   - `RESEARCH_MCP_FULLTEXT_RETRIEVAL_RESOLVE_CMD`
3. **builtin 或 auto-discovered stub fallback**
   如果没有环境变量，运行时会优先寻找 `scripts/mcp_<name>.py`。如果 env var 和 builtin/stub 都不存在，就显示 ⚠ 提示，并以降级模式继续执行。

## MCP 能力矩阵

先看这张表。它回答的是：仓库里默认已经有什么、什么时候值得接外部 provider、什么时候 stub 就够。

| MCP | 仓库内 builtin baseline | 推荐外接模式 | 什么时候该接外部 provider | 什么时候 stub 就够 | 主要环境变量 |
|---|---|---|---|---|---|
| `metadata-registry` | 有。本地规范化、合并、citekey 生成、本地文献产物摄取。 | 通常是 **overlay**，不是 replace。 | 当你要在本地 reference state 之上叠加 OpenAlex/Crossref 这类权威 enrichment。 | 基本不需要；builtin 已经有实际价值。 | `RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD`, `RESEARCH_MCP_METADATA_REGISTRY_CMD` |
| `fulltext-retrieval` | 有，但只是 planning。能草拟 manifest 和全文追踪。 | 通常是 **overlay resolver**，不是 replace。 | 当你需要真实 PDF/全文解析，而不只是 manifest planning。 | 仅当当前阶段只关心 planning / audit，不急着真实下载全文。 | `RESEARCH_MCP_FULLTEXT_RETRIEVAL_RESOLVE_CMD`, `RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD` |
| `screening-tracker` | 没有 builtin provider。 | 直接接外部 MCP 或本地 stub。 | 当你需要 PRISMA 决策持久化、双人筛选或盲筛流程。 | 单人使用、非系统综述、或筛选状态在仓库外部管理时够用。 | `RESEARCH_MCP_SCREENING_TRACKER_CMD` |
| `extraction-store` | 没有 builtin provider。 | 直接接外部 MCP 或本地 stub。 | 当你需要跨 B/E 阶段共享结构化 extraction store，或多人维护提取结果。 | 当提取仍然主要落在 markdown / CSV 产物里时够用。 | `RESEARCH_MCP_EXTRACTION_STORE_CMD` |
| `stats-engine` | 没有 builtin provider。 | 直接接外部 MCP。 | 当你要真实执行模型、meta-analysis、Bayesian 计算或数值诊断。 | 只有在你现在还停留在 plan/spec 阶段，不做真实计算时才够。 | `RESEARCH_MCP_STATS_ENGINE_CMD` |
| `code-runtime` | 没有 builtin provider。 | 直接接外部 MCP。 | 当你要在框架内安全执行 Python/R 研究代码，而不是只写代码计划。 | 只有当任务只是设计/规范化，或代码会在框架外独立运行时才够。 | `RESEARCH_MCP_CODE_RUNTIME_CMD` |
| `reporting-guidelines` | 没有 builtin MCP，但 `reporting-checker` skill 已经提供强 fallback。 | 直接接外部 MCP 或本地 checklist stub。 | 当你要把 guideline lookup、清单覆盖和审计外部化。 | 通常够用，因为 repo 内 skill 已带核心规范逻辑。 | `RESEARCH_MCP_REPORTING_GUIDELINES_CMD` |
| `submission-kit` | 没有 builtin MCP，但 `submission-packager` skill 已经提供强 fallback。 | 直接接外部 MCP。 | 当你需要和期刊系统、Overleaf 等下游系统直接联动。 | 通常够用，只要你现在需要的是本地 artifact 生成。 | `RESEARCH_MCP_SUBMISSION_KIT_CMD` |

## 快速决策规则

- 如果仓库默认已经能产出你需要的 artifact，而且你关心的是 contract 完整性，优先选 **builtin only**。
- 对 `metadata-registry` 和 `fulltext-retrieval`，如果你希望 artifact ownership 继续留在仓库内，但权威来源来自外部系统，优先选 **builtin + overlay**。
- 只有当外部 MCP 明显比 builtin 更适合完全接管该 slot 时，才选 **full external override**。
- 如果你只是想消除 warning，或者先满足 orchestration contract，但暂时并不真正使用这项能力，就选 **thin local stub**。

---

## 各 MCP Provider 说明与接入方式

### 1. `metadata-registry` — 文献元数据标准化

**作用：** 补全和标准化论文的 DOI、期刊、年份、作者信息。  
**使用场景：** B 阶段（文献处理）、C1 任务等。

这个 provider 现在已经有仓库内置的本地 reference 实现，可用于 identifier 规范化、本地记录合并和 citekey 生成。它可以直接读取 `bibliography.bib`、`references.json`、`references.ris`、`search_results.csv` 和 `notes/*.md`。如果你需要更权威的 enrichment，优先建议在 builtin provider 上叠加 OpenAlex 这类外部实现。

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

只有当你想完全替换掉 builtin metadata-registry 时，才去设置 `RESEARCH_MCP_METADATA_REGISTRY_CMD`。

---

### 2. `fulltext-retrieval` — 论文全文获取

**作用：** 解析并获取论文 PDF 全文、追踪版本来源。  
**使用场景：** B1（系统综述）、B2（全文提取）任务。

这个 provider 现在也有仓库内置的 retrieval-planning stub。内置模式不会真正下载 PDF，但会根据本地文献产物草拟 `retrieval_manifest.csv` 和 `screening/full_text.md`，保留已有 manifest 行、检查本地路径是否存在，并标出需要 OA/manual follow-up 的条目。

resolver handoff 现在按分层合同处理：
- 默认保留 builtin stub 作为 planning baseline
- 当你希望外部解析器把真实获取结果写回 manifest 时，设置 `RESEARCH_MCP_FULLTEXT_RETRIEVAL_RESOLVE_CMD`
- 只有想完全替换 builtin provider 时，才设置 `RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD`

当前 resolver merge policy 不是简单覆盖，而是带 source-aware 优先级：
- resolver 返回的 `retrieved_*` 状态会优先覆盖 builtin 的 `not_retrieved:*` planning 状态
- resolver 提供的 `fulltext_path`、`license`、`version_label` 会在优先级相同或更高时替换 builtin 占位值
- builtin planning notes 会保留，resolver notes 会追加进去，方便审计

**推荐工具：**

| 工具 | 类型 | 地址 |
|------|------|------|
| 内置 retrieval-planning stub | 仓库内置 | `scripts/mcp_fulltext_retrieval.py` |
| Zotero MCP Server | Node.js，接入本地 Zotero 库 | [github.com/zcaceres/zotero-mcp](https://github.com/zcaceres/zotero-mcp) |
| Unpaywall API wrapper | 可获取开放获取全文 | 自定义脚本接入 `api.unpaywall.org` |

```bash
# 如果你想保留 builtin planning，再叠加 Zotero 的真实解析结果（需要 Zotero 桌面客户端运行中）
npm install -g @zcaceres/zotero-mcp-server
export RESEARCH_MCP_FULLTEXT_RETRIEVAL_RESOLVE_CMD="npx -y @zcaceres/zotero-mcp-server"
```

只有当你想完全把 builtin planning stub 替换成“外部全文解析 provider”时，才需要设置 `RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD`。

> **提示：** 详细的 Zotero 接入流程见 [`mcp-zotero-integration.md`](./mcp-zotero-integration.md)。

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

```bash
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
