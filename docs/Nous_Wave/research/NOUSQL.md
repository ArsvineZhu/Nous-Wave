# NousQL 与身份寻址

状态：查询语言仍是设计材料；v0.5 已单独收敛身份与寻址合同。仓库尚无 NousQL parser/binder。语法的完整 v0.2/v0.4 正文和 Quick Reference 未随材料提供，本文不补写正式 grammar。

来源：[v0.2 讨论](sources/workspace-migration/data_sources/2026-09-10T065227Z__收敛新架构实现方式__file_00000000ed948209b7a8641922982238.txt)、[后续语法简化及身份讨论](sources/architecture-conversation.txt)后半部、[v0.5 身份原文](sources/identity-addressing-v0.5.md)。

## 语言边界

NousQL 是面向 Agent 的 Cognitive Query 语言，编译成 typed intent。Observation、上传、Memory mutation、Session/Focus 生命周期、ReportUse、配置修改继续使用 typed Client API。

目标路径：

```text
NousQL source
-> TS parser / source canonicalization
-> 绑定名称与引用 / bound canonicalization
-> typed CognitiveQuery / QueryExpr
-> Host 向 Kernel 提交规范身份与查询语义
```

Kernel 保留检索执行，不解析 NousQL 源文本或 LexicalRef。语言表达认知需求、约束和 effort；不公开 `$useDense`、`$wave` 等物理检索开关。

## 明确的语法演进

| 问题 | 较早记录 | 后续讨论/合同 |
|---|---|---|
| 自然语言 cue | v0.2 使用 `~"school"` | tmp1 后半部改为表达式位置的 `"school"` |
| 正负软偏好 | `+("school")`、`-(...)` | 一元 `+"school"`、`-"small talk"`；单 operand 的冗余括号可省 |
| 关系方向参数 | `$rel(from=@e("Alice"),...)` | v0.5 中参数已是 EntityLocator，改为 `$rel(from="Alice",to="Bob",kind="trust")` |
| Entity identity | 早期留待身份约定 | v0.5 分离 Name、LexicalRef、Authority identity |
| 组作用域 | v0.2 出现 `(...) {$memory}` | 后续示例出现 `(...) $memory`；缺少完整 grammar，不能推定全部旧语法已废止 |
| 时间表达 | v0.2 `$occurred/$observed/$valid` | 后续例子使用 `$time(occurred,within=30d)`；三轴语义保留，完整 constructor 规则待原文或规格补齐 |

压缩包恢复 checkpoint 仍使用 `~` 和 `+(...)`，因此不能作为语法最新版本的唯一依据。v0.5 标题中的版本只保证身份/寻址合同的覆盖范围，不意味着一份完整 NousQL v0.5 grammar 已提供。

以下是后续讨论中的设计示例，尚不可在当前程序中执行：

```text
"Rust ownership mistakes"
@e("Alice") && "graduate school"
@e("Alice") +"graduate school"
@e("Alice") -"small talk"
```

表达式位置的字符串是 SemanticText；`@e("Alice")` 参数中的字符串是 Entity name locator；`kind="trust"` 中则是普通 typed 参数。上下文承担消歧，不让同一裸字符串兼任 exact lexical match。

`&&` 表示硬交集，`||` 表示 union；不引入 implicit AND。`+/-` 只改变软相关性，负号不意味着排除。后续讨论中 `$exclude(source=tool)` 表示硬排除的设计例子；具体 constructor spelling 仍需语言规格统一。

`+/-` 绑定紧邻的一个 PreferenceAtom，先于 Boolean 组合；括号只用于必要 grouping。`recent` 作为已注册 preference key 可以出现在 `+recent`，不能据此推定它本身是独立查询。

## 查询对象、语义与作用域

v0.2 提出的紧凑 selector 集：`@e`、`@tag`、`@anchor`、`@r`、`@object`、`@ref`。不为每种内部 CognitiveRef 新造一个缩写。

`#school` 在 v0.2 中表达 semantic concept，`@tag("school")` 是 exact persisted Tag。两者不可自动等同。后续材料未完整说明 `#concept` 在最终 grammar 中的去留。

Anchor/Tag 可作为拓扑入口，`$explore` 表达进一步探索。Association 在现有 reference 词汇中没有通用对象引用，研究采用解释两个 endpoint 关联的查询 intent，而不凭空建立 `@association(id)`。

`occurred`、`observed`、`valid` 是独立时间轴。source、modality、authority、memory/evidence class、effort、limit、materialization、diagnostics 各自表达不同维度。

