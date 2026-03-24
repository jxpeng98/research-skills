# Paper Type 路线图

这一页给出五种 canonical paper type 的标准示例路线：

- `systematic-review`
- `empirical`
- `qualitative`
- `methods`
- `theory`

它们不是唯一正确路径，但在你需要一个可辩护、可落地的默认流程时，这些路线最适合作为起点。

## 怎么读这些范例

每个 playbook 都会包含：

- 推荐路线
- 更窄的轻量路线
- 常见会用到的 skills
- 常见产物
- 一个起手命令

你可以先把它们当默认操作模板，再根据自己的约束决定要不要收窄或加深。

## 1. Systematic Review

### 什么时候用

当你在做 PRISMA 风格系统综述、证据综合，或者需要一个透明检索与筛选逻辑的 structured related work 基础时，用这条。

### 推荐路线

1. `A1`：澄清问题与边界
2. `B1`：跑可复现检索
3. `B1_5`：收敛概念与 Boolean 检索式
4. `B2`：提取论文
5. `B3`：构建 literature map
6. `E1`：综合证据
7. `E2`：做质量 / 偏倚评估
8. `G1`：跑 PRISMA 检查
9. `F3`：写综述全文

### 更窄路线

如果你已经有比较稳定的 corpus，可直接用：

1. `B2`
2. `B3`
3. `E1`
4. `F3`

### 典型 skills

- `academic-searcher`
- `concept-extractor`
- `paper-screener`
- `paper-extractor`
- `literature-mapper`
- `evidence-synthesizer`
- `quality-assessor`
- `prisma-checker`
- `manuscript-architect`

### 典型产物

- search log
- screening log
- extraction table
- literature map
- synthesis memo 或 meta-analysis 结果
- quality assessment
- PRISMA compliance report
- manuscript draft

### 起手命令

```bash
python3 -m bridges.orchestrator task-run \
  --task-id B1 \
  --paper-type systematic-review \
  --topic ai-in-education \
  --cwd . \
  --research-depth deep
```

### 常见收窄规则

如果辅助产物开始失控，优先保留：

- `B2`
- `B3`
- `E1`
- `F2` 或 `F3`

再配合 `--focus-output` 和 `--output-budget`。

## 2. Empirical Paper

### 什么时候用

当你在写标准实证论文，核心问题是设计、数据、分析、结果解释和投稿收口时，用这条。

### 推荐路线

1. `A1`：定义问题
2. `A1_5`：生成假设
3. `C1`：搭建设计
4. `C2` / `C3`：变量操作化与数据路径确认
5. `C4`：补稳健性逻辑
6. `I1` / `I2` / `I3`，或在代码量较大时切入完整 Stage-I 代码链
7. `F1`：设计稿件结构
8. `F3`：起草全文
9. `F4`：补表格 / 图 / 结果解释
10. `G2`：做 reporting check
11. `H1`：整理 submission package

### 更窄路线

如果研究已经跑完，主要需求是写作和提交前检查，可以直接用：

1. `F1`
2. `F3`
3. `F4`
4. `G2`
5. `H1`

### 典型 skills

- `question-refiner`
- `hypothesis-generator`
- `study-designer`
- `variable-constructor`
- `dataset-finder`
- `robustness-planner`
- `analysis-interpreter`
- `table-generator`
- `figure-specifier`
- `reporting-checker`

### 典型产物

- question / hypothesis set
- design spec
- variable / dataset plan
- robustness plan
- manuscript draft
- 表格与图规格
- reporting compliance memo
- submission bundle

### 起手命令

```bash
python3 -m bridges.orchestrator task-run \
  --task-id C1 \
  --paper-type empirical \
  --topic policy-effects \
  --cwd .
```

### 常见判断规则

如果代码只是辅助，就留在 design + writing 路线。
如果代码本身已经成为论文贡献的一部分，就切到完整 Stage-I 代码链，不要只靠普通写作流程。

## 3. Qualitative Paper

### 什么时候用

当论文的核心证据来自访谈、案例、民族志、文档或过程追踪，并且目标是解释意义、机制或过程，而不是做统计估计时，用这条。

### 推荐路线

1. `A1`：收敛 qualitative research question、setting 和 unit of analysis
2. `A1_5`：定义 working propositions 或 sensitizing concepts
3. `A3`：锚定理论镜头或过程框架
4. `A4`：明确 qualitative gap 和预期贡献
5. `B2`：做定向论文提取
6. `B6`：围绕 mechanism、process 和 rival explanation 建 literature map
7. `C1`：完成 qualitative design
8. `C2`：写 interview / observation / document protocols
9. `C3`：锁定 coding、memoing 和 comparison logic
10. `C1_5`：定义 rival interpretations / disconfirming cases
11. `D1`：补 ethics 与 data governance
12. `F1`：设计 manuscript structure
13. `F3`：写 qualitative 全稿
14. `G1`：做 reporting check（SRQR / COREQ）
15. `H4`：做 fatal-flaw 压力测试

### 更窄路线

如果 fieldwork 已经完成，主要需求是把分析转成稿件，可直接用：

1. `C3`
2. `F1`
3. `F3`
4. `G1`
5. `H1`

