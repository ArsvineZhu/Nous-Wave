# 10 — Memory Evaluation and Acceptance

## 1. Evaluation principle

Memory quality is not one Recall@K score.

The current wave evaluates:

```text
source/provenance correctness
memory formation correctness
temporal correctness
associative recollection
work-cycle behavior
learning/forgetting behavior
purge/portability correctness
traditional IR quality
latency/resource cost
```

Use a scorecard/Pareto view, not one opaque “NousScore”.

## 2. Source/material correctness

Required scenarios:

- one short human message;
- one very long Markdown/text source;
- multi-turn conversation -> coherent Episode;
- JSON MCP/tool result with invocation provenance;
- tool error/partial result;
- file artifact with no parser available;
- file + externally supplied derivation;
- same Markdown media type with different semantic/epistemic classes;
- future-style self-generated narrative/simulated artifacts retained without truth pollution.

Prove classification and provenance survive Memory formation/Recall.

## 3. Artifact-memory separation

Tests must show:

- one Artifact can produce multiple MemoryItems;
- multiple source Artifacts can form one Episode;
- raw source remains addressable after chunking/summarization;
- deleting/rebuilding a serving projection does not delete source/memory Authority.

## 4. Temporal correctness

Cases include:

- “what happened around date X?”;
- old state versus new state;
- later correction of an earlier interpretation;
- source observed later than event occurrence;
- approximate time range;
- recency must not cause an invalid newer/older fact to replace correct historical selection.

## 5. Private experiential association

Construct private synthetic names/events absent from model pretraining so semantic priors cannot solve the task alone.

Examples test:

- personally associated concepts far apart in embedding space;
- narrative order;
- multi-hop episode/entity association;
- hub pollution;
- short A↔B loops;
- independent multiple-seed support.

This inherits the useful experimental spirit of the VCP lineage without copying its benchmark implementation.

## 6. Work Cycle behavior

Compare logically:

```text
independent recall calls
vs
same-cycle incremental recollection
```

Measure:

- target recovery;
- recall rounds;
- repeated candidates;
- index queries/model calls;
- time-to-sufficient-context;
- association edge visits;
- context exposure waste.

The product implementation must demonstrate reuse, not merely accept a `cycle_id` parameter while restarting internally.

## 7. Retrieved != reinforced

Required regression:

1. repeatedly surface a top result without inspect/exposure/use;
2. verify its learned accessibility/association does not strengthen solely from surfacing;
3. then generate meaningful follow/use events;
4. verify the permitted learning signal changes.

## 8. Forgetting/accessibility

Required scenario:

- a long-unused memory becomes less likely under `light/normal` associative recall;
- exact reference can still recover it;
- `deep/maximum` with appropriate cues can recover it more often;
- no data was physically deleted.

Do not require one universal numeric threshold to define human-like forgetting.

## 9. Suppression/purge

### Suppression

Ordinary recall excludes suppressed items immediately; authorized direct inspect still sees suppression/history.

### Purge

After purge:

- canonical target is gone according to policy;
- derived Memory/association/projection data is gone;
- exclusive object bytes are removed;
- shared bytes remain only if another retained reference justifies them;
- projection rebuild cannot resurrect purged content.

## 10. Correction/history

Prove:

- immutable source retained;
- wrong interpretation has historical revision;
- corrected current head is returned for current interpretation;
- historical query can explain what was previously believed/remembered when semantics support it;
- correction provenance is visible.

## 11. Consolidation

Prove consolidation:

- can merge/abstract repeated memories;
- preserves parent/support refs;
- does not erase raw evidence;
- respects contradictory evidence;
- is only invoked explicitly by caller/host;
- restart does not duplicate the same consolidation result when operation identity is reused.

## 12. Traditional IR diagnostics

Track at least where ground truth permits:

- Recall@K;
- Precision@K;
- MRR;
- nDCG;
- hit rate;
- reranker accuracy.

These diagnose retrieval but do not replace cognitive scenarios.

## 13. Effort-quality frontier

For benchmark targets record:

- success/quality at light/normal/deep/maximum;
- observed RecallEffortTrace;
- minimum successful effort under defined cue/context;
- latency/CPU/model-call cost.

This is central to studying accessibility and algorithm improvements.

## 14. Public benchmark references

Where compatible, adapt/use:

- LongMemEval / LongMemEval-V2 style longitudinal memory tasks;
- LoCoMo-style long conversational memory;
- temporal and multi-hop tasks inspired by Graphiti/Zep and related systems;
- static RAG retrieval as a control baseline.

Do not distort Nous semantics merely to optimize a public benchmark schema.

## 15. Scale profiles

Use staged datasets rather than assuming billion-scale deployment:

```text
100K objects/material units -> correctness/debug profile
1M                       -> early realistic scale
10M                      -> long-lived stress profile
50M/100M                 -> optional ceiling probes when affordable
```

The current release does not require proving every ceiling before useful development continues.

## 16. Performance metrics

Measure:

- P50/P95 Recall latency;
- P50/P95 cycle latency;
- ingestion acceptance latency;
- derivation/projection lag;
- recalls per resolved cycle;
- repeated candidate ratio;
- candidate waste;
- context waste;
- CPU time per cycle;
- model inference time/calls;
- memory footprint of hot association graph;
- storage growth.

## 17. Algorithm acceptance

The initial competitive activation implementation must beat or meaningfully complement dense/lexical baselines on private experiential/associative cases without unacceptable hub/loop behavior.

If it fails, that is evidence to reopen the algorithm, not permission for Coding Agent to silently add five alternate graph frameworks.

## 18. Release acceptance for this wave

Memory MicroSystem is complete when all are true:

```text
Subject + Character Seed creation works
heterogeneous material ingestion works
tool/MCP provenance works
artifact/source lineage works
memory formation works
structured recall works
work-cycle reuse is real
association/accessibility feedback works
correction/forgetting/suppression/purge work
consolidation works only when explicitly invoked
export/import + projection rebuild work
current required semantic/performance tests pass
future MicroSystems remain absent rather than fake
no internal scheduler exists
```
