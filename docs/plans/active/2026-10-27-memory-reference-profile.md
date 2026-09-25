# 2026-10-27 Memory Reference Profile 实施计划

状态：ACTIVE。代码基线：`e9af4ebd8a77d316c78764fd85836e6de8d9b7ee`。Vault 目标设计及已接受决定拥有长期语义；本计划授权当前 Memory reference profile 的有界规格、实现和验证工作。

## 目标

形成一条完整的 Memory reference profile 纵向闭环：从材料与观察、Evidence/Provenance、Memory formation/revision、Authority 持久化、固定计划检索、真实使用回执、生命周期、projection consistency、crash/restart 恢复到来源追溯。

该计划服务 10 月 27 日中期检查，也为后续认知领域落到规格与代码提供范式。它不把研究候选自动提升为 production default。

## 当前代码基线

当前 workspace 已有 TypeScript Core、Rust Kernel、Subject Core、Cognitive Runtime、material/Memory owners、Protobuf、NousQL 子集及独立 Serving generations。实现差距见[当前状态](../../current-state/CURRENT_STATE.md)和[实现差距审查](../../current-state/audits/2026-09-25-implementation-gap.md)。

开始实现前先更新实现架构和决定索引，使其指向 Vault 的 Memory、Evidence、Retrieval、Serving 和 Self 目标语义。旧 `Specific / Integrative / Procedural` 分类、Anchor Authority 和旧 Architecture Rebase 执行规格不再作为本计划的施工基础。

## 必须完成的语义

- Artifact / ObservationOccurrence、Evidence 与 Provenance 分离；来源可以从派生材料回溯到来源区域和原始观察。
- Memory object、不可变 revision、object epoch、head revision fencing、RevisionLifecycle、修正/重新解释、contradiction 和 derivation。
- 认知角色与形成方式正交表示；移除互斥 `Specific / Integrative / Procedural` 分类。
- Entity mention 与 Memory revision aboutness 分离；Occurrence 拥有 occurred/observed 时间证据。
- CognitiveSchema 归 Memory Authority；AssociativeTopology 为可独立重建的 Serving projection，不再把 Anchor 作为目标认知对象。
- formation、revision、consolidation 保留来源提案与 owner validation；局部 Action 或 Effect 不代表长期 Memory/目标条件已满足。
- 精确、实体、词项、稠密基线；QueryPlan 在生成候选前固定通道、预算、过滤与融合参数。
- 多视图先聚合到对象 revision，再执行稳定融合；返回前核验 revision、状态、权限、accessibility、suppression 与 purge。
- `ReportUse` 使用调用方稳定事件 ID 与规范摘要；相同事件重试幂等，同 ID 不同内容冲突。
- 自动 accessibility 是查询时回忆资格。Suppression、restore、purge 和 Session eviction 仍是不同生命周期。
- Serving generations 保留来源 Authority watermark、producer/embedding identity、独立失效与重建边界；拒绝已过期候选。
- restart 后从 canonical Authority 恢复；provenance 能从 Memory 追溯到 Observation、Artifact/Region 或派生材料。

## Self Profile 辅助边界

可将静态 Self Profile 作为独立、只读的辅助系统纳入 reference profile。它从 Character Seed 的 Subject 快照初始化，可分为身份、自我概念、人格倾向、价值偏好和叙事起点等只读文档。

该 Profile 不合并进 Memory，不实现自动 Self evolution，也不取得安全政策、管理员强制行为或外部行为 Authority。完整 Self 仍由 Vault 单独设计。

## 非目标

- 完整 Self evolution、Social Cognition、Motivation/Pursuit；
- Nous 内部 scheduler、Dream、完整离线认知工作流；
- 完整日记领域及 CognitiveSchema 高级形成/拆分/合并算法；
- EPA/Residual、Wave、PageRank 或任一高级联想算法成为 production default；
- Heptalogos 的行为对象、live connector、provider 选择或行为权威实现；
- 为支持旧 schema 增加兼容双读/双写、fallback 或 legacy migration。

这些能力继续由 Vault 设计或后续任务负责，不应削弱 Memory reference profile 的 Authority 正确性。

## 执行顺序

### A. 更新当前实现架构与决定来源

按 Vault 目标设计更新实现架构说明，区分 current code、计划和目标。检查依赖/物理选择的权威来源；workspace manifests、Protobuf sources 与 lockfiles 持有精确实现选择，不在 prose 重复版本。

### B. 记忆 Authority 规格

定义 Memory object/revision/head、分类双轴、Evidence/Provenance、Entity aboutness、时间、矛盾/修订/派生关系、accessibility/suppression/purge 和 ReportUse 幂等合同。更新 Rust types、Protobuf、PostgreSQL schema 与事务边界前，先确认同一语义 owner。

### C. 查询与检索规格

冻结 typed Cognitive Query、NousQL 边界、候选通道、fixed QueryPlan、预算、过滤、多视图对象聚合、稳定融合、诊断/降级和返回前核验。消除依赖本批候选启用通道的归一化及不可解释的计划外拓扑奖励。

### D. Projection 与恢复规格

明确每个 serving family 的 generation、水位、producer/embedding-space 身份、独立 invalidation、stale-result validation、原子 publish、crash/restart recovery 和 provenance trace-back。

### E. 实现与资格验证

先完成语义规格与迁移，再实现 owner operations、Rust/TypeScript/Protobuf 与 query path。通过单 owner tests、Kernel/Core integration、restart/rebuild 与 end-to-end provenance scenarios；不得以 mock 结果代表 live provider 或跨平台资格。

### F. 静态 Self Profile

将只读 Profile 作为单独辅助边界实现，不改变 Memory schema 或生命周期。其内容来源绑定 Character Seed snapshot；没有 profile 不得阻断核心 Memory reference path。

## 中期演示主线

1. 创建 Subject 并保留 Character Seed 来源；
2. 摄入多份消息或材料，形成独立 Artifact、ObservationOccurrence 和 Evidence；
3. 形成不同认知角色/形成方式的 Memory；
4. 新证据产生修订或矛盾关系，保留旧 revision 与来源；
5. 经 exact/entity/lexical/dense 检索返回带证据族的结果；
6. 展示候选召回/上下文呈现不自动强化长期 Memory；
7. 通过稳定事件 ID 报告真实引用或行动，重复请求不重复记账；
8. 重启后恢复 canonical Authority、Session 与 serving generation；
9. 执行 suppression、restore 与 purge，保持互不混淆；
10. 从 Memory revision 与 query trace 追溯到 Observation、SourceRegion 和 Artifact；
11. 读取静态 Self Profile 时保留来源，不触发自动演化。

## 验收与停止

实现前更新规格、数据库和 Protobuf contracts。按计划运行 `corepack pnpm generate`、`corepack pnpm check`、Kernel/Core focused integration tests 和 `just verify`。每项结果按 `PASS`、`FAIL`、`NOT_RUN` 或 `BLOCKED` 记录；代码审阅或源码搜索不替代实际验证。

如果实现要求改变 Vault 目标语义，新增 Memory owner、持久状态、provider、failure policy 或跨系统 authority，停止在窄 `PLAN_GAP`，先更新 Vault 目标设计/决定。完成验收后停止，不扩展为 Self evolution、Social/Motivation implementation 或高级算法研究。
