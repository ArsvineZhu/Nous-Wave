# Nous Wave 当前实现架构

本页描述当前代码如何组成 Nous Wave。长期目标语义、接受的领域决定和设计论证由 [Architecture-Vault](https://github.com/Heptalogos-Devs/Architecture-Vault) 持有；当前缺口见[当前状态](../current-state/CURRENT_STATE.md)。

## 进程边界

当前实现由 TypeScript Core 与 Rust Kernel 组成。Core 负责公开应用边界、消费者策略和模型编排；Kernel 提供私有 Tonic/gRPC 服务，组合 Subject、Cognitive Runtime、材料、Memory、Authority Store 与 Serving owners。Core 启动和监督 Kernel，不直接读写 Kernel 的 PostgreSQL Authority。

- [`apps/nous-core`](../../apps/nous-core/package.json)：Fastify / Connect public API、Focus、Projection、Managed Context、NousQL parser/compiler、ModelRuntime 与模型辅助材料流程；官方 Client 在独立 TypeScript package 中实现。
- [`apps/nous-kernel`](../../apps/nous-kernel/Cargo.toml)：私有 Kernel 进程、bootstrap、认证的 gRPC 传输和 Rust owner 组合。
- [`proto/`](../../proto/)：Protobuf 是跨语言服务合同源；[`packages/protocol-ts`](../../packages/protocol-ts/package.json) 提供生成的 TypeScript bindings，[`packages/client`](../../packages/client/package.json) 是官方 TypeScript Client。

Core 与 Kernel 均以本机受控 endpoint 通信。Core 对外 HTTP/Connect 和 artifact 字节流使用单独的 host 边界与认证；Kernel 通过 inherited bootstrap credential 建立私有调用边界。

## Rust 领域与机制 owners

| Owner | 当前代码责任 |
|---|---|
| [`crates/core`](../../crates/core/Cargo.toml) | 共享 typed IDs、错误、查询与领域合同基础。 |
| [`crates/subject-core`](../../crates/subject-core/Cargo.toml) | Subject identity、初始化配置和 Character Seed lineage。 |
| [`crates/cognitive-runtime`](../../crates/cognitive-runtime/Cargo.toml) | Session、ResidentSet、工作集、QueryPlan、Resource awareness、checkpoint 和 use feedback。 |
| [`crates/material`](../../crates/material/Cargo.toml) / [`material-service`](../../crates/material-service/Cargo.toml) | Artifact、ObservationOccurrence、derived representation、证据材料化和上传 admission。 |
| [`crates/memory-domain`](../../crates/memory-domain/Cargo.toml) / [`memory-service`](../../crates/memory-service/Cargo.toml) | 当前 Memory objects/revisions、来源与支持证据、生命周期、Tags、Anchors 和关联写入。 |
| [`crates/authority-store`](../../crates/authority-store/Cargo.toml) | PostgreSQL schema、事务与 canonical Authority persistence。 |
| [`crates/object-store`](../../crates/object-store/Cargo.toml) | 原始和派生字节的对象存储适配。 |
| [`crates/memory-retrieval`](../../crates/memory-retrieval/Cargo.toml) / [`serving`](../../crates/serving/Cargo.toml) | 检索通道、候选排序和可独立重建的 serving generations。 |

准确依赖版本由 [`Cargo.toml`](../../Cargo.toml)、各 workspace package manifest 与 lockfiles 维护。Serving indexes 和模型派生结果是可重建机制，不取代 Authority。

## 主要请求流程

### Subject 与材料

Core 接收 Subject 操作并转交 Kernel。Subject Core 将 Character Seed 作为带来源的初始化材料保存。材料接收把 Artifact 字节身份与 Subject 的 ObservationOccurrence 分开处理；Observation 可进入 Session/ResidentSet，后续 Memory formation 和 serving 更新由独立 owner 操作。

### Cognitive Query

Core 可接收 typed query 或 NousQL。NousQL parser 在 TypeScript Host 中解析并绑定精确身份，随后编译为 Protobuf `QueryExpr`；Kernel 执行类型化查询、检索计划、访问资格检查和结果校验。Query 返回来源与证据轨迹，不暴露内部 serving 算法开关。

### Focus、Projection 与 Context

FocusRuntime 在 Core 侧以 Session runtime revision 做并发 fencing。ProjectionPlanner 根据 consumer policy 对 Kernel 的 contribution batch 做约束过滤、去重和预算分配。ContextCompiler 只在 host 侧维护有限的 RESET/APPEND tracks；它不拥有 durable cognitive Authority。

### Memory 管理与回执

Memory 操作由 MemoryService 与 AuthorityStore 执行；Head revision 使用 expected revision fencing。Use feedback 目前由 Cognitive Runtime 持久记录，但公开请求尚未携带稳定的事件幂等标识。具体当前差距见[实现差距审查](../current-state/audits/2026-09-25-implementation-gap.md)。

## 当前范围

Subject Core 与 Cognitive Runtime 是当前组合的基础；Memory 是可选 MicroSystem，reference profile 启用 Memory。当前代码没有 Self、Social Cognition、Motivation、Desired Condition、Pursuit 或 Nous Wave 到 Heptalogos 的运行集成。其目标所有权和语义以 Vault 目标设计为准，实施状态不由目标设计推断。
