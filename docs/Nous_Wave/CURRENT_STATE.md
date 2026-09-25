# 当前实现状态

基线：`e9af4ebd8a77d316c78764fd85836e6de8d9b7ee`。

本文只描述代码现状。目标语义以 Architecture-Vault 的 Nous Wave 目标设计为准。

## 已存在的工程基础

仓库已经形成 TypeScript Host 与 Rust Kernel 的组合。Rust workspace 包含 `subject-core`、`authority-store`、`material`、`material-service`、`memory-domain`、`memory-service`、`memory-retrieval`、`cognitive-runtime`、`serving`、`object-store`、`protocol` 等模块。

当前实现已经具备 Subject 基础身份、材料与观察相关结构、Memory 对象和不可变修订、证据引用、修订关系、Tag、部分关联结构、Memory 服务、查询与服务投影、认知运行时反馈等能力。

最近一轮代码已经把部分旧问题改为更明确的语义，例如修订生命周期与矛盾关系分离，并保存独立的证据、时间和实体指向信息。

## 与目标设计仍有差距的部分

### Memory 模型

当前 `memory-domain` 仍使用 `Specific / Integrative / Procedural` 的 `MemoryClass`。目标设计已经将认知角色和形成方式分别表示。

当前代码仍包含 `Anchor`、`AnchorRevision` 和 `AnchorSupport`。目标设计使用认知图式表达长期结构性认知，联想拓扑只作为可重建服务投影。

当前 `AssociationEvidence` 仍保存通用 `support_value: f64`。该字段尚未证明具有稳定的认知含义。

### 可访问性与使用反馈

当前实现仍含具体时间分档和现有使用反馈路径。目标设计只固定可访问性、相关性、抑制和清除的语义区分；默认函数和参数仍需实验。

长期使用事件需要稳定的调用方事件身份，避免网络重试形成重复认知使用。当前公开接口仍需按后续规格核对并收敛。

### 检索

当前代码已经存在词项、稠密、关联等服务机制和多种实验算法。它们属于当前实现与研究资产。目标设计要求查询计划在候选产生前固定、多个服务视图先聚合到对象修订、拓扑检索作为普通检索通道参与融合。

EPA、Residual、Wave 等实现不能因为已经存在而成为默认长期算法。算法选择在语义合同稳定后通过实验决定。

### 尚未实现的主要认知领域

目标设计中的 Self、Social Cognition、Motivation、Episode、Journal、CognitiveSchema 完整生命周期和 Offline Cognition 尚未形成对应的完整代码系统。

10 月 27 日前的工程重点是 Memory Reference Profile；Self 只需要提供静态、只读的结构化档案入口，不在本阶段实现自动演化。

## 当前工程原则

现有实现可复用的基础设施应继续使用。语义与目标设计冲突的类型和流程直接修改，不维护开发阶段兼容层。新的实现工作必须由当前 Plan 和后续可执行规格授权。
