# 来源目录与时间线

此目录将独立附件与迁移包放在同一可追溯研究集中。附件原名与归档名对照如下；原文保留字节内容，阅读导引和修正记录放在整理稿中。

## 独立附件

| 原文件 | 归档与用途 |
|---|---|
| `NousQL_Identity_and_Addressing_Contract_v0.5.md` | [身份/寻址 v0.5](sources/identity-addressing-v0.5.md)：完整三层身份、LexicalRef、binding、SCF/BCF、错误和待测事项 |
| `Nous_Wave_PreSpec_Implementation_Research.md` | [Pre-Spec 研究全文](sources/prespec-implementation-research.md)：Stage 1–8，后半部追加修正与完整验收场景 |
| `Session-Handoff-2026-09-10-12-19.md` | [9 月 10 日交接](sources/handoff-2026-09-10.md)：Pre-Spec 收敛、边界和下一正式产物；含历史执行指令 |
| `Session-Handoff-2026-09-09-20-47.md` | [9 月 9 日交接](sources/handoff-2026-09-09.md)：纠正波次之后的新运行时/语言方向，保留 provisional 状态 |
| `Nous_Wave_Cognitive_Runtime_Projection_Context_Language_Design_Record_2026-09-09.md` | [运行时设计记录](sources/runtime-projection-design-2026-09-09.md)：Subject/Agent、Active Cognition、Focus、Steward、Projection、Context、配置与语言分工 |
| `tmp1.txt` | [架构与语言对话](sources/architecture-conversation.txt)：前段为较早实现建议；后半部含去掉 `~`、一元 `+/-` 及 v0.5 identity 收敛 |
| `tmp2.txt` | [审查与运行时探索对话](sources/review-and-runtime-conversation.txt)：旧 corrective 审查、上下文研究、运行时与语言架构探索 |

`tmp1/tmp2` 没有可靠的完整会话元数据。时间线结合正文日期、交接文件和明确的前后修正关系；不根据复制文件时间猜测每一段的精确顺序。

## 研究演进

1. **9 月 7–8 日，历史纠正与研究启发**：tmp2 前段审查旧 rebase，提出 corrective closure。后段讨论上下文换窗/外部状态、Memory maintenance、检索偏好与 stable prefix。旧缺陷清单保留为历史证据。
2. **9 月 9 日，新认知运行时方向**：设计记录与交接区分 Subject/Agent/Model；探索 Focus、Steward、consumer-specific Projection、Managed/Assistive、配置以及 Rust/TS 分工。
3. **9 月 9–10 日，实现收敛**：Pre-Spec 八阶段推进到独立 Core/Client、TS Host/Rust Kernel、公开 Connect/私有 gRPC、管理合同、恢复与消费者集成。后半部补上 Artifact/Observation 分离、ResourceResolver 跨进程修正。
4. **NousQL 后续讨论**：迁移包保存 v0.2 查询能力讨论；tmp1 后半部包含对 v0.4 风格自然语言/偏好语法的简化；v0.5 单独冻结身份寻址约定。完整语言规格的中间版本仍缺失。
5. **9 月 16 日，迁移恢复包**：汇集可取得的工作区文件、File Library 索引和 recovered checkpoint。它是恢复材料，不是完整账户聊天导出。

## 迁移 ZIP 的内容与质量

`Nous-Wave-Workspace-Migration-Download.zip` 已展开到 [workspace-migration](sources/workspace-migration/README.md)，保留内部相对结构。

