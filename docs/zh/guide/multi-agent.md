# 多 Agent 运行指南

当你要运行 `parallel`、`task-run`、`team-run`，或者任何由 orchestrator 同时协调 Codex、Claude、Gemini 的流程时，先看这一页。

## 这份文档解决什么问题

这一页主要说明：

- orchestrator 如何在多个 runtime 之间路由任务
- Gemini 的 `broker`、`direct`、`auto` 分别代表什么
- 每条路径各自接受什么认证方式
- 只支持 Google 登录订阅时，怎样稳定地自动化使用 Gemini
- 哪些场景适合桌面、本地、CI、纯无头服务器

## Runtime 结构

orchestrator 目前协调三个 runtime agent：

- `codex`
- `claude`
- `gemini`

其中 `codex` 和 `claude` 的路径比较直接：先做 preflight，再直接启动对应 CLI。

`gemini` 则拆成三种 transport：

- `direct`：一次一进程地直接启动 Gemini CLI
- `broker`：把请求发给本地常驻 broker
- `auto`：优先 broker，broker 不可用时再尝试 direct

当前内置 Gemini broker 的默认 backend 已经改成常驻的 `gemini --acp`。

## 为什么 Gemini 必须分成两条路

Gemini 自动化里真正有差异的是这两类认证：

1. `API key` 或 `Vertex`
   适合非交互子进程
2. `Google 登录` 订阅
   只有在 broker 持有一个常驻 Gemini 会话时才稳定

真正要记住的是：

- 终端里出现 `Signed in with Google /auth`，只说明你手工交互用 Gemini 没问题
- 这不等于 orchestrator 的 `direct` 路径也稳定
- 但对 `broker` 路径是成立的，因为 broker 维持的是一个长生命周期的 ACP Gemini 会话，不是每次重新拉起新的子进程

## Gemini Transport 模式

你可以全局设置：

```bash
export RESEARCH_GEMINI_TRANSPORT=auto
```

也可以在 agent profile 里通过 `runtime_options.gemini.transport` 单独覆盖。

支持的值：

- `auto`
- `broker`
- `direct`

### `auto`

默认推荐。

行为是：

1. 先检查 broker 是否已配置且健康
2. 如果健康，Gemini 任务走 broker
3. 如果 broker 不可用，再检查 direct 路径的非交互认证是否可用

这是最稳妥的默认值。

### `broker`

当你明确要让 Gemini 只走常驻 broker 时使用。

行为是：

- 需要配置 `RESEARCH_GEMINI_BROKER_URL`
- 如果 broker 内部已经持有 Gemini 的认证态，orchestrator 端不要求 `GEMINI_API_KEY`
- 这是 `Google 登录订阅` 最正确的模式

### `direct`

只有在你明确想保留一次一进程的 Gemini 子进程调用时才使用。

行为是：

- 即使 broker 在运行，也会被绕过
- 必须有 direct 路径可用的非交互认证
- 不应该依赖缓存的浏览器登录态

## 认证矩阵

| 认证方式 | `broker` | `direct` |
|---|---|---|
| `GEMINI_API_KEY` | 支持 | 支持 |
| Vertex AI 环境变量 | 支持 | 支持 |
| 缓存的 Google 登录态 | 通过常驻 ACP broker 支持 | 不可靠 |

所以规则很简单：

- 只有 Google 登录 => 用 `broker`
- 有 API key 或 Vertex => `auto` 或 `direct` 都行，但仍然推荐 `auto`

## 如何启动常驻 Gemini Broker

### 标准桌面流程

先在桌面会话里启动 broker：

```bash
python3 scripts/gemini_session_broker.py --backend acp --host 127.0.0.1 --port 8767
```

再告诉 orchestrator 去哪里找它：

```bash
export RESEARCH_GEMINI_BROKER_URL=http://127.0.0.1:8767
export RESEARCH_GEMINI_TRANSPORT=broker
```

如果你希望 broker 掉线时还能回退到 direct：

```bash
export RESEARCH_GEMINI_TRANSPORT=auto
```

### 自定义 ACP 命令

如果 Gemini 安装位置特殊，或者你想换启动参数：

