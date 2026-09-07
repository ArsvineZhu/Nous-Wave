# Nous Wave 系统说明

**日期：** 2026-09-07  
**对应实现规范：** `Nous_Wave_Production_Implementation_Spec_2026-09-07.md`  
**性质：** 面向项目所有者/架构讨论的详细说明，而非 Coding-Agent 执行合同。

---

# 0. 用一句话描述 Nous Wave

Nous Wave 是一个**长期存在的 AI Subject 的认知底座**：它不仅保存“记忆文本”，而是持续管理 Subject 看见过什么、记住了什么、目前脑中还活跃着什么、这些内容彼此如何关联、哪些外部资源才是当前事实的权威来源，以及在下一次模型调用之前应该把哪些认知材料带回前景。

如果把传统 RAG 理解为：

```text
文件
→ 切块
→ embedding
→ 相似度搜索
→ 塞进 Prompt
```

那么 Nous Wave 的目标更接近：

```text
Subject 遇到世界中的事件/材料
        ↓
保留证据与发生语境
        ↓
形成（或不形成）长期认知
        ↓
在时间中不断产生新的解释、联系、概括和程序性经验
        ↓
维持当前认知运行时
        ↓
根据“我现在为什么需要回忆”进行多通道寻址
        ↓
沿个体长期认知拓扑进行有界联想
        ↓
把可解释、带证据、带时态、知道来源权威性的认知贡献交给模型/Host
```

这里最重要的变化是：**Nous Wave 不是“帮模型搜文本”，而是承载一个 Subject 跨模型调用、跨进程、跨会话持续存在的认知状态。**

---

# 1. 它首先不是一个“记忆数据库”

传统 AI Memory 很容易被设计成：

```text
add(text)
search(query)
```

这在简单聊天机器人中够用，但很快会遇到一系列根本问题。

例如用户说：

> “我最近越来越喜欢埃塞俄比亚咖啡。”

系统至少面对六个不同问题：

1. 这句话作为原始消息，要不要完整保存？
2. “用户说过这句话”是不是事实？——是一个观察事实。
3. “用户喜欢埃塞俄比亚咖啡”是不是当前世界真理？——未必，它只是 Subject 形成的认知。
4. 这条认知要不要形成长期 Memory？——不等于消息入库。
5. 用户下一句继续谈咖啡时，是否应该立刻记得？——应该，即使长期 Memory/embedding 还没生成。
6. 三个月后用户说“现在不怎么喝咖啡了”，旧记录怎么办？——不能把历史抹掉，也不能继续把旧偏好当当前有效事实。

所以 Nous Wave 从一开始就把这些概念拆开：

```text
Artifact              原始内容本体
ObservationOccurrence Subject 在某个时间/语境下遇到它这件事
DerivedRepresentation 对材料的 OCR、转录、模型解读等
Memory                 Subject 真正形成并长期保留的认知
Runtime                现在脑中可直接获得的认知
Serving Projection     为了搜索性能建立的索引/向量/拓扑快照
```

这种分层看起来比 `add/search` 多，但它并不是为了“架构漂亮”，而是为了不在未来遇到无法修复的语义混乱。

---

# 2. 最重要的 Authority 边界：Nous 拥有认知，不拥有世界

Nous Wave 的核心原则是：

> **外部系统拥有自己的当前事实；Nous Wave 拥有 Subject 对这些事实的认知，以及知道如何再次访问它们。**

例如：

```text
Schedule
  拥有：明天下午 3 点是否真的有会议

Messaging
  拥有：某条消息是否真实存在、发送者、原始时间

Tasks / Projects
  拥有：任务当前是否完成

Contacts / Identity
  拥有：某账号实际对应哪一个人

Simulated Reality
  拥有：当前世界状态
```

Nous 可以记住：

> “我记得明天下午好像有一个组会。”

但当用户问：

> “明天下午我到底有什么安排？”

如果 Schedule 当前可用，Nous 必须重新查询 Schedule。

这避免一个非常危险但常见的 Memory 设计错误：**把过去检索/记住的外部状态偷偷升级成当前 Authority。**

因此 Nous 还会保存一种很轻的 `ResourceRef` 认知，例如：

```text
我知道有一个 Schedule 资源
它权威负责 appointment / commitment / deadline
可以按时间、实体、项目查询
当前 READY
查询成本很低
```

这不是把 Schedule 全复制进脑中，而是类似人知道：

> “这件事我不用死记，我知道应该去哪里查。”

---

# 3. Subject 不等于模型

Nous Wave 设计中另一个最重要的区别是：

```text
Subject != Model
```

一次 GPT、Claude、Gemini 或本地模型调用只是 Subject 当前使用的一个认知/表达执行器。

Subject 的长期状态不能寄生在某一个模型的：

- Context Window；
- KV Cache；
- hidden state；
- provider session；
- prompt prefix。

这些都可以作为性能缓存，但随时可以丢。

因此 Nous 的认知运行时是：

```text
CognitiveSession
        ↓
ResidentSet
        ↓
ConsumerWorkingSet
        ↓
ContextContribution
        ↓
某一次模型调用
        ↓
KV / hidden cache（可丢）
```

换模型、模型崩溃、进程重启，都不等于 Subject “失忆”。

这也使未来多模型协作自然成立：一个 Subject 同时可能有聊天模型、后台解释模型、视觉模型，但它们消费的是同一个认知主体，而不是各自拥有一份彼此割裂的“记忆库”。

---

# 4. 三种“现在记得”必须区分

一个内容可以处于至少三种不同状态：

### 4.1 已持久化

它存在于长期 Memory 或 Evidence 中。

### 4.2 当前 Resident

它现在就在 Session 的认知前景/近景中，不需要重新做冷检索。

### 4.3 已投影进某次模型 Context

它被选进了某个模型调用的输入。

这三者不是同一件事。

例如：

```text
一条十年前的童年 Memory
  durable = yes
  resident = no
  projected = no

刚刚用户说的一句话
  durable memory = maybe not yet
  resident = yes
  projected next turn = likely yes

某个 Resident Memory
  resident = yes
  projected to Model A = yes
  projected to Model B = no
```

这使系统能够避免传统 RAG 的荒谬路径：

```text
用户刚说完一句话
→ 系统先形成长期 Memory
→ 做 embedding
→ 下一轮再向量搜索把它找回来
```

Nous 的正常路径是：

```text
新 Observation
→ 立即进入 Runtime
→ 下一轮直接可用
→ 是否形成长期 Memory，另行决定
```

---

# 5. Evidence、Specific Memory 与 Integrative Memory

Nous Wave 不把所有长期内容都视作一种扁平“记忆”。

## 5.1 Evidence / Cognitive Material

Evidence 是 Subject 接触过的材料：

- 消息；
- 文件；
- 图片；
- 音频；
- 视频；
- 工具结果；
- Web 页面；
- API 返回；
- 外部系统事件；
- 模拟世界事件。

Evidence 可以很重要，但**读到它不等于学会它**。

## 5.2 Specific / Episodic Memory

这是对具体经历的认知痕迹。

例如：

> “9 月 7 日用户说最近喜欢埃塞俄比亚咖啡。”

它通常：

- 形成快；
- 语境丰富；
- 有明确来源；
- 对“发生过什么”很重要。

## 5.3 Integrative Memory

这是跨多个经历形成的概括、schema、长期认识。

例如多个月的对话后形成：

> “用户通常偏好花香、浅烘焙、酸质明显的咖啡。”

