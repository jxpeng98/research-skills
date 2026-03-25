# 严格 Academic Literature Search

当你希望 `research-skills` 的文献检索更接近可审计、可复现的 academic review 工作流，而不是单一引擎的便捷搜索时，使用这份指南。

## 核心原则

把文献检索拆成四层证据能力，而不是让一个引擎包办一切：

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

以下层仍然是外部 provider 插槽：

- `fulltext-retrieval`
- `screening-tracker`
- `extraction-store`

所以当前最现实的“严格 baseline”是：

- 用内置 Semantic Scholar 做发现
- 用内置 metadata-registry 做本地规范化，再用 OpenAlex MCP 做权威 enrichment
- 用内置 citation graph 做 snowballing
- 用 Zotero / OA resolver 做全文获取

## 标准 Literature Bundle

当 literature workflow 运转正常时，建议最终收敛到这组共享产物：

- `search_strategy.md`
- `search_log.md`
- `search_results.csv`
- `dedup_log.csv`
- `snowball_log.md`
- `bibliography.bib`
- `screening/full_text.md`
- `retrieval_manifest.csv`

## 配置矩阵

先用这张表判断你到底需不需要配置：

| 层 | 零配置可否运行 | 是否建议 key | 是否需要 `RESEARCH_MCP_*_CMD` | 说明 |
|---|---|---|---|---|
| `scholarly-search` | 可以，走内置 Semantic Scholar | 建议 | 可选 | 零配置能跑，并能产出 query variants 和 dedup-ready 结果行，但没有 key 时更容易被限流 |
| `citation-graph` | 可以，走内置 Semantic Scholar graph | 不强制 | 可选 | 即使不接外部 MCP，也能先做 snowballing；内置模式会优先从本地产物抽 seed |
| `metadata-registry` | 可以，走内置本地 reference provider | 本地模式不需要 | 可选 | 内置模式能先做 identifier 规范化；要权威 enrichment 时再接 OpenAlex 等外部实现 |
| `fulltext-retrieval` | 不可以 | 取决于 provider | 需要 | 需要接 Zotero 或其他全文解析器 |
| `screening-tracker` | 不可以 | 取决于 provider | 需要 | 主要用于 systematic review |
| `extraction-store` | 不可以 | 取决于 provider | 需要 | 主要用于 systematic review |

## 常用配置总表

如果你不确定“除了 search 之外还有哪些要配”，直接看这张表：

| 功能层 | 是否必须 | 什么时候该配 | 对应环境变量 | 推荐实现 |
|---|---|---|---|---|
| 模型调用 | 必须，至少配你实际使用的一个 | 只要你直接跑 orchestrator / bridge / CLI agent | `OPENAI_API_KEY` / `ANTHROPIC_API_KEY` / `GEMINI_API_KEY` | 按你实际使用的模型提供商填写 |
| CLI 语言 | 可选 | 你希望 CLI 输出固定为中文或英文时 | `RESEARCH_CLI_LANG` | `zh-CN` 或 `en` |
| 文献发现检索 | 非必须，但强烈建议 | 你要做 related work、gap analysis、literature review 时 | `SEMANTIC_SCHOLAR_API_KEY`；可选 `RESEARCH_MCP_SCHOLARLY_SEARCH_CMD` | 内置 Semantic Scholar；更严格时换成多源 scholarly MCP |
| 引文扩展 | 可选 | 你要做 backward / forward snowballing 时 | 可选 `RESEARCH_MCP_CITATION_GRAPH_CMD` | 默认内置 graph；更严格时换成自定义 graph MCP |
| 元数据标准化 | 严格检索时建议视为必须 | 你要统一 DOI、作者、venue、年份，或做可复现 bibliography 时 | 可选 `RESEARCH_MCP_METADATA_REGISTRY_CMD` | 内置本地 reference provider；权威 enrichment 时用 `python3 -m openalex_mcp` |
| 全文获取 | 做系统综述或深度阅读时建议配置 | 你要批量拿 PDF、全文、版本来源链时 | `RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD` | Zotero MCP / OA resolver |
| 筛选跟踪 | systematic review 场景下建议配置 | 你需要记录纳入/排除决策、双人筛选流程时 | `RESEARCH_MCP_SCREENING_TRACKER_CMD` | Rayyan MCP 或本地 stub |
| 结构化提取库 | systematic review 场景下建议配置 | 你要维护 extraction table、effect size、study attribute 时 | `RESEARCH_MCP_EXTRACTION_STORE_CMD` | Covidence 类 MCP 或 CSV/SQLite stub |
| 统计分析引擎 | 做 meta-analysis、统计建模时建议配置 | 你希望把统计运行外包给 R/Python engine 时 | `RESEARCH_MCP_STATS_ENGINE_CMD` | R script MCP / Python stats MCP |
| 代码运行时 | 做 research code 执行时建议配置 | 你希望实际执行生成代码而不只是写代码时 | `RESEARCH_MCP_CODE_RUNTIME_CMD` | Jupyter MCP / 其他 sandbox runtime |
| 报告规范检查 | 投稿前建议配置 | 你要外部化 PRISMA、CONSORT、STROBE 等 guideline 检查时 | `RESEARCH_MCP_REPORTING_GUIDELINES_CMD` | EQUATOR 类 MCP 或本地 checklist stub |
| 投稿包管理 | 非必须 | 你要管理投稿信、CRediT、supplement 清单时 | `RESEARCH_MCP_SUBMISSION_KIT_CMD` | OJS / Overleaf / submission MCP |

