# Qiongli 2 CLI: installation and command boundaries

Qiongli 2 is CLI-first. GitHub binary archives, npm and PyPI distribute the same
native executable for a given version and target. No Qiongli App is required.

## Standalone binary download

**Recommended for direct use: download, extract, and run. No Python, Node.js,
Rust or package-manager setup is needed.** You can run from the extracted folder;
adding it to PATH is optional.

Download a platform archive from [Release v2.0.0-alpha.8](https://github.com/jxpeng98/qiongli/releases/tag/v2.0.0-alpha.8).
It contains `qiongli` (Windows: `qiongli.exe`), `README.md` and `LICENSE`.
The executable embeds the research Skills, templates and Lite/Full MCP resources;
you do not need a separate resource directory or a checkout of this repository.
Configure models, Host applications and online literature services separately.

| Your platform | Complete CLI archive |
|---|---|
| macOS Apple Silicon / ARM64 | [qiongli-2.0.0-alpha.8-aarch64-apple-darwin.tar.gz](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-alpha.8/qiongli-2.0.0-alpha.8-aarch64-apple-darwin.tar.gz) |
| Windows x64 | [qiongli-2.0.0-alpha.8-x86_64-pc-windows-msvc.zip](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-alpha.8/qiongli-2.0.0-alpha.8-x86_64-pc-windows-msvc.zip) |
| Linux x64 / glibc 2.35+ | [qiongli-2.0.0-alpha.8-x86_64-unknown-linux-gnu.tar.gz](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-alpha.8/qiongli-2.0.0-alpha.8-x86_64-unknown-linux-gnu.tar.gz) |

**Windows alpha.8 exception:** the published executable imports `VCRUNTIME140.dll`
and needs the Visual C++ runtime. A Windows release without that requirement is
still pending; these links point to the existing alpha.8 build. Linux uses its
system libraries, including glibc 2.35+.

Choose these files under **Assets**. GitHub's **Source code** archives require a
build; the `.tgz` npm package, `.whl` Python packages and `qiongli-next-…-plugin-…`
archives serve different installation paths. Intel macOS, Linux ARM and native
Windows ARM builds are not part of this release.

### 1. Verify the download

Download [SHA256SUMS](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-alpha.8/SHA256SUMS)
from the same release. In the directory containing your download, run the command
for your platform and compare its hash with the line for that exact filename in
`SHA256SUMS`. Continue only if they match.

```sh
# macOS
shasum -a 256 qiongli-2.0.0-alpha.8-aarch64-apple-darwin.tar.gz
# Linux
sha256sum qiongli-2.0.0-alpha.8-x86_64-unknown-linux-gnu.tar.gz
```

```powershell
# Windows
Get-FileHash .\qiongli-2.0.0-alpha.8-x86_64-pc-windows-msvc.zip -Algorithm SHA256
```

### 2. Extract and run

Extract into a new directory, keeping existing installations and research files
intact. On macOS:

```sh
mkdir qiongli-2.0.0-alpha.8-macos-arm64
tar -xzf qiongli-2.0.0-alpha.8-aarch64-apple-darwin.tar.gz -C qiongli-2.0.0-alpha.8-macos-arm64
cd qiongli-2.0.0-alpha.8-macos-arm64
./qiongli --version
./qiongli --help
./qiongli content list
```

On Linux:

```sh
mkdir qiongli-2.0.0-alpha.8-linux-x64
tar -xzf qiongli-2.0.0-alpha.8-x86_64-unknown-linux-gnu.tar.gz -C qiongli-2.0.0-alpha.8-linux-x64
cd qiongli-2.0.0-alpha.8-linux-x64
./qiongli --version
./qiongli --help
./qiongli content list
```

On Windows, open PowerShell in your download directory:

```powershell
Expand-Archive -Path .\qiongli-2.0.0-alpha.8-x86_64-pc-windows-msvc.zip -DestinationPath .\qiongli-2.0.0-alpha.8-windows-x64
Set-Location .\qiongli-2.0.0-alpha.8-windows-x64
.\qiongli.exe --version
.\qiongli.exe --help
.\qiongli.exe content list
```

The version output should be `qiongli 2.0.0-alpha.8`. The archive supplies the
`qiongli` executable; npm/PyPI additionally provide the `ql` command alias.

### 3. Optional PATH and Host setup

You can keep using the absolute executable path. To run `qiongli` from any
directory, add the extracted directory to your user PATH using your shell profile
or Windows user environment settings, then open a new terminal. Use
`type -a qiongli` on macOS/Linux or `Get-Command qiongli -All` in PowerShell to
check which installation will run. Verify `qiongli --version` again.

[Connect a Host](#connect-a-host) using the absolute executable path for Lite or
Full MCP, or [export a local Plugin source](#export-a-local-plugin-source).
Downloading and running the CLI does not automatically activate a Host Plugin.

To upgrade, extract the new version into a separate directory, test it, then
update PATH or the Host command. Keep the previous binary and research data for
rollback; switching binaries does not reverse data migrations.

## Package managers

Alternatively, choose one package manager for the command on your PATH:

```sh
npm install --global qiongli@next
```

Or install in a Python virtual environment:

```sh
python -m pip install --pre qiongli==2.0.0a8
```

Both expose `qiongli` and `ql`. Check both with `--version` before comparing
behavior. Alpha.8 targets macOS ARM64, Windows x64, and Linux x64/glibc 2.35+.
npm needs Node 18+; PyPI needs Python 3.9+.

Cargo support is being added for the next qualified release. It builds the
same CLI from source, requires Rust 1.97+ and the target's native linker, and
will expose both command names. Cargo uses the exact SemVer prerelease version,
such as `2.0.0-alpha.8`, with `cargo install qiongli --version VERSION --locked`.
There is no Cargo `next` channel. Alpha.8 has not been published to crates.io;
do not use its GitHub/npm availability as proof of Cargo availability.

If npm reports a successful install but the command is missing, inspect
`npm prefix -g`. On Unix the command directory is `<prefix>/bin`; on Windows
it is `<prefix>`. Compare it with PATH. A malformed npm prefix is a local
configuration problem; reinstalling into the same prefix does not fix PATH.
Use `type -a qiongli ql` on Unix or `Get-Command qiongli,ql -All` in PowerShell
to detect an older binary shadowing the new installation. Review the npm
configuration before changing a prefix shared by other globally installed tools.

## Which surface owns which command?

| Outcome | Native entry | Boundary |
|---|---|---|
| Inspect installation and resolved paths | `qiongli --version`, `paths`, `doctor`, `install status`, `install inventory` | Inspection is not Host activation |
| Register, inspect, refresh, import or export a project | `qiongli project --help` | Project writes retain revision and approval checks |
| Inspect embedded research content | `qiongli content list` | Does not install a Plugin |
| Inspect global configuration | `qiongli config show`, `config backend status` | The Host owns model configuration and execution |
| Start tool transport | `qiongli mcp serve --profile full --transport stdio` | Full exposes project/graph/handoff tools; Lite exposes the bounded literature subset |
| Plan Skills installation or maintenance | `qiongli app plan skills-reconcile --preset qiongli-managed --profile full` | `app` is the retained command namespace; no GUI is needed; apply requires the returned plan/digest and explicit approval |
| Register or repair a managed Plugin | `qiongli app plan integrations-install --target codex` (or `claude`) | Standalone/registry binaries lack the packaged-product authority required by this owner; use the separate user-approved local source export below |
| Research routing, literature review, writing and critique | Host workflow entries such as `/paper`, `/lit-review`, `/paper-read` | Host instructions, not standalone shell subcommands |
| Inspect/update a managed installation | `qiongli update --help`, `migrate-1x --help` | Managed authority remains required; use the package manager to upgrade a registry-installed CLI |

Use `--help` on the owning command group for exact arguments. Legacy commands
such as `qiongli setup`, `check`, `provider setup`, `provider doctor`,
`install --target ... --parts mcp`, and `project init` are not the native 2.x
contract. They remain documented in the explicitly labeled 1.x reference.

## Connect a Host

A CLI package installation does not register or upgrade a Plugin. A bundled
Plugin uses its own executable; check the Plugin version independently of PATH.
The native Plugin source is projected from `content/`; installed caches are
derived outputs and must not be edited as source.

For an explicit standalone MCP connection, configure the Host to launch the
absolute installed `qiongli` path with these arguments:

```text
mcp serve --profile full --transport stdio
```

Use a command path and runtime PATH visible to the actual Host process. A GUI
Host may not inherit your terminal's PATH; npm wrappers additionally need Node.
The connection is ready only when real Qiongli tools are visible and a read-only
call succeeds. Configure literature providers through the visible
`qiongli_config_status` / `qiongli_configure_provider` tools, then inspect
`qiongli_literature_status`. Models and credentials remain owned by the Host.

## Export a local Plugin source

Alpha.8 supports a user-approved local Plugin source without a signed App
package. It bundles the
running native executable, Full MCP and canonical workflow content, so the Plugin
does not depend on npm's Node/PATH wrapper after export.

Choose a secure directory outside Host configuration/cache and `.qiongli` paths.
For Codex, create an export parent and inspect the plan:

```sh
mkdir -p "$HOME/qiongli-plugins/codex"
qiongli app plan plugin-source-install --target codex \
  --destination "$HOME/qiongli-plugins/codex/qiongli-next" > "$HOME/plugin-plan.json"
```

Review the JSON, including destination, binary hash, expected receipt and
`plan_digest_sha256`, then apply that exact digest:

```sh
qiongli app apply --plan "$HOME/plugin-plan.json" \
  --expected-plan-digest <plan_digest_sha256> --approve-filesystem-write
qiongli app plugin-source-status --target codex \
  --destination "$HOME/qiongli-plugins/codex/qiongli-next"
```

Use `--target claude` with a separate export parent for Claude Code. The destination
must end in `qiongli-next`; its parent must already exist and reject writes by
other users. These commands never write Host configuration or install into caches.
A local source is user-approved content, not a signed publisher attestation.

To update after upgrading your CLI, preview `plugin-source-update` with the same
target/destination, review and apply its new digest. Only a fully matching receipt
may be replaced. To remove an export, first unregister its Plugin through the Host,
then preview/apply `plugin-source-remove`. Unknown files, changed content, symlinks
and stale receipts refuse; there is no force-delete option.

The exported marketplace is `qiongli-cli-local` and the selector is
`qiongli-next@qiongli-cli-local`. Use the Host's official local marketplace and
Plugin commands to register the export, avoiding duplicate enabled Qiongli Plugins.
For Codex, the isolated compatibility check used:

```sh
codex plugin marketplace add "$HOME/qiongli-plugins/codex/qiongli-next"
codex plugin add qiongli-next@qiongli-cli-local
```

For Claude Code, validate the export with `claude plugin validate <export-path>`
before the Host registration increment. Its actual session activation is still
unqualified.
Source updates require a Host refresh/new session; deleting a source does not
unregister its Host Plugin. `source-current` and `source-ready-host-action-required`
only describe the exported files. Automatic Host registration and real tool
visibility/read/handoff/approved-write checks are the next increment.