这不是任何一句原话，而是整合出来的认知。

因此它必须保留：

```text
Integrative Memory
→ 支撑它的多条 Specific Memory / Evidence
```

而不能把原经历覆盖掉。

## 5.4 Procedural / Experience Memory

它描述“怎么做”、失败模式、适用边界和经验方法。

例如：

> “当某个站点返回 403 时，先检查 cookie/session，而不是立刻切代理。”

未来 Agent 执行工作时，这类 Memory 的价值甚至可能高于单纯事实记忆。

---

# 6. 为什么 Message != Memory、File != Memory

一段 500 页 PDF 不能等于一个 Memory。

一条消息也未必值得成为 Memory。

反过来，一条消息可能形成多种认知：

```text
“我下周三去大阪出差，最近特别想吃寿司，记得提醒我带充电器。”

可能形成：
- Specific Memory：用户提到大阪出差
- Preference cognition：最近想吃寿司
- prospective/commitment-like cognition：带充电器（未来由 Goals/Commitments owner 处理）
- Entity/Place association：大阪
```

但原始消息仍然只是一份 Evidence。

于是数据关系允许：

```text
1 Artifact → N Memories
N Artifacts → 1 Integrative Memory
1 Memory → N immutable revisions
```

这也是为什么 Nous 不应该直接套用“聊天记录向量数据库”。

---

# 7. Artifact 和 ObservationOccurrence 为什么必须分开

假设完全相同的一张图片被：

1. 用户第一次私聊发来；
2. 半年后另一个人转发；
3. 又被网页爬取一次。

原始 bytes 完全相同，所以 CAS 可以只存一份：

```text
Artifact(content_hash = X)
```

但 Subject 的经历是三次：

```text
Occurrence A
Occurrence B
Occurrence C
```

每次的：

- 谁给的；
- 什么时候看到；
- 当时上下文；
- 社会意义；
- 来源可信度；
- 后来形成什么 Memory

都可能不同。

如果 Artifact 与 Occurrence 混为一张表，内容去重会把经历去重，这是不可接受的。

---

# 8. 多媒体不是“附件”，而是第一类认知材料

Nous 从底层就允许：

```text
ArtifactRef + RegionRef
```

所以 Memory 不必把整张图片、整段音频、整部视频复制进去。

它可以精确指向：

```text
PDF 第 7 页某个 bbox
图片中的一个区域
视频某条 stream 的 PTS 12.3s–16.8s
JSON 的 /users/3/profile
OOXML 中某个结构节点
字节区间
```

## 8.1 SourceRegion 与 DerivedRegion

这是一个非常重要、而且容易被普通 RAG 忽视的区别。

`SourceRegionRef` 指向原材料自身稳定坐标。

例如：

```text
PDF page 7 + bbox
image pixel bbox
video stream + PTS + time_base
JSON Pointer
```

而：

```text
OCR 的第 57–92 字符
Docling 分出来的第 12 个 paragraph span
ASR transcript 的第 8 个 segment
```

属于某一次派生结果，所以是：

```text
DerivedRepresentationRevision
    └── DerivedRegionRef
```

如果以后换 parser，旧 DerivedRegion 仍然稳定，因为它绑定旧 representation，而不是假装“chunk 12”永远是源文件坐标。

---

# 9. “模型看过图片”必须留下可降级使用的认知材料

如果今天配置了一个视觉模型，它解释图片：

> “照片中有两个人站在东京塔附近，右侧的人穿红色外套……”

Nous 不应该只把这个描述临时拿去 embedding 然后丢掉。

它应持久化为：

```text
DerivedRepresentation
  kind = image_description
  source = exact Artifact/Region
  producer = Provider/Model/Revision/Config
  text = ...
```

以后视觉模型被移除，系统仍可：

- 全文搜索这段旧描述；
- 用文本 embedding 搜索；
- 让 text-only LLM 阅读；
- 根据旧描述形成/检索 Tags、Anchors、Memories。

同时系统必须知道：

```text
原始图片
!= OCR
!= 视觉模型描述
!= 从描述形成的 Memory
!= 世界真理
```

这是 Nous 对“能力退化”特别重要的设计：**模型能力可以消失，已经形成的认知材料不应一起失踪。**

---

# 10. Provider 能力不是“选一个模型”

现实模型供应商不会整齐地提供同一套功能。

可能出现：

```text
Provider A
  text.embedding READY

Provider B
  image.interpretation READY
  text.interpretation READY

Provider C
  speech.transcription READY

没有 reranker
```

因此 Nous 不采用：

```text
embedding model
rerank model
generation model
```

三个固定槽位作为最终架构。

它采用：

```text
Capability = operation × modality
```

例如：

```text
text.embedding
image.interpretation
speech.transcription
memory.formation.text
text.rerank
```

查询 Planner 看的不是：

> “系统现在是什么模式？”

而是：

> “这次操作需要哪些 semantic operators？当前有哪些 capability READY？”

所以系统可以在**零模型**情况下正常启动。

---

# 11. ProducerSignature 和 EmbeddingSpaceSignature 是两回事

这是实际工程中非常关键的细节。

## 11.1 ProducerSignature

回答：

> 这份派生结果是谁、用什么配置产生的？

包含：

- provider/implementation；
- operation；
- model identity/revision；
- preprocessing；
- config digest。

## 11.2 EmbeddingSpaceSignature

回答：

> 两个向量能不能直接做距离/相似度比较？

包含：

- weights revision；
- task；
- input representation；
- preprocessing；
- dimension；
- normalization；
- output semantics。

因此：

```text
两个 provider 名字不同
≠ 两个向量空间一定不同
```

如果它们实际托管完全相同的权重和预处理，空间可以一致。

反过来：

```text
两个向量都是 1024 维
≠ 可以混搜
```

这能够彻底避免“模型换了但旧向量还偷偷混在同一索引里”的灾难。

---

# 12. Entity：认识“这个人”不能依赖名字或 embedding

长期认知系统如果把人建成：

```text
name:Alice
```

迟早会坏。

一个人历史上可能出现为：

```text
“猫猫”
“王学姐”
“Wang”
“🐱”
微信账号 A
Discord 账号 B
```

Nous 不负责猜它们是不是同一个人。

Host/Identity 系统负责：

```text
各种账号/别名/表面
        ↓
stable EntityRef
```

Nous 保存：

```text
EntityRef
+ 历史 surface snapshot
+ 哪一处 Evidence 当时怎样提到 TA
```

## 12.1 为什么还要 Entity exact posting

即使 Nous 已经知道：

```text
“猫猫” = EntityRef X
```

历史 Memory 文本可能完全没有相似 embedding。

例如：

> “王学姐当时说她不吃香菜。”

后来查询：

> “猫猫吃不吃香菜？”

如果只做向量搜索，昵称变化可能导致检索不稳定。

所以必须有：

```text
EntityRef X
→ 所有关联 Memory / mention / Anchor / relation
```

的确定性索引。

## 12.2 错误合并怎么修

身份系统最危险的操作不是漏合并，而是错合并。

所以 Nous 不使用不可逆的 Union-Find 作为 Authority。

每一个 `EntityMention` 有 revisioned binding：

```text
Mention A → Entity X
Mention B → Entity X
Mention C → Entity X
```

后来发现 C 其实是另一个人：

```text
Mention C binding revision 2 → Entity Y
```

历史文本不变，其他 mention 不受影响，Entity posting generation 重建即可。