Canonicalization 可以按固定阶段排列同一 scope 内的 attachments：domain/view、硬约束、topology/resource、软偏好、effort、ranking、result shaping、enrichment。不能改变 scope：每分支 limit 5 与 union 后全局 limit 5 不等价；仅 Bob 分支的 `$memory` 不能被移到整个 union。

完整语言可覆盖未来能力，但发给模型的 Quick Reference 只教授当前 runtime 支持的 vocabulary。Persona/Relationship 等尚未实现，不应因语言中出现 `$persona`/`$relation` 就创建虚假 capability。

## v0.5 三层身份

| 层 | 作用 | 性质 |
|---|---|---|
| DisplayName / Alias | 首次发现、方便查询 | 可变、可能重名 |
| LexicalRef | 模型跨轮精确引用 | 稳定、typed、不复用 |
| Authority Identity | 存储与规范身份 | Nous-owned UUIDv7 或 Host-owned opaque ref |

```text
@e("Alice")
@e(ent:amber-lotus-cello-river)
@e("Alice",ent:quiet-piano-mint-cloud)
```

名称绑定查询显示名、明确 alias 或 Host exact name binding：零匹配 `UNKNOWN_REFERENCE`，唯一匹配绑定，多匹配 `AMBIGUOUS_REFERENCE` 并给出候选。不得用向量相似度猜测身份。已获得精确 LexicalRef 后，后续精确引用保留它。

`@e("Alice","Bob")` 是无序 joint participant set；不能按参数顺序推断关系方向。bound canonical 按稳定 LexicalRef 排序；方向由 `$rel(from=...,to=...,kind=...)` 显式表达。

## LexicalRef 编码与生命周期

v0.5 约定：

```text
<type>:<word>-<word>-<word>-<word>
```

四词来自冻结/版本化的 4096 词词表，形成 48-bit 随机候选空间。使用 CSPRNG 生成，再以中央数据库 UNIQUE insert 与碰撞重试保证唯一性。随机空间本身不提供绝对无碰撞保证。

正文是语义中立随机词，不编码姓名、关系、记忆内容或会变化的认知含义。规范写法为 lowercase ASCII 和连字符，无大小写 alias 或多套分隔符。

初始类型词汇为 `ent / mem / memrev / tag / anchor / art / obs / src / repr / region / session / res`；只有确实需要模型寻址的对象才创建地址。

唯一性范围是一套部署/Authority namespace，包含类型前缀，不能仅按 Subject 局部唯一。删除或 purge 后保留 tombstone，永不分配给另一个对象。LexicalRef 不是访问凭据，仍须独立校验 type、Subject/visibility、Host binding 和 liveness。

Nous-owned 对象映射到 UUIDv7；Entity、Resource、外部 Object 保留原 Host canonical identity。一次性 external object 不必自动铸造 LexicalRef。ArtifactId 与 ContentDigest 也分开：摘要用于字节去重/完整性，不能代替可演化语义对象身份。

概念 binding 记录包括 LexicalRef、ObjectKind、AuthorityNamespace、CanonicalIdentity、Status、CreatedAt 和可选 TombstonedAt。

## Canonical forms、结果与错误

SCF 保留表层名称，例如 `@e("Alice")`；BCF 完成绑定后使用精确 LexicalRef。Kernel 最终接收 typed canonical IDs/Host refs，而非把可读地址当内部 Authority。

模型结果通常返回 `lexical_ref + display_label + type + result data`；不常规暴露 raw UUID。歧义候选的 hint 是呈现元数据，不参与 identity。

身份错误包括 `INVALID_LEXICAL_REF`、`UNKNOWN_REFERENCE`、`AMBIGUOUS_REFERENCE`、`REFERENCE_TYPE_MISMATCH`、`REFERENCE_NOT_VISIBLE`、`REFERENCE_TOMBSTONED`、`DUPLICATE_ENTITY_PARTICIPANT`。不采用 fuzzy auto-correction。

## 剩余研究与缺失材料

4096 词表尚需具体选择、冻结和跨 tokenizer 实测。复制错误是否需要 checksum、高量一次性对象是否默认配地址、离线/分布式铸造是否有真实需求，均保留待验证。

完整 grammar、operator/attachment 结合关系、concept 与 exact text 查询的最终语法、canonical serializer 规则、模型 few-shot 测试结果仍需完整来源或后续规格。现有材料提出过实验建议，未提供足以宣称 parser 或多模型验证完成的证据。
