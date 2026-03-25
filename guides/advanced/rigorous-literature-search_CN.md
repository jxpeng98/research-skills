# 严格 Academic Literature Search

当你希望 `research-skills` 的文献检索更接近可审计、可复现的 academic review 工作流，而不是单一引擎的便捷搜索时，使用这份指南。

## 核心原则

把文献检索拆成四层证据能力：

1. `scholarly-search`：发现候选文献
2. `metadata-registry`：标准化 DOI、作者、期刊、年份
3. `citation-graph`：前向 / 后向引文扩展
4. `fulltext-retrieval`：拿全文并保留来源链

严格检索不应只依赖一个搜索源。

## 当前仓库内置了什么

目前仓库内置的文献 provider 有：

- `scholarly-search` → 内置 Semantic Scholar 适配器，已带 query variants、结果规范化和基础 dedup
- `citation-graph` → 内置 Semantic Scholar 引文图适配器，能先从 `search_results.csv`、`bibliography.bib`、`notes/` 中抽 seed
- `metadata-registry` → 内置本地 reference provider，用于 identifier 规范化

其余层仍需外部 provider：

- `fulltext-retrieval`
- `screening-tracker`
- `extraction-store`

## 标准 Literature Bundle

- `search_strategy.md`
- `search_log.md`
- `search_results.csv`
- `dedup_log.csv`
- `snowball_log.md`
- `bibliography.bib`
- `screening/full_text.md`
- `retrieval_manifest.csv`

## 配置矩阵

| 层 | 零配置可否运行 | 是否建议 key | 是否需要 `RESEARCH_MCP_*_CMD` | 说明 |
|---|---|---|---|---|
| `scholarly-search` | 可以 | 建议 | 可选 | 内置 Semantic Scholar 能跑，并能产出 query variants 和 dedup-ready 结果行，但可能限流 |
| `citation-graph` | 可以 | 不强制 | 可选 | 内置 graph adapter 可用，并会优先从本地产物解析 seed |
| `metadata-registry` | 可以 | 本地模式不需要 | 可选 | 内置模式能先做 identifier 规范化；权威 enrichment 时再接 OpenAlex 或其他 metadata MCP |
| `fulltext-retrieval` | 不可以 | 取决于 provider | 需要 | 建议接 Zotero 或其他全文解析器 |
| `screening-tracker` | 不可以 | 取决于 provider | 需要 | systematic review 支持 |
| `extraction-store` | 不可以 | 取决于 provider | 需要 | systematic review 支持 |

## 常用配置总表

| 功能层 | 是否必须 | 什么时候该配 | 对应环境变量 | 推荐实现 |
|---|---|---|---|---|
| 模型调用 | 必须，至少一个 | 直接跑 orchestrator / bridge / CLI agent 时 | `OPENAI_API_KEY` / `ANTHROPIC_API_KEY` / `GEMINI_API_KEY` | 按实际模型提供商填写 |
| CLI 语言 | 可选 | 想固定中文或英文输出时 | `RESEARCH_CLI_LANG` | `zh-CN` 或 `en` |
| 文献发现检索 | 非必须，但强烈建议 | 做 related work、gap analysis、literature review 时 | `SEMANTIC_SCHOLAR_API_KEY`；可选 `RESEARCH_MCP_SCHOLARLY_SEARCH_CMD` | 内置 Semantic Scholar 或多源 scholarly MCP |
| 引文扩展 | 可选 | 做 snowballing 时 | 可选 `RESEARCH_MCP_CITATION_GRAPH_CMD` | 内置 graph 或自定义 graph MCP |
| 元数据标准化 | 严格检索时建议视为必须 | 做 DOI、作者、venue 标准化时 | `RESEARCH_MCP_METADATA_REGISTRY_CMD` | `python3 -m openalex_mcp` |
| 全文获取 | 系统综述时建议配置 | 批量拿 PDF / 全文 / provenance 时 | `RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD` | Zotero MCP / OA resolver |
| 筛选跟踪 | systematic review 时建议配置 | 记录纳入/排除决策时 | `RESEARCH_MCP_SCREENING_TRACKER_CMD` | Rayyan MCP 或本地 stub |
| 结构化提取库 | systematic review 时建议配置 | 维护 extraction table 时 | `RESEARCH_MCP_EXTRACTION_STORE_CMD` | Covidence 类 MCP 或 CSV/SQLite stub |
| 统计分析引擎 | 分析导向任务时建议配置 | 做 meta-analysis、统计建模时 | `RESEARCH_MCP_STATS_ENGINE_CMD` | R / Python stats MCP |
| 代码运行时 | 代码执行任务时建议配置 | 需要真正执行生成代码时 | `RESEARCH_MCP_CODE_RUNTIME_CMD` | Jupyter MCP / sandbox runtime |
| 报告规范检查 | 投稿前建议配置 | 外部化 guideline 检查时 | `RESEARCH_MCP_REPORTING_GUIDELINES_CMD` | EQUATOR 类 MCP 或本地 stub |
| 投稿包管理 | 非必须 | 处理 cover letter、CRediT、supplement 时 | `RESEARCH_MCP_SUBMISSION_KIT_CMD` | OJS / Overleaf / submission MCP |

