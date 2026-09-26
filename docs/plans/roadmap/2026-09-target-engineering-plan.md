# Nous Wave Target Engineering Plan

状态：TARGET / PRE-SPEC
日期：2026-09-26

## 1. 计划定位

本计划连接：

`Architecture-Vault Target Design`

与：

`Executable Specs / Code / Qualification`

语义 Authority 是 [Architecture-Vault Nous Wave Target Design](https://github.com/Heptalogos-Devs/Architecture-Vault/blob/main/docs/Nous-Wave/TARGET_DESIGN.md) 及其已接受设计决定；本计划只定义从该目标到当前代码 owner、迁移顺序和验证闭环的工程轨迹。

它已经深入：

- 当前真实 code owners；
- semantic migration unit；
- protocol / persistence / runtime / serving blast radius；
- 实施依赖关系；
-阶段进入和停止条件。

它暂时不冻结：

- Protobuf message / field；
- SQL table / column；
- exact Rust type；
- crate/package 物理划分；
-算法参数；
- provider backend；
- migration script。

这些内容由后续 Specs 决定。

本计划不是中期 MVP 计划。长期目标不会为了 2026-10-27 时间点缩小。

---

# 2. 当前起点

当前 repo 已形成一条可复用基础设施骨架：

- `apps/nous-core`
  - public API；
  - Focus / Projection / Managed Context；
  - NousQL；
  - model orchestration。
- `apps/nous-kernel`
  - private Rust Kernel composition。
- `crates/subject-core`
  - Subject identity / Character Seed lineage。
- `crates/material` + `material-service`
  - Artifact / ObservationOccurrence / DerivedRepresentation。
- `crates/memory-domain` + `memory-service`
  - current Memory ontology and operations。
- `crates/cognitive-runtime`
  - Session / ResidentSet / use feedback / QueryPlan-related runtime。
- `crates/authority-store`
  - PostgreSQL Authority persistence。
- `crates/memory-retrieval`
  - candidate lanes / ranking。
- `crates/serving`
  - rebuildable serving generation and artifacts。
- `proto/nous/wave/v1alpha1`
  - cross-language contracts。
- `packages/protocol-ts` + `packages/client`
  - generated protocol and official client。

当前主要问题是 **semantic rebase**，不是基础设施从零开始。

---

# 3. 工程推进原则

## 3.1 以 semantic vertical slice 为迁移单位

一个目标语义只有在以下各层一致时才算完成：

`domain → Authority persistence → protocol → service → client → query/serving impact → tests`

禁止只改 Rust enum、只改数据库或只改 Proto，然后长期保留半迁移语义。

## 3.2 不维持无真实义务的旧 ontology compatibility

当前项目没有已批准的外部兼容义务。

如果旧 `MemoryClass`、Anchor、ranking semantics 与 Target Design 冲突，优先直接替换，而不是建立双读、双写、legacy fallback 或 permanent adapter。

## 3.3 Authority migration 先于高级算法

先保证：

- identity；
- revision；
- provenance；
- valid-time；
- lifecycle；
- idempotency；
- query-plan stability；
- serving generation correctness。

再研究：

- graph propagation；
- Wave；
- learned fusion；
- advanced consolidation。

## 3.4 Spec gate

每个真正进入代码施工的阶段，先形成足够窄的 Executable Spec。

Spec 的作用是冻结“这一轮代码必须实现什么”，而不是重新讨论 Target Design。

---

# 4. Phase 0 — 文档 Authority 与施工基线

## 目标

消除后续 Agent 在不同文档层读取到冲突语义的风险。

## 工作

Architecture-Vault：

- Nous Wave Target Design 迁移为 canonical Markdown；
- 合并本轮设计收敛；
- 更新 Decisions；
- 收敛 Open Questions；
- 停止把 PDF/LaTeX 当活动 Authority。

Nous-Wave repo：

- 建立本 Target Engineering Plan；
- 更新 10/27 active milestone plan；
- 明确 future Spec partition；
- 保持 current-state / architecture docs 只描述实际代码。

## 未来开始编码前的 baseline cleanup

当前已知 verification noise：

- `buf lint` 与 bundled health proto naming conflict；
- `cargo fmt --check` 有现有格式差异。

第一轮代码施工开始前先使基础验证链能够给出干净、可重复的 PASS/FAIL。

这属于 engineering hygiene，不是研究成果。

---

# 5. Phase 1 — Core semantic substrate rebase

## 目标

让后续所有 cognition owner 共享正确的 identity / reference / provenance / time primitives。

## 涉及 owner

主要：

- `crates/core`
- `crates/material`
- `crates/material-service`
- `crates/authority-store`
- protocol / client

## 计划语义

需要明确支撑：

- stable ObjectId / RevisionId / Epoch；
- Subject commit ordering；
- EvidenceRef；
- provenance derivation；
- producer identity；
- entity mention vs cognition aboutness；
- occurred / observed / valid / formed / recorded time；
- stable idempotency event identity；
- dependency references。

## 结果

后续 Memory/Self/Social/Motivation 不各自重造 provenance、revision、time primitives。

## Spec gate

未来先写：

`Evidence, Provenance and Cognitive Identity Spec`

可与 Memory Authority Spec 合并，若独立文档不能降低重复则不拆。

---

# 6. Phase 2 — Memory Authority ontology rebase

## 目标

把当前 Memory 代码从旧 ontology 迁移到 Target Design。

## 主要代码面

- `crates/memory-domain`
- `crates/memory-service`
- `crates/authority-store`
- `proto/nous/wave/v1alpha1`
- protocol-ts/client
- Core routes
- integration tests

## 必须完成的 semantic change

### Memory classification

从：

`Specific / Integrative / Procedural`

迁移为正交：

- Cognitive Role；
- Formation Mode。

### Object / Revision

实现：

- cognitive matter identity；
- revision intent；
- correction vs new world state；
- valid-time；
- object epoch / head fencing；
- contradiction / derivation / temporal-successor 等关系。

### Evidence

Memory revision 持有 typed evidence 与 provenance references。

### CognitiveSchema

建立 Memory-owned CognitiveSchema semantics。

当前 Anchor Authority 退出目标模型。

### AssociationEvidence

保留 typed relation / provenance / polarity / validity。

移除“通用 support_value 作为认知强度真值”的语义。

## 不在本阶段固定

- automatic schema induction algorithm；
- graph edge scoring；
- advanced consolidation model；
- learned confidence function。

---

# 7. Phase 3 — Authority persistence 与 protocol rebase

## 目标

让 Phase 2 语义在存储和跨语言边界中完整成立。

## 主要代码面

- `crates/authority-store/migrations`
- `crates/authority-store/src`
- Proto
- generated TypeScript bindings
- official Client
- Core routes

## 原则

### Schema replacement

由于当前没有真实 backward compatibility obligation，优先做 clean semantic migration，而不是保留旧 Anchor/MemoryClass 双模型。

### Idempotency

所有 durable mutation / use event 按适用范围绑定：

- caller stable operation/event id；
- normalized request digest；
- same-id/same-content idempotent；
- same-id/different-content conflict。

### Transaction boundary

模型调用、远程 resource、embedding/rerank 等继续在 Authority transaction 外执行。

提交时重新验证：

- object epoch；
- current revision；
- lifecycle；
- provenance dependency；
- permission/scope。

---

# 8. Phase 4 — Cognitive Runtime 与真实使用语义

## 目标

让 Runtime 明确区分：

- retrieval trace；
- context presentation；
- semantic reference；
- acted-on use；
- downstream outcome。

## 主要代码面

- `crates/cognitive-runtime`
- Core Focus / Projection / Context
- protocol/client
- authority-store use-event persistence

## 工作

### UseEvent

为 durable use event 加 stable caller identity 与 digest contract。

### WorkContext

在现有 Session / ResidentSet 之上建立 WorkContext 的目标 runtime boundary。

支持：

- active；
- paused；
- resumed；
- ended；
- durable checkpoint when needed。

### Accessibility

从固定 90/365 天门槛重构为 query-time policy input。

年龄只成为一个 signal。

### ResidentSet / Active Cognition

保持 Runtime state 与 durable Cognition Authority 分离。

---

# 9. Phase 5 — Cognitive Query 与 Serving rebase

## 目标

建立稳定、可解释、可重建、不会被当前候选集合反向改写的 retrieval path。

## 主要代码面

- `crates/memory-retrieval`
- `crates/serving`
- `crates/cognitive-runtime`
- NousQL compiler
- protocol/client
- model embedding / rerank adapters

## QueryPlan

在候选生成前固定：

- lanes；
- lane budget；
- hard filters；
- soft preferences；
- fusion rule；
- topology budget；
- rerank budget；
- material expansion budget；
- consistency requirement。

## Reference baseline

目标参考基线：

- exact；
- entity；
- lexical；
- dense；
- runtime；
- temporal where applicable。

先完成 object-revision level multiview aggregation，再执行跨 lane fusion。

## Stable fusion

中期 reference profile 优先采用简单、可解释、候选集合稳定的固定 fusion baseline。

Advanced learned/calibrated fusion 保持 research lane。

## Serving generation

每类 projection generation 至少绑定：

- Authority watermark；
- producer/build identity；
- embedding/vector-space identity；
- artifact digest；
- build configuration identity。

查询返回前仍重新验证 Authority。

## AssociativeTopology

从有效 AssociationEvidence / Cognition refs 重建。

它是普通 QueryPlan lane。

不得通过隐藏加分改变 baseline。

---

# 10. Phase 6 — Memory Reference Profile closure

## 目标

完成第一条真正从 Source 到 Retrieval/Use/Recovery 的纵向 reference implementation。

## 必须形成的闭环

`Artifact`
→ `ObservationOccurrence`
→ `Evidence / Provenance`
→ `Memory proposal`
→ `Authority Memory`
→ `Revision / Contradiction`
→ `Serving generation`
→ `QueryPlan`
→ `Context contribution`
→ `semantic UseEvent`
→ `lifecycle`
→ `restart/rebuild`
→ `trace-back`

## Qualification focus

必须专门覆盖：

- duplicate derived evidence independence；
- temporal update vs correction；
- stale serving generation；
- retried use event；
- source/aboutness isolation；
- suppression / restore / purge；
- restart；
- provenance trace；
- budget-limited retrieval vs unknown cognition。

这是后续 Self/Social/Motivation 的 architecture proving ground。

---

# 11. Phase 7 — Self owner

## 进入条件

Memory reference spine 已稳定：

- object/revision；
- provenance；
- cross-domain proposal；
- query；
- serving；
- lifecycle。

## 目标

实现独立 Self Authority。

至少覆盖：

- self facets；
- seed initialization；
- self revision；
- Narrative Identity references；
- query projection；
- cross-domain proposal in/out。

## 物理边界

Target Design 固定 semantic owner。

是否单独 crate/service，在 Self Spec 中根据当前 Kernel 组合成本决定。

---

# 12. Phase 8 — Social Cognition owner

## 目标

实现：

- social entity references；
- Relation Type semantics；
- directed relation cognition；
- degree vs certainty；
- LanguageConvention；
- evidence / valid-time / revision；
- social query projection。

Entity binding 必须复用 shared identity/provenance substrate。

---

# 13. Phase 9 — Motivation 与 Desired Condition

## 目标

实现：

- Desire；
- Concern；
- Desired Condition；
- bounded typed condition expression；
- temporal forms；
- coverage-aware evaluation；
- revision；
- query / context contribution。

仍不实现 Pursuit。

Pursuit 是 Heptalogos behavior owner。

---

# 14. Phase 10 — Offline Cognition 与跨领域维护

## 目标

把已有离线认知目标落实为受宿主 lease 约束的可恢复工作流。

包括：

- MaintenanceNeed；
- input snapshot；
- Hypothesis；
- counterexample search envelope；
- target-owner proposal；
- revalidation；
- preemption / cancellation。

不建立 second authority。

---

# 15. Phase 11 — Heptalogos live cognition integration

## 进入条件

至少：

- Memory；
- runtime query/context；
- stable use event；
- Desired Condition；

具备正式 contracts。

## 目标

建立：

- shared SubjectId binding；
- cognition availability/readiness；
- versioned query/context；
- revision invalidation；
- behavior outcome receipts；
- stable use event feedback；
- Desired Condition revision feed；
- purge/permission invalidation；
- maintenance lease handoff。

Nous Wave 不取得 behavior Authority。

---

# 16. Phase 12 — 高级认知与 retrieval research lanes

以下保持独立、可插拔研究：

- CognitiveSchema induction；
- Journal maintenance；
- Episode segmentation；
- topology retrieval；
- PPR；
- Wave；
- residual semantic cues；
- EPA/PCA family；
- learned fusion；
- advanced multiview；
- reranking。

只有经过 reference-profile evaluation 后显示稳定收益的机制，才进入 production/reference default。

---

# 17. 依赖图

```text
Phase 0  Documentation authority
   ↓
Phase 1  Shared semantic substrate
   ↓
Phase 2  Memory ontology
   ↓
Phase 3  Persistence + protocol
   ↓
Phase 4  Runtime/use
   ↓
Phase 5  Query/Serving
   ↓
Phase 6  Memory reference profile
   ├────────→ Phase 7 Self
   ├────────→ Phase 8 Social
   └────────→ Phase 9 Motivation
                  ↓
Phase 10 Offline cognition
                  ↓
Phase 11 Heptalogos integration

Phase 12 research lanes attach after Phase 5/6 baseline exists.
```

---

# 18. 未来 Spec 分区

不要现在创建大量空 Spec。

当本计划文档完成后，第一轮建议最多建立四份：

1. **Memory Authority & Provenance Spec**
2. **Cognitive Runtime & Use Semantics Spec**
3. **Cognitive Query & Serving Spec**
4. **Memory Reference Profile Qualification Spec**

如果第一份过大，可以把 Evidence/Provenance 单独拆出；只有实际减少重复或 owner 冲突时才拆。

Self、Social、Motivation 在进入各自 Phase 前再写独立 Specs。

---

# 19. Target Plan 完成标准

本计划本身只有在以下条件满足后才算完成其职责：

- Target Design 不再存在重大领域边界未决；
- 每个 Phase 都映射到当前真实 owner 或明确的新 semantic owner；
- 不用 Plan 伪装 Spec；
- 10/27 milestone 可以作为它的子集，不需要改写目标语义；
- 后续 Agent 能从 Phase 1/2 直接写第一批 Specs，而无需再次重做宏观架构。
