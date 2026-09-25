# 2026-09-25 Nous Wave 实现差距审查

代码基线：`e9af4ebd8a77d316c78764fd85836e6de8d9b7ee`。本审查依据当前 source、Protobuf 和 integration tests 作定向核对；它描述已有能力与未对齐处，不把研究材料或目标设计当作实现证明。

## 可直接继承的代码基础

| 主题 | 当前证据 | 状态 |
|---|---|---|
| Core/Kernel 进程分层 | [`apps/nous-core`](../../../apps/nous-core)、[`apps/nous-kernel`](../../../apps/nous-kernel)、[`proto/`](../../../proto) 与官方 Client | 已有运行实现；最终验证按仓库验收命令报告。 |
| Subject Core / Character Seed | [`crates/subject-core`](../../../crates/subject-core) | 主体身份、初始化材料与来源引用已有 owner。 |
| Session / Cognitive Runtime | [`crates/cognitive-runtime`](../../../crates/cognitive-runtime) | Session、ResidentSet、QueryPlan、资源查询、checkpoint、使用事件和工作集已有实现。 |
| Artifact / Observation 分离 | [`crates/material`](../../../crates/material)、[`crates/material-service`](../../../crates/material-service) | 字节对象、观察事件、派生表示与 Evidence 有独立身份。 |
| Observation 重试 | [`types.proto`](../../../proto/nous/wave/v1alpha1/types.proto)、[`observation.rs`](../../../crates/material-service/src/observation.rs) | `request_id`、规范请求摘要和事务绑定已实现；不是当前差距。 |
| NousQL | [`parser.ts`](../../../apps/nous-core/src/nousql/parser.ts)、[`compiler.ts`](../../../apps/nous-core/src/nousql/compiler.ts)、[`nousql.test.ts`](../../../apps/nous-core/tests/nousql.test.ts) | 当前 parser/compiler 子集和测试存在；完整产品语言语义未冻结。 |
| Memory/Serving owners | [`memory-domain`](../../../crates/memory-domain)、[`memory-service`](../../../crates/memory-service)、[`memory-retrieval`](../../../crates/memory-retrieval)、[`serving`](../../../crates/serving) | 形成、revision、生命周期、检索与 projection generation 已有代码路径。 |

## 需要与 Vault 目标设计对齐的实现合同

### Memory 分类

当前 [`MemoryClass`](../../../crates/memory-domain/src/lib.rs) 是 `Specific / Integrative / Procedural` 互斥枚举。Vault 目标设计要求分别记录认知角色与形成方式。实现更改会触及 Rust domain、Protobuf、数据库 schema、formation/revision 操作与查询过滤。

### Anchor 与 CognitiveSchema

当前 Rust domain、Management Protobuf 与 PostgreSQL schema 都把 `Anchor / AnchorRevision / AnchorSupport` 作为权威对象。目标设计要求 CognitiveSchema 归 Memory Authority，AssociativeTopology 归可重建 Serving；两类职责不能继续通过一个 Anchor 对象承担。

### 关联证据权重

[`AssociationEvidence`](../../../crates/memory-domain/src/lib.rs) 用 `support_value: f64` 表示支持强度。目标设计要求 Authority 保留类型化来源证据，Serving 根据有效证据计算投影权重；不能把单一数值解释成认知置信度或永久边权。

### 长期使用事件幂等

当前 [`ReportUseRequest`](../../../proto/nous/wave/v1alpha1/types.proto) 不带调用方事件标识；[`use_feedback.rs`](../../../crates/cognitive-runtime/src/use_feedback.rs) 为每个事件生成新的 `Uuid::now_v7()`。同一 HTTP/RPC 请求重试可能形成多个使用事件。计划需要为相同 ID、相同摘要的事件定义幂等写入，并拒绝相同 ID 的不同内容。

### 可访问性

当前 [`AccessibilityPolicy`](../../../crates/memory-service/src/accessibility.rs) 默认为 90 天和 365 天的 NORMAL/DEEP/EXPLICIT 阈值。目标设计要求访问资格在查询时派生，并把年龄阈值当作可调整策略，而不是永久事实门槛。

### 查询融合

当前 [`rank_candidates`](../../../crates/memory-retrieval/src/ranking.rs) 先按当前批次实际出现的 evidence families 计算分母，并根据相近基分候选计算 topology innovation。这样最终得分可能随无关候选变化。目标设计要求在候选产生前固定通道/预算/权重，先把多视图聚合到对象修订，再执行稳定融合。

## 实现边界

当前工作区未见 Self、Social Cognition、Motivation、Pursuit 或 Heptalogos integration 的实现 owner/spec。它们属于 Vault 设计及后续计划，不应被记作当前 Memory 子系统能力。

本审查记录的是源码对应，不等同于运行/集成测试 PASS。实际 `corepack pnpm check` 与 `just verify` 结果记录在[当前状态](../CURRENT_STATE.md)。
