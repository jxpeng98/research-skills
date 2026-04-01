# 发布指南（PyPI Package Publishing）

本指南说明如何将 `research-skills-installer` 发布到 PyPI，以及日常版本发布的完整流程。

## 0) 前置条件（一次性配置）

### 0.1 PyPI Trusted Publisher

本项目使用 [Trusted Publisher](https://docs.pypi.org/trusted-publishers/) 机制发布（无需管理 API Token）。

1. 登录 [pypi.org](https://pypi.org)，进入你的账号
2. 如果是**首次发布**（PyPI 上还没有这个包），进入 [Publishing](https://pypi.org/manage/account/publishing/) 页面，在 "Add a new pending publisher" 中填写：
   - **PyPI Project Name**: `research-skills-installer`
   - **Owner**: `jxpeng98`
   - **Repository name**: `research-skills`
   - **Workflow name**: `publish-pypi.yml`
   - **Environment name**: `pypi`
3. 如果**已有该包**，进入包的 Settings → Publishing → "Add a new publisher"，填写同上

### 0.2 GitHub Environment

1. 进入 GitHub 仓库 Settings → Environments
2. 点击 "New environment"，名称填 `pypi`
3. （可选）添加 protection rules（如仅允许 `main` 分支部署、需要审批等）

### 0.3 TestPyPI Trusted Publisher

本仓库已提供 TestPyPI 专用 workflow：`.github/workflows/publish-testpypi.yml`。

1. 登录 [test.pypi.org](https://test.pypi.org)
2. 进入 Account settings → Publishing
3. 添加 pending publisher（或在已有项目下添加 publisher），填写：
   - **PyPI Project Name**: `research-skills-installer`
   - **Owner**: `jxpeng98`
   - **Repository name**: `research-skills`
   - **Workflow name**: `publish-testpypi.yml`
   - **Environment name**: `testpypi`
4. 回到 GitHub 仓库 Settings → Environments，创建环境 `testpypi`

---

## 1) 日常发布流程

### 1.1 一条命令准备到可发布状态

推荐直接执行：

```bash
./scripts/release_ready.sh --version 0.2.0
./scripts/release_ready.sh --version 0.2.0b1 --from-tag v0.2.0
```

`release_ready.sh` 会串起三段：

- `bump-version.sh`：统一同步以下版本位点：

- `pyproject.toml`
- `research_skills/__init__.py`
- `research-paper-workflow/VERSION`
- `skills/registry.yaml`

- `release_automation.sh pre`：运行 strict validator、仓库单元测试、release smoke，并校验 release 文档
- `pypi_preflight.sh`：构建包、执行 `twine check`，并对生成 wheel 做安装 smoke

当前 release 文档策略：

- stable 正式版统一维护在 `CHANGELOG.md`
- beta / prerelease 继续使用 `release/<tag>.md`

你可以传入稳定版 `0.2.0`，也可以传入 beta 版 `0.2.0b1`。
版本同步阶段会自动规范成三种表示：

| 层 | 稳定版 | Beta |
|------|------|------|
| PyPI package | `0.2.0` | `0.2.0b1` |
| Skill metadata / registry | `0.2.0` | `0.2.0-beta.1` |
| Portable skill `VERSION` / git tag | `v0.2.0` | `v0.2.0-beta.1` |

其中 package 版本遵循 [PEP 440](https://peps.python.org/pep-0440/)，skill metadata 使用 SemVer 兼容的 prerelease 语法。

当前 release tooling 只支持 `stable` 和 `beta`。

如果你想手动拆开执行，旧入口仍然可用：

```bash
./scripts/bump-version.sh 0.2.0
./scripts/release_automation.sh pre --tag v0.2.0
bash scripts/pypi_preflight.sh
```

### 1.2 Commit + TestPyPI 验证

```bash
git add pyproject.toml research_skills/__init__.py research-paper-workflow/VERSION skills/registry.yaml skills CHANGELOG.md
git commit -m "chore: prepare release 0.2.0"
```

然后运行 GitHub Actions 的 `Publish to TestPyPI` workflow，先完成 TestPyPI 安装和 CLI 行为验证，再创建正式发布 tag。

---

## 2) 打 Tag 并发布到 PyPI

TestPyPI 验证通过后：

```bash
git tag v0.2.0
git push origin main --tags
```

> **注意**：tag 必须以 `v*` 开头，并使用 repo release 语法，例如 `v0.2.0` 或 `v0.2.0-beta.1`，GitHub Actions 的 `publish-pypi.yml` 才会触发。

Push tag 后，GitHub Actions 会自动：

1. Checkout 代码
2. 运行 `inject_project_toml.sh`（把当前仓库 slug 写入 `research_skills/project.toml`）
3. `python -m build` 构建 sdist + wheel
4. `twine check` 验证包元数据
5. 使用 Trusted Publisher 发布到 PyPI

在 GitHub 仓库的 **Actions** 页面可以监控进度。

---

## 3) 本地验证（手动 / 可选）

如果你想绕开 `release_ready.sh` 单独跑包预检，可以执行：

```bash
bash scripts/pypi_preflight.sh
bash scripts/pypi_preflight.sh --no-build
```

等价的手动步骤如下：

```bash
# 安装构建工具
pip install build twine

# 注入上游 repo 信息
bash scripts/inject_project_toml.sh

# 构建
python -m build

# 验证
twine check dist/*
```

本地试装：

```bash
pip install dist/research_skills_installer-*.whl
research-skills --help
rsk check --repo jxpeng98/research-skills
```

---

## 4) 发布到 TestPyPI（建议先做）

使用 GitHub Actions workflow（手动触发，无需打 tag）：

1. 打开 GitHub Actions
2. 选择 **Publish to TestPyPI**
3. 在目标分支点击 **Run workflow**

该 workflow 会自动构建、校验并通过 Trusted Publishing 发布到 TestPyPI。

发布后从 TestPyPI 安装验证：

```bash
pip install --index-url https://test.pypi.org/simple/ research-skills-installer
```

推荐顺序：

- 先运行 **Publish to TestPyPI**，验证安装与 CLI 功能
- 验证通过后，再 push `v*` tag 触发正式 **Publish to PyPI**

---

## 5) 完整发布 Checklist

发版时按以下步骤执行：

- [ ] 确认所有功能已合入 `main`
- [ ] CI 通过（`ci.yml` 绿色）
- [ ] 运行 `./scripts/release_ready.sh --version <version>` 或 `./scripts/release_automation.sh publish --version <version>`
- [ ] 如果走的是 `release_ready.sh`，手动 commit release prep 变更
- [ ] 运行 GitHub Actions `Publish to TestPyPI`，并完成 TestPyPI 安装验证
- [ ] 如果没有使用 `publish`，手动创建并 push tag
- [ ] 在 GitHub Actions 确认 `Publish to PyPI` workflow 成功
- [ ] 如果没有使用 `publish`，运行 release postflight：`./scripts/release_automation.sh post --tag v<version> --create-release`
- [ ] 验证安装：`pipx install research-skills-installer && rsk --help`

---

## 6) 常见问题

### Q: tag 推送后 Actions 没有触发？

确认 tag 格式是 `v` 开头（如 `v0.1.0-beta.7`），且 `.github/workflows/publish-pypi.yml` 文件已在 `main` 分支上。

### Q: PyPI 发布失败 "403 Forbidden"？

通常是 Trusted Publisher 配置问题：

- 确认 PyPI 上的 workflow name 完全匹配 `publish-pypi.yml`
- 确认 GitHub environment name 完全匹配 `pypi`
- 确认 owner 和 repository name 正确

### Q: 版本号已存在导致上传失败？

PyPI 不允许覆盖已发布的版本。如果需要修复，必须递增版本号（如 `0.1.0b7` → `0.1.0b8`）。

### Q: TestPyPI 是自动触发吗？

不是。`publish-testpypi.yml` 仅支持 `workflow_dispatch`（手动触发），不会增加仓库 tag。
正式 PyPI 仍由 `publish-pypi.yml` 在 `v*` tag 上自动触发。

### Q: 如何撤回一个已发布的版本？

在 PyPI 项目页面可以 "yank" 一个版本（不会从已安装用户处删除，但 `pip install` 不会默认选择被 yank 的版本）。