## 分步配置说明

### 方案 A：零配置起步

只用内置 provider。

当前内置 `scholarly-search` baseline 仍会给你：

- 基于 topic / question / keywords 的多组 query variants
- 规范化后的 `search_results` 结果行
- 机器可读的 `dedup_log`
- 按 query 记录的 `search_log` 执行条目

### 方案 B：推荐的轻量增强版

```env
SEMANTIC_SCHOLAR_API_KEY="your-semantic-scholar-key"
RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
```

### 方案 C：Review-Grade 多源方案

```env
RESEARCH_MCP_SCHOLARLY_SEARCH_CMD="python3 /path/to/multi_source_search_mcp.py"
RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
RESEARCH_MCP_CITATION_GRAPH_CMD="python3 /path/to/graph_mcp.py"
RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD="npx -y @zcaceres/zotero-mcp-server"
```

### 方案 D：本地文献库受控方案

```env
RESEARCH_MCP_SCHOLARLY_SEARCH_CMD="npx -y @zcaceres/zotero-mcp-server"
RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD="npx -y @zcaceres/zotero-mcp-server"
RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
```

## 严格模式说明

启用 `--mcp-strict` 后，所有必需 provider 都必须已配置。当前通常最先需要显式接入的是 `fulltext-retrieval`，而 `metadata-registry` 现在可以先回落到仓库内置的本地 reference provider。

## 推荐的检索栈

### 1. 轻量增强版

```env
SEMANTIC_SCHOLAR_API_KEY="your-semantic-scholar-key"
RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
```

### 2. Review-Grade 方案

```env
RESEARCH_MCP_SCHOLARLY_SEARCH_CMD="python3 /path/to/multi_source_search_mcp.py"
RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
RESEARCH_MCP_CITATION_GRAPH_CMD="python3 /path/to/graph_mcp.py"
RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD="npx -y @zcaceres/zotero-mcp-server"
```

### 3. 本地文献库受控方案

```env
RESEARCH_MCP_SCHOLARLY_SEARCH_CMD="npx -y @zcaceres/zotero-mcp-server"
RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD="npx -y @zcaceres/zotero-mcp-server"
RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
```

## 推荐查询流程

1. 先写一个 canonical research question，再拆出 2-4 组 query 变体。
2. 每组 query 至少跑两个 discovery source。
3. 先标准化、去重，再进入 screening。
4. 记录 source、日期、query string 和结果数。
5. 从高价值 seed papers 做 citation snowballing。
6. 拿到全文后再进入 extraction 和 synthesis。

建议保留这些产物：

- `search_strategy.md`
- `search_log.md`
- `bibliography.bib`
- `screening_decisions.csv`
- `fulltext_manifest.csv`

## 引擎职责

| 引擎 | 更适合的用途 | 说明 |
|---|---|---|
| Semantic Scholar | 快速发现、相关性扫描、citation count | 仓库内置；无 API key 时更容易遇到限流 |
| OpenAlex | 元数据标准化、实体图谱、作者 / 期刊清洗 | 很适合作 discovery 的补充层 |
| Crossref | DOI 定位、规范化 metadata harvesting | 很适合作 verification / normalization 层 |
| Europe PMC / PubMed | 生物医学与生命科学检索 | 提高特定领域召回率 |
| arXiv | CS、physics、math 的 preprints | 适合 preprint 占比高的领域 |
| CORE | open-access 全文发现 | 适合 OA retrieval |
| Lens | 学术 + 专利全景 | 常见于机构级接入，先确认权限 |

## 验证方式

```bash
python3 -m bridges.orchestrator doctor --cwd .
printf '%s' '{"provider":"scholarly-search","task_packet":{"topic":"causal inference in management research"}}' | python3 scripts/mcp_scholarly_search.py
```

如果内置 provider 返回 `HTTP Error 429`，优先补 `SEMANTIC_SCHOLAR_API_KEY`，或者把 `scholarly-search` 切到你自己的多源 MCP。