---

# 13. Tag、Anchor、Association 到底是什么

## 13.1 Tag

Tag 不是一个扁平 topic string。

它是认知寻址和拓扑中的一个可复用语义结点，可参与：

- 查询 cue；
- 记忆组织；
- 关联传播；
- runtime activation；
- consolidation；
- Resource routing；
- novelty/contradiction 感知。

例如：

```text
coffee
Ethiopia
presentation
deployment failure
Bob
project X
```

都可能形成某种 Tag，但系统不需要现在就做一个庞大的 ontology taxonomy。

## 13.2 Anchor

Anchor 是比 Tag 更像“认知地标”的对象。

例如一个 Subject 长期经历中可能形成：

> “2026 春季项目危机”

这个 Anchor 可能连接：

- 多条 Memory；
- 几个人；
- 一个项目 Resource；
- 一些失败模式；
- 一组 Tags；
- 另一个 Anchor。

Anchor 不是：

```text
一个摘要字符串
一个平均 embedding
一个关键词
```

它是有：

```text
稳定 identity
revision
support evidence
supersession/revocation
```

的认知拓扑对象。

## 13.3 Association

Association 也不是 cosine similarity。

两个概念可以 embedding 很远，却因为个人经历产生非常强的联系：

```text
某个人
↔
某次答辩失败
```

这种个体经验联系才是 Nous 想捕捉的东西。

Association 可以来自：

- explicit relation；
- temporal/structural evidence；
- consolidation；
- meaningful co-use；
- experience；
- procedural relation。

而 embedding similarity 只能用于**找可能相关的候选**，不能自动创造永久经历关系。

---

# 14. 我们从 A-MEM 吸收“记忆会演化”，但不允许历史被偷偷改写

A-MEM 最有价值的思想之一是：新经验进入后，旧记忆的上下文理解可能发生变化。

Nous 接受这个认知事实，但实现方式不同。

不做：

```text
旧 Memory text
→ 直接覆盖成新的描述
```

而做：

```text
旧 MemoryRevision 1
        ↓ 新经验
MemoryRevision 2（如果真的是同一长期认知的修订）

或者

Specific Memory A + B + C
        ↓ consolidation
Integrative Memory D

或者

新 Anchor / Tag / Association
```

这样 Subject 的当前认知可以演化，同时仍可回答：

> “当时我是怎么记得这件事的？”

---

# 15. Graphiti/Zep 给 Nous 带来的不是“图数据库”，而是时间与证据纪律

Graphiti/Zep 最值得吸收的一点，不是 Neo4j，而是：

```text
一个事实/关系什么时候在世界中有效
!=
系统什么时候知道它
```

因此 Nous Memory 会同时保留：

```text
occurred_at
observed_at
valid_from
valid_to
created_at
```

例如：

1 月 1 日发生：

> Alice 开始在公司 X 工作。

Subject 1 月 10 日才知道。

6 月 1 日 Alice 离职，Subject 6 月 5 日才知道。

这四个时间点不能压成一个 `timestamp`。

因此系统可以区分：

> “5 月 20 日时 Alice 在哪里工作？”

和：

> “我是在什么时候知道 Alice 去公司的？”

新信息使旧结论失效时，也不是删除旧 Memory，而是关闭它的适用区间/建立 contradiction/supersession。

---

# 16. Letta 给 Nous 带来的不是“Memory Block”，而是 Context 应该是可管理的投影

Letta 的 memory block 证明了一件很重要的工程事实：

> 长期状态不能简单靠“每次重新搜索几个 chunk”，应该有一部分持久认知能被有意识地维持在模型附近。

Nous 把这个想法拆成：

```text
ResidentSet
→ ConsumerWorkingSet
→ ContextContribution
```

而不是复制一个 `persona block / human block`。

好处是：

- ResidentSet 是 Subject runtime；
- Model A 与 Model B 可得到不同 WorkingSet；
- Host 仍然拥有最终 prompt/compositor；
- Context Window 容量不会反过来定义 Memory 的本体结构。

---

# 17. Mem0 给 Nous 带来的不是 `memory.search()`，而是多信号检索必须落地

Mem0 v3 已经从单纯向量路线走向：

```text
semantic
+ BM25
+ entity matching
```

这证明了一个非常实际的问题：

- 向量适合语义；
- BM25 适合精确罕见词、名字、代码、错误码；
- entity signal 适合身份连续性。

Nous 会进一步走得更远。

候选来源包括：

```text
Runtime
Exact
Entity
Lexical
Dense
Tag
Anchor
Wave topology
Temporal
Resource
```

但这些不会被压成一个不透明的“combined score”然后丢掉来源。

每个结果都会保留：

> 它为什么被找到？

例如：

```text
Entity exact
+ lexical match
+ Wave transfer-field
+ direct seed contact
```

这样将来模型或开发者能真正审查一次回忆，而不是只能看到 `score = 0.8421`。

---

# 18. GraphRAG / LightRAG / HippoRAG 为什么都没有被“整套搬进来”

这些系统分别证明了不同能力：

### GraphRAG

大语料的全局主题问题需要层级 summary/community representation。

Nous 因此为大型 Resource 保留：

```text
resource synopsis
integrative derived representation
```

但不会让每条个人 Memory 都经过昂贵 community clustering + report generation。

### LightRAG

图结构和向量检索应该能增量协作。

Nous 因此明确：

```text
Authority
→ LexicalGeneration
→ DenseGeneration
→ WaveGraphGeneration
```

都是独立可重建 projection。

### HippoRAG

PPR/图传播证明多跳联想有实际价值。

但 Nous 的长期个体认知图有：

- hub；
- 高频通用概念；
- 叙事方向；
- 跨域 bridge；
- 循环回流；
- query-specific actual flow。

因此最终选择的是 VCP lineage 中更针对这些问题的 bounded Wave，而不是静态 PPR 作为生产定律。

---

# 19. 为什么 VCP Toolbox 是算法来源，但不会成为代码依赖

VCP Toolbox 当前最有价值的地方，是它已经把“长期私有记忆为什么不能只 KNN”做了大量工程演进。

目前吸收的核心问题分解包括：

```text
EPA
Residual Pyramid
有界、守恒的传播
hub 抑制
立即回流抑制
跨域 bridge
查询实际边流
Local / Transfer 双尺度场
候选有序语义曲线
相对拓扑
Ω 可观测性
Direct seed contact
```

但 VCP 的代码形状是它自己的：

```text
JS
SQLite
Vexus/N-API
TagMemo classes
RiverMemo protocol
```

Nous 不应该复制这些形状。

更重要的是，VCPToolBox 当前使用 CC BY-NC-SA 4.0。

因此 Nous 的规则是：

```text
VCP = research reference
VCP != source donor
VCP != runtime dependency
```

我们会按照已经明确的数学行为和问题定义，用自己的 Rust 类型、PostgreSQL Authority、USearch/Tantivy、CSR 和 Query protocol 做 clean-room 实现。

---

# 20. “浪潮”算法直觉：一次回忆不是搜一个点，而是形成一个受约束的信息场

这是整个系统最具特色的部分。

假设用户问：

> “为什么我一看到 Alice 就会想到上次那场 presentation？”

普通 vector search 可能分别找到：

```text
Alice 相关文本
presentation 相关文本
```

但真正的问题是：

> 这两件东西在这个 Subject 的长期经历中，是怎么连起来的？

Nous 先产生种子：