### 典型 skills

- `question-refiner`
- `hypothesis-generator`
- `theory-mapper`
- `gap-analyzer`
- `paper-extractor`
- `literature-mapper`
- `study-designer`
- `rival-hypothesis-designer`
- `analysis-interpreter`
- `reporting-checker`
- `manuscript-architect`

### 典型产物

- qualitative RQ 与 contribution memo
- case / participant sampling logic
- interview guide 或 observation protocol
- coding 与 memoing plan
- findings interpretation memo
- manuscript draft
- SRQR / COREQ checklist
- fatal-flaw memo

### 起手命令

```bash
python3 -m bridges.orchestrator task-run \
  --task-id C1 \
  --paper-type qualitative \
  --topic platform-governance-practices \
  --domain business-management \
  --cwd .
```

### 常见判断规则

当论文需要对 process、meaning、interpretation 或 mechanism 做深度解释，并且证据基础是访谈、案例、fieldnotes 或 documents 而不是可直接建模的数据集时，就走 qualitative 路线。

## 4. Methods Paper

### 什么时候用

当核心贡献是方法、算法、pipeline 或代码支持的新程序，并且代码本身是第一类证据时，用这条。

### 推荐路线

1. `A1`：定义问题与贡献声明
2. `A3`：完成理论 / 方法定位
3. `C1`：规定评估设计
4. `I5`：写 code specification
5. `I6`：写 zero-decision execution plan
6. `I7`：实现与 profiling
7. `I8`：复核逻辑与统计有效性
8. `I4`：做 reproducibility audit
9. `F1`：设计稿件结构
10. `F3`：写 methods paper 全稿
11. `H3`：做模拟评审压力测试

### 更窄路线

如果你还在锁定实现边界，不需要立刻编码，可先用：

1. `A1`
2. `C1`
3. `I5`
4. `I6`

### 典型 skills

- `theory-mapper`
- `study-designer`
- `code-specification`
- `code-planning`
- `code-execution`
- `code-review`
- `reproducibility-auditor`
- `stats-engine`
- `manuscript-architect`

### 典型产物

- method positioning memo
- evaluation design
- code specification
- execution plan
- performance profile
- code review
- reproducibility audit
- methods manuscript draft

### 起手命令

```bash
python3 -m bridges.orchestrator code-build \
  --method "Staggered DID" \
  --topic policy-effects \
  --domain economics \
  --focus full \
  --paper-type methods \
  --cwd .
```

### 常见判断规则

如果你不确定是否需要完整代码链，就问自己：

- 代码是不是核心贡献？
- reviewer 会不会直接检查可复现性和实现质量？
- 这篇文章是否必须给出 `code_review.md`、`reproducibility_audit.md` 这类严格产物？

如果答案是“会”，就走 Stage-I 代码链。

## 5. Theory Paper

### 什么时候用

当论文的主贡献是概念、理论结构、机制构建，而不是数据执行或代码实现时，用这条。

### 推荐路线

1. `A1`：收敛核心问题
2. `A1_5`：把问题转成 propositions
3. `A2`：映射理论基础
4. `A4`：识别 unresolved theoretical gap
5. `B2`：做定向文献提取
6. `E1`：综合概念性证据
7. `F1`：设计 manuscript logic
8. `F3`：写理论全文
9. `G4`：做语气压缩
10. `H4`：做 fatal-flaw 压力测试

### 更窄路线

如果理论基础已经稳定，可直接用：

1. `A2`
2. `A4`
3. `F1`
4. `F3`

### 典型 skills

- `question-refiner`
- `hypothesis-generator`
- `theory-mapper`
- `gap-analyzer`
- `paper-extractor`
- `evidence-synthesizer`
- `manuscript-architect`
- `tone-normalizer`
- `fatal-flaw-detector`

### 典型产物

- 概念问题定义
- propositions
- theory map
- theoretical gap memo
- theory manuscript draft
- 语气归一化日志
- fatal-flaw memo

### 起手命令

```bash
python3 -m bridges.orchestrator task-run \
  --task-id A2 \
  --paper-type theory \
  --topic organizational-ai-governance \
  --cwd .
```

### 常见判断规则

不要把代码链硬塞进 theory paper，除非 simulation 或 method 本身就是贡献的一部分。

## 跨 Playbook 的通用建议

### 什么情况下该走深一点

这些情况建议走更完整的链：

- reviewers 会期待 reproducibility 或 checklist evidence
- 论文类型本身就有强 reporting 标准
- 证据基础存在争议或高度异质
- 你需要更强的 adversarial review

### 什么情况下该收窄

这些情况建议收窄：

- 输入已经比较稳定
- 当前是 revision，不是从零开始
- artifact sprawl 已经太大
- 你只需要一个核心交付物

常用控制参数：

- `--focus-output`
- `--output-budget`
- `--research-depth deep`
- `--only-target`

## 相关页面

- 想看按任务场景的指导：去 [任务场景](/zh/guide/task-recipes)
- 想看完整 skill 地图：去 [Skills 指南](/zh/reference/skills)
- 想看精确命令参数：去 [CLI 参考](/zh/reference/cli)
