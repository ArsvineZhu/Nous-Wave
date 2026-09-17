# 认知运行时与上下文

状态：Session/Resident/use 已有基础；Focus、Steward 和新投影/上下文运行时属于后续研究设计。依据 [Pre-Spec](sources/prespec-implementation-research.md) Stage 3–4；概念起点见[9 月 9 日设计记录](sources/runtime-projection-design-2026-09-09.md) §2–18。

## 认知状态的层次

```text
Subject Cognitive Authority
    -> Session ResidentSet
    -> FocusWorkingSet
    -> consumer-specific CognitiveProjection
    -> 消费者的一次模型调用
```

Session 表示认知连续性，独立于会话 UI、单次 Reaction、模型线程和进程寿命。ResidentSet 保存可恢复的引用与使用元数据。FocusWorkingSet 从当前 Focus、ResidentSet、Authority 和检索结果派生；消费者投影再施加角色需求、预算和材料访问策略。

Active Cognition 需要区别稳定认知、当前任务、工作知识和可丢弃执行细节。早期讨论中的多维 activation/salience 是探索动机；Pre-Spec 用显式状态、来源、使用事件和需求等级落实，未冻结一套统一认知分数或永久 N×M consumer activation 表。

## Focus 与恢复

第一轮设计中，每个 Session 最多一个前台 `ACTIVE` Focus，可以有多个 `SUSPENDED` Focus 和历史 `CLOSED` Focus。真正独立的并行工作先由不同 Session 承载。

Focus 的 descriptor 是有界任务说明和来源引用。`SwitchFocus` 是带 Session runtime revision 的原子操作：检查预期版本、保存当前 checkpoint、挂起旧 Focus、激活目标、推进版本，一并提交。并发过期调用得到 `ABORTED`，重新读取状态。

`RuntimeCheckpointStore` 提供一个有界私有存储机制：

```text
subject_id + session_id + owner_kind + owner_key
schema_version + revision + payload_bytes + timestamps
```

Kernel 管理 Subject/Session 范围、大小限制和 CAS；Host 管理版本化 Protobuf payload 的语义。用途限于 Focus、Context、Steward 等非 Authority 连续性状态。

Focus checkpoint 保存引用、必要的 continuation summary、来源 runtime revision 与 producer。恢复时重新解析当前 Authority；已失效、被清除或修订的引用必须重新处理。summary 是可替换解释，不能被恢复成另一份 Memory/Persona Authority。

## Consumer 与 Projection

消费者使用稳定 opaque ID，Host 将其解析到已配置的 `ConsumerPolicy`，包括允许的认知族、材料化权限、预算上限和默认需求。自报 consumer ID 或 `REQUIRED` 需求不会自动授予权限。

需求强度采用 `REQUIRED / PREFERRED / OPTIONAL / FORBIDDEN`。表达 Agent 和机器操作 Agent 可以从同一 Subject 得到不同投影；Persona 尚未实现时，不能通过虚构 Persona 数据证明差异化。

拟议 `ProjectionRequest` 含 Subject、Session、可选 Focus、Consumer、当前 Situation、可选 Query、effort 和预算。Situation 提供当前外部引用和线索，避免把消费者工作流状态复制进 Nous。

确定性路径为：

```text
解析 ConsumerPolicy -> 校验 Session/Focus
-> 合并 Situation refs、检索候选、Resident/Focus refs
-> 收集已实现 contributor
-> 权限与需求过滤 -> 去重 -> 模态/材料化/预算限制
-> 渲染 ContextSegments -> CognitiveProjection + trace/degradation
```

现有 Rust `ConsumerWorkingSet` 的引用校验、去重、预算和材料化机制保留；目标职责收窄为 Kernel candidate/materialization primitive。Host 拥有最终消费者认知选择。

Contributor 是静态组合的窄内部接口。共用 envelope 记录 source refs、evidence、provenance、freshness、需求等级、大小和材料化状态，具体 payload 由认知所有者定义 typed union。

