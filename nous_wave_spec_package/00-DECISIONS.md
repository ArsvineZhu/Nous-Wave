# 00 — Current Decisions

This file separates decisions by status so implementation defaults do not masquerade as permanent cognitive theory.

## Status vocabulary

- **LOCKED** — current implementation baseline; do not reopen without concrete contradictory evidence.
- **DERIVED** — strongly follows from locked semantics and may be implemented directly in this wave.
- **DEFAULT** — selected implementation route; replaceable if real evidence shows a better route.
- **OPEN** — intentionally unfrozen research seam.
- **FUTURE** — recognized architectural area, not authorized for internal implementation in the current wave.

## D1. Project identity — LOCKED

The project name is **Nous Wave**.

Nous Wave is an independent, general-purpose cognitive system. It does not name, import, document, or depend on a particular host product as part of its own identity.

## D2. Subject initialization — LOCKED

A subject can be created with only:

```text
Subject identity
+ Character Seed
+ basic configuration
```

The Character Seed is human-authored free-form source text, analogous to a role card or initial character prompt. It is a legitimate initialization primitive.

`State > Prompt` means durable cognitive state must not be reduced to prompt text; it does **not** prohibit a foundational prompt/role-card source.

A subject MUST NOT require preloaded memories, relationships, beliefs or persona ontology to exist.

## D3. Character Seed and evolved cognition are distinct — LOCKED

The original Character Seed is retained with provenance and history.

Future Persona/Self state may interpret, refine, diverge from or evolve beyond the seed. The seed is not overwritten into “current personality”.

## D4. MicroSystem architecture — LOCKED

Nous Wave uses logically independent cognitive **MicroSystems**.

```text
MicroSystem = semantic owner + typed operations + explicit dependencies + readiness
MicroSystem != process
MicroSystem != container
MicroSystem != network service
MicroSystem != marketplace plugin
```

Current deployment is a modular monolith with static/startup composition.

No runtime unload, ABI layer, universal plugin registry, generic `execute(JSON)` contract or distributed service discovery is authorized.

## D5. Optional cognitive systems — LOCKED

The system must remain runnable when optional cognitive MicroSystems are absent.

Operation availability is capability-dependent. If an operation genuinely requires an absent MicroSystem, return unavailable/degraded truthfully; do not fabricate fallback cognition.

The current required core is Subject Core. Memory is the first implemented optional cognitive MicroSystem and is expected to be enabled in the reference profile.

## D6. No internal scheduler — LOCKED

Nous Wave does not own autonomous scheduling policy.

It may expose operations that are useful offline or asynchronously, including consolidation, reflection, diary generation or simulation when those systems exist. Invocation timing belongs to the host, user, external scheduler, automation system or agent runtime.

No `nightly`, `every N hours`, cron-like or self-wakeup machinery belongs in Nous Wave core.

## D7. Current implementation wave — LOCKED

This wave fully implements the **Memory MicroSystem**.

Persona/Self, Social, Epistemic, Goals/Commitments, Reflection, Diary and Dream/Simulation are boundary-only FUTURE MicroSystems in this package.

Memory completeness is not reduced because other MicroSystems are deferred.

## D8. Cognitive material is broader than messages — LOCKED

Memory ingestion is source-agnostic.

Potential material includes:

- one short message;
- a long conversation slice;
- arbitrary-length text;
- Markdown/HTML/JSON/code;
- MCP/tool invocation results;
- browser/search/API/database outputs;
- files and documents;
- images/audio/video through artifact references and derivations;
- host events;
- imported archives;
- future self-generated diary/reflection/simulation artifacts.

`Message != Memory` and `File != Memory`.

## D9. Classification axes are orthogonal — LOCKED

At minimum distinguish:

```text
Origin / Source Class
Semantic Class
Epistemic Class
Representation / Media Type
```

File extension or MIME type does not determine semantic meaning.

Two Markdown artifacts may respectively be a user document, a tool report, a diary, or a dream.

## D10. Artifact and cognitive memory are distinct — LOCKED

Raw/received/generated artifacts preserve source material. Cognitive memory objects are derived or formed from one or more artifacts/evidence items.

Supported cardinalities include:

```text
one artifact -> many cognitive objects
many artifacts -> one episode/memory
one cognitive object -> many revisions/interpretations
```

Preserving a source does not imply indexing its entire payload as one recall unit.

## D11. Tool/MCP output is evidence, not automatic truth — LOCKED

A tool result records that a tool/system reported or returned something under a specific invocation and context.

It may support claims or memory formation, but does not bypass epistemic interpretation because it came from a tool.

## D12. Rust core — LOCKED

Rust owns:

- service runtime;
- canonical memory semantics;
- revision/history logic;
- recall planner;
- association algorithms;
- Cognitive Work Cycle;
- hot working state;
- local retrieval integration;
- ingestion/projection orchestration.

Python remains appropriate for research, training and external model workers. Hosts may use any language through the public protocol/client surface.

## D13. PostgreSQL 18.x Authority — LOCKED

PostgreSQL owns canonical structured durable state:

- Subject Core metadata;
- cognitive-material metadata;
- source/provenance records;
- artifact catalog;
- episodes and memory objects;
- revisions/current heads;
- entity/relation metadata required by Memory;
- association evidence and learned accessibility state;
- Cognitive Work Cycle durable events where persistence is justified;
- corrections/suppression/purge state;
- derivation metadata;
- projection obligations.

