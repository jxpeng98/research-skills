# Academic Deep Research Skills

一套专为学术研究场景设计的 Claude Code 深度研究技能系统。

## 功能特性

- 📚 **系统性文献综述** - 遵循 PRISMA 方法论
- 📖 **论文深度阅读** - 结构化笔记 + BibTeX
- 🔍 **研究 Gap 识别** - 5类学术空白分析
- 🧠 **理论框架构建** - 概念关系映射
- ✍️ **学术写作辅助** - 符合学术规范

## 快速开始

### 安装

将此仓库克隆到你的项目中，Claude Code 会自动识别 `.agent/workflows/` 中的命令。

```bash
git clone <repository-url> research-skills
```

### 使用命令

| 命令 | 用途 | 示例 |
|------|------|------|
| `/lit-review` | 系统性文献综述 | `/lit-review transformer architecture 2020-2024` |
| `/paper-read` | 深度阅读论文 | `/paper-read https://arxiv.org/abs/2303.08774` |
| `/find-gap` | 识别研究空白 | `/find-gap LLM in education` |
| `/build-framework` | 构建理论框架 | `/build-framework technology acceptance` |
| `/academic-write` | 学术写作辅助 | `/academic-write introduction AI ethics` |

## 核心工作流

### 1. 系统性文献综述 `/lit-review`

遵循 PRISMA 2020 方法论：

```
研究问题定义 (PICO/PEO)
       ↓
多学术数据库检索 (Semantic Scholar, arXiv, Google Scholar)
       ↓
标题/摘要筛选 → 全文筛选
       ↓
数据提取 + 质量评估
       ↓
综合报告 + PRISMA 流程图
```

### 2. 论文深度阅读 `/paper-read`

从论文中提取：
- 研究问题 (RQs)
- 理论框架
- 研究方法 (设计、样本、分析)
- 核心发现
- 贡献与局限性
- Future Work

输出格式：Markdown 笔记 + BibTeX 引用

### 3. 研究 Gap 识别 `/find-gap`

识别以下类型的研究空白：
- **理论 Gap** - 框架不完整或冲突
- **方法论 Gap** - 研究方法存在局限
- **实证 Gap** - 缺乏特定情境证据
- **知识 Gap** - 某主题研究不足
- **人群 Gap** - 特定群体未被研究

### 4. 理论框架构建 `/build-framework`

- 现有理论梳理与对比
- 概念关系映射 (Mermaid 图谱)
- 假设/命题推导

### 5. 学术写作辅助 `/academic-write`

支持论文各章节：
- Introduction (研究背景、问题陈述)
- Literature Review (主题组织、批判性综述)
- Methodology (方法论证明)
- Discussion (发现解释、理论对话)
- Conclusion (贡献总结、局限性)

## 学术证据评级

| 等级 | 证据类型 |
|------|----------|
| **A** | 系统性综述、Meta 分析、RCT |
| **B** | 队列研究、高影响因子期刊论文 |
| **C** | 案例研究、专家意见、会议论文 |
| **D** | 预印本、工作论文 |
| **E** | 轶事、理论推测 |

## 目录结构

```
research-skills/
├── .agent/workflows/     # 用户命令
├── skills/               # 可复用技能模块
├── templates/            # 输出模板
├── RESEARCH/             # 研究输出目录
├── CLAUDE.md             # Claude Code 快速参考
└── README.md             # 本文件
```

## 学术数据库支持

- **Semantic Scholar** - 跨领域 (2亿+论文)，API 直接调用
- **arXiv** - 物理/数学/CS/AI，API 直接调用
- **Google Scholar** - 跨领域，通过网页搜索
- **PubMed** - 生物医学，通过网页搜索

## License

MIT
