# 任务场景指南

这一页适合“我知道自己要做什么，但还不知道应该用哪些 stage / Task ID / skill”的使用者。

它是按真实任务来组织的：

- 先从你要完成的工作出发
- 再映射到 stages 和 Task IDs
- 再看背后通常会用到哪些 skills
- 最后选一条尽量窄、但又足够可辩护的执行路径

如果你想看完整的 stage-by-stage skill 地图，请直接看 [Skills 指南](/zh/reference/skills)。
如果你想看“systematic-review / empirical / methods / theory”这类 paper type 的标准默认路线，请看 [示例](/zh/examples/)。

## 怎么使用这一页

看每个场景时，按这个顺序读：

1. **什么时候用**
2. **最小路径**
3. **深入路径**
4. **典型 skills**
5. **典型产物**

不要默认自己必须把所有 stage 都跑完。
通常最好的路线是：只跑那条最窄、但足以满足论文类型与证据要求的路径。

## 1. 我需要把一个宽泛主题收敛成可研究的问题

### 什么时候用

当你的主题还很散、贡献点不清楚、理论锚点不稳、目标 venue 也没定下来时，用这条。

### 最小路径

- `A1`：收敛问题和边界
- `A4`：找出最强的 research gap

### 深入路径

- `A1`
- `A1_5`：生成假设 / propositions
- `A2` / `A3`：补理论和定位
- `A5`：做 venue fit 检查

### 典型 skills

- `question-refiner`
- `hypothesis-generator`
- `theory-mapper`
- `gap-analyzer`
- `venue-analyzer`

### 典型产物

- 研究问题集合
- contribution framing
- theory map
- gap memo
- venue fit 约束说明

### 建议起手命令

```bash
python3 -m bridges.orchestrator task-plan \
  --task-id A1 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd .
```

## 2. 我需要一个靠谱的 related work / systematic review 基础

### 什么时候用

当真正的瓶颈在于文献质量、筛选纪律、提取一致性、或 PRISMA 透明度时，用这条。

### 最小路径

- `B1_5`：补概念和检索词
- `B2`：重点读文献 / 做结构化提取
- `B3`：做 literature mapping

### 深入路径

- `B1`：全量检索
- `B1_5`
- `B2`
- `B3`
- `B4` / `B5`：做 citation expansion 或 synthesis support
- `G1`：提交前做 PRISMA 核查

### 典型 skills

- `academic-searcher`
- `concept-extractor`
- `paper-screener`
- `paper-extractor`
- `citation-snowballer`
- `literature-mapper`
- `prisma-checker`

### 典型产物

- search log
- screening log
- extraction table
- literature map
- PRISMA-ready 计数与 compliance memo

### 建议起手命令

```bash
python3 -m bridges.orchestrator task-run \
  --task-id B2 \
  --paper-type systematic-review \
  --topic ai-in-education \
  --cwd . \
  --research-depth deep
```

## 3. 我需要先把实证研究设计清楚，再去写作或编码

### 什么时候用

当研究问题已经比较稳，但设计、变量、稳健性方案、数据路径仍然薄弱时，用这条。

### 最小路径

- `C1`：主设计
- `C2` 或 `C3`：变量或数据可得性

### 深入路径

- `C1`
- `C1_5` / `C2`：竞争解释和变量逻辑
- `C3`：数据可得性
- `C3_5` / `C4`：稳健性和数据管理
- `C5`：预注册式 handoff

### 典型 skills

- `study-designer`
- `rival-hypothesis-designer`
- `dataset-finder`
- `variable-constructor`
- `robustness-planner`

### 典型产物

- design spec
- analysis plan
- variable specification
- dataset plan
- robustness plan

### 建议起手命令

```bash
python3 -m bridges.orchestrator task-run \
  --task-id C1 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd .
```

## 4. 我已经有分析结果，现在要把它写成论文

### 什么时候用

当分析已经存在，但问题在于结构、表格、图、结果解释、摘要标题这些写作层工作时，用这条。

### 最小路径

- `F1`：做结构 / outline
- `F2`：做段落级或 section 级写作

### 深入路径

