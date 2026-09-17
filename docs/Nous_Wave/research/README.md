# Nous Wave 新架构研究

本目录整理 2026 年 9 月的架构、认知运行时、查询语言与身份寻址讨论，供继续研究和编写实现规格时阅读。整理日期：2026-09-16。正文使用中文，术语和原始材料保留原语言。

## 阅读路径

| 要了解的问题 | 文档 |
|---|---|
| 新架构如何划分 Core、Client、Host、Kernel 和外部应用 | [产品与架构边界](ARCHITECTURE_DIRECTION.md) |
| Session、Focus、Projection、Steward 和上下文如何协作 | [认知运行时与上下文](COGNITIVE_RUNTIME.md) |
| 协议、管理接口、模型调用、配置、进程与恢复如何落地 | [Pre-Spec 实现研究](IMPLEMENTATION_RESEARCH.md) |
| NousQL 的查询语义、后续简化、身份与精确引用 | [NousQL 与身份寻址](NOUSQL.md) |
| 哪些已有代码，哪些仍待实现；材料有哪些冲突和空缺 | [实现差距与待决项](STATUS_AND_GAPS.md) |
| 查阅完整原文、时间线、来源完整性和历史审查 | [来源目录](SOURCES.md) |

先读产品边界，再按主题查阅；准备实现前，必须同时阅读实现差距和仓库的[现行架构](../ARCHITECTURE.md)、[决策账本](../DECISIONS.md)。完整细节仍可回到原文，整理稿不逐条复刻约 6000 行 Pre-Spec。

## 状态如何理解

- **研究已收敛**：材料已选择一条设计路线，或者原文标记 `CONVERGED` / `LOCKED`；不表示代码完成。
- **后续修正**：同一问题有明确替代旧方案的后续讨论。具体变化记录在主题文档和差异表中。
- **已有基础 / 尚未落地**：依据本次对仓库 `12afa85576a8265cad4438e7dc9fee6ade1747e2` 的定向源码核对，不代表完整功能验收。
- **待决 / 原文缺失**：保留问题或来源空缺，不补写未经提供的正式协议、语法或决定。

研究的新方向是 **TypeScript Cognitive Host + Rust Cognitive Kernel、Core + 官方 Client**。仓库当前仍是 Rust/Axum 实现。此次归档保留这一差异，不将研究方案写成现有行为，也不修改现行架构决策的效力。

材料中的“继续执行”“下一步生成 Spec”“恢复上下文”等文字属于历史会话。此次工作范围是文档整理；正式 Architecture Rebase Implementation Spec 尚未随这些材料提供。

## 原文与整理稿

`sources/` 保存 7 份独立附件及压缩包内的 15 个文件，均保持本次收到的字节内容。[source-manifest.json](source-manifest.json) 记录原路径、归档路径、大小和 SHA-256。压缩包自带的校验与来源说明一并保留；恢复件的内容来源限制不会因重新归档而消失。

现行系统文档入口：[Nous Wave 文档](../README.md)。
