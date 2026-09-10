# Qiongli 2 CLI：安装与直接下载

Qiongli 2 以原生 CLI 为入口，不需要打开或安装 Qiongli App。
同一版本、同一平台的 GitHub 二进制包、npm 和 PyPI 包使用相同的原生可执行文件。

## 独立二进制下载 {#standalone-binary-download}

**推荐直接下载：解压后就能运行，不用先安装 Python、Node.js、Rust 或包管理器。**
你可以直接在解压目录使用，PATH 配置是可选项。

从 [GitHub Release v2.0.0-beta.2](https://github.com/jxpeng98/qiongli/releases/tag/v2.0.0-beta.2)
选择与你的操作系统和 CPU 对应的压缩包：

| 平台 | 完整 CLI 二进制包 |
|---|---|
| macOS Apple Silicon / ARM64 | [qiongli-2.0.0-beta.2-aarch64-apple-darwin.tar.gz](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.2/qiongli-2.0.0-beta.2-aarch64-apple-darwin.tar.gz) |
| Windows x64 | [qiongli-2.0.0-beta.2-x86_64-pc-windows-msvc.zip](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.2/qiongli-2.0.0-beta.2-x86_64-pc-windows-msvc.zip) |
| Linux x64 / glibc 2.35+ | [qiongli-2.0.0-beta.2-x86_64-unknown-linux-gnu.tar.gz](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.2/qiongli-2.0.0-beta.2-x86_64-unknown-linux-gnu.tar.gz) |

Windows Beta.1 已将 C 运行库编入程序，不需要另装 Visual C++ 运行库。
Linux 使用系统自带库，要求 glibc 2.35+。

压缩包内有 `qiongli`（Windows 为 `qiongli.exe`）、`README.md` 和 `LICENSE`。
研究 Skills、模板及 Lite/Full MCP 资源已经嵌入可执行文件，无需额外下载资源目录或克隆仓库。
模型 Host 和在线文献服务仍需单独配置。

请在 Release 的 **Assets** 中选择上述平台包。页面自动生成的 **Source code** 是需要编译的源码，
`.tgz` 是 npm 包，`.whl` 是 Python 包，`qiongli-next-…-plugin-…` 是 Host 插件包。
当前版本不提供 Intel Mac、Linux ARM 或 Windows ARM 原生构建。

### 1. 校验下载文件

下载同一 Release 的 [SHA256SUMS](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.2/SHA256SUMS)。
在下载目录中，运行与你的平台对应的命令，将结果与 `SHA256SUMS` 中该文件名对应的摘要比较；一致后再继续。

```sh
# macOS
shasum -a 256 qiongli-2.0.0-beta.2-aarch64-apple-darwin.tar.gz
# Linux
sha256sum qiongli-2.0.0-beta.2-x86_64-unknown-linux-gnu.tar.gz
```

```powershell
# Windows
Get-FileHash .\qiongli-2.0.0-beta.2-x86_64-pc-windows-msvc.zip -Algorithm SHA256
```

### 2. 解压后直接运行

使用一个新目录，保留已有安装和研究文件。在 macOS 终端中：

```sh
mkdir qiongli-2.0.0-beta.2-macos-arm64
tar -xzf qiongli-2.0.0-beta.2-aarch64-apple-darwin.tar.gz -C qiongli-2.0.0-beta.2-macos-arm64
cd qiongli-2.0.0-beta.2-macos-arm64
./qiongli --version
./qiongli --help
./qiongli content list
```

在 Linux 终端中：

```sh
mkdir qiongli-2.0.0-beta.2-linux-x64
tar -xzf qiongli-2.0.0-beta.2-x86_64-unknown-linux-gnu.tar.gz -C qiongli-2.0.0-beta.2-linux-x64
cd qiongli-2.0.0-beta.2-linux-x64
./qiongli --version
./qiongli --help
./qiongli content list
```

Windows 用户在下载目录打开 PowerShell：

```powershell
Expand-Archive -Path .\qiongli-2.0.0-beta.2-x86_64-pc-windows-msvc.zip -DestinationPath .\qiongli-2.0.0-beta.2-windows-x64
Set-Location .\qiongli-2.0.0-beta.2-windows-x64
.\qiongli.exe --version
.\qiongli.exe --help
.\qiongli.exe content list
```

版本应显示 `qiongli 2.0.0-beta.2`。独立包只提供 `qiongli` 可执行文件；`ql` 别名由 npm / PyPI / Cargo 安装提供。

### 3. 可选：加入 PATH

你可以一直使用可执行文件的绝对路径。若希望在任意目录输入 `qiongli`，将解压目录加入用户 PATH：
macOS / Linux 使用自己的 Shell 配置，Windows 使用用户环境变量设置；完成后打开新终端。

macOS / Linux 用 `type -a qiongli`，PowerShell 用 `Get-Command qiongli -All` 检查实际运行的路径，
再运行 `qiongli --version`，避免旧版本或包管理器的入口优先于新版本。

升级时将新版本解压到另一个目录，验证后再更新 PATH 或 Host 配置中的路径。
保留旧二进制及研究数据以便回退；切换二进制不会撤销数据迁移。

## 检查和迁移已有 CLI

Beta.2 可以列出本机可见的穷理安装，并提供交互式迁移建议。如果旧版在 PATH 中
排在前面，请用新安装程序的完整路径运行下面的命令：

```sh
qiongli install inventory --paths exact
qiongli install migrate --interactive
```

清单会合并同一安装的别名，并区分包元数据版本与当前运行版本。检测范围包括
PATH、常见用户安装目录及已配置的 Cargo、Python、npm 位置；其他环境和 Shell
函数需要单独检查。检测不会执行来源不明的程序。`doctor` 默认隐藏实际路径，
需要时再明确查看完整路径。

向导让你选择准备使用的安装，并逐项查看其他版本的归档或卸载说明。直接按回车
会保留现有设置。选择默认版本只生成建议，不会修改 PATH 或 Host 配置，也不会
删除、移动或归档任何文件。卸载前要确认文件归属：旧包可能与新版共用启动入口。
研究文件、配置和 Plugin 缓存不属于 CLI 清理范围。

在终端中无参数运行新版 `qiongli` 也会打开向导。npm 可以通过
`npm install -g qiongli@next --foreground-scripts` 在安装时显示向导，但要求输入和
输出都连接终端。禁用安装脚本仍可正常使用 CLI。pip 和 Cargo 用户在安装完成后
运行向导。脚本、帮助与版本查询、MCP 启动不会弹出交互提示。

包管理器安装应保留在原位置，并记录版本及原环境以便重装；确认是独立发布包后，
才适合另行复制完整备份并校验。归档副本本身不会停用旧命令。


## 接入 MCP 与 Plugin

在 Host 的 MCP 配置中，将 command 设为 `qiongli` 可执行文件的**绝对路径**，参数设为：

```text
mcp serve --profile full --transport stdio
```

需要 Lite MCP 时使用 `--profile lite`。Full 提供项目、Graph 和交接等工具，Lite 提供较小的文献工具集。
模型与凭据由 Host 管理；下载 CLI 不会自动注册或激活 Plugin。
接入后应能看到真实工具，并成功执行一次只读调用。

Alpha.8 也支持将当前可执行文件、Full MCP 与研究资源导出为用户批准的本地 Plugin 来源，
再通过 Host 的插件机制注册；详见[本地 Plugin 导出步骤](../../guide/cli-2x.md#export-a-local-plugin-source)。
现有项目写入仍需要对应的预览、批准和修订检查，安装不改变这些要求。

## npm / pip 安装 {#package-managers}

如果更习惯包管理器，可任选一个入口：

```sh
npm install --global qiongli@next
```

或者在 Python 虚拟环境中运行：

```sh
python -m pip install --pre qiongli==2.0.0b2
```

npm 需要 Node 18+，PyPI 需要 Python 3.9+；两者都提供 `qiongli` 和 `ql`。
安装后分别用 `--version` 核对版本。通过原包管理器升级包管理器安装的版本。

Cargo 从源码构建同一个 CLI，需要 Rust 1.97+ 和本机链接器。

```sh
cargo install qiongli --version 2.0.0-beta.2 --locked
```

安装后可使用 `qiongli` 和 `ql`。Cargo 没有 `next` 渠道，预发布版需指定完整版本号。
如果希望解压后立即使用，请选择上方的独立二进制包。

更多命令边界见[英文 CLI 指南](../../guide/cli-2x.md#which-surface-owns-which-command)。
旧版 `qiongli setup`、`check`、`project init` 等命令属于 1.x，不能直接套用到原生 2.x。