```text
Entity Alice
Tag presentation
当前 Session 中相关 Anchor/Tag
文本 query 的 residual sensed Tags
```

然后在长期认知图中传播。

不是无限扩散，而是：

```text
每个节点只有固定总出流预算
高频 hub 被抑制
bridge 只能在预算内竞争
立即 i→j→i 回流被压制
最多有限 hop
最多有限状态
很弱的流直接停止
```

于是一次 Query 会形成一个局部的：

```text
QueryRiver
```

它不只是：

```text
哪些节点最后分数高
```

还记录：

```text
这次信息真正流过哪些边
每条边承载多少流
哪些节点是 seed
哪些是 emergent
从哪个 hop 出现
最强 parent 是谁
```

这使“联想”第一次成为可以被审计的请求级结构，而不是一句模糊的“graph score”。

---

# 21. 为什么要守恒：否则长期记忆图必然被 hub 污染

长期记忆图中一定会出现：

```text
“用户”
“项目”
“工作”
“聊天”
“问题”
```

这种超级通用节点。

如果普通图扩散直接按边累计，它们会因为连接多不断吸收/释放质量，最终任何查询都能经过它们连接到任何东西。

Nous Wave 构图时会：

1. 对重复证据 `log1p` 压缩，不让重复次数线性垄断；
2. 统计每个 target 的整体入流；
3. 用中位入流估计 hub 程度；
4. 温和降低 hub conductance；
5. 最后把每个 source row 统一归一化到固定 outbound budget。

因此：

```text
一个节点边再多
→ 总共也只有 0.9 左右的传播预算
```

这不是为了模拟真实神经元，而是一个非常有效的工程纪律：**拓扑不能凭连接数量创造无限相关性。**

---

# 22. Bridge / “虫洞”在 Nous 中是什么

长期个人认知最有价值的联想，往往恰恰不是 embedding 最相似的两个东西。

例如：

```text
Alice
↔
某次 presentation
```

文本语义可能并不接近，但如果 Subject 的经历中它们有强关系，就应该保留一条跨域 bridge。

Nous 不给 bridge 免费能量。

每个节点的总预算中只预留一部分：

```text
main mass
+ bridge reserve
= 仍然不超过总 outbound budget
```

所以 bridge 可以“穿越语义距离”，但必须跟其他 bridge 竞争。

这是一种很重要的平衡：

```text
不让 vector space 决定一切
也不让 graph relation 无限制决定一切
```

---

# 23. 为什么有 SourceField、LocalField、TransferField 三层

一次查询的种子是 `SourceField`。

例如：

```text
Alice 0.4
presentation 0.3
某个 residual Tag 0.2
某个 Runtime Anchor 0.1
```

然后在同一 WaveGraph 传输算子上得到两个尺度：

### LocalField

更接近 seed。

适合：

> “与当前问题直接有关的认知邻域是什么？”

### TransferField

允许更远的结构传播。

适合：

> “有哪些通过长期关系才能联想到的内容？”

重要的是，这不是：

```text
两个图数据库
两个算法
两个 index
```

而是：

```text
same graph
same source
same transition operator
不同有限尺度
```

因此它仍然可解释，也不会制造一套“deep mode retrieval system”。

---

# 24. EPA：先判断 query 在全局语义空间中处于什么状态

EPA 使用长期 Tag embedding 构造一个有界的正交语义基底。

查询向量先减去整体 mean，再投影到这些主轴：

```text
q - mean
→ axis 1
→ axis 2
→ ...
```

得到：

- 哪些语义轴被激活；
- 能量是否集中；
- entropy；
- 是否同时跨多个正交方向。

一个非常聚焦的问题可能类似：

```text
80% 能量集中在一个方向
```

而复杂、混合的问题可能：

```text
多个轴同时显著激活
```

EPA 不会宣称：

> “entropy = 人类思想复杂度”。

它只是帮助 Retrieval 判断：

> 当前 query 是否可能包含多个需要拆开的语义因素？

---

# 25. Residual Pyramid：强主题不能吃掉弱但关键的 cue

假设 query embedding 里有：

```text
80% = project X
15% = Alice
5%  = presentation failure
```

普通 KNN 往往只会不断返回 project X。

Residual Pyramid 做的是：

```text
当前向量
→ 搜最接近的 Tags
→ 用这些 Tag 张成一个子空间
→ 把能被解释的方向投影掉
→ 对剩余 residual 再搜
```

数学上使用 Modified Gram-Schmidt 建正交基。

每一层都问：

> “已经被这些 Tag 解释掉以后，还剩下什么？”

于是一个弱但独立的 cue 有机会在第二/第三层浮出来。

它不是简单：

```text
搜一次 → 排除 topK → 再搜一次
```

而是在向量几何上真正把已解释方向剥离。

这会直接提高长查询、复合情境和隐性联想的寻址能力。

---

# 26. Candidate generation 和 Ranking 为什么必须分开

Nous 不会要求某一个 search engine 给最终答案。

候选池可能来自：

```text
Exact
Resident Runtime
Entity posting
Tantivy lexical
USearch direct dense
Residual dense
LocalField dense
TransferField dense
Tag posting
Anchor posting
Wave node
Temporal
Resource
```

这些只是：

> “哪些东西值得进入下一轮判断？”

不是：

> “谁的 raw score 大谁就是最终结果。”

因此流程是：

```text
Candidate Generation
        ↓
Evidence-family consolidation
        ↓
Field / topology audit
        ↓
Unified ranking
        ↓
optional language rerank
        ↓
result diversity
```

这与把 cosine、BM25、PageRank 直接相加有本质区别。

---

# 27. 为什么不会让 Residual / Local / Transfer 多投三票

这是 Memoria Next 研究中一个很有价值的工程教训。

如果同一个 semantic query 衍生出：

```text
direct vector
residual vector
local field vector
transfer field vector
```

然后每一路都 RRF 投一票，一个候选可能只是因为“同一份 dense 证据被复制了四次”而上升。

Nous 会把它们归为：

```text
SemanticDense evidence family
```

保留：

- 最好 rank；
- 哪些 variant 都命中；
- 每个 variant 的 trace；

但不会把它们当成四份独立证明。

真正独立的证据比如：

```text
Entity exact
Lexical
Temporal
Direct Tag
```

才具有额外独立意义。

---

# 28. CandidateSemanticTrail：只有真的存在顺序，才谈“路径”

VCP 的长期日记有有序 Tag 曲线，这是很有价值的结构。

但 Nous 的 Memory 来源更广，不能假装每条 Memory 都有一个有意义的 Tag sequence。

所以每个候选只有三种情况：

```text
Ordered
Unordered
Unavailable
```

例如：

### Ordered

一段文档中：

```text
problem
→ investigation
→ root cause
→ fix
```

来源结构能够证明这个顺序。

### Unordered

Memory 只知道它关联：

```text
Alice
presentation
stress
```

但没有证据说谁先谁后。

这种情况下，Nous 可以计算 Field contact，但绝不能制造：

```text
Alice → presentation → stress
```

然后给自己一个漂亮的 path score。

这是我们特别强调的“不可观测就不要伪装成 0/伪数据”。

---

# 29. QueryRiver 与候选 Trail 怎么比较

QueryRiver 记录本轮实际流过的边：

```text
Alice → event-X : flow 0.32
event-X → presentation : flow 0.21
presentation → failure-pattern : flow 0.12
```

某个候选 Memory 的真实有序 Trail 是：

```text
Alice → event-X → presentation
```

