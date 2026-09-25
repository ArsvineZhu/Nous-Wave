# Nous Wave 当前实现状态

代码基线：`e9af4ebd8a77d316c78764fd85836e6de8d9b7ee`。文档重组分支只修改文档；本页描述该代码基线中的实现事实。

## 当前系统

当前实现由 TypeScript Core、Rust Kernel、Protobuf contracts 和官方 TypeScript Client 组成。Subject Core、Cognitive Runtime、材料/证据、Memory、Authority Store、对象存储、检索与 Serving 分别由 workspace 中的 Rust owners 维护。Memory 在 Runtime composition 中可选，并在 reference profile 启用。

TypeScript Core 已实现 Session/Cognition/Memory/Material 服务路由、Focus 状态管理、consumer Projection、Managed Context、模型角色路由、artifact stream 和 NousQL parser/compiler。Rust Kernel 持有 PostgreSQL Authority 操作、材料、Memory、查询与 Serving owners。当前模块图见[实现架构](../architecture/current-implementation.md)。

## 已有能力

- Subject 与 Character Seed 初始化；Artifact 与 ObservationOccurrence 分开写入，Observation `request_id` 已有规范摘要绑定和重试幂等处理。
- Session、ResidentSet、Focus、ConsumerWorkingSet、ContextContribution 与 RESET/APPEND Context tracks。
- Memory 对象与不可变 revision、来源 Evidence、formation/revision/consolidation、抑制/恢复/清除、Tag/Anchor/Association 管理。
- Query 支持 exact/entity/lexical/dense/resource/topology 等候选族；Serving generation 独立构建、发布与恢复。
- NousQL 在 TypeScript Core 中解析、身份绑定并编译为 Protobuf `QueryExpr`；当前支持子集见[NousQL 参考](../reference/NOUSQL.md)。

## 尚未实现的目标领域

当前代码没有 Self、Social Cognition、Motivation、Desired Condition、Pursuit 或 Heptalogos live cognition integration。Character Seed 是 Subject 初始化来源，不是 Self Profile 或 Memory。产品目标与未决跨系统语义由 [Architecture-Vault](https://github.com/Heptalogos-Devs/Architecture-Vault)维护。

## 主要语义差距

- Memory 仍使用 `Specific / Integrative / Procedural` 单轴分类。
- `Anchor` 仍是持久 Authority object；目标设计要求 Memory-owned CognitiveSchema 与可重建 AssociativeTopology 分离。
- 关联证据 `support_value` 仍为通用 `f64`。
- `ReportUseRequest` 没有调用方稳定事件 ID；服务为事件内部生成 UUID，重试可能重复记账。
- AUTO accessibility 默认由 90/365 天阈值分档。
- 候选排序的融合分母根据当前候选集合启用的证据族计算，并另有相近分数候选间的 topology innovation。

差距和对应代码路径见[实现差距审查](audits/2026-09-25-implementation-gap.md)。下一实施范围由[2026-10-27 Memory Reference Profile Plan](../plans/active/2026-10-27-memory-reference-profile.md)授权。

## 2026-09-25 文档审查验证

- 源码与协议核对：完成；以本页及差距审查中列出的路径为范围。
- 本地 Markdown 相对链接检查：PASS，0 个失效链接。
- `corepack pnpm check`：FAIL，`buf lint` 在未由本次文档分支修改的 `proto/grpc/health/v1/health.proto` 上报告标准 Health proto 的 enum/service 命名规则冲突；由于命令链在 Proto 检查终止，TypeScript typecheck 与 tests 为 NOT_RUN。
- `just verify`：FAIL，首项 `cargo fmt --all -- --check` 在未由本次文档分支修改的 Rust 源文件报告格式差异；后续 check、Clippy、tests、deny、cargo-shear 与 source-shape tasks 未运行。
