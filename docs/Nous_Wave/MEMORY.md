# Memory 语义合同

状态：2026-09-17 执行基线，纳入本轮 Architecture Rebase；实现和验收仍在进行。

## 范围

Memory 负责经历保存、记忆形成、演化、拓扑、回忆可访问性、证据来源和生命周期。长期记忆保留 Specific/Episodic、Integrative、Procedural/Experience 三类；Procedural 记录经验性做法，不提交行为策略。

Self/Persona、Social/Relationship、Affect、Motivation、群体语言惯例以及独立信念系统不属于 Memory Authority。它们可通过明确引用、证据和查询线索协作；最终产品、仓库或进程归属保持 OPEN。当前不改变 Core/Client、Host/Kernel 的重构方向。

## 对象与修订

- 同一 remembered cognition 的修正、重述、重新解释：同一 Memory 的新 revision，必须提供预期 head revision 和明确修订意图。
- 新的独立经历：新 Specific Memory。
- 新认知与旧认知冲突：保留两者，以不可变 revision refs 建立 `Contradicts` 关系。
- 整合多个来源：默认新 Integrative Memory，以 `Integrates` 指向来源 revisions。
- 提取经验做法：默认新 Procedural Memory，以 `Proceduralizes` 指向来源 revisions。

RevisionLifecycle 仅表示 Current、Superseded、Revoked。Contradiction 属于 revision/evidence 关系，不能使另一个对象的当前 revision 自动过时。普通同对象修订使用 Supersedes 或显式撤回意图；跨对象关系有独立操作，不能通过修改历史叙述伪造新经历。

## 时间与证据

ObservationOccurrence 拥有事件发生时间 `occurred_at` 和 Subject 接触时间 `observed_at`。MemoryRevision 拥有形成时间 `created_at` 以及可选 claim validity。Memory 不拥有一个人为挑选的 observation 时间。

Memory 的 occurred/observed 查询沿 evidence lineage 解析 ObservationOccurrence。Occurrence evidence 指向一次确切 encounter；SourceRegion/DerivedRepresentation/DerivedRegion evidence 沿其 Artifact 查找关联 encounters。查询两条轴时要求同一个 encounter 同时满足；未知时间不命中有界时间条件。没有 encounter 的 source evidence 不凭空生成时间。

读取模型可展示派生的 observed/occurred min/max，但这些范围不作为 Memory Authority 写入。时间过滤检查真实 evidence rows，不能用 min/max 跨过证据间的空白区间制造命中。

删除通用 Memory confidence 和 evidence weight。保留 EpistemicClass、SupportRole、ProducerSignature、关系与来源；具体模型的质量指标只在派生表示的质量元数据中解释。

EntityMention 仅记录真实 source mention。MemoryRevision 与 EntityRef 的“涉及”关系由独立 `memory_revision_entities` 保存，包含 role/provenance；不得把它附着到任意一条证据上。修订需要明确给出保留的实体关联，不能从文字或第一条 evidence 猜测。

## 可访问性与遗忘

可访问性是回忆资格，与内容真实性、相关性排序、suppression、purge 和 Session eviction 分开。Memory 保存显式策略 `AUTO / NORMAL / DEEP / EXPLICIT`；AUTO 的实际等级在查询时派生，无后台定时修改。

默认运行策略：以 Memory 创建时间与该 Memory 或其 revision 的最后 meaningful-use 时间中较晚者为起点；90 天内 NORMAL，90–365 天 DEEP，超过 365 天 EXPLICIT。阈值是可配置的产品运行默认值，不是认知科学定律或置信概率。不同算法可替换该派生策略，不能改变下表的资格语义。

| 等级 | 普通回忆资格 |
|---|---|
| NORMAL | light/normal/deep/maximum 均可进入候选 |
| DEEP | deep/maximum，或确切 Memory/Revision 引用，或直接 Entity/Tag/Anchor 接触 |
| EXPLICIT | 仅确切 Memory/Revision 引用；管理读取仍可访问 |

“直接接触”依据已绑定身份和已有拓扑证据，不使用一个新造的 cue-strength 分数。纯软偏好不得放宽硬资格。ResidentSet 中存在引用也不会绕过可访问性或 suppression。

Surfaced、Inspected、Selected、Exposed 不更新 meaningful-use 起点；Referenced、ActedOn、Corroborated、Corrected、Pinned 是已定义的 meaningful-use 信号。它们影响后续可访问性，不增加真值 confidence。明确固定 NORMAL 可表达持续可访问的管理意图。

Suppressed 不参与普通 cognition；显式管理读取和单独声明的诊断查询可检查它。EXPLICIT 仍可精确回忆；purge 删除 Authority；runtime eviction 只移出当前工作集。

## 拓扑与 consolidation

Tag proposal 明确选择已有 TagId 或创建新 Tag；同名只产生 resolver 候选，绝不自动合并身份。

AssociationEvidence 的 `support_value` 定义为单条证据的 normalized contribution，范围 `[0,1]`。它不是 truth probability、主观重要性或最终 graph edge weight；Serving 使用保留证据类型的确定性规则聚合。

Consolidation 的输入是 immutable MemoryRevision refs，默认输出新 Memory，保留所有支持和矛盾证据。修订已有 Memory 必须显式指定目标、预期 head revision 和 intent，在同一事务中校验。未经确认的 consolidation Anchor 仍需至少两个独立来源根。

## 必需证明

验收覆盖：矛盾与 current 共存；多个 encounter 的时间过滤；MemoryEntityLink 不增加 source mentions；同名 Tag 不合并；非法 association contribution 拒绝；stale consolidation revision 回滚；可访问性三等级的精确/普通/deep 行为；曝光不刷新可访问性；meaningful use 不改变证据或真假；suppression、purge、eviction 互不混淆。