那么它不仅“节点碰到了”，而且路径方向也和本轮 QueryRiver 一致。

另一个候选可能只有：

```text
presentation → Alice
```

反向也可能有意义，但获得折扣信用。

因此 topology score 能表达：

> “这个候选不仅包含相似概念，它的内部叙事结构也与本轮长期认知流动一致。”

这是普通 vector + graph-neighbor 检索很难表达的一层。

---

# 30. Ω（WaveObservability）为什么必要

图算法最危险的问题之一是：

> 只要能计算一个 graph score，系统就会把它当真。

Nous 不允许这样。

`WaveObservability` 先问：

> **本轮传播真的产生了足够可观察的结构吗？**

它看四件事：

1. 实际边流有多少；
2. 是否真的出现了非 seed 的 emergent nodes；
3. 流是不是有一定结构分布，而不是只有单边独占；
4. 是否因为状态预算丢掉了大量质量。

然后：

```text
Ω = geometric_mean(activity, emergence, entropy, completeness)
```

如果本轮只是：

```text
一个 seed
→ 一条很弱的边
```

即使某个候选 graph path 看起来不错，Ω 也会很低，它无权获得大幅“拓扑创新奖励”。

因此 Ω 不是“信心”，而是：

> **本轮拓扑证据有没有资格影响排序。**

---

# 31. DirectSeedEvidence 不是 Anchor

VCP 使用过 “Direct Anchor” 这个术语，但 Nous 已经有自己的 `Anchor` 认知对象。

两者完全不是一回事。

所以在 Nous 中统一叫：

```text
DirectSeedEvidence
```

它只表示：

> 候选是否直接接触本轮 hop-0 query seeds？

例如 query seed 有 `Alice`，候选 Memory 本身直接关联 Alice，那么获得 direct-seed 证据。

它不创建 Anchor，也不改变 Anchor Authority。

---

# 32. 最终排名为什么只允许拓扑做“有界正向修正”

如果一个 Memory 通过：

```text
Entity exact
+ lexical exact quotation
```

已经明显相关，但它没有可观察的有序 topology，不能因为：

```text
StructuralScore = 0
```

就被惩罚。

所以 Nous 的最终结构是：

```text
BaseRankScore
+ bounded Field bonus
+ Ω-gated bounded topology innovation
+ bounded DirectSeed bonus
```

Topology 缺失：

```text
bonus = 0
```

而不是：

```text
penalty < 0
```

这能避免图结构成为新的独裁信号。

---

# 33. 为什么仍然保留一个 rank-based BaseScore

不同检索系统的 raw score 根本不是同一个物理量：

```text
cosine 0.82
BM25 13.4
Entity exact = true
时间距离 = 2 天
```

直接加起来毫无数学意义。

因此第一版 BaseRankScore 使用的是**证据族内排名效用**，而不是 raw score 求和。

同时：

- correlated semantic variants 先合并；
- hard constraints 先过滤；
- exact target 可直接 pin；
- 每个结果保留原始 evidence trace。

这不是把 RRF 升级成“认知理论”，只是一个在没有 pre-execution benchmark 的情况下稳健、可解释、不会混单位的 baseline ranking mechanic。

以后有真实数据，可以调整这些 implementation weights，但不需要改 CognitiveQuery 或 Memory Authority。

---

# 34. Optional Reranker 的地位

Cross-encoder / LLM reranker 很擅长回答：

> “这段内容是不是在直接回答当前语言问题？”

但它通常不擅长知道：

> “为什么这条并不直接回答问题的个人经历，是当前 Subject 联想链中的关键一步？”

所以 Nous 不让 reranker 替换 Wave。

它只是：

```text
LanguageRerank evidence family
```

加入最终判断。

它不能：

- 创造候选；
- 恢复被 hard filter 排除的内容；
- 推翻 Host EntityRef；
- 创建永久 association；
- 把一个 stale Memory 变成 current Authority。

---

# 35. 记忆为什么不会因为“被搜出来”越来越强

很多 Memory 系统会无意中形成反馈环：

```text
Memory A 排名高
→ 被召回更多
→ 访问次数更多
→ 权重更高
→ 更容易排名高
```

最后早期热门 Memory 会永久垄断。

Nous 明确区分：

```text
candidate
surfaced
inspected
selected
referenced
acted_on
corroborated
pinned
```

只有真正的独立 meaningful use：

```text
referenced
acted_on
corroborated
explicit pin
```

才可能更新 accessibility 或形成 adaptive association evidence。

“出现在 Prompt 中”本身不算学习。

---

# 36. Forgetting、Eviction、Suppression、Purge 是四件事

### Runtime Eviction

只是：

> 现在不放在脑中近景了。

Memory 仍存在。

### Forgetting / Accessibility

只是：

> 冷回忆时更不容易被优先唤起，需要更强 cue/更高 effort。

Memory 仍存在。

### Suppression

主动规定：

> 普通召回不应再返回它。

历史/管理仍可查。

### Purge

才是真正：

> 删除 Authority，并清理相关 evidence/derived/index/raw object（在无其他引用时）。

把这四个概念分开以后，系统不会出现：

> “缓存淘汰 = 忘掉人生经历”

或：

> “用户要求删除 = 只把 vector index 隐藏”

这种错误。

---

# 37. 为什么 PostgreSQL 是 Authority，而不是 vector DB

Nous 的长期认知有：

- immutable revision；
- evidence lineage；
- entity rebind；
- temporal validity；
- Anchor support；
- derivation lease；
- Session durability；
- purge transaction；
- serving generation pointer。

这些都是关系型、事务型状态。

PostgreSQL 非常适合。

而 vector DB 的优势是：

```text
nearest-neighbor serving
```

所以第一版明确：

```text
PostgreSQL = canonical structured Authority
USearch     = dense projection
Tantivy     = lexical projection
CSR         = Wave topology projection
CAS         = raw bytes
```

任何 projection 坏了：

```text
认知仍然存在
```

这是整个系统可维护性的基础。

---

# 38. 为什么选 USearch + Tantivy，而不是让 LanceDB/Qdrant 接管整个 Retrieval

如果 Nous 的目标只是：

```text
metadata filter + vector + BM25
```

一个一体化 retrieval DB 很合理。

但 Nous 已经明确需要：

```text
Entity exact
Residual cue
Tag/Anchor
QueryRiver
Local/Transfer fields
CandidateSemanticTrail
Ω
Resource Authority
Runtime
```

最终 retrieval semantics 已经由 Nous 自己拥有。

因此底层引擎最好做自己最擅长的一件事：

```text
USearch → dense candidate addressing
Tantivy → lexical candidate addressing
```

而不是再引入一个有自己 hybrid query DSL、graph/vector semantics 的服务，然后在上面又做一套 Nous Planner。

Memoria Next 与 VCP 两条独立实现路线都实际使用/收敛到 USearch 相关方向，这提供了额外工程信心；但决定的根本原因仍然是 Nous 自己的 workload shape。

---

# 39. ServingGeneration：索引更新不能让一次 Query 看到两个世界

假设后台刚好在：

```text
重建 Wave graph
更换 embedding model
发布新 Tantivy reader
```

如果 Query 中途切代，就可能出现：

```text
dense candidate 来自 generation A
Wave topology 来自 generation B
Entity posting 来自 generation C
```

结果很难解释，甚至引用不存在的 serving IDs。

所以所有 projection 都通过 immutable generation 发布。

Query 开始时：