### 三个最常见的配置层级

| 使用目标 | 最少配置 |
|---|---|
| 先跑起来 | 一个模型 API key |
| 论文写作 / related work 更稳 | 一个模型 API key + `SEMANTIC_SCHOLAR_API_KEY`；要更强 metadata enrichment 时再加 `RESEARCH_MCP_METADATA_REGISTRY_CMD` |
| systematic review / review-grade | 上述配置 + `RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD`，必要时再补 `RESEARCH_MCP_SCREENING_TRACKER_CMD` 和 `RESEARCH_MCP_EXTRACTION_STORE_CMD` |

## 如果什么都不配置，会发生什么

如果你完全不配置：

- `scholarly-search` 仍会尝试使用内置 Semantic Scholar
- `citation-graph` 仍会尝试使用内置 Semantic Scholar graph
- `metadata-registry` 仍会尝试使用内置本地 reference provider
- `fulltext-retrieval` 处于未配置状态
- 只要你没有显式启用严格 MCP 校验，任务通常仍能继续运行

这足够做 exploratory search，但不适合作为严格 review-grade 检索的长期默认配置。

## 分步配置说明

### 方案 A：零配置起步

什么都不配，直接使用内置 provider。优点是最快，缺点是召回、metadata 质量和限流稳定性都有限。

当前内置 `scholarly-search` baseline 仍会帮你产出：

- 基于 topic / question / keywords 的多组 query variants
- 规范化后的 `search_results` 结果行
- 机器可读的 `dedup_log`
- 按 query 记录的 `search_log` 执行条目

### 方案 B：推荐的轻量增强版

在项目根目录创建 `.env`：

```env
SEMANTIC_SCHOLAR_API_KEY="your-semantic-scholar-key"
RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
```

这是一套对大多数用户最划算的默认增强方案，不需要自己做太多工程接入。

### 方案 C：Review-Grade 多源方案

把 discovery、metadata 和 full text 分成独立层来接：

```env
RESEARCH_MCP_SCHOLARLY_SEARCH_CMD="python3 /path/to/multi_source_search_mcp.py"
RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
RESEARCH_MCP_CITATION_GRAPH_CMD="python3 /path/to/graph_mcp.py"
RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD="npx -y @zcaceres/zotero-mcp-server"
```

适合需要稳定 search log、合并候选集、可追溯 provenance 的项目。

### 方案 D：本地文献库受控方案

如果 discovery 和全文都必须限定在你的本地精选库中：

```env
RESEARCH_MCP_SCHOLARLY_SEARCH_CMD="npx -y @zcaceres/zotero-mcp-server"
RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD="npx -y @zcaceres/zotero-mcp-server"
RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
```

适合审稿前证据链要求更严、或者必须限定语料范围的项目。

## 严格模式说明

如果你用 `--mcp-strict` 跑任务，所有必需的外部 provider 都必须真的配置好。实际含义是：

