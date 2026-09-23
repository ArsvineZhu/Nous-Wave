# Architecture Rebase 实施规格

状态：执行中。授权：2026-09-16，完整首轮重构；可直接制定必需开放项，不保留旧接口兼容。

## 范围和所有权

实现独立 Core + 官方 Client，TypeScript Cognitive Host + Rust Cognitive Kernel。公开 Connect，私有 gRPC；Protobuf 是合同源。保留 Rust Authority、检索/Serving 与证据语义。Host 拥有 Focus 策略、Projection、Managed Context、Steward、模型调用和 NousQL；Kernel 拥有可靠持久化与 owner 校验。

GUI、远程多用户、Persona/Relationship 等未来认知域、分布式检索、内部调度器不属于本轮。原始研究材料保持历史状态。

## 本轮开放项决定

1. Node 24 LTS；pnpm workspace；版本固定在 lockfile。生成协议纳入显式 Just/pnpm 入口，不在运行时生成。
2. Core 默认仅 loopback，私有 Kernel 绑定随机 loopback 端口。启动凭据按进程生成，父子通过继承 pipe 传递和保持存活。用户指定的配置文件和数据根是发现边界；凭据不进入普通日志。
3. 无模型配置是合法可用基线。模型角色由显式配置提供，使用 AI SDK；不猜测用户凭据、不自动付费调用模型。模型不可用或 proposal 校验失败时回到确定性结果并记录降级。
4. 一个 Session 最多一个 ACTIVE Focus。所有 Focus 生命周期变更以 Session revision 为 CAS fence；checkpoint 保存有界 refs/summary，恢复重新核验引用。
5. ConsumerPolicy 使用静态配置和 REQUIRED/PREFERRED/OPTIONAL/FORBIDDEN；默认只启用已实现的 memory/runtime/resource 贡献。预算以 bytes/items 强制，tokens 只在指定估算器时强制。
6. Managed Context 只支持 RESET/APPEND。track 按 Subject/Session/Focus/Consumer 隔离；源修订、策略或渲染语义变化 RESET；重启可以安全 RESET。
7. NousQL 使用裸字符串 semantic atom、一元软偏好 +/-、显式 &&/||、括号 scope；不接受旧 ~ 前缀，不提供兼容 parser。统一时间 constructor 为 `$time(axis,...)`；Query modifiers 绑定所在表达式节点，不能跨 scope canonicalize。
8. Identity v0.5：四个随机词、冻结 4096 词表、部署范围 UNIQUE 与重试、永久 tombstone；内部 UUIDv7 或外部规范 ref。名称 exact binding，歧义返回候选，不用向量猜测。词表版本作为持久化编码合同；实测与未知的模型 tokenizer 表现分开报告。
9. Observation request_id 是可选 durable 幂等键，绑定 canonical request digest，与 occurrence 同事务提交；Memory head mutation 需要 revision fence。Artifact upload 与 Observation 分开。
10. Resource Awareness 返回 action suggestions；外部访问属于调用方/Host，不建立 Kernel 回调。公开 consumer 查询不暴露 provider/算法开关。
11. 公开列表采用有界 keyset page token，并绑定 filter/sort；取消/deadline 不推断 mutation 是否提交。模型/schema 校验之外仍由 Kernel 验证 Subject、证据和修订。

## 执行顺序与证明

2026-09-17 补充：用户要求参考“重新复述结果”中的审查继续开发。经源码核对，将 [Memory 语义合同](MEMORY.md) 纳入当前执行范围；先修正 Authority 时间、修订、实体关联、无定义数值、拓扑身份和可访问性，再进行最终验收。既有 Core/Client 和 Host/Kernel 工作继续保留。

- 协议、工具链、Kernel/Core 进程及认证链。
- 管理与材料操作 parity、请求幂等与并发 fencing；随后删除旧公开入口。
- Focus/checkpoint、确定性 Projection、Context、身份绑定与 NousQL。
- ModelRuntime/Steward 和派生材料的单向调用；无模型降级。
- 端到端 Client→Core→Kernel→真实 PostgreSQL 验收、文档更新及 `just verify`。

测试优先保护跨进程边界、并发、重试、scope、来源和失败行为。现场没有外部模型凭据时只报告受控模型契约测试，不宣称真实 provider 验收。当前机器之外的平台不宣称实测。

## 完成条件

全部上述能力有实际调用路径，无兼容旧入口，无假成功、占位实现或未执行却标 PASS 的证明。完成后将稳定决定和行为写回架构/系统文档，删除本临时执行规格。