```bash
export RESEARCH_GEMINI_ACP_CMD="gemini --acp"
python3 scripts/gemini_session_broker.py --backend acp --host 127.0.0.1 --port 8767
```

### 旧的一次一进程 backend

如果你明确要保留旧行为：

```bash
python3 scripts/gemini_session_broker.py --backend cli --host 127.0.0.1 --port 8767
```

但这一条不会保住常驻的 Google 登录会话，只适合 API key 风格的自动化。

## Multi-Agent 工作流里怎么用

### 预检

在跑大任务前先执行：

```bash
python3 -m bridges.orchestrator doctor --cwd .
```

现在 `doctor` 会把 Gemini 拆成三行单独展示：

- `Gemini transport`
- `Gemini broker`
- `Gemini direct auth`

这三项要分开理解。只要 transport 最终落到 broker，那么 direct auth 缺失并不等于配置错误。

### `task-run`

典型命令：

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --triad
```

在 triad 模式里，Gemini 只是运行计划中的一个参与者。如果 Gemini transport 解析成 broker，那么只有 Gemini 任务会走常驻 broker，Codex 和 Claude 仍然是直接运行。

### `parallel`

适合要多个 agent 独立草稿或独立评审的场景。现在 orchestrator 会在真正发送大 prompt 之前先做 runtime 级 preflight，所以如果 Gemini direct 本来就不可用，但 broker 可用，就不会在错误路径上白白消耗那一整份 prompt token。

### `team-run`

`team-run` 用的也是同一套路由逻辑。这里最重要的是一致性：你要提前决定 Gemini 是必须在所有阶段都参与，还是允许在 fallback 策略下被跳过。

## Profile 级控制

可以在 profile 里固定 transport：

```json
{
  "runtime_options": {
    "gemini": {
      "transport": "broker",
      "approval_mode": "plan"
    }
  }
}
```

适用场景：

- 某个项目必须固定走常驻 Google 登录 Gemini
- 某个 CI profile 必须避免 broker 前提
- 某个本地 review profile 希望 Gemini 永远只在只读 `plan` 模式运行

## 环境支持矩阵

### 本地桌面

支持：

- `codex` direct
- `claude` direct
- `gemini` direct 配合 API key 或 Vertex
- `gemini` broker 配合 resident ACP
- `gemini` broker 配合缓存的 Google 登录

这是 Google 登录 Gemini 自动化的主场景。

### CI

支持：

- `codex` direct
- `claude` direct
- `gemini` direct 配合 API key 或 Vertex

不推荐：

- 依赖 Google 登录的 resident Gemini broker

原因很简单：CI 不应该依赖浏览器登录，也不应该依赖人工维持的桌面会话。

### 纯无头服务器

支持：

- `codex` direct
- `claude` direct
- `gemini` direct 配合 API key 或 Vertex
- `gemini` broker，但前提是 broker 自己也使用非交互认证

不推荐：

- 没有稳定桌面驻留环境时，依赖缓存 Google 登录的 Gemini broker

## 常见失败模式

### Broker 已配置但不健康

在 `broker` 模式下：

- Gemini 直接不可用
- orchestrator 会报告 broker transport 失败

在 `auto` 模式下：

- 只有在 direct auth 同时可用时，才会回退到 direct

### Direct auth 缺失

这只在 transport 最终走到 `direct` 时才构成问题。

如果 transport 最终走的是 `broker`，那 direct auth 缺失本身不是错误。

### Resident broker 中途丢失认证

如果最近一次 prompt 失败看起来像 auth 问题，broker 会把 Gemini 标记为 `auth_lost`。这时要么 reset broker 状态，要么重新完成认证后重启 broker。

## 推荐默认值

除非你有明确反例，否则按下面配置：

- 本地桌面，只有 Google 登录：
  - 启动 ACP broker
  - 设置 `RESEARCH_GEMINI_TRANSPORT=broker`
- 本地桌面，有 API key：
  - 设置 `RESEARCH_GEMINI_TRANSPORT=auto`
- CI：
  - 用 API key 或 Vertex
  - 不要假设 Google 登录可用

## 相关页面

- [快速开始](/zh/quickstart)
- [任务场景](/zh/guide/task-recipes)
- [故障排除](/zh/guide/troubleshooting)
- [系统架构](/zh/architecture)