排序先使用需求等级、精确/当前引用、证据族内顺序、Focus/Resident 状态、适用的时效与稳定 tie-breaker。现有检索证据不能被一个通用 importance 浮点数覆盖。

预算以 `max_items` 和 `max_text_bytes` 为可强制上限。`max_estimated_tokens` 依赖具体 tokenizer/estimator 合同；没有估算器仍可进行确定性投影。

## Steward 与模型增强

Steward 是 Session 范围内的有界 proposal engine。Host 提供当前 Focus、runtime revision、近期 Observation refs、Resident refs、候选和预算。模型返回驻留建议、checkpoint 草稿、查询/上下文建议及维护意图。

应用建议前校验候选集合、Subject 范围、版本、Focus 状态和操作边界，再通过 Kernel CAS 或对应认知 owner 提交。Steward 第一轮没有任意工具访问，也不自主唤醒。

触发来自 Observation admission、消费者报告的交互边界、Focus 切换、上下文压力、显式 checkpoint 或维护调用。`StewardGeneration` 记录模型绑定、prompt/schema revision 和配置摘要，供解释派生状态来源。

模型辅助 Projection 发生在确定性选择之后，只压缩或组织已选来源。输出结构、引用或生成失败时退回确定性渲染。零模型配置时，基础 Session、检索、Memory 管理、驻留和投影仍应有真实可用路径。

## Assistive 与 Managed Cognition

Assistive 模式向消费者提供 Query、证据与 context contribution，适合只允许工具集成的外部 Agent。Managed 模式进一步管理消费者 prompt 中的 **Nous cognitive lane**，其余产品规则、工具、行为协议和最终 InvocationSpec 仍由消费者组合。

Managed context 的 track 概念键为：

```text
(subject_id, session_id, focus_id?, consumer_id)
```

同步合同只包含两种操作：

| 操作 | 消费者行为 |
|---|---|
| `RESET(epoch snapshot)` | 替换整个 Nous lane |
| `APPEND(delta)` | 保留本 epoch 内容，追加不可变增量 |

每个 epoch 内 revision 单调推进。Focus 切换、来源重大修订、policy/renderer 语义变化、预算调整或压缩会触发新 epoch。旧内容需要删除或改写时使用 RESET。

未知或丢失 cursor 返回 RESET；Host 重启后如果无法证明 append 连续性，也从当前 Authority 重编译 RESET。第一轮不承诺跨重启保留完整追加链，不保存无界 rendered-context 日志。

## Context Compiler 与缓存

编译顺序为确定性 Projection、可选 synthesis、ContextSegment 渲染、对比 track、选择 RESET/APPEND。优先排列稳定内容、Focus epoch snapshot、近期增量。

公开层只表达 `IMMUTABLE / EPOCH_STABLE / DYNAMIC` 等语义稳定性提示。具体 provider cache-control 在模型适配层映射。语义过时必须更新，即使因此损失缓存命中。Provider thread、KV cache 或私有 continuation state 不能成为恢复认知所需状态的唯一副本。

## 使用、强化与维护

生成投影只证明选择；实际曝光、引用、采取行动、佐证和纠正由消费者按真实发生阶段反馈。检索与重复投影本身不续命、不强化。

Memory maintenance、Dreaming、自动 Tags/Anchors/拓扑综合保留显式 operation 与 proposal-first 路线。维护调用时机由用户、Host 或外部自动化决定；“Dreaming”研究不授权新后台调度器或未来 MicroSystem 实现。

## 研究验收场景

后续实现需证明：并发 Focus 切换不能形成两个 ACTIVE；恢复按当前 Authority 解析引用；相同输入和 policy revision 得到确定选择；两个 consumer 可获得不同合法投影；无模型仍能投影；非法模型引用不被接受；过期 context cursor 安全 RESET；ReportUse 失败不回滚消费者已提交行为。这里记录的是预期证明，尚未执行这些新架构测试。
