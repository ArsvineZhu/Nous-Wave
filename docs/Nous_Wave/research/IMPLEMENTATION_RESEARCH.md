# Pre-Spec 实现研究

状态：原研究标记 Stage 1–8 为 `CONVERGED`，尚未形成随本批材料提供的正式执行 Spec。这里整理的是拟议实现与验收，不是已发布 API。主要来源：[Pre-Spec 全文](sources/prespec-implementation-research.md)及[收敛交接](sources/handoff-2026-09-10.md)。

## 八阶段研究地图

| 原文章节 | 核心问题 | 本目录归属 |
|---|---|---|
| Stage 1 | 协议、进程、Core/Client 主干 | 本文、[架构方向](ARCHITECTURE_DIRECTION.md) |
| Stage 2 | 管理合同、官方 Client | 本文 |
| Stage 3 | Session、Focus、Projection | [认知运行时](COGNITIVE_RUNTIME.md) |
| Stage 4 | ModelRuntime、Steward、Managed Context | 本文、认知运行时 |
| Stage 5 | 配置、存储、恢复、Artifact 传输 | 本文 |
| Stage 6 | Heptalogos 参考消费者边界 | 架构方向 |
| Stage 7 | 工程拓扑、生成、迁移 | 本文 |
| Stage 8 及后续同编号补充 | 最终修正、服务清单、验收和剩余 OPEN | 本文、[实现差距](STATUS_AND_GAPS.md) |

原文 Stage 8 后追加了另一组 §8.1–8.46；后半部明确修正 Artifact/Observation 与跨进程 ResourceResolver。引用时应同时核对节标题，避免只按编号定位。

## 协议与工具路线

研究选择 Protobuf + Buf 为统一协议源，公开 Connect 面向 Node/浏览器 Client，Host→Kernel 使用标准 gRPC/Tonic。Fastify 承载公开服务和字节数据面；生成代码经 adapter 映射到领域类型。

拟议工程布局：

```text
apps/nous-core/                      TypeScript Host
apps/nous-kernel/                    Rust 应用入口
packages/client/                    官方领域化 Client
packages/protocol-ts/                TS 生成协议
proto/nous/wave/v1alpha1/            公开协议
proto/nous/wave/kernel/v1alpha1/     私有协议
crates/                             继续使用既有 Rust owners
```

研究中的 TypeScript/Node、AI SDK、Buf/生成插件等版本与稳定性说法属于当时依赖调查。正式执行前仍需重新核对兼容版本和生成链；本次归档未重新作依赖选型或在线验证。不将实验版本或“conformance 已通过”的历史描述提升为当前资格证明。

## 管理合同

GUI-complete 的要求是通过官方 Client 完成管理闭环，当前波次不要求实际 GUI。浏览列表与 cognitive search 分开：列表使用明确 filter/sort 和分页；认知查询遵循 cue、effort、证据与来源语义。

| 服务 | 研究确定的能力范围 |
|---|---|
| SubjectService | Create/Get/ListSubject，读取和修订 Character Seed；未顺带定义 Subject purge |
| CognitionService | Session 生命周期、Focus 生命周期与切换、RecordObservation、Query、BuildProjection、ReportUse |
| MemoryService | 有界列表、详情、revision/history、Form/Revise/Consolidate、Suppress/Restore/Purge |
| MaterialService | Artifact/Occurrence/SourceRegion/DerivedRepresentation 元数据、Evidence materialization；大字节另走 stream |
| ResourceService | ResourceDescriptor 的 List/Get/Register/Update/Remove；不包含通用可执行 resolver 注册 |
| TopologyService | Tag、Anchor、Association、Entity rebind 与有界 Neighborhood；不公开内部 CSR 图 |
| SystemService | Status、Capabilities、脱敏 EffectiveConfig、各 Serving family 的 ProjectionStatus |

公开字段保持 typed；时间用 Timestamp/Duration，标量缺失需要 presence。`Struct` 只用于明确非核心 metadata，`Any` 不作为通用认知对象容器。Memory revision 是语义操作，不能用一个通用 PATCH 掩盖证据和生命周期约束。

分页使用 `page_size/page_token -> items/next_page_token`；token 对消费者 opaque，并绑定同一 effective filter/sort。Memory summary、详情、revision、evidence 分层读取，避免详情页无界展开与 N+1。

公开边界可用 Protovalidate 处理 schema 约束；Kernel 再校验 Subject、引用归属、证据、修订、producer/embedding-space 和生命周期。传输校验不能替代认知 owner 的校验。

错误使用 Connect/gRPC canonical code 与 typed details：无效参数、未找到、版本冲突、前置条件不满足、能力不可用、资源超限各有可辨认结果。Client 保留 code/details/cause，GUI 不解析错误文案。

## 并发、幂等和失败

Memory 的 immutable revision 与 mutable head 分开。目标 head revision/opaque etag 在修订、抑制、恢复、清除等状态变化时提供 fence，防止并发覆盖及 suppress→restore 的 ABA。过期 etag 返回 `ABORTED`，调用者重新读取。

Observation 的 `request_id` 是首个要求 durable 幂等的操作：

| 请求情况 | 目标行为 |
|---|---|
| 同 request_id、同 canonical request digest | 返回同一或等价 AcceptedObservation，仅一个 Occurrence |
| 同 request_id、不同内容 | 结构化冲突 |
| 不同 request_id、同 external_object_ref | 允许形成两次真实 encounter |
| 不提供 request_id | 允许普通非幂等语义 |

