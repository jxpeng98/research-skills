---
layout: home

hero:
  name: Qiongli
  text: "用 AI agent 做学术研究，同时保留可复查证据链。"
  tagline: "把一个研究主题拆成论文路线、Task ID、质量门、文献和引用证据、写作与代码产物，以及可追踪的审阅交接。"
  actions:
    - theme: alt
      text: 下载 2.x CLI
      link: /zh/guide/cli-2x#standalone-binary-download
    - theme: brand
      text: 快速开始
      link: /zh/quickstart
    - theme: alt
      text: 安装
      link: /zh/guide/install
    - theme: alt
      text: 选择工作流
      link: /zh/guide/task-recipes

features:
  - title: "先小后大"
    details: "按任务选择 native plugin、Desktop ZIP、bootstrap、npm 或 pipx，不必一开始装完整运行时。"
  - title: "把工作路由清楚"
    details: "把研究目标映射到 paper type、stage、Task ID、预期产物和质量门。"
  - title: "证据保持可见"
    details: "把 claim、citation、search log、diagnostics、method、code、review status 和 handoff 放到稳定路径。"
  - title: "更新和刷新分开"
    details: "`qiongli update` 升级 package；`qiongli upgrade` 刷新本地 assets。边界明确。"
---

## 先选入口

| 你想做什么 | 从这里开始 |
|---|---|
| 只想先在一个客户端试用 | [安装](/zh/guide/install) |
| 从零跑到第一个 workspace | [快速开始](/zh/quickstart) |
| 安装后不知道怎么调用 | [使用 Agent Skills](/zh/guide/using-agent-skills) |
| 查看每个模型/Host 真正通过了哪些实测 | [Agent Host 实测能力矩阵](/zh/guide/agent-host-capability-matrix) |
| 选择论文工作流 | [任务场景](/zh/guide/task-recipes) |
| 使用 validator、`doctor` 或 orchestrated task | [多 Agent 运行](/zh/guide/multi-agent) |
| 自动化安装、检查、更新或发版 | [CLI 参考](/zh/reference/cli) |

## 最新稳定版下载

当前稳定版是 [v1.17.0](https://github.com/jxpeng98/qiongli/releases/tag/v1.17.0)。下面这些直达链接覆盖常见安装路径；需要 subject 专精 Desktop ZIP 或维护者 artifacts 时，再打开下载指南。

| 需求 | 链接或命令 |
|---|---|
| npm CLI | [`qiongli@1.17.0`](https://www.npmjs.com/package/qiongli/v/1.17.0)：`npm install -g qiongli@latest` |
| PyPI CLI | [`qiongli 1.17.0`](https://pypi.org/project/qiongli/1.17.0/)：`pipx install qiongli` |
| Claude Desktop 推荐插件 | [`qiongli-claude-desktop-plugin-v1.17.0.zip`](https://github.com/jxpeng98/qiongli/releases/download/v1.17.0/qiongli-claude-desktop-plugin-v1.17.0.zip) |
| Claude Desktop/Web fallback skill ZIP | [`qiongli-claude-desktop-skill-core-v1.17.0.zip`](https://github.com/jxpeng98/qiongli/releases/download/v1.17.0/qiongli-claude-desktop-skill-core-v1.17.0.zip) |
| Claude Desktop literature MCPB | [`qiongli-literature-provider-0.1.5.mcpb`](https://github.com/jxpeng98/qiongli/releases/download/v1.17.0/qiongli-literature-provider-0.1.5.mcpb) |
| Zotero Desktop companion | [`qiongli-zotero-companion-0.2.2.xpi`](https://github.com/jxpeng98/qiongli/releases/download/v1.17.0/qiongli-zotero-companion-0.2.2.xpi) |
| 全部 release assets | [下载指南](https://github.com/jxpeng98/qiongli/releases/download/v1.17.0/qiongli-downloads-v1.17.0.md) 和 [GitHub Release](https://github.com/jxpeng98/qiongli/releases/tag/v1.17.0) |

## 系统覆盖什么

Qiongli 提供便携的 `qiongli-workflow` package，也提供可选的本地 literature search 和 orchestration 运行时。

- **定义问题：** question、gap、contribution claim、venue 和边界。
- **文献工作：** provider-aware search、diagnostics、bundle、screening、extraction 和 snowballing。
- **研究设计：** variables、datasets、robustness、preregistration、ethics 和 data management。
- **论文写作：** claim-evidence map、tables、figures、limitations、proofreading、submission 和 rebuttal。
- **研究代码：** Stage-I specification、planning、execution、review。
- **多模型协作：** solo、duo、triad roles，以及可追溯 handoff。

## 运行时边界

安装 workflow assets 不等于启动本地 agent execution。你可以在没有 Python 的情况下使用 skill/plugin。只有运行 `doctor`、validator、MCP orchestration 或真实 task execution 时，才需要 Python 3.12+、模型 CLI 和对应认证。

## 文档地图

- [入门](/zh/guide/): 安装、使用、升级、故障排除和 runtime 选择。
- [示例](/zh/examples/): 不同 paper type 的 playbook。
- [参考](/zh/reference/): CLI 行为和 skill catalog。
- [架构](/zh/architecture): package surfaces、contracts、roles 和 bridges。
- [高级](/zh/advanced/): MCP providers、Zotero、subject packaging、plugin-first distribution。
- [维护者](/zh/maintainer/): release policy、naming policy 和贡献者指南。