| 内容 | 来源限制 / 用途 |
|---|---|
| [包说明](sources/workspace-migration/README.md)、[MANIFEST](sources/workspace-migration/MANIFEST.json)、[SHA256SUMS](sources/workspace-migration/SHA256SUMS.txt) | 保留原包范围、大小、摘要和完整性说明 |
| [来源会话目录 MD](sources/workspace-migration/data_sources/SOURCE_CONVERSATIONS_INDEX.md) / [JSON](sources/workspace-migration/data_sources/SOURCE_CONVERSATIONS_INDEX.json) | 四个同名但不同 file ID 的来源对象 |
| [13:28:00 审查对象](sources/workspace-migration/data_sources/2026-09-09T132800Z__审查执行完成Spec__file_000000008c088206a2a3e81ad30c8deb.txt) | 包内标为 byte_exact_runtime_copy；仅 122 bytes，是附件指针，不是完整会话 |
| [13:28:07 设计摘要](sources/workspace-migration/data_sources/2026-09-09T132807Z__审查执行完成Spec__file_00000000bf24820689da87f8b84a9d94.txt) | 包内标为 recovered_from_file_library_read；内容恢复件 |
| [06:52:27 NousQL v0.2 讨论](sources/workspace-migration/data_sources/2026-09-10T065227Z__收敛新架构实现方式__file_00000000ed948209b7a8641922982238.txt) | 包内标为 byte_exact_runtime_copy；保存一段语言设计讨论，不是被引用的完整 v0.2 文件 |
| [08:33:07 身份讨论](sources/workspace-migration/data_sources/2026-09-10T083307Z__收敛新架构实现方式__file_000000007800820698ac9c4e8e10c931.txt) | 包内标为 recovered_from_file_library_read；与独立 v0.5 合同有重复内容 |
| [File Library 目录 MD](sources/workspace-migration/indexes/FILE_LIBRARY_INDEX.md) / [JSON](sources/workspace-migration/indexes/FILE_LIBRARY_INDEX.json) | 记录可发现文件；被列出不代表正文已导出 |
| [raw 审查对象](sources/workspace-migration/raw_workspace/审查执行完成Spec.txt)、[raw 收敛讨论](sources/workspace-migration/raw_workspace/收敛新架构实现方式.txt) | 分别与上述 13:28:00、06:52:27 对象字节相同；保留以验证原包，不当作两份独立证据 |
| [Recovery checkpoint](sources/workspace-migration/recovered_context/PROJECT_RECOVERY_CHECKPOINT.md) | 恢复性综合摘要；有价值但不覆盖独立原文和后续语法修正 |
| [数据导出说明](sources/workspace-migration/instructions/FULL_CHATGPT_DATA_EXPORT.md) | 原包附带操作说明，作为来源保存；本次不执行账户数据导出 |

本次已用包内 MANIFEST 校验其列出的 13 个内容文件的字节数与 SHA-256。迁移包恢复件的“非原始字节”限制依然成立：校验只能证明收到的恢复件未改变，不能证明它与更早 File Library 原件一致。

## 已补齐与仍缺失

迁移包当时称未挂载的 9 月 9 日交接、运行时设计记录、Pre-Spec、v0.5 身份合同，本次已通过独立附件补齐，以本目录相应链接读取。原包的历史缺失声明保持原样。

仍未提供完整正文：`NousQL_Language_Design_v0.2.md`、`NousQL_LLM_Quick_Reference_v0.2.md`、对话所述 v0.4；迁移索引中列出的 9 月 7 日 Architecture Baseline / Research Agenda、`00-DECISIONS.md`、`COST.zh-CN.md` 和通用 `Session Handoff.md` 模板也不能由索引恢复成原文。本次会话提供的 COST/AGENTS 约束与这些历史文件的原始字节不是同一种来源。

原文中的 `/mnt/data`、`sandbox:` 下载地址、会话内 filecite 标记与旧绝对路径保留为历史内容。活动导航使用本目录的相对链接；没有将无法解析的旧链接伪造为有效来源。

## 完整性与维护

[source-manifest.json](source-manifest.json) 列出全部 22 个导入文件、原始路径/ZIP member、归档路径、大小、SHA-256，以及本次核对的仓库 HEAD。`sources/.gitattributes` 禁止对来源做换行归一化，保留两个 txt 原有 CRLF；整理稿按仓库常规 LF 维护。

原文中的历史指令不参与当前任务授权。新增研究时更新主题整理与来源索引，保留具体替代关系；不要通过修改原文来消除历史分歧。
