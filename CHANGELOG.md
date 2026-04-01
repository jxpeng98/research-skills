# Changelog

本文件汇总自 `v0.3.0`（2026-03-25）以来到当前 `HEAD`（2026-04-01）的主要更新，重点记录用户可感知的新能力、安装体验变化与重要修复。`0.4.0` 采用正式版 summary 写法，已将 `0.3.0` 之后的 beta 演进合并整理，不再按小 beta 分段展开。

## [Unreleased] - 2026-04-01

### Added

- 为 `scripts/bootstrap_research_skill.ps1` 增加用于生成用户 `PATH` 更新命令的辅助函数，进一步完善 Windows 安装后的环境变量刷新体验。

### Changed

- 正式版发布流程改为以 `CHANGELOG.md` 作为 GitHub Release 的说明来源；beta / prerelease 继续使用 `release/<tag>.md`。

## [0.4.0] - 2026-04-01

### Added

- 新增严格文献工作流能力，包括 `literature_search`、`metadata_registry`、`fulltext_retrieval`、`citation_graph` 与 `overlay_runtime`，并配套 smoke tests、fixtures 与集成测试。
- 新增学术演示阶段 `K_presentation`，覆盖 presentation planning、slide architecture、Slidev scholarly builder 与 Beamer builder。
- 引入一键 bootstrap 安装流程，支持 `partial` / `full` 两种安装档位。
- 扩展 Codex / Claude Code / Gemini 的集成资产，补充 agent profiles、路由文档与多客户端安装说明。
- CLI 新增 command runtime utilities，为命令解析、执行与安装流程打基础。
- 新增 `install-check` GitHub Actions 工作流，并增强发布自动化的 publish mode、CI 检查、push 事件支持和标签校验。
- Universal installer 增加 `install_manifest.tsv` 打包支持，改善通过 Python 包安装时的资产分发一致性。
- 安装脚本新增缺失客户端 CLI 的提示文案，并加入 `antigravity` 目标支持。
- bootstrap 脚本支持安装最新 beta / prerelease 标签，便于验证预发布版本。
- 为 bootstrap 增加 PowerShell 7+ 版本检查和更稳健的命令执行流程。
- 支持从本地源码仓库安装，便于本地开发、离线测试和验证未发布改动。
- `full` 模式增强了 `PATH` 管理和用户环境持久化，安装 `mise` 后会同步更新当前会话与后续 shell 环境。

### Changed

- 大幅增强 skills 语料与工作流文档，覆盖 literature、compliance、design、writing、code、submission 和 presentation 阶段。
- MCP provider 文档补充 resolver handoff、source-aware merge policy 和环境变量说明。
- Validator 简化冗余兼容性检查，降低维护复杂度。
- 安装与 quickstart 文档重写，推荐使用 `mise` 管理 Python，并明确 Windows 侧需要 PowerShell 7+。
- README 与安装文档补充 prerelease 安装方式以及 `full` 模式行为说明。

### Fixed

- 修复早期 Windows 兼容性问题，并提升跨平台校验一致性。
- 改进 bootstrap 的 dry-run 输出、清理逻辑以及从源码仓库安装 `partial` profile 的处理。
- 改善 Python 环境处理和跨平台脚本执行细节，显式使用 Bash 运行 postflight 与测试脚本。
- 修复 release automation 和 postflight 脚本中的 `git fetch` 分支引用处理问题。

## [0.3.0] - 2026-03-25

### Baseline

- 这是 `0.4.0` beta 系列之前的稳定基线版本，完成了 skill 版本元数据收敛：以 `skills/registry.yaml` 作为单一版本源，并更新了对应的校验与发布流程。