request binding 与 Occurrence 原子提交；不把 external object identity 当请求去重键。其他 mutation 按 owner 的 revision/refetch/reconcile 规则处理，不默认自动重试。取消或 deadline 只能说明调用停止等待，不能证明服务端未提交。

GUI 手工纠正同样要形成带 actor/source provenance 的显式管理证据，再进行 Memory revision；不能因为来源是人工界面就绕过证据规则。

## 材料与 Observation 的最终修正

原文后半部 §8.2 明确收紧语义：

```text
UploadArtifact -> 持久化字节和 Artifact 元数据
RecordObservation(artifact_ref) -> 新建 Subject encounter
observeFile -> Client 便利组合
```

上传成功、Observation 失败时，Artifact 已存在，调用者可以单独重试 Observation。不能隐式伪造一个跨操作回滚。

目标字节通道采用公开 HTTP multipart streaming → Host 有界转发 → 私有 gRPC streaming → Kernel/OpenDAL。上传大小在各边界限制；下载使用标准 HTTP Range 与对象存储 range read。大媒体不放入 unary protobuf，也不在 Node 一次性缓冲。

首轮不引入分片重传或 resumable upload 平台；有实际需求时再选择成熟机制。字节 HTTP endpoint 不演变为第二套 Memory/Query REST API。

## Resource 的跨进程修正

原文后半部 §8.3 指出，现有 Rust `ResourceResolver` trait-object map 不能原样跨进程。目标为：

- Kernel 保存 ResourceDescriptor、覆盖/时效/代价/ready 信息及本地关联，返回有界 `ResourceActionSuggestion`。
- Host/消费者在具有真实 resolver 时访问外部规范事实，再经正常证据/Observation/Situation 合同回传。
- `resolver_key` 不进入稳定公开认知语义；第一轮不添加反向 RPC 或通用 provider 框架。
- 要求 current authority 而数据未取得时，可以返回本地认知与 `PARTIAL`，并说明尚需执行的 Resource action；不能暗示外部查询已完成。

## 单一 ModelRuntime

研究选择 TS AI SDK Core 承担模型/Provider 通用机制，Nous 只增加认知角色绑定和 readiness。Gateway 是可选 route；Steward、projection synthesis、formation、embedding 等分别报告可用性。

内部结构化输出使用成熟 schema validation（研究提及 Zod/`Output.object`），经语义和引用校验后才可进入 owner operation。模型输出 schema 比完整 Authority mutation 权限更窄。

查询 dense cue 由 Host 在需要且能力可用时生成，携带 producer 与 embedding-space identity 一起提交给 Kernel；Rust 保留检索计划、执行与证据族排序。索引 embedding/多模态 interpretation 同样由 Host 读取已有 coverage/derivation need、调用模型、回传派生结果；控制方向保持 Host→Kernel。

`ConsiderSpecific` formation 在 Observation 已持久化后请求模型 proposal，再由 Kernel 接受或拒绝。模型失败不撤销 encounter；确定性的显式 formation 可以保留 owner 内部操作。Rust 原 provider 路径在新路径完成后移除，避免长期双实现。

## 配置、启动和恢复

BootstrapConfig 只覆盖启动前必须知道的路径、端点、数据根、数据库模式和发现凭据。ResolvedRuntimeConfig 按 owner 划分模型绑定、consumer policies、投影预算、Steward、Session、Serving、材料和维护限制。

研究提出配置优先级：typed defaults < 显式 TOML < 环境变量白名单 < 有真实需求的 CLI/bootstrap override。默认通过重启生效。EffectiveConfig 输出来源、owner、可见/可编辑性和 restart_required，敏感值脱敏。

Kernel 唯一访问 PostgreSQL、对象库、Serving 文件和 checkpoint 存储；保留 managed/external PostgreSQL 模式。Host 不直接读写 Kernel 数据库。

Core 启动 Kernel，使用有界 bootstrap 输出发现端点，再通过 gRPC Health 检查服务 ready。继承的 liveness pipe 在父进程退出时 EOF，Kernel 开始有界关闭。Kernel 意外退出时 Core 报 `UNAVAILABLE`，可通过完整重启恢复；第一轮不要求自动 crash-loop supervisor。

本地公开端点拟限制 loopback，使用本地 bearer credential 和受保护 discovery record；默认拒绝任意跨域。私有 Kernel 也需要启动范围内的认证边界。确切跨平台 credential/discovery 路径仍需实施时明确；远程 OAuth/RBAC 不在此轮。

Serving family 的 ready/stale/building/unavailable 与 Authority 状态分开。Host 重启本身不要求重建所有 Serving；provider 变化按 producer/space 影响定向失效。

## 实现与验收的组织

原文 §8.17 的 A–E 依赖顺序是：A 协议与进程主干 → B 公开 Core/Client 管理 parity → C 移除被替代的公开/provider 入口 → D Focus、checkpoint、确定性 Projection 和 Managed Context → E ModelRuntime、Steward 与模型增强。移除重复 provider 路径仍以新路径 parity 为前提；此处记录研究顺序，不新增阶段或授权执行。

关键证明包括：无模型管理闭环、Artifact/Observation 分离、Observation 重试只产生一个 occurrence、Memory 过期 etag 拒绝、Focus CAS、非法 proposal 拒绝、RESET/APPEND、Kernel 父进程退出、能力降级、浏览器 Client 运输，以及源码之外的可执行路径配置。

Heptalogos 接入应由其后续独立 Spec 承接。生产安装包、远程多用户、最终 GUI 和未来认知领域不因归档而进入实现范围。
