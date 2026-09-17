# 实现差距与待决项

核对日期：2026-09-16；仓库 HEAD：`12afa85576a8265cad4438e7dc9fee6ade1747e2`。本次仅对研究涉及的工程结构、类型、操作和 schema 作定向核对，未运行新架构验收，也未重新审计全部检索行为。

## 研究与实现的对应

| 研究主题 | 本次源码可确认的状态 | 差距 / 后续意义 |
|---|---|---|
| Core + Client / TS Host + Rust Kernel | [Cargo workspace](../../../Cargo.toml) 仍以 `apps/nous-wave` 和 Rust crates 组成；[应用](../../../apps/nous-wave/src/lib.rs) 静态组合 owners | 未见 `apps/nous-core`、`apps/nous-kernel`、TS Client、proto 或 Buf 工程 |
| 公开 Connect、私有 gRPC | [HTTP router](../../../apps/nous-wave/src/http.rs) 是 Axum `/v1` 路由 | 新协议和监督进程尚未落地 |
| UUIDv7 与外部规范引用 | [core types](../../../crates/core/src/lib.rs) 已使用 `Uuid::now_v7()`；EntityRef/ResourceRef 为 Host-owned opaque string | v0.5 的内部身份基础已有；LexicalRef 词表、铸造、binding/tombstone 和 binder 尚未出现 |
| Artifact identity / digest 分离 | [Material](../../../crates/material/src/lib.rs) 与 [Authority schema](../../../crates/authority-store/migrations/202609070001_authority.sql) 已分开对象身份和内容哈希 | 不能把这一基础能力标成全部待实现 |
| Artifact upload 与 Observation 分离 | [ingest_stream](../../../crates/material-service/src/observation.rs) 保存 Artifact 后仍调用 `record_observation` | Pre-Spec 后续修正要求将公开字节上传与 encounter admission 解耦 |
| Observation 请求幂等 | [ObservationInput](../../../crates/material-service/src/types.rs)、record_observation 和当前 SQL 未见 `request_id` binding 机制 | 新跨系统重试合同尚未落地；同 bytes 去重不等于同一次 encounter 幂等 |
| Session / ResidentSet / use | [runtime types](../../../crates/cognitive-runtime/src/types.rs)、[sessions](../../../crates/cognitive-runtime/src/sessions.rs) 和 SQL 已存在 | 有可复用连续性基础；Focus 和 runtime checkpoint CAS 目标尚未出现 |
| Consumer-specific working set | [working_set](../../../crates/cognitive-runtime/src/working_set.rs) 已有 ConsumerProfile、items/bytes/modality/materialization 和结果结构 | 尚未升级为研究中的语义 ConsumerPolicy、typed contributor 与 Host Projection Runtime |
| Managed Context / Steward | 在 `apps/`、`crates/` 中未找到相关目标实现 | RESET/APPEND、consumer context tracks、Steward proposal engine 仍属研究 |
| ModelRuntime | [RuntimeOptions/ProviderConfig](../../../apps/nous-wave/src/lib.rs) 和 [Serving provider](../../../crates/serving/src/provider.rs) 仍是 Rust 组合 | TS 单一模型调用层、Host-supplied query embedding 与新 proposal 路径待实现 |
| Resource Awareness | [resources](../../../crates/cognitive-runtime/src/resources/mod.rs) 已有 descriptor/action 机制；[runtime service](../../../crates/cognitive-runtime/src/lib.rs) 仍有进程内 ResourceResolver map | descriptor/awareness 基础保留；跨进程调用方向及公开 resolver_key 边界待迁移 |
| 使用反馈 | [UseKind](../../../crates/cognitive-runtime/src/types.rs) 已分 Surfaced/Selected/Referenced/ActedOn 等，Selected 不属于 meaningful use | 新研究明确 EXPOSED 阶段，当前 enum 未包含该项 |
| Memory 管理 | [HTTP router](../../../apps/nous-wave/src/http.rs) 已有详情、revision/history、form/consolidate/suppress/restore/purge 等操作 | 不等于完整管理 parity；统一公开分页、etag 和 typed error contract 仍需新规格 |
| 检索、Serving 和证据 | [QueryPlan](../../../crates/cognitive-runtime/src/query/plan.rs)、[Memory query](../../../crates/memory-service/src/query.rs)、[Serving](../../../crates/serving/src/lib.rs) 已有独立模块 | 本轮语言/进程研究要求保留和适配这些 owners，不能笼统当作重写对象 |

