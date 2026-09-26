# 2026-10-27 Nous Wave 中期检查计划

状态：ACTIVE MILESTONE PLAN
父计划：[Target Engineering Plan](../roadmap/2026-09-target-engineering-plan.md)

语义 Authority：[Architecture-Vault Nous Wave Target Design](https://github.com/Heptalogos-Devs/Architecture-Vault/blob/main/docs/Nous-Wave/TARGET_DESIGN.md) 及其已接受设计决定。

## 1. 定位

10 月 27 日中期检查交付：

**Memory Reference Profile R1**

它是 Nous Wave 长期目标工程计划的一条完整纵向阶段成果。

它不是：

- Nous Wave Lite；
- 产品 MVP；
- 目标设计缩水版；
- 为演示写的独立 demo architecture。

时间不足时，优先删除：

- optional topology；
- advanced algorithms；
- static Self stretch；
- UI/presentation；

而不削弱：

- Authority；
- provenance；
- revision；
- valid-time；
- idempotency；
- serving consistency；
- lifecycle correctness。

---

# 2. 中期研究问题

阶段成果至少回答：

1. 一个长期 Subject 如何把材料、实际观察和持久 Memory 区分开？
2. Memory 如何保留来源和派生关系，而不把多个同源表示误当成独立佐证？
3. 如何区分“同一认知被纠正”和“世界在后来发生变化”？
4. 检索服务如何高效工作但不成为认知真值？
5. QueryPlan 如何避免结果集合反向改变融合定义？
6. 系统如何记录真实 use，而不是把 retrieval hit 自动强化？
7. suppression / restore / purge / accessibility 如何保持不同语义？
8. restart / rebuild 后 Authority 和 Serving 是否仍一致？
9. provenance 能否从 retrieved Memory 反向追溯到 Observation / Artifact？
10. advanced topology 是否真的比强 lexical+dense baseline 提供增量？

---

# 3. 范围

中期范围对应 Target Engineering Plan：

- Phase 0；
- Phase 1；
- Phase 2；
- Phase 3；
- Phase 4 中与 Memory reference path 直接相关的部分；
- Phase 5 baseline；
- Phase 6 closure。

不进入：

-完整 Self evolution；
- Social Cognition；
- Motivation implementation；
- Desired Condition implementation；
- Offline Cognition full workflow；
- Heptalogos live integration；
- production-hardening；
- full cross-platform qualification。

---

# 4. 中期前需要编写的 Specs

文档整理完成后的下一步才开始 Spec。

建议按施工顺序写：

## Spec A — Memory Authority & Provenance

冻结：

- object/revision identity；
- cognitive role / formation mode；
- evidence/provenance；
- aboutness；
- time semantics；
- correction vs new object；
- relation semantics；
- CognitiveSchema boundary；
- lifecycle；
- idempotent mutation semantics。

## Spec B — Cognitive Runtime & Use

冻结：

- stable UseEvent identity；
- use kinds；
- retrieval trace vs semantic use；
- Session / ResidentSet impact；
- accessibility input/output contract；
- minimum WorkContext interaction required by reference profile。

## Spec C — Cognitive Query & Serving

冻结：

- typed query；
- QueryPlan；
- lanes；
- budgets；
- hard filter；
- multiview aggregation；
- baseline fusion；
- generation/watermark；
- stale validation；
- restart/rebuild。

## Spec D — Reference Profile Qualification

冻结：

- scenario corpus；
- PASS/FAIL expectations；
- restart / purge / idempotency tests；
- baseline evaluation；
- demo flow；
- benchmark/report output。

---

# 5. Must-have semantic closure

## Authority

- Memory Object 有稳定 identity；
- Revision immutable；
- head/epoch fenced；
- role × formation 正交；
- valid time；
- contradiction / correction / derivation；
- provenance references；
- source dependency 可追溯。

## Runtime

- retrieval trace 不自动成为 long-term use；
- use event 稳定幂等；
- accessibility 是 query-time policy；
- exact/management/provenance access 不被 auto accessibility 错误屏蔽。

## Query

- QueryPlan 先于 candidate generation；
- lane budget 固定；
- candidate collection 不反向改变其他 lane weighting；
- view 先聚合到 object revision；
- return 前 Authority revalidation。

## Serving

- immutable/rebuildable generation；
- Authority watermark；
- producer identity；
- vector-space identity；
- stale result rejection；
- restart/rebuild。

## Lifecycle

- suppression；
- restore；
- purge；
- session eviction；
- serving invalidation；

分别验证。

---

# 6. Reference retrieval baseline

中期默认先证明强、简单、稳定的 baseline：

```text
exact
+ entity
+ lexical
+ dense
+ runtime/temporal where applicable
→ object-revision aggregation
→ fixed fusion
→ optional bounded rerank
→ final Authority validation
```

不要求 topology 成为默认路径。

## Topology research

在 baseline 稳定后，作为实验 lane 比较：

- no graph；
- simple bounded expansion；
-现有 Wave；
- PPR 或其他候选。

评价增量时必须固定：

- same corpus；
- same QueryPlan；
- same candidate budget；
- same embedding；
- same final validator。

如果没有可靠增量，就保持 experimental。

---

# 7. Nous Integrity Suite

中期必须建立项目自己的 integrity scenario set。

至少包括：

## Provenance

1. 同一 Artifact 的两个 summary 不算两份 independent evidence；
2. derived representation 可以追溯到原始 region；
3.多个模型对同一来源的重述不自动增加 source independence。

## Temporal / Revision

4. 同一 valid interval 的错误事实被纠正 → revision；
5. 后续新 world state → new cognition / successor；
6. 旧 valid-time 历史仍可精确读取。

## Entity isolation

7. 同一 Observation 出现 A/B，不因 A 的 cognition aboutness 把 B 错绑定到 Memory；
8. EntityMention 与 CognitionAboutness 可以不同。

## Use

9. retrieval hit 不产生 meaningful use；
10. 同一 stable use event 重试只记一次；
11. same event id + different digest → conflict。

## Serving

12. stale generation 不返回已经失效的 revision；
13. rebuild 后结果仍指向同一 Authority identity；
14. dense vector-space identity 不匹配时禁止直接混合分数。

## Lifecycle

15. suppression 不等于 purge；
16. restore 不新建虚假 content revision；
17. purge 后 lexical/dense/derived projection 不能恢复被清内容。

## Diagnostics

18. budget 不足与 Authority 中不存在对象区分；
19. service projection unavailable 与“主体不知道”区分。

## Recovery

20. process restart 后 Authority 恢复；
21. serving generation 可以从 Authority 重建；
22. use event / revision idempotency 在 restart 后仍成立。

---

# 8. 外部 benchmark 使用方式

中期不追求用 benchmark 排名证明整个 Nous Wave。

选取适合验证目标的部分：

## LongMemEval / LongMemEval-V2

重点映射：

- dynamic state tracking；
- premise awareness；
- workflow / procedural experience；
- context gathering。

## Memora

重点：

- obsolete memory；
- knowledge update；
- forgetting-aware correctness。

## Mem2ActBench

暂时只借鉴评价思想：

- 真实 use / action utilization 与 passive retrieval 区分。

Heptalogos live action integration 尚未进入本里程碑，所以不声称完成完整 Mem2Act capability。

---

# 9. 时间安排

## 9/26–10/1：文档与 Specs

- 完成本轮文档 Authority 整理；
- 修订 Target Plan / Active Plan；
- 编写 Spec A–D；
- 使既有验证链基线可用。

## 10/2–10/9：Authority rebase

- Memory classification；
- identity/revision；
- evidence/provenance；
- valid-time；
- CognitiveSchema boundary；
- protocol/persistence alignment。

## 10/10–10/15：Runtime / Query

- UseEvent idempotency；
- accessibility；
- fixed QueryPlan；
- multiview aggregation；
- stable baseline fusion。

## 10/16–10/20：Serving / lifecycle / recovery

- generation/watermark；
- stale validation；
- rebuild/restart；
- suppression/restore/purge；
- trace-back。

## 10/21–10/24：Evaluation

- Nous Integrity Suite；
- selected external benchmark cases；
- baseline vs optional topology experiments；
- latency/resource observations。

## 10/25–10/27：Qualification / presentation

- full verification；
- qualification report；
- architecture diagram；
- experiment results；
- demo data；
-阶段性 PDF / report 仅在此时按需要生成。

---

# 10. 中期可选 Stretch

只有 Must-have 已稳定后再做。

优先级：

1. static read-only Self Profile；
2. Episode reference model demonstration；
3. topology experiment；
4. advanced rerank/fusion experiment。

这些都不能阻塞 Reference Profile acceptance。

---

# 11. 中期交付物

至少包括：

- updated Target Design / Decisions；
- Target Engineering Plan；
- four Executable Specs；
- target-aligned Memory code；
- migration/protocol implementation；
- Integrity Suite；
- selected benchmark harness；
- Qualification report；
- demo script；
- research comparison results。

中期检查应能说明：

> Nous Wave 已经证明一套长期 Subject Memory 的 Authority、时间、来源、检索、使用和生命周期语义可以在真实代码中形成闭环。

而不是只展示“模型记住了几句话”。

---

# 12. 停止规则

达到中期 Must-have 与 Qualification 后停止扩张。

不要在最后一周临时加入：

- Social；
- Motivation；
-完整 Self evolution；
- Heptalogos live connector；
- graph algorithm rewrite；
- UI。

如果核心语义尚未闭环，删除 optional research lane，而不是削弱核心合同。