```text
snapshot = Arc<ServingSnapshot>
```

从头到尾使用同一快照。

新 generation 构建完成后：

```text
ArcSwap publish
```

新 Query 才看到它。

老 Query 持有旧 Arc，完成后旧 generation 才能 GC。

这是一个非常低成本但很高价值的并发设计。

---

# 40. Provider replacement 会发生什么

假设原来使用 embedding model A，后来切成 B。

错误做法：

```text
所有 Memory 重新写一遍
或者
把 A/B 1024 维向量混在一起
```

Nous 做：

```text
Authority Memory 不变
Producer/EmbeddingSpace 记录新空间
→ build DenseGeneration(B)
→ build EpaBasisGeneration(B)
→ 必要时 build field-node vectors(B)
→ atomic publish
→ A generation 退休
```

Tag/Anchor/Association 如果本身是显式/认知 Authority，也不会因为换 embedding 被删除。

只有“这个模型产生的 projection”失效。

---

# 41. Derivation worker 为什么不算内部 Scheduler

Nous 明确不拥有：

```text
每天凌晨 3 点反思
每 6 小时 consolidation
自动做梦
```

这些是 Host/用户/未来认知系统的 policy。

但当用户刚上传一个视频，并明确需要 transcript 时：

```text
CoverageNeed 已经成立
Derivation 已经创建
```

此时后台 worker 负责把这项已存在的工作做完，只是执行机制。

因此：

```text
Event-driven pending-work executor = allowed
Autonomous cognition schedule       = not owned by Nous
```

崩溃后 worker 可以根据 lease 恢复未完成工作，但它不会自己发明“现在该不该形成一条新人生总结”。

---

# 42. Memory formation 为什么是 Proposal → Validate → Commit

LLM 很适合：

- 提取 gist；
- 判断哪些信息值得记；
- 建议 Tags；
- 建议 associations；
- consolidation。

但它不能直接写 Authority。

因此：

```text
Evidence
→ Model Proposal
→ deterministic/service validation
→ Cognitive Authority commit
```

Validator 至少检查：

- Evidence 是否真的存在；
- EntityRef 是否由 Host 提供；
- 时间区间是否合法；
- 模型是否把一个 tool result 误写成无条件真理；
- proposal 是否超限；
- 是否在当前 Subject scope。

这里的“Authority”也要准确理解：

> commit 后它成为 **Subject 的认知事实**，不是整个现实世界的真理。

---

# 43. Anchor 为什么不能由单个模型输出随便创建

Anchor 会影响未来很多次拓扑寻址，因此它比普通 Tag 更具有结构权力。

所以模型提议 Anchor 时，默认要求：

```text
至少两个独立 support roots
```

例如来自两个不同 Observation/Memory lineage。

这不是说“两条证据就一定是真理”，而是防止：

```text
一次 LLM 幻觉
→ 永久创建全局认知地标
→ 以后不断影响回忆
```

显式用户/Host 创建 Anchor 时则不需要这条限制，因为意图已经明确。

---

# 44. Resource awareness 让“知道在哪里查”成为认知能力

很多人类认知不是把内容完整记在脑中，而是：

> “我知道这东西在那本书/那个数据库/日历里。”

Nous 把这个能力正式化。

例如一个大型项目数据库：

```text
ResourceRef project-db
coverage:
  tasks
  decisions
  documents
query_dimensions:
  project
  status
  time
  entity
authority:
  current project state
```

Runtime 中甚至可以只 Resident：

```text
“project-db 是当前项目状态权威来源”
```

而完全不 Resident 里面的百万条记录。

Query 需要时再 progressive disclose。

这对未来大规模认知非常关键，因为“全部复制进 Memory”从根本上不可扩展。

---

# 45. 系统在零模型情况下会是什么样

把所有 AI provider 关掉，Nous 仍然不是“启动失败”。

它还能：

- 保存 Artifact/Occurrence；
- 精确查 Memory/Artifact/Entity/Tag/Anchor；
- Tantivy 全文检索；
- 使用已经存在的 Tags/Anchors/Associations；
- 运行 deterministic Wave；
- 维持 Session/ResidentSet；
- 查询 Resource；
- 查看 provenance/evidence；
- 读取此前已经生成的 textual surrogates。

它不能：

- 自动理解一张从未解释过的新图片；
- 自动生成新的 LLM Memory；
- 自动做复杂 consolidation；
- 生成缺失的 embedding。

这叫：

```text
degraded but truthful
```

而不是：

```text
system failed
```

---

# 46. 一条普通聊天消息的完整生命周期

用户发来：

> “我最近越来越喜欢埃塞俄比亚咖啡。”

假设 Host 选择 `consider_specific`。

### 46.1 Material

创建文本 Artifact：

```text
hash = BLAKE3(...)
```

### 46.2 Occurrence

创建：

```text
source = messaging
occurred_at = 消息发送时间
observed_at = Subject 收到时间
conversation_ref = 当前会话
```

### 46.3 Runtime

Occurrence 立即 Resident。

下一轮模型不用等 embedding。

### 46.4 Derived coverage

文本原生即可：

```text
raw text READY
lexical projection pending/ready
text embedding preferred if provider ready
```

### 46.5 Memory formation

模型可能提出：

```text
Specific Memory:
“用户最近越来越喜欢埃塞俄比亚咖啡。”
```

Evidence 指回原始 occurrence/region。

EntityRef 如果涉及用户本人，由 Host context 提供，不由模型猜。

### 46.6 Tag / topology

可能连接：

```text
coffee
Ethiopia
preference
```

但 embedding similarity 本身不自动创建永久 association。

### 46.7 Serving generation

Memory 成为 PostgreSQL Authority 后，相关 projection 增量/新 generation 发布。

### 46.8 Meaningful use

以后模型真的引用：

> “你之前提到最近更喜欢埃塞俄比亚豆……”

Host 回传 `referenced` use event，才构成 meaningful use。

---

# 47. 同一张图片被转发两次会发生什么

第一次：

```text
Artifact X
Occurrence A from Alice
```

第二次：

```text
Artifact X        ← CAS 去重
Occurrence B from Bob
```

视觉模型只需要对 Artifact/Region 解释一次时，可以共享 DerivedRepresentation；但：

```text
“Alice 给我发过这张图”
“Bob 后来又转发给我”
```

是两段不同 Specific cognition。

这就是内容去重和经历连续性可以同时成立的方式。

---

# 48. 图片模型消失后的行为

昨天：

```text
image provider READY
→ OCR
→ image_description
→ persisted textual surrogate
```

今天 provider 被删除。

Query：

> “之前那张东京塔照片里是谁穿红衣服？”

系统可能：

```text
image-native retrieval unavailable
BUT
persisted image_description READY
→ Tantivy/text embedding 找到
→ 返回时明确：这是历史视觉解释
→ 提供 raw image materialization handle
```

如果用户要求：

> “重新看原图确认一下。”

而当前没有视觉 capability，系统必须说该能力不可用，而不是把旧 description 冒充重新观察结果。

---

# 49. 一个当前 Schedule 问题的完整流程

用户问：

> “我明天下午有安排吗？”

CognitiveQuery 包含：

```text
TemporalCue tomorrow afternoon
current authority preferred/required
```

Planner 看 Resource directory：

```text
Schedule READY
owns current appointments
```

于是：

```text
Memory lane
→ 可提供历史相关认知

Schedule resolver
→ 提供当前 Authority
```

即使 Memory 里有：

