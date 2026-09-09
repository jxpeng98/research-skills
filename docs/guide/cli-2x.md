# Qiongli 2 CLI: installation and command boundaries

Qiongli 2 is CLI-first. No Qiongli App window is required. npm and PyPI
distribute the same native executable for a given version and target.

## Install and identify the version

Choose one package manager for the command on your PATH:

```sh
npm install --global qiongli@next
```

Or install in a Python virtual environment:

```sh
python -m pip install --pre qiongli==2.0.0a7
```

Both expose `qiongli` and `ql`. Check both with `--version` before comparing
behavior. Alpha.7 targets macOS ARM64, Windows x64, and Linux x64/glibc 2.35+.
npm needs Node 18+; PyPI needs Python 3.9+. Standalone archives are available in
the [GitHub Release](https://github.com/jxpeng98/qiongli/releases/tag/v2.0.0-alpha.7).

Cargo support is being added for the next qualified release. It builds the
same CLI from source, requires Rust 1.97+ and the target's native linker, and
will expose both command names. Cargo uses the exact SemVer prerelease version,
such as `2.0.0-alpha.8`, with `cargo install qiongli --version VERSION --locked`.
There is no Cargo `next` channel. Alpha.7 has not been published to crates.io;
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
| Register or repair a managed Plugin | `qiongli app plan integrations-install --target codex` (or `claude`) | Alpha.7 registry binaries lack the packaged-product authority required by this owner; `source-build-read-only` is a blocker, not success |
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

The next integration increment will make registry-installed CLI to Plugin
registration a complete approved lifecycle. It must preserve preview, approval,
CAS, ownership and exact removal; bypassing packaged-product checks is not an
installation fix.