- `F1`
- `F2`
- `F3`：全稿起草
- `F4`：结果解释、表格、图支持
- `F5` / `F6`：摘要、标题、关键词和最后收口

### 典型 skills

- `manuscript-architect`
- `analysis-interpreter`
- `effect-size-interpreter`
- `table-generator`
- `figure-specifier`
- `meta-optimizer`

### 典型产物

- manuscript outline
- section drafts
- 保留不确定性的 results narrative
- 论文级表格与图规格
- title / abstract / keywords 优化稿

### 建议起手命令

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --focus-output manuscript/manuscript.md \
  --output-budget 2
```

## 5. 我需要的是学术代码，不是通用产品工程流程

### 什么时候用

当你的工作是 methods paper、实证流水线、可复现包、或强统计导向实现时，用这条。

### 最小路径

- `I5`：spec
- `I6`：zero-decision plan

如果你还在锁定边界、约束和执行方式，这就是正确的最小起点。

### 深入路径

- `I5`
- `I6`
- `I7`：实现与 profiling
- `I8`：代码 / 统计复核
- `I4`：可复现性审计

### 典型 skills

- `code-specification`
- `code-planning`
- `code-execution`
- `code-review`
- `reproducibility-auditor`
- `stats-engine`

### 典型产物

- `code/code_specification.md`
- `code/plan.md`
- `code/performance_profile.md`
- `code/code_review.md`
- `code/reproducibility_audit.md`

### 建议起手命令

```bash
python3 -m bridges.orchestrator code-build \
  --method "Staggered DID" \
  --topic policy-effects \
  --domain economics \
  --focus full \
  --paper-type methods \
  --cwd .
```

### 如果结果太宽、辅助文件太多

优先用这些参数收窄：

- `--only-target`：只重跑局部 target
- `--research-depth deep`：当推理深度不够时
- `--focus-output` 和 `--output-budget`：当产物扩散太宽时

## 6. 我需要做投稿前 stress test、回复审稿或最终打包

### 什么时候用

当稿件已经接近提交、已经进入审稿轮次，或者你想先做一次 harsh pre-submission 检查时，用这条。

### 最小路径

- `H1`：submission package
- `H2`：rebuttal support

### 深入路径

- `G1` / `G2`：先做 reporting / PRISMA 检查
- `G4`：语气收敛
- `H1`
- `H2`
- `H3`：模拟评审
- `H4`：fatal flaw 扫描

### 典型 skills

- `submission-packager`
- `rebuttal-assistant`
- `peer-review-simulation`
- `fatal-flaw-detector`
- `reviewer-empathy-checker`
- `reporting-checker`

### 典型产物

- cover letter 和 submission bundle
- point-by-point rebuttal matrix
- 模拟 reviewer 报告
- fatal-flaw memo
- response tone 调整日志

### 建议起手命令

```bash
python3 -m bridges.orchestrator task-run \
  --task-id H3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --triad
```

## 7. 我只需要补跑局部，不想整条链重来

### 什么时候用

当你已经有 Stage-I 结构化 artifact，只想重跑某几个 plan step，或者只修某几个 review finding 时，用这条。

### 常见路径

- `task-run --only-target <target>`
- `code-build --only-target I6:S1`
- `code-build --only-target I8:P1-01`

### 示例

```bash
python3 -m bridges.orchestrator task-run \
  --task-id I6 \
  --paper-type methods \
  --topic policy-effects \
  --cwd . \
  --only-target S1
```

```bash
python3 -m bridges.orchestrator code-build \
  --method "Staggered DID" \
  --topic policy-effects \
  --domain economics \
  --focus full \
  --only-target I8:P1-01 \
  --cwd .
```

## 接下来该看哪一页？

- 想看全局 skill 地图：去 [Skills 指南](/zh/reference/skills)
- 想看按 paper type 组织的默认路线：去 [Paper Type 路线图](/zh/examples/paper-type-playbooks)
- 想看精确命令参数：去 [CLI 参考](/zh/reference/cli)
- 想看运行时协同细节：去 [Agent + Skill 协同](/zh/advanced/agent-skill-collaboration)
- 想看安装与升级：回到 [入门首页](/zh/guide/)
