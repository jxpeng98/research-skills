---
layout: home

hero:
  name: Research Skills
  text: 学术工作流文档站
  tagline: 将安装、CLI、工作流入口、架构、MCP 集成与维护者说明收敛到一个统一文档入口。
  actions:
    - theme: brand
      text: 开始使用
      link: /zh/guide/
    - theme: alt
      text: CLI 参考
      link: /zh/reference/cli
    - theme: alt
      text: 系统架构
      link: /zh/architecture

features:
  - title: 使用者路径
    details: 先看快速开始、安装、升级、故障排除，再进入 shell / Python CLI 的具体命令。
  - title: 契约优先
    details: 把 standards、capability routing、roles、skills、pipelines、bridges 放进同一张结构图里理解。
  - title: 高级集成
    details: 把 MCP Provider、Zotero、Agent + Skill 协同与扩展方式整合为连续文档路径。
  - title: 维护者入口
    details: 把 CLAUDE.md、约定规范、发布流程和维护说明整理成站内可导航页面。
---

## 文档导航

- [入门](/zh/guide/): 先看安装、升级、故障排除与快速开始。
- [参考](/zh/reference/): 查看 CLI 命令参考与系统规范。
- [架构](/zh/architecture): 了解层次模型、依赖方向、领域挂载和多模型协同。
- [高级](/zh/advanced/): 进入 MCP、Zotero、扩展与协同增强等主题。
- [维护者](/zh/maintainer/): 查看 CLAUDE 指南摘要、发布操作和维护约定。

## 文档来源

本网站整理并重组了以下原始文档内容：

- `README.md`
- `README_CN.md`
- `docs/`
- `guides/basic/`
- `guides/advanced/`
- `CLAUDE.md`

原始 Markdown 文件仍保留在仓库中；VitePress 站点主要解决的是“内容分散、入口不统一、使用路径不清晰”的问题。
