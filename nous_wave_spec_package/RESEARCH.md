# Research Basis and Algorithm Lineage

This file records references that informed the current rolling-wave decisions. It is not implementation Authority where it conflicts with the normative specs.

## 1. VCP Toolbox lineage

Repository: https://github.com/lioensky/VCPToolBox

Relevant ideas recovered from the VCP/TagMemo lineage:

- residual/query factor decomposition to discover weaker semantic cues;
- ordered/private experiential association distinct from embedding similarity;
- bounded spreading/competitive activation concepts;
- association hub control and anti-loop behavior;
- diary as file-oriented self-authored material with tags/references/archival operations;
- AgentDream as memory-seeded association/simulation prototype.

Important architectural rejection:

VCP intentionally integrates memory, tools, autonomy, scheduling and agent runtime very tightly. Nous Wave reuses cognitive hypotheses, not that coupling. Diary/Dream are future MicroSystems; scheduling belongs to the host.

The current VCP whitepaper describes DailyNote `.txt/.md` diary material and AgentDream memory/tool/diary integration, which reinforces the need for generic Artifact + semantic/epistemic classification rather than a chat-message-only Memory model.

## 2. Memoria lineage

Repositories previously studied:

- https://github.com/ArsvineZhu/Memoria
- memoria-next development lineage

Recovered lesson:

successive JS -> TS -> Rust rewrites changed algorithm semantics. Current code is therefore reference implementation/evidence, not the canonical definition of the original cognitive mechanism.

Nous Wave specifies algorithms by problem/invariant first, then implements them cleanly.

## 3. Hindsight

Reference: https://github.com/vectorize-io/hindsight

Useful comparison points:

- explicit retain/recall/reflect separation;
- multi-route memory retrieval;
- long-term agent memory as a standalone service concern.

Nous Wave does not adopt Hindsight wholesale because its memory/world/mental-model semantics would overlap with future Nous Wave domain owners.

## 4. Graphiti / Zep temporal knowledge graph

Reference: https://github.com/getzep/graphiti

Useful ideas:

- incremental ingestion of unstructured/structured information;
- temporal/historical facts;
- hybrid semantic + lexical + graph retrieval;
- provenance and evolving relationships.

Nous Wave does not make a knowledge graph the universal cognitive Authority. Graph/association structures remain typed domain state or serving projections.

## 5. Mem0

Reference: https://github.com/mem0ai/mem0

Useful ideas:

- memory extraction from unstructured interactions;
- entity-linked graph retrieval;
- host/user/agent scoping;
- practical memory-service API ergonomics.

Current Mem0 graph memory mainly connects entities and memories through extracted entity co-occurrence. Nous Wave needs richer private experiential association and explicit Cognitive Work Cycle dynamics.

## 6. Letta

References:

- https://github.com/letta-ai/letta
- https://github.com/letta-ai/letta-code

Useful ideas:

- stateful agents with persistent editable memory;
- separation of in-context blocks and external recall memory;
- explicit memory evolution/history.

Nous Wave remains a cognitive substrate, not a generic agent runtime.

## 7. HippoRAG / graph associative retrieval

HippoRAG and HippoRAG 2 are relevant baselines for personalized PageRank/graph-augmented associative retrieval.

PPR remains a known alternate/secondary method. It is not frozen as a mandatory stage because Nous Wave's Work Cycle calls for incremental, stateful activation behavior in addition to static graph ranking.

## 8. SYNAPSE and spreading activation research

Recent work combining embedding retrieval, spreading activation, temporal decay and inhibition supports continued investigation of associative activation for long-term memory.

Nous Wave does not copy biological metaphors literally. Each mechanism must have a measurable retrieval/cognitive role.

## 9. Long-memory evaluation

Relevant families include:

- LongMemEval;
- LongMemEval-V2;
- LoCoMo and later long-conversation derivatives;
- private zero-shot synthetic concept tests;
- temporal evolution/correction tests;
- multi-step work-cycle tests.

Traditional Recall@K/MRR/nDCG remain diagnostics rather than the sole objective.

## 10. Technology references verified near spec time

As of 2026-09-06:

- Rust stable 1.98.1: https://blog.rust-lang.org/2026/09/03/Rust-1.98.1/
- PostgreSQL 18.6: https://www.postgresql.org/docs/release/18.6/
- SQLx 0.9.0: https://docs.rs/crate/sqlx/latest
- Axum 0.8.9: https://docs.rs/axum/latest/axum/
- LanceDB Rust crate 0.38.0: https://docs.rs/crate/lancedb/latest
- Apache OpenDAL 0.58.2: https://docs.rs/crate/opendal/latest

Patch/minor upgrades remain maintenance choices when they do not change the adopted role.

## 11. Query factorization / residual cue discovery

VCP's EPA and Residual Pyramid explored decomposition of query embeddings so weaker semantic factors survive a dominant direction. The underlying research question remains valid, especially for long/multi-factor free-text recall cues.

The current implementation prioritizes Structured Recall Intent and does not freeze the historical Gram-Schmidt/residual formulas. Latent cue enrichment remains an explicit future algorithm seam.
