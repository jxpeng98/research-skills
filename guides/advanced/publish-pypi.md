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

---

## 1) 日常发布流程

### 1.1 更新版本号

使用 `bump-version.sh` 脚本同时更新 `pyproject.toml` 和 `research_skills/__init__.py`：

```bash
./scripts/bump-version.sh 0.2.0
```

版本号格式遵循 [PEP 440](https://peps.python.org/pep-0440/)：

| 阶段 | 格式 | 示例 |
|------|------|------|
| Beta | `X.Y.ZbN` | `0.1.0b7` |
| RC | `X.Y.ZrcN` | `0.1.0rc1` |
| 正式版 | `X.Y.Z` | `0.1.0`、`1.0.0` |

### 1.2 Commit + Tag + Push

```bash
git add pyproject.toml research_skills/__init__.py
git commit -m "chore: bump version to 0.2.0"
git tag v0.2.0
git push origin main --tags
```

> **注意**：tag 格式必须是 `v*`（如 `v0.2.0`、`v0.1.0b7`），GitHub Actions 的 `publish-pypi.yml` 才会触发。

### 1.3 自动构建 & 发布

Push tag 后，GitHub Actions 会自动：

1. Checkout 代码
2. 运行 `inject_project_toml.sh`（把当前仓库 slug 写入 `research_skills/project.toml`）
3. `python -m build` 构建 sdist + wheel
4. `twine check` 验证包元数据
5. 使用 Trusted Publisher 发布到 PyPI

在 GitHub 仓库的 **Actions** 页面可以监控进度。

---

## 2) 本地验证（发布前可选）

如果你想在发布前先确认包能正确构建：

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
rs check --repo jxpeng98/research-skills
```

---

## 3) 发布到 TestPyPI（可选）

如果想先发布到 TestPyPI 进行测试，可以手动上传：

```bash
# 先在 test.pypi.org 注册并获取 API token
twine upload --repository testpypi dist/*

# 从 TestPyPI 安装测试
pip install --index-url https://test.pypi.org/simple/ research-skills-installer
```

---

## 4) 完整发布 Checklist

发版时按以下步骤执行：

- [ ] 确认所有功能已合入 `main`
- [ ] CI 通过（`ci.yml` 绿色）
- [ ] 运行 release preflight：`./scripts/release_automation.sh pre --tag v<version>`
- [ ] 更新版本号：`./scripts/bump-version.sh <version>`
- [ ] Commit 版本号变更
- [ ] 创建 tag：`git tag v<version>`
- [ ] Push：`git push origin main --tags`
- [ ] 在 GitHub Actions 确认 `Publish to PyPI` workflow 成功
- [ ] 运行 release postflight：`./scripts/release_automation.sh post --tag v<version>`
- [ ] 验证安装：`pipx install research-skills-installer && rs --help`

---

## 5) 常见问题

### Q: tag 推送后 Actions 没有触发？

确认 tag 格式是 `v` 开头（如 `v0.1.0b7`），且 `.github/workflows/publish-pypi.yml` 文件已在 `main` 分支上。

### Q: PyPI 发布失败 "403 Forbidden"？

通常是 Trusted Publisher 配置问题：
- 确认 PyPI 上的 workflow name 完全匹配 `publish-pypi.yml`
- 确认 GitHub environment name 完全匹配 `pypi`
- 确认 owner 和 repository name 正确

### Q: 版本号已存在导致上传失败？

PyPI 不允许覆盖已发布的版本。如果需要修复，必须递增版本号（如 `0.1.0b7` → `0.1.0b8`）。

### Q: 如何撤回一个已发布的版本？

在 PyPI 项目页面可以 "yank" 一个版本（不会从已安装用户处删除，但 `pip install` 不会默认选择被 yank 的版本）。