> “上周说过明天下午可能开会。”

如果 Schedule 当前显示取消，最终 current answer 应服从 Schedule。

这就是“记忆”和“现实状态”真正分开的效果。

---

# 50. Entity 改名场景

历史：

```text
2024: “王学姐”
2025: “猫猫”
2026: “Wang”
```

三类 Evidence 都绑定：

```text
EntityRef X
```

Query：

> “Wang 以前提到过什么咖啡？”

先：

```text
EntityRef X exact posting
```

得到候选域，再结合 lexical/dense/Wave 排序。

不需要：

- 把历史文本改成 Wang；
- 重新 embedding 全部历史；
- 期待向量模型自己知道昵称等价。

---

# 51. 错误身份合并场景

最初 Host 错把两个“Alex”都绑定 Entity X。

后来纠正：

```text
Mention 17, 19, 31 → Entity Y
其余 → 仍为 X
```

Nous 产生新的 binding revisions。

然后：

```text
Entity posting generation 更新
WaveGraph affected edges 更新
```

原始 Artifact、Occurrence、surface snapshot、Memory evidence 全部不改。

所以系统能真正支持“修正认知索引，而不是重写历史”。

---

# 52. 一次跨域联想 Query 的完整内部流程

Query：

> “为什么看到 Alice 会让我想到上次 presentation？”

### 52.1 Runtime

先看当前 Session 是否已经 Resident：

```text
Alice
某个 project Anchor
presentation context
```

### 52.2 Exact/Entity

EntityRef Alice 直接取 posting。

### 52.3 Lexical

Tantivy 找包含：

```text
Alice
presentation
相关错误码/具体措辞
```

的内容。

### 52.4 Dense

Text embedding 产生 direct semantic candidates。

### 52.5 EPA / Residual

发现 query 同时触及：

```text
person/social axis
presentation/work axis
```

Residual Pyramid 可能感应出：

```text
failure
stage fright
project-X
```

### 52.6 SourceField

形成 seed mass。

### 52.7 QueryRiver

沿 WaveGraph 有界传播，实际出现：

```text
Alice
→ project-X
→ presentation
→ failure-pattern
```

### 52.8 Dual fields

LocalField 保持 Alice/project-X 附近。
TransferField 延伸到 failure-pattern/相关经历。

### 52.9 Candidate superset

综合所有 lane 收集 Memory 候选。

### 52.10 Candidate Trail

某条具体 Memory 的 ordered trail 可能正好：

```text
Alice
→ project-X
→ presentation failure
```

### 52.11 Ω + topology

本轮实际边流充分、有 emergent nodes、flow 分布非退化，因此 Ω 足够高。

这个候选获得 topology innovation bonus。

### 52.12 Result

返回的不只是：

> “Memory 42 score=0.91”

而是：

```text
Entity Alice exact
Dense match
TransferField contact
QueryRiver path support:
  Alice -> project-X -> presentation
Evidence:
  Occurrence ...
  SourceRegion ...
```

这才真正接近“可解释的联想回忆”。

---

# 53. 一次进程重启会发生什么

进程崩溃前 Session 有：

```text
resident Alice
resident current project Anchor
resident last observation
```

USearch/Tantivy/Wave generation 都是 immutable artifacts。

重启后：

1. PostgreSQL 恢复 Session/ResidentSet；
2. 加载当前 serving-generation pointers；
3. 打开对应 Tantivy/USearch/Wave artifacts；
4. ArcSwap 发布 ServingSnapshot；
5. pending derivation lease 到期后可恢复；
6. model KV cache 丢失，不影响语义 runtime。

Subject 不会因为 Rust 进程重启就从“我刚才在谈 Alice”退化成完全冷启动。

---

# 54. 为什么没有 Graph Database

我们需要图算法，但这不意味着需要 graph DB。

Nous 的典型 topology workload 是：

```text
Authority 中增量写 relation evidence
→ 周期/触发式生成 immutable serving graph
→ Query 在有限局部范围高频只读传播
```

这非常适合：

```text
PostgreSQL canonical edges
→ compact CSR snapshot
→ petgraph/Rust numerical traversal
```

引入 Neo4j/Kuzu 等服务只会再产生：

- 第二份 durable graph state；
- 同步一致性；
- schema ownership；
- 部署依赖；
- query DSL。

目前没有语义需求证明这些成本值得。

Library-first 的含义不是“凡有数据库就用数据库”，而是：

> generic graph data structure/algorithm 用成熟库；项目只拥有自己真正特殊的认知语义。

---

# 55. 为什么没有“万能 Memory 对象”

未来 Nous 还会有：

- Persona/Self；
- Relationship cognition；
- Epistemic cognition；
- Goals/Commitments；
- Reflection；
- Diary；
- Dream/Simulation。

一个常见过度抽象会想立刻设计：

```text
CognitiveObject {
  type: string,
  payload: JSON,
  execute(...)
}
```

Nous 不这么做。

当前先完整实现 Memory MicroSystem 所需的：

```text
CognitiveRef
Evidence
Runtime
Tags/Anchors/Associations
Query
Resource
```

未来其他 MicroSystem 可以复用这些公共语义，但自己拥有自己的 domain state。

这符合 JDD：保留低成本未来 seam，但不提前制造万能框架。

---

# 56. 未来 Persona/Self 会怎样接入

未来 Persona 不需要把人格写成一个 Prompt 字符串然后每次拼接。

更合理的路径是：

```text
Character Seed（来源材料）
        ↓
Persona/Self Authority
        ↓
当前人格/自我认知结构
        ↓
ContextContribution
```

Memory 可以给 Persona 提供：

- Specific experiences；
- Integrative patterns；
- meaningful use；
- contradictions；
- Anchors。

Persona 又可以形成新的 cognitive contributions，但不会把所有东西塞回 Memory 表。

这就是为什么当前 Nous Wave 被定义为 Cognition system，而不是 Memory DB：Memory 是第一个完整 MicroSystem，不是最终边界。

---

# 57. 未来 Relationship cognition 会怎样接入

Relationship owner 可以维护：

```text
EntityRef X
→ relationship state/history
```

Memory 提供：

- 与 X 有关的 Specific memories；
- interaction Evidence；
- relationship-related Anchors；
- experiential associations。

Identity 仍由 Host 负责。

Relationship state 也不会被 Name/embedding 定义。

因此今天把 EntityRef/mention/rebind 设计好，是为未来 Social cognition 打基础，而不是提前实现 Social ontology。

---

# 58. “梦”“日记”“反思”以后不会变成定时脚本塞进 Memory

这些未来能力如果实现，会是独立认知操作：

```text
Dream/Simulation
→ 产生明确 origin=synthetic 的 Artifact/Occurrence/Cognitive output

Diary
→ 产生明确 origin=self-authored 的整合叙事

Reflection
→ 产生 Integrative/Persona/Epistemic proposals
```

它们仍然必须保留 provenance：

> “这是 Subject 自己模拟/总结的”，而不是“外部世界发生过”。

Nous 也不拥有：

```text
每晚 2 点必须做梦
```

调度属于 Host/Automation。

---

# 59. 系统为什么能随着数据量增长而不是越来越笨重

大规模长期认知的关键，不是把更多内容全部塞进 prompt。

Nous 的扩展策略是：

### Raw storage

CAS 去重，冷数据不占数据库大行。

### Authority

PostgreSQL 保存结构和历史。

### Serving