- 内置 `scholarly-search` 和 `citation-graph` 仍可满足这两层，前提是你没有把它们 override 掉
- 内置 `metadata-registry` 可以先满足本地规范化这一层，不需要额外配置
- 只有当你想让外部权威 enrichment 覆盖 builtin reference 模式时，才需要设置 `RESEARCH_MCP_METADATA_REGISTRY_CMD`
- `fulltext-retrieval` 没有设置 `RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD` 时，也会成为 strict 阻塞项

## 推荐的检索栈

### 1. 轻量增强版

适合想提升 rigor，但不想自己做太多工程接入的情况：

```env
SEMANTIC_SCHOLAR_API_KEY="your-semantic-scholar-key"
RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
```

这套组合意味着：

- 保留内置 Semantic Scholar 检索
- 有 API key 时更不容易频繁遇到 429
- 用 OpenAlex 做 DOI、作者、期刊清洗

### 2. Review-Grade 方案

适合 systematic review、严谨的 related work、或任何强调可复现性的项目：

```env
RESEARCH_MCP_SCHOLARLY_SEARCH_CMD="python3 /path/to/multi_source_search_mcp.py"
RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
RESEARCH_MCP_CITATION_GRAPH_CMD="python3 /path/to/graph_mcp.py"
RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD="npx -y @zcaceres/zotero-mcp-server"
```

推荐职责分配：

- `scholarly-search`：并行调用多个学术索引，返回合并候选集
- `metadata-registry`：统一 DOI、作者、venue 元数据
- `citation-graph`：结构化前向 / 后向引文扩展
- `fulltext-retrieval`：获取全文并记录 provenance

### 3. 本地文献库受控方案

如果你的 review 必须严格限定在本地精选文献库内，可以这样做：

```env
RESEARCH_MCP_SCHOLARLY_SEARCH_CMD="npx -y @zcaceres/zotero-mcp-server"
RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD="npx -y @zcaceres/zotero-mcp-server"
RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
```

它更窄，但通常更容易审计。

## 推荐的查询流程

1. 先写一个 canonical research question，再拆出 2-4 组 query 变体。
2. 每组 query 至少跑两个 discovery source。
3. 先标准化、去重，再进入 screening。
4. 记录 source、日期、query string 和结果数。
5. 从高价值 seed papers 做 citation snowballing。
6. 完成全文解析后再进入 extraction 和 synthesis。

建议在 `RESEARCH/[topic]/` 下保留这些产物：

- `search_strategy.md`
- `search_log.md`
- `bibliography.bib`
- `screening_decisions.csv`
- `fulltext_manifest.csv`

## 不同引擎分别适合什么

按职责选引擎，不要默认某一个引擎能包办全部：

| 引擎 | 更适合的用途 | 说明 |
|---|---|---|
| Semantic Scholar | 快速发现、相关性扫描、citation count | 仓库内置；无 API key 时可能更容易遇到限流 |
| OpenAlex | 元数据标准化、实体图谱、作者 / 期刊清洗 | 很适合作 discovery 的补充层 |
| Crossref | DOI 定位、规范化 metadata harvesting | 很适合作 verification / normalization 层 |
| Europe PMC / PubMed | 生物医学与生命科学检索 | 适合提升特定领域召回率 |
| arXiv | CS、physics、math 的 preprints | 适合 preprint 占比高的领域 |
| CORE | open-access 全文发现 | 适合 OA retrieval |
| Lens | 学术 + 专利全景 | 常见于机构级接入，先确认使用权限 |

## 不建议作为主可复现层的引擎

Google Scholar 仍可用于人工 spot check，但不建议作为这套系统里 review-grade 自动检索的主来源，因为其查询行为和复现性都更难控制。

## 如何验证

配置完成后运行：

```bash
python3 -m bridges.orchestrator doctor --cwd .
```

如果你想单独 smoke-test 内置 `Semantic Scholar` provider，可以运行：

```bash
printf '%s' '{"provider":"scholarly-search","task_packet":{"topic":"causal inference in management research"}}' | python3 scripts/mcp_scholarly_search.py
```

如果返回 `HTTP Error 429`，说明 provider 本身是可达的，但当前被限流了。这时应优先补 `SEMANTIC_SCHOLAR_API_KEY`，或者把 `scholarly-search` 切到你自己的多源 MCP。