Current stable implementation baseline at spec time is PostgreSQL 18.6. Patch-level updates within 18.x are maintenance, not architecture changes.

## D14. Object repository — LOCKED

Large/raw payloads use a content-addressed object repository rather than being forced into PostgreSQL rows or retrieval indexes.

Apache OpenDAL owns storage-provider mechanics.

First backend: local filesystem. Future server deployments may use S3-compatible storage without changing Memory semantics.

## D15. Local Retrieval Projection — LOCKED default

LanceDB OSS embedded is the default PC/local Retrieval Projection unless actual implementation or benchmark evidence reveals a hard blocker.

This is a serving projection, not cognitive Authority.

Qdrant Edge, pgvector, Milvus, Vespa or another engine may be evaluated later when a real workload justifies it; their existence does not require a multi-provider implementation now.

## D16. Authority history — LOCKED

Durable cognitive state uses:

```text
stable logical object identity
+ immutable revision snapshots
+ lineage/parent relations where needed
+ explicit current heads
```

Do not default to mutable overwrite or long patch chains.

## D17. Direct read and recollection are different — LOCKED

Current-state cognitive facts should be directly readable from their owner when such an owner exists.

Recollection reconstructs past/episodic/associative material and must not become the only way to retrieve current state.

For the current wave this mainly affects Memory current metadata and future integration seams.

## D18. Structured Recall Intent — LOCKED

Recall is not only `query: string`.

The caller may provide known semantics such as target, objective, temporal perspective, cues, constraints, result needs and explicit requested effort.

Query text remains one cue.

## D19. Explicit recall effort — LOCKED

Semantic effort levels:

```text
light
normal
deep
maximum
```

They express willingness to spend cognitive/retrieval work, not hard-coded execution presets.

Planner chooses the physical route.

## D20. Cognitive Work Cycle — LOCKED

Multiple recalls in one reasoning activity share a `cycle_id` and process-local working cognitive state.

Later recalls inherit useful frontier/candidate/activation/inspection state instead of restarting the whole retrieval pipeline.

## D21. Retrieved != reinforced — LOCKED

Candidate generation or ranking alone never strengthens memory.

Useful stages are distinguished, e.g.:

```text
candidate
surfaced
inspected
selected/exposed
referenced/followed
useful downstream effect
```

Learning uses meaningful cognitive-use events, not raw retrieval count.

## D22. Forgetting, suppression and purge — LOCKED

- **Forgetting/accessibility change** affects likelihood/effort of recollection while retaining the memory.
- **Suppression** makes an object intentionally unavailable to ordinary recall without claiming physical deletion.
- **Purge** is an authoritative deletion workflow across source, artifacts, derivations and serving projections as required by scope.

## D23. Association is not semantic similarity — LOCKED

Personal/experiential association may connect objects far apart in embedding space.

Embedding similarity is a retrieval signal. It must not automatically create permanent experiential associations.

## D24. Memory association graph — DERIVED

Memory uses a heterogeneous derived association projection over typed memory-relevant nodes instead of a Tag-only universal ontology.

Canonical domain objects retain their own types. Association projection is optimized for recall.

## D25. VCP/Memoria lineage — DERIVED

VCP Toolbox and Memoria are algorithmic reference lineages, not code authority.

Preserve demonstrated problems and useful invariants; do not preserve historical coupling, naming or implementation shape by default.

## D26. Current association algorithm direction — DEFAULT

The primary research/implementation path is bounded competitive spreading activation informed by the VCP lineage, with finite budgets, directional evidence, hub control and anti-backtracking behavior.

PPR/diffusion remains a known alternate/secondary algorithm seam, not a mandatory fixed pipeline stage.

Do not create a generic algorithm marketplace merely to preserve this seam.

## D27. Quantification discipline — LOCKED

Do not invent numeric scales for cognitive concepts merely because they are easy to store.

Numerical values are appropriate when they have operational meaning, e.g.:

- confidence;
- retrieval score;
- evidence support;
- accessibility estimate;
- association evidence/strength;
- recency/temporal distance;
- resource/effort metrics.

Concepts such as personality, relationship, emotion, belief content or self-narrative remain structured/qualitative unless their future owner establishes a justified measurement model.

## D28. Future Persona/Social/Epistemic/Diary/Dream semantics — FUTURE

Only their cross-MicroSystem contracts are frozen in this package.

Their detailed ontology, learning algorithm, schedule, prompt strategy, file organization and evolution rules remain open for later research.

VCP Diary/Dream prototypes are references, not implementation specifications.

## D29. Model roles — OPEN

Exact embedding model, reranker, generative extraction model, dimensions, quantization, sparse representation and local inference runtime are intentionally unfrozen.

Implement stable model-service contracts and provenance; choose concrete defaults in configuration without turning them into cognitive architecture.

## D30. Server-scale retrieval/backend — OPEN

Server deployments may later use a different retrieval projection or object backend. The current wave implements local/PC defaults only, behind capability boundaries justified by the already-known deployment split.

## D31. Query factorization / residual cue discovery — OPEN

The VCP lineage's EPA/Residual Pyramid addresses a real problem: dominant semantic directions can hide weaker independent cues in a complex query.

The current Memory wave does not freeze EPA, Gram-Schmidt residual peeling or a fixed residual-pyramid algorithm as production law.

Structured Recall Intent already preserves caller-known factors; future latent-cue discovery may complement it when free-text cues are underspecified. Preserve an internal planner seam for cue enrichment without implementing a generic query-language framework.