USearch/Tantivy/CSR 只保存为检索优化需要的 projection。

### Runtime

只维持 bounded ResidentSet。

### Resource awareness

知道大型资源在哪里，而不是复制完整资源。

### Progressive disclosure

```text
Ref
→ synopsis
→ relevant region
→ exact evidence
→ raw bytes
```

所以增长的是可寻址认知空间，不是每轮 context 长度。

---

# 60. 一次 Query 的成本为什么是有边界的

所有危险的搜索都受硬边界约束：

```text
candidate union cap
USearch topK
Tantivy topK
Wave max hops
Wave max states
Wave max neighbors/node
Residual levels
Tag topK/level
rerank candidate cap
result limit
```

并且 Wave 不做每次全图稳态求解。

因此无论长期图变多大，正常 Query 都应该是：

```text
索引级候选访问
+ bounded local computation
```

而不是：

```text
scan all memories
scan all graph edges
```

这也是为什么最终实现使用 immutable CSR，而不是请求时临时构整张图。

---

# 61. 对用户来说，系统最终会表现成什么

一个成功的 Nous Wave 不应该让用户感受到：

> “系统突然搜索了一下数据库。”

而应该表现为长期连续：

- 刚说过的事情下一句自然还记得；
- 很久以前的具体经历在合适 cue 下能回来；
- 人换昵称后仍然认识；
- 一个词/人/事件能触发个体化联想，而不是公共语义 KNN；
- 新经验能够形成更高层的理解，但旧经历仍然可追溯；
- 图片/音视频曾经理解过后，即使模型能力变弱仍留有认知痕迹；
- 当前日程/任务不会被过期记忆冒充；
- 模型更换不会把 Subject 换掉；
- 被回忆得多不等于自动被强化；
- 删除、压制、遗忘、当前不活跃各有准确含义。

---

# 62. 最终系统结构图

```text
┌──────────────────────────────────────────────────────────────┐
│                  Host / External World Authorities           │
│ Messaging · Schedule · Tasks · Identity · Tools · Resources │
└──────────────────────────────┬───────────────────────────────┘
                               │ observations / stable refs
                               ▼
┌──────────────────────────────────────────────────────────────┐
│                     Evidence / Material                      │
│ Artifact · ObservationOccurrence · SourceRegion              │
│ DerivedRepresentation · DerivedRegion · provenance           │
└──────────────────────────────┬───────────────────────────────┘
                               │ formation / consolidation
                               ▼
┌──────────────────────────────────────────────────────────────┐
│                    Cognitive Authority                       │
│ Specific Memory · Integrative Memory · Procedural Memory     │
│ Tags · Anchors · Association Evidence · Resource Awareness   │
│ immutable revisions · temporal validity · EntityRef binding  │
└───────────────┬──────────────────────────────┬───────────────┘
                │                              │
                │ runtime                      │ rebuild projections
                ▼                              ▼
┌──────────────────────────────┐   ┌───────────────────────────┐
│ Semantic Cognitive Runtime   │   │ Serving Generations       │
│ CognitiveSession             │   │ Exact/Postings            │
│ ResidentSet                  │   │ Tantivy lexical           │
│ Meaningful Use               │   │ USearch vectors           │
└───────────────┬──────────────┘   │ Wave CSR + EPA basis      │
                │                  └──────────────┬────────────┘
                └───────────────┬─────────────────┘
                                ▼
┌──────────────────────────────────────────────────────────────┐
│                    Cognitive Query Planner                   │
│ Runtime · Exact · Entity · Lexical · Dense · Tag · Anchor   │
│ Resource · Temporal · EPA · Residual Pyramid                 │
│ Wave → QueryRiver → Local/Transfer → Topology                │
└──────────────────────────────┬───────────────────────────────┘
                               ▼
┌──────────────────────────────────────────────────────────────┐
│              Evidence-aware Cognitive Results               │
│ refs · revisions · temporal state · provenance · match trace │
│ materialization handles · degradation · resource actions     │
└──────────────────────────────┬───────────────────────────────┘
                               ▼
┌──────────────────────────────────────────────────────────────┐
│ ConsumerWorkingSet → ContextContribution → Host Compositor   │
│                    → any model/provider                      │
└──────────────────────────────────────────────────────────────┘
```

---

# 63. 这版架构真正解决了哪些此前悬而未决的问题

到这一版为止，已经明确：

```text
Authority store          PostgreSQL
raw artifact             OpenDAL+BLAKE3 CAS
lexical                  Tantivy
dense                    USearch
topology                 PostgreSQL evidence → immutable CSR
runtime                   Session/ResidentSet in semantic Authority
provider model            operation × modality capability
vector compatibility      explicit EmbeddingSpaceSignature
multimodal                raw + persistent textual surrogate + optional native vectors
entity identity            upstream EntityRef + revisioned mention binding
memory lifecycle           Evidence → optional formation → immutable revisions
association               evidence-backed, not cosine
runtime reinforcement      meaningful use only
retrieval API              typed CognitiveQuery AST
retrieval strategy         multi-addressing candidate superset
primary topology algorithm clean-room VCP-informed bounded Wave
query topology             request-local actual-flow river
ranking                    evidence families + field/topology bounded positive correction
current facts              external Resource Authority
index consistency          immutable ServingGeneration snapshot
async work                 event-driven derivation ledger, no autonomous cognition scheduler
```

这意味着 Coding Agent 不再需要在实现中替架构师决定这些问题。

---

# 64. 仍然故意没有冻结的东西

即使 Production Spec 很详细，也不应该假装整个 Cognition 已经研究完。

仍然可以以后演化：

- Persona/Self 的内部表示；
- Relationship cognition 的完整状态模型；
- Anchor 更高级的形成/合并策略；
- Tag taxonomy；
- Integrative/Procedural consolidation prompt/model；
- 真正长期 empirical tuning 后的 Wave 参数；
- accessibility 的更精细学习模型；
- 多模态 native embedding 的默认模型；
- Resource synopsis/GraphRAG-like global representation 何时值得构建；
- Dream/Reflection/Diary；
- server-scale specialized projection backend。

这些没有冻结，不意味着要现在做 adapter framework。

当前系统已经为它们保留了足够低成本的语义接口：

```text
CognitiveRef
Observation
Memory
Tag/Anchor/Association
ResourceRef
Capability
ContextContribution
```

只有真正开始研究某个 MicroSystem 时，再增加它自己的 Authority。

---

# 65. 最后：Nous Wave 与“高性能记忆系统”的关系

如果只看性能，最简单的做法可能是：

```text
SQLite/Qdrant
+ embedding
+ topK
```

但那解决的是 search performance，不是长期 Subject cognition。

Nous Wave 的性能目标更难：

> 在维持 Evidence、身份连续性、时间、运行时、个体化拓扑、多媒体、provider 可替换性和外部 Authority 边界的前提下，仍然让正常认知寻址保持局部、有界、索引化、可并行。

因此这里的“高性能”不是为了做一个更大的 ANN benchmark，而是：

```text
大量长期数据存在
但一次思考只触及真正相关的很小一部分
```

这也是“认知系统”与“存储系统”的真正分界。

Nous Wave 最终要达到的状态不是：

> “它什么都保存。”

而是：

> **它能在长期时间尺度上保持一个 Subject 的认知连续性；知道自己经历过什么、当前在想什么、哪些关系只属于自己、哪里能找到最新事实，并且在需要时以可解释、可追溯、有边界的方式把这些认知重新带回前景。**