“未见”限定于本次 checked-out 源码与 schema；不推断其他分支、外部仓库或未提供文件的实现状态。已有测试不在此次重跑，历史 PASS 不转换为本次 PASS。

## 需要显式处理的记录差异

| 差异 | 整理结果 |
|---|---|
| 现行 `DECISIONS D12` 为 Rust runtime/core LOCKED，现行物理架构为 modular monolith/static composition | 新 Pre-Spec 提议后续正式 rebase 为 polyglot；此次归档保留两者状态，未直接重写现行 Authority |
| 恢复 checkpoint 称旧 Rust-only 决策已被 supersede | 视为研究状态恢复描述；Pre-Spec §8.43 则把仓库真值更新列为下一 Spec 的工作 |
| 早期 tmp1 推荐嵌入 Host 与 connect-rust | 后续 Pre-Spec 采用独立 Core/Client、公开 Connect、私有 gRPC/Tonic |
| Stage 5 上传流程文字仍把 Artifact/Observation 并提 | 采用原文后半部 §8.2 的显式分离修正 |
| Rust 进程内 resolver 可直接访问外部源 | 后半部 §8.3 改为 Kernel suggestion + Host/consumer 实际访问；无反向 RPC |
| 恢复摘要保留 NousQL `~`/`+(...)` | tmp1 后半部有明确语法简化；[NousQL](NOUSQL.md) 按局部演进记录，不合成缺失的全版本 grammar |
| tmp2 前段列出旧 corrective-closure P0/P1 | 属于当时审查，不自动重开为当前缺陷；后续 handoff 记录该纠正波次已结束 |

## 尚未提供或尚未定案

- **正式执行规格**：未提供 Nous Wave Architecture Rebase Implementation Spec，也未提供 Heptalogos integration Spec。
- **语言规格**：缺完整 NousQL Language Design v0.2、Quick Reference v0.2、对话提到的 v0.4 正文；v0.5 已提供的范围是身份与寻址。canonical syntax、scope 和复杂组合需规格化。
- **词表和实验**：4096 词表实体、跨 tokenizer 测量、复制错误与 checksum 资格、LLM few-shot/语法压力测试结果未提供。
- **执行前刷新**：实际 Node/TS/AI SDK/Buf/Tonic 及生成插件版本、跨语言互操作与跨平台进程行为要由实施验证。
- **物理细节**：本地 discovery record/credential 的平台路径、具体 model binding/token estimator、runtime/checkpoint payload 与 proto 精确字段待执行规格落细。
- **未来产品责任**：GUI 框架、远程 principal/RBAC、多前台 Focus、自动 Focus 分段、配置热改、自动模型路由、resumable upload、live push、分布式 Serving、最终安装器、跨产品备份恢复、Persona/Relationship 等保持 OPEN/FUTURE。

Pre-Spec 的“已收敛”意为当时认为可进入规格编写，不等于所有具体字段、依赖 pin、实现证明已经完成。新的身份/语法材料还需要与该规格接合。

## 历史研究的证据边界

tmp2 涉及上下文换窗、外部状态、稳定 prefix、Dreaming、检索计划等研究启发，其中也包含第三方产品描述、版本、性能数字及会话内部 citation 标记。它们保留在原文；此次未在线复核，整理稿不将其用作当前产品事实或性能依据。

本次确认的是文件归档完整性、文档路径、研究阶段差异和上述源码对应。新协议、运行时、语言和跨仓库验收均为 **NOT_RUN**。后续何时执行以新的明确任务范围为准。
