# Install Qiongli

> Native 2.x users: follow the [CLI installation and command guide](cli-2x.md).
> The npm/Python differences and installer commands below describe legacy 1.x.

> **Product-line notice:** This page documents the npm, Python, marketplace, and bootstrap/shell
> installers for Qiongli 1.x. Native Qiongli 2.x Community Alpha users must follow the single
> [2.x Alpha installation authority](../alpha/install-2x.md); a 1.x install cannot satisfy 2.x
> readiness.

Qiongli has several installation surfaces because users need different levels of runtime control. Start with the smallest surface that gives you the workflow you need.

## Latest Stable Downloads

Current stable release: [v1.17.0](https://github.com/jxpeng98/qiongli/releases/tag/v1.17.0). These direct links cover the common install paths; use the download guide for subject-specific Desktop ZIPs and maintainer artifacts.

| Need | Link or command |
|---|---|
| npm CLI | [`qiongli@1.17.0`](https://www.npmjs.com/package/qiongli/v/1.17.0): `npm install -g qiongli@latest` |
| PyPI CLI | [`qiongli 1.17.0`](https://pypi.org/project/qiongli/1.17.0/): `pipx install qiongli` |
| Claude Desktop recommended plugin | [`qiongli-claude-desktop-plugin-v1.17.0.zip`](https://github.com/jxpeng98/qiongli/releases/download/v1.17.0/qiongli-claude-desktop-plugin-v1.17.0.zip) |
| Claude Desktop/Web fallback skill ZIP | [`qiongli-claude-desktop-skill-core-v1.17.0.zip`](https://github.com/jxpeng98/qiongli/releases/download/v1.17.0/qiongli-claude-desktop-skill-core-v1.17.0.zip) |
| Claude Desktop literature MCPB | [`qiongli-literature-provider-0.1.5.mcpb`](https://github.com/jxpeng98/qiongli/releases/download/v1.17.0/qiongli-literature-provider-0.1.5.mcpb) |
| Zotero Desktop companion | [`qiongli-zotero-companion-0.2.2.xpi`](https://github.com/jxpeng98/qiongli/releases/download/v1.17.0/qiongli-zotero-companion-0.2.2.xpi) |
| All release assets | [Download guide](https://github.com/jxpeng98/qiongli/releases/download/v1.17.0/qiongli-downloads-v1.17.0.md) and [GitHub Release](https://github.com/jxpeng98/qiongli/releases/tag/v1.17.0) |

## Install Entry Comparison

| Entry | Positioning | Includes | Can do | Boundary | Python required |
|---|---|---|---|---|---|
| Marketplace plugin / extension | Client-native, lowest setup | Client plugin plus `qiongli-workflow`; Codex and Claude Code include the bundled Rust Lite literature MCP | Skill workflows, prompts, subject packages, and bundled literature-provider tools inside one client | Does not expose the full Python orchestrator or package self-update | No for skill use or bundled literature MCP |
| Claude Desktop Skill ZIP | Desktop/Web skill-only path | Personal `qiongli` Skill upload, usually paired with optional literature MCPB | Claude Desktop/Web workflows without a terminal or code environment | Skill ZIP stores no secrets and does not execute provider or orchestrator calls | No |
| Claude Desktop Literature MCPB | Desktop local provider path | `qiongli-literature-provider.mcpb` with bundled Rust Lite MCP executable | Provider config/status, local literature search, evidence export, Zotero import files | Provider-only; does not install Qiongli workflows or the Python orchestrator | No |
| npm / npx | Python-free asset manager | npm CLI, pre-materialized skills by default, optional plugin-lite assets with `--surface plugin|both`, Node project commands | Scripted installs, CI/dotfiles, current-package asset refresh, `project init/status/set-subject`, plugin-lite where bundled | Does not update npm/Python packages and does not run `doctor`, `mcp serve`, provider setup, or task orchestration | No |
| pipx / pip full runtime | Python CLI and managed full local runtime | Python CLI, setup wizard, full plugin surface, unified MCP server, provider setup, doctor, task/orchestrator commands | Full local validation, provider configuration, MCP/orchestrator tools, package self-update, release-archive refresh | Requires local Python and relevant client/model CLIs before real agent execution | Yes, Python 3.12+ |
| Bootstrap `partial` | Release-script skills install | Global skills and workflow discovery across clients | No-package-manager install of portable workflow assets | No full runtime validation or orchestration | No |
| Bootstrap `full` | Release-script full runtime install | `partial` plus shell CLI, MCP registration part, and `doctor` support | Full runtime setup from release scripts | Does not install Python; the machine must already have Python 3.12+ | Yes, Python 3.12+ |

The user-visible skill name is `qiongli`. The installed directory is still `qiongli-workflow` for compatibility with existing clients and release artifacts. For normal full-runtime CLI/local plugin use, install the stable `core/complete` runtime once and store everyday subject behavior in `.qiongli/guidance_manifest.yaml`. npm installs the skills surface by default; use `--surface plugin` or `--surface both` only when you deliberately want the bundled plugin-lite output where supported. Specialized CLI/npm installs default to `coverage=complete`, but they are advanced compatibility paths for Desktop ZIPs, focused packages, release payloads, and install-surface testing.

In the full Python runtime, `qiongli install` defaults to `--profile full --surface plugin` from v1.9.0 onward. Codex, Claude Code, and Antigravity receive CLI-managed local plugin bundles backed by `qiongli mcp serve --transport stdio`; Antigravity bundles that server in the plugin root `mcp_config.json`, and Hermes receives managed full MCP client config. npm/npx remains Python-free and defaults to `--surface skills`.

npm/npx is the Python-free asset manager surface. Install the full runtime with `pipx install qiongli` when you need full runtime commands such as `doctor`, `mcp serve`, `provider setup`, `task-plan`, `task-run`, or `customize`.

## Native Plugin And Extension

Use this when you only need Qiongli inside one supported client.

Codex installs through the shared [Skillsplace](https://github.com/jxpeng98/skillsplace) marketplace:

```bash
codex plugin marketplace add jxpeng98/skillsplace --ref main
codex plugin marketplace list
```

Then install or enable `qiongli` from the Codex plugin UI for the default core package. Subject entries such as `qiongli-economics`, `qiongli-accounting`, `qiongli-business`, `qiongli-finance`, `qiongli-political-economy`, `qiongli-geoeconomics`, and `qiongli-economics-accounting` install the corresponding `subject/complete` package from the same marketplace.

The Codex plugin bundles its MCP registration through `.mcp.json` and includes the Rust Lite literature-provider executable at `bin/qiongli-literature-provider`. Codex users do not need to hand-write a separate MCP config or install Node, Python, or the `qiongli` CLI for those bundled literature tools. Provider keys remain outside the plugin and can be configured with the bundled local setup tool `qiongli_configure_provider`, with `qiongli_save_provider_config`, or with `qiongli mcp configure` / `qiongli provider setup` when the CLI is installed. For full local Qiongli, use the CLI-generated local plugin: `qiongli install --profile full --target codex --surface plugin`. That local plugin keeps the Codex-native plugin container, but its `.mcp.json` launches the full Python-backed `qiongli mcp serve --transport stdio` server.

After installing or upgrading the plugin, start a new Codex thread or restart the local Codex client so the plugin-bundled MCP server is loaded. In Codex CLI, `codex mcp list` should show `qiongli` or `qiongli-next` with command `./bin/qiongli-literature-provider` and the plugin cache path as `cwd`. In the Codex App, use Settings > Integrations & MCP or the thread MCP status view to confirm the server is enabled before testing provider tools.

Codex plugin installs use plugin-bundled MCP by default. The installer writes `.mcp.json` inside the Qiongli plugin and the plugin manifest points to it; it does not write `~/.codex/config.toml` on the plugin path. Use `qiongli install --target codex --parts mcp` only when you need the standalone MCP fallback.

Codex currently treats plugin-bundled MCP servers as plugin assets: the settings UI can enable the server and manage tool policy, but it is not the right place to add provider keys for this bundled server. Claude Desktop MCPB, Claude Code, Cursor-style clients, and other local stdio MCP clients should use the same Qiongli provider setup contract. Configure keys through the Qiongli provider config instead:

1. Ask Codex to run `qiongli_config_status` and note the redacted status plus `config_path`.
2. Ask the client to run `qiongli_configure_provider`, then open the returned `127.0.0.1` URL.
3. Enter the OpenAlex API key, optional OpenAlex email, and Semantic Scholar API key in the local browser page. The page writes the shared provider config without putting secrets in the conversation.
4. Re-run `qiongli_config_status` or `qiongli_literature_status`; credentials should be reported only as `configured` or `missing`, never printed in full.

Do not put provider keys in `.mcp.json`, `.codex-plugin/plugin.json`, release ZIPs, or research artifacts. The same shared provider config is read by the Codex plugin MCP, the Claude Code plugin MCP, the Claude Desktop MCPB, and the full CLI MCP server.

Claude Code uses the same Skillsplace catalog:

```bash
claude plugin marketplace add jxpeng98/skillsplace@main
claude plugin install qiongli@skillsplace
# Subject-specialized install:
claude plugin install qiongli-economics@skillsplace
```

Inside an interactive Claude Code session, use:

```text
/plugin marketplace add jxpeng98/skillsplace@main
/plugin install qiongli@skillsplace
/plugin install qiongli-economics@skillsplace
```

The Claude Code plugin also bundles the Rust Lite literature-provider MCP runtime under `bin/qiongli-literature-provider`, using the same provider, search, search-plan, evidence-export, Zotero import-file, and status tools as the Codex plugin. This bundled runtime covers literature-provider tools and literature-provider MCP without installing Node, Python, or the `qiongli` CLI. Python-backed orchestration tools, including `qiongli_task_run` and `qiongli_orchestrator_doctor`, require the full runtime: `pipx install qiongli`. Then run `qiongli install --profile full --target claude --surface plugin` to generate a local Claude Code plugin that launches the unified `qiongli mcp serve --transport stdio` server. Use `--target antigravity` to generate the Antigravity plugin with its root `mcp_config.json`, `--target hermes` for Hermes MCP config, or `--target all --surface plugin` when Codex/Claude Code/Antigravity should use local plugins and Hermes should receive managed full MCP config.

After installing or upgrading a Claude Code plugin, run `/reload-plugins` or start a new Claude Code session. Then open `/mcp`; the plugin server should appear as a plugin-provided MCP server and report a non-zero tool count. Plugin MCP tool names are scoped by plugin and server name, so the exact callable names include the plugin namespace rather than only the bare `qiongli_literature_search` tool name.

Claude Desktop and Claude.ai do not install third-party Claude Code plugin marketplaces. For Desktop, prefer the direct plugin ZIP when that install path is available:

1. Download `qiongli-claude-desktop-plugin-<tag>.zip` from the GitHub Release assets.
2. Install it through Claude Desktop's direct plugin or plugin upload flow.
3. Restart Claude Desktop and confirm the user-visible entry is `qiongli`.

The direct plugin is the recommended Desktop package because it keeps the `qiongli` skill, generated workflow command wrappers including `/qiongli`, and the bundled lightweight literature MCP runtime in one plugin-shaped asset. It does not provide the Python-backed orchestrator; install the full runtime separately when you need `qiongli_orchestrator_route`, `qiongli_task_plan`, or `qiongli_task_run`.

If direct plugin install is unavailable, use the fallback skill ZIP path. Download exactly one focused skill ZIP, such as `qiongli-claude-desktop-skill-core-<tag>.zip`, `qiongli-claude-desktop-skill-economics-<tag>.zip`, `qiongli-claude-desktop-skill-business-<tag>.zip`, `qiongli-claude-desktop-skill-finance-<tag>.zip`, `qiongli-claude-desktop-skill-political-economy-<tag>.zip`, `qiongli-claude-desktop-skill-geoeconomics-<tag>.zip`, or `qiongli-claude-desktop-skill-economics-accounting-<tag>.zip`. In Claude Desktop, drag the ZIP into the Skills upload/install flow, or open `Customize > Skills`, click `+`, choose `Create skill`, then `Upload a skill`. Claude.ai uses the same `Customize > Skills` upload flow. Public Desktop ZIP subjects in this phase are `core`, `economics`, `business`, `finance`, `political-economy`, `geoeconomics`, and `economics-accounting`; there is no standalone accounting Desktop ZIP yet.

The fallback skill ZIP uses `coverage=focused` to stay under the current 180-file upload budget. It is a subject-specialized Desktop/Web package, not a reduced-quality cut. It preserves executable workflows, prompts, templates, standards, selected profiles, `skills-summary.md`, and `skills-core.md`; specialized ZIPs also include selected effective skill markdown generated with layered overlays. This Desktop skill ZIP is skill-only: it contains workflows/prompts/templates, stores no secrets, and does not execute provider calls. Detailed canonical source remains available through CLI/npm `coverage=complete`, the Codex / Claude Code plugin packages, and the source repository.

The Qiongli Literature Provider `.mcpb` (`qiongli-literature-provider.mcpb`) is a separate Claude Desktop local provider asset. It runs Desktop literature search through OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv, exposes Desktop configuration fields for provider credentials, and uses Claude Desktop sensitive-field handling instead of putting keys in the skill ZIP. arXiv is enabled without credentials. It supports the Rust Lite provider subset: provider status/config, bounded basic search, normalization and deduplication, evidence export, Zotero status/import-file generation, and preview-only route/task planning. Domain deep search, citation expansion, Zotero library search/writes, and agent execution require Full. It contains its own Rust Lite MCP executable, so Desktop users do not need Node, Python, the `qiongli` CLI, or an npm install to use this MCPB. A beta marked `current-host-only` must be installed only on the target triple recorded in its identity file; such a beta does not advance generic marketplace dist refs. CLI, Codex, Claude Code, Antigravity, and Hermes users can still run `qiongli provider setup`, then verify `provider_connected` or `strategy_only` with `qiongli provider doctor`. Desktop users need the `qiongli-literature-provider` MCPB or platform-native search before claiming `provider_connected`; if no MCPB or platform-native search is available, record the run as `strategy_only` and treat platform search or user-supplied corpus as the evidence source. Finance/economics data APIs such as FRED and SEC EDGAR belong in a separate data MCP surface, documented in [Finance/Economics Data MCP Boundary](../advanced/finance-econ-data-mcp.md).

Manual Desktop fallback installs can combine two local assets:

- Skill ZIP: enables Qiongli agent instructions, workflows, subject overlays, and skill guidance inside Claude Desktop/Web.
- Literature MCPB: enables local literature MCP calls and provider configuration.

Those two assets do not by themselves expose the full Python-backed orchestrator. If a local client should call `qiongli_orchestrator_route`, `qiongli_task_plan`, `qiongli_task_run`, or `qiongli_orchestrator_doctor` as MCP tools, install the full runtime first:

```bash
pipx install qiongli
qiongli install --profile full --target codex --surface plugin
qiongli install --profile full --target claude --surface plugin
qiongli install --profile full --target all --surface plugin
qiongli mcp serve --transport stdio
qiongli mcp doctor --json
```

In `--surface plugin` mode, Codex, Claude Code, and Antigravity get local plugin bundles. Antigravity stores its MCP server definition in the plugin root `mcp_config.json`; `--parts mcp --target antigravity` remains available for the older client-level config path at `~/.gemini/config/mcp_config.json`. Hermes receives managed client-level MCP config when included by `--target all`.

Use `qiongli_orchestrator_route` from Codex, Claude Code, Antigravity, or another local MCP client when deciding whether a request should move from skill-only execution to the full orchestrator. `qiongli_task_run` defaults to preview mode. It launches local runtime agents only when the MCP caller explicitly sends JSON boolean `run_agents: true` and the local runtime passes `doctor`.

## Use After Install

Restart the target client after installing or upgrading. Then use the entrypoint that client exposes:

| Client | Discovery | Invocation |
|---|---|---|
| Codex | `/skills` should list `qiongli` and plugin workflow wrappers such as `qiongli-lit-review` | `$qiongli <research task>` or `$qiongli-lit-review <topic>` |
| Claude Desktop / Claude.ai | `Customize > Skills` should show `qiongli`, or the direct plugin entry should be enabled | Ask naturally for a research task; use `/qiongli` when workflow commands are visible |
| Claude Code | Plugin UI, `/plugin`, or global command discovery | `/qiongli`, `/paper`, `/lit-review`, `/paper-write`, `/code-build` |
| Shell | `qiongli check` | `qiongli doctor`, `qiongli upgrade`, `python3 -m bridges.orchestrator ...` |

Codex does not expose custom `/qiongli` or `/lit-review` slash commands. Use `/skills` to confirm the main skill and generated wrapper skills exist, then invoke `$qiongli` or a wrapper such as `$qiongli-lit-review`. Each wrapper routes to the same canonical workflow file used by Claude Code slash commands.

Use `qiongli check --json` when the client UI and filesystem state disagree. It reports the active install surface for each client: `plugin`, `mcp`, `legacy_skill`, or `none`. Use `qiongli doctor --cwd .` for runtime/orchestrator health; it also prints a non-fatal `Client Integration` summary.

If Codex shows `qiongli` under Personal plugins but opening it says `Plugin detail unavailable`, Codex found the marketplace entry but could not read the local plugin payload. The usual causes are invalid `.codex-plugin/plugin.json`, invalid YAML frontmatter in `skills/qiongli-workflow/SKILL.md`, or a missing plugin path. Refresh the CLI-managed local plugin with:

```bash
qiongli install --target codex --surface plugin --overwrite
```

## Bootstrap Partial

Use `partial` for the cross-client workflow package without Python:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile partial --project-dir "$PWD" --target all
```

Windows PowerShell 7+:

```powershell
winget install --id Microsoft.PowerShell --source winget
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.ps1 -OutFile .\bootstrap_qiongli.ps1
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile partial -ProjectDir "$PWD" -Target all
```

`partial` installs workflow assets and discovery links. It does not require Python and does not run full runtime validation.

## Bootstrap Full

Use `full` when you need local validation or orchestrated task execution:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile full --project-dir "$PWD" --target all
```

Windows PowerShell 7+:

```powershell
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile full -ProjectDir "$PWD" -Target all
```

`full` requires Python 3.12+ to already be on `PATH`. It does not install Python or `mise`.

After `full`, check a workspace:

```bash
qiongli doctor --cwd .
python3 -m bridges.orchestrator doctor --cwd .
qiongli mcp doctor --json
```

## npm / npx

Use npm when you want a Node-distributed installer with the workflow payload bundled:

```bash
npm install -g qiongli
qiongli install --target all --surface skills
qiongli project init --project-dir .
qiongli project set-subject finance --project-dir .
qiongli project status --project-dir .
```

For scripted compatibility, `qiongli install --target all --surface skills --project-dir "$PWD"` is equivalent for the default install surface; ordinary subject behavior still belongs to `qiongli project ...`. Plugin-lite output is opt-in with `--surface plugin` or `--surface both` where bundled and supported.

For normal npm asset/project use, keep the installed npm package stable and store subject behavior in `.qiongli/guidance_manifest.yaml`. If no manifest exists, Qiongli uses implicit `active_subject: auto`, starts from core guidance, infers temporary subject or method lenses from the task, and writes auditable proposals before changing project-local state.

Use npm for Python-free asset refresh and project operations:

```bash
qiongli update
qiongli refresh
qiongli upgrade --target all
qiongli project status --project-dir .
```

`qiongli update`, `qiongli refresh`, and `qiongli upgrade` reapply bundled assets from the currently installed npm package. `upgrade` is an overwrite refresh alias. They do not update the npm package or the full Python CLI.

For one-off runs:

```bash
npx qiongli@latest install --target all --surface skills
npx qiongli@latest project status --project-dir .
npx qiongli@latest check --json
```

Prerelease testing remains available through the `next` dist-tag:

```bash
npx qiongli@next install --target all --surface skills
```

Advanced compatibility, Desktop ZIP, focused package, release payload, and install-surface testing examples:

Subject packages and runtime activation are separate. A marketplace, npm-lite, or Desktop ZIP package may include subject content for compatibility, but the adaptive runtime only suggests `runtime_enabled` subjects whose runtime-enabled gate passes. Candidate subjects can still be installed or inspected; they are not automatically activated.

```bash
qiongli install --subject core --target all --project-dir "$PWD"
qiongli install --subject core --target all --surface skills --project-dir "$PWD"
qiongli install --subject economics --target all --surface skills --project-dir "$PWD"
qiongli install --subject accounting --target all --surface skills --project-dir "$PWD"
qiongli install --subject economics-accounting --target all --surface skills --project-dir "$PWD"
qiongli install --subject economics --coverage focused --target all --surface skills --project-dir "$PWD"
npx qiongli@latest install --subject economics --target all --surface skills --project-dir "$PWD"
npx qiongli@next install --subject economics --target all --surface skills --project-dir "$PWD"
```

Remove CLI-installed assets before switching fully to marketplace plugins:

```bash
qiongli remove --target all --dry-run
qiongli remove --target all
```

`qiongli remove` only removes CLI-installed global workflow assets and discovery links by default. Native marketplace plugins remain managed by the client/plugin manager that installed them.

For npm plugin-lite roots, remove only npm-managed plugin assets:

```bash
qiongli remove --surface plugin --target claude
```

npm marks owned plugin-lite roots with `.qiongli-npm-lite.json` (or a sidecar marker for linked installs). Full-runtime local plugin roots use `.qiongli-managed.json` and Codex marketplace entries marked with `metadata.managedBy = "qiongli-cli"`. `remove` preserves unmanaged plugin directories and marketplace lite plugins.

## Recommended CLI Setup Wizard

After installing the full runtime with pipx, pip, or the bootstrap `full` profile, run the interactive setup wizard before hand-writing install flags:

```bash
pipx install qiongli
qiongli setup
qiongli setup --dry-run
qiongli setup --project-dir "$PWD" --no-doctor
qiongli setup --provider-mode prompt --no-browser
```

The wizard guides CLI, Codex, Claude Code, and Antigravity users through:

- setup path: `install` for first-time bundled asset installation, or `upgrade` for an upstream refresh
- runtime surface: CLI, Codex, Claude Code, Antigravity, or multi-platform
- subject choice
- coverage choice: `complete` or `focused`
- install mode: `--mode copy` for normal use, or `--mode link` for local checkout development
- install scope: `all`, `globals`, `project`, or `cli`
- shell CLI directory when CLI wrappers are enabled
- overwrite policy: `--overwrite` for replacing managed installs, or `--no-overwrite` on upgrade when you want to preserve existing managed files
- upgrade source: latest stable, latest beta, an explicit `--ref` tag, an explicit `--ref-type branch`, and optional `--repo`
- optional literature provider setup. By default this opens one local browser page with fields and key-access guidance for OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv. Use `--no-browser` to print the URL only, `--provider-mode prompt` for terminal-only entry, or `--provider-mode skip` to bypass provider setup.
- doctor verification, unless `--no-doctor` is set

Every prompt prints a short `Tip:` comment explaining what the choice changes, so new users can follow the install or upgrade path without knowing the full CLI flag set first.

Provider keys saved through setup use the same provider config as `qiongli provider setup` and `qiongli provider doctor`. The local page includes links and short steps for getting each supported credential; arXiv is marked as available without an API key. Secrets are stored outside generated research artifacts. The provider step configures credentials and runs doctor/capability checks; it does not guarantee external search results.

On npm/npx, `qiongli setup` is client-asset setup, not the full interactive wizard. It stays on the Python-free asset manager path for client assets. For full runtime commands, install the full runtime first: `pipx install qiongli`. Use explicit `qiongli install ...` commands when you want the Node-only asset installer.

## pipx / pip

Use pipx when you specifically want the Python-distributed updater CLI:

```bash
pipx install qiongli
qiongli setup
qiongli install --target auto
qiongli project init --project-dir .
qiongli project set-subject finance --project-dir .
qiongli project status --project-dir .
```

`qiongli setup` can guide installation interactively. Scriptable installs can use `qiongli install --target auto` to refresh only detected client CLIs, or `--target all` to write every supported platform path. Project subject behavior changes with `qiongli project set-subject`.

Upgrade the full runtime with:

```bash
qiongli update
qiongli update --dry-run
qiongli update --yes
```

In an interactive terminal, bare `qiongli update` opens a wizard for stable/beta channel, refresh target, surface, profile, refresh/check, and confirmation behavior. Manual package-manager updates are still valid: run `pipx upgrade qiongli` or `python -m pip install --upgrade qiongli`, then refresh client surfaces with `qiongli install --target auto --surface plugin --profile full --overwrite`. `qiongli upgrade` remains the release-archive refresh path when you intentionally want to download an upstream GitHub release.

`--subject` defaults to `core`, and `--coverage` defaults to `complete`, but subject install flags are advanced compatibility controls. Use them for deliberate slim installs, Desktop/Web-equivalent packages, release payload validation, and install-surface testing. Current official subjects are `core`, `economics`, `accounting`, `business`, `finance`, `political-economy`, `geoeconomics`, and the named composite `economics-accounting`; `political-economy` and `geoeconomics` are independent subject choices, not a composite. Official composite subjects are not arbitrary comma-separated stacking. Ordinary project subject behavior changes with `qiongli project set-subject`; installed subject or coverage changes are only intentional specialized package refreshes. `qiongli check --json` reports the active installed subject and coverage per target; legacy installs without a `SUBJECT_MANIFEST.json` or `SUBJECT` file are treated as `core` / `complete`.

Create a custom scaffold before materializing local overlays:

```bash
qiongli customize --subject economics --name my-econ-lab --out ./qiongli-custom/econ-lab
```

Local custom overlays are supported by the source materializer:

```bash
python3 scripts/materialize_subject_package.py \
  --subject economics \
  --coverage complete \
  --source . \
  --custom-dir /path/to/custom-qiongli \
  --out /tmp/qiongli-workflow
```

Use this when you need local overlays, profiles, registry entries, or custom skill markdown. Custom overlays affect generated output only and do not rewrite canonical source files. `qiongli customize` plus `--custom-dir` materialization is for the Python/source checkout workflow; npm runtime installs pre-generated payloads and do not accept `--custom-dir` in this phase.

## What Gets Installed

Depending on the surface, Qiongli may install:

- `qiongli-workflow` skill assets under client home directories, visible to users as `qiongli`
- workflow command discovery links such as `/paper`, `/lit-review`, `/paper-write`, and `/code-build` in clients that support that discovery model
- shell commands `qiongli`, `ql`, and compatibility aliases `research-skills`, `rsk`, `rsw`
- optional project integration files when you explicitly run `qiongli init --project-dir .`

Project-local files are not written by default. The global workflow package can be used from any research workspace.

For invocation details, see [Using Agent Skills](/guide/using-agent-skills).

## Keep Versions Aligned

If you use multiple surfaces, keep plugin, global skill assets, npm payload, and Python CLI aligned. For the Python full runtime:

```bash
qiongli check
qiongli update
qiongli update --dry-run
qiongli update --yes
```

On npm/npx, omit the full-runtime flags and refresh current-package assets with `qiongli update`, `qiongli refresh`, or `qiongli upgrade --target all`.

If you move fully to native plugins and no longer need legacy global skill directories or slash discovery, inspect cleanup first:

```bash
qiongli remove --target all --dry-run
```
