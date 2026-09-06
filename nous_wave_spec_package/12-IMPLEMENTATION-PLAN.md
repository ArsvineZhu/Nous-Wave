# 12 — Decision-Complete Implementation Plan

## 1. Mission

Build a new standalone **Nous Wave** repository that realizes:

```text
Subject Core
+ MicroSystem composition boundary
+ complete Memory MicroSystem
```

Do not implement future Persona/Social/Epistemic/Goals/Reflection/Diary/Dream internals in this plan.

The executor implements the decisions in this package; it does not run architecture bake-offs or invent substitute designs.

## 2. Toolchain and adopted mechanics

Use:

```text
Rust stable 1.98.1
Rust edition 2024
Tokio 1.x
Axum 0.8.9
SQLx 0.9.0 with PostgreSQL
PostgreSQL 18.x (reference patch 18.6)
LanceDB 0.38.0
Apache OpenDAL 0.58.2
Serde / serde_json
UUID 1.x with UUIDv7 support
BLAKE3
tracing + tracing-subscriber
clap 4.x
thiserror 2.x
reqwest or equivalent mature HTTP client only for external model-service calls
```

Pin actual resolved versions in `Cargo.lock`.

Use `sqlx` directly rather than introducing an ORM/repository framework above it.

### Build prerequisite

LanceDB/transitive native tooling may require `protoc`; document it as a developer build prerequisite if the selected dependency graph still requires it. Do not abandon LanceDB merely because `protoc` is absent from one machine.

## 3. Do not add

Unless a concrete blocker in this plan requires reopening:

```text
Redis
Kafka
Neo4j/graph database
Milvus/Qdrant second production backend
Kubernetes/service mesh
runtime plugin marketplace
DI framework
universal repository/provider abstraction
internal cron/scheduler
MCP server
Persona/Diary/Dream placeholder crates
legacy compatibility readers
TDD/qualification ceremony
```

## 4. Repository bootstrap

Create:

```text
Cargo.toml workspace
rust-toolchain.toml -> stable 1.98.1 (or exact installed compatible stable when patch superseded before execution)
rustfmt/clippy config only when needed
AGENTS.md
README.md
specs/ <- this package or canonicalized equivalent
crates/core
crates/material
crates/object-store
crates/memory-domain
crates/memory-store
crates/memory-retrieval
crates/memory-service
apps/nous-wave
```

Do not create empty future MicroSystem packages.

## 5. Stage A — executable local spine

Implement the smallest real composition:

```text
config load
PostgreSQL connection
schema migrations
OpenDAL local object repository
LanceDB local database open
Axum service
CLI entry
health/readiness
```

Use real PostgreSQL and real LanceDB in integration tests for their claimed boundaries.

Proof:

- service starts with local profile;
- migrations initialize empty DB;
- object put/get by content hash works;
- LanceDB projection opens;
- process restarts cleanly.

## 6. Stage B — Subject Core and Character Seed

Implement:

- SubjectId and subject table;
- Character Seed Artifact + immutable seed revisions;
- subject create/read/list;
- subject revision;
- MicroSystem readiness projection with `subject.core` and `memory`;
- explicit subject purge orchestration hook limited to current owners.

Proof:

- create a subject from only a Markdown role card;
- zero memories is valid;
- seed replacement creates history rather than overwrite;
- disabling Memory still permits Subject Core startup/profile.

## 7. Stage C — Cognitive Material and Artifact ingestion

Implement canonical types/tables/services for:

```text
SourceRecord
Artifact
classification axes
Derivation provenance
source/artifact links
```

Implement:

- inline small text/JSON;
- OpenDAL CAS large/object payload;
- file upload/registration;
- arbitrary-length text source;
- tool observation helper;
- derived-artifact submission;
- processing status.

Initial semantic classes cover current needs but support namespaced extension strings.

Proof matrix:

- short message;
- long Markdown;
- JSON tool/MCP result;
- binary unsupported file retained;
- same `.md` classified differently;
- duplicate idempotency source does not duplicate canonical record.

## 8. Stage D — canonical Memory domain

Implement:

- MemoryObject / immutable MemoryRevision / current head;
- Episode and Episode membership;
- lightweight Memory entity/concept refs;
- relation refs required for recall;
- correction/supersession;
- suppression;
- accessibility state;
- association evidence/state schema;
- provenance/time/scope semantics.

Do not add Persona/Social/Belief tables.

Proof:

- multi-source episode;
- one source -> multiple MemoryItems;
- revision correction preserves source/history;
- approximate temporal range survives round-trip.

## 9. Stage E — processing and model-service adapters

Implement narrow clients:

```text
EmbeddingService
RerankService
StructuredGenerationService
```

All requests/results record model identity/revision/config provenance.

Implement Memory processing:

- deterministic text segmentation;
- derived section artifacts;
- optional structured generation for episode/entity/semantic memory extraction;
- schema/domain validation;
- embedding generation;
- canonical commit then projection obligation.

### Durable processing obligation

Do **not** build a generic job queue.

Implement a small domain-owned PostgreSQL `processing_obligation`/operation model with:

- stable obligation identity;
- type/payload version;
- pending/running/succeeded/failed state;
- claim using `FOR UPDATE SKIP LOCKED`;
- best-effort `LISTEN/NOTIFY` wakeup plus periodic/resume scan;
- lease/attempt identity sufficient to recover crashed workers;
- idempotent owner handlers.

This is justified processing mechanics for Memory, not a reusable workflow framework.

Proof:

- crash after source commit before processing -> source rediscovered;
- duplicate processing attempt cannot create duplicate canonical Memory/projection row;
- model unavailable leaves truthful pending/degraded status.

## 10. Stage F — retrieval projections and deterministic channels

Implement candidate sources:

```text
exact/reference -> PostgreSQL
source/artifact -> PostgreSQL
entity/concept -> PostgreSQL
lexical -> PostgreSQL FTS/trigram where appropriate
temporal -> PostgreSQL
episodic/relation -> PostgreSQL
Dense -> LanceDB
```

Implement projection obligations/rebuild.

Implement RRF-style heterogeneous fusion with deterministic precedence for exact references.

Implement bounded neural rerank only when configured/available.

Proof:

- current index can be deleted/rebuilt;
- exact reference does not require embedding;
- embedding outage does not falsely claim dense readiness;
- temporal filters are admissibility rules, not recency boosts.

## 11. Stage G — Structured Recall Intent and effort

Implement public/internal types for:

- target;
- objective;
- temporal perspective;
- cues;
- constraints;
- result need;
- effort.

Implement rule-based progressive planner.

Do not expose provider/index tuning knobs in API.

Implement `RecallEffortTrace`.

Proof:

- explicit ref route runs minimal work;
- semantic route uses appropriate dense/lexical channels;
- deep effort can broaden after insufficient initial result;
- light does not blindly execute all channels.

## 12. Stage H — heterogeneous association and competitive activation

Implement Derived Association Projection over current Memory node types.

Do not build universal graph ontology.

Implement baseline competitive activation with:

- directed edge evidence;
- log/compressed evidence mapping;
- fixed/bounded outbound mass;
- neighbor competition;
- target hub/specificity correction;
- soft immediate-backtrack penalty;
- priority/best-first frontier rather than FIFO where it matches strongest-path semantics;
- max hops / edges / states / minimum activation;
- path/support provenance;
- multi-seed support.

Keep constants typed/configured and documented as algorithm defaults, not public cognitive law.

Do not implement PPR/diffusion in this stage unless a blocking correctness issue requires the already-known alternate and the plan is explicitly amended.

Proof:

- high-degree node cannot create unbounded activation mass;
- A->B->A immediate oscillation is suppressed;
- generic hubs do not dominate private associations;
- private zero-shot association cases recover targets missed by pure semantic similarity.

## 13. Stage I — Cognitive Work Cycle

Implement:

```text
begin cycle
recall in cycle
inspect
follow/refine
close cycle
```

Process-local Working Cognitive State must actually reuse:

- candidate/frontier;
- inspected/exposed refs;
- active entities;
- association activation/visited region;
- exhausted candidates/regions.

Persist only bounded cycle metadata/use events required for learning/audit; do not persist the whole ephemeral frontier by default.

Proof:

- second recall in same cycle performs less repeated work in a representative multi-hop case;
- new intent can continue from a discovered object;
- restart may end ephemeral cycle state truthfully rather than fabricating restoration unless durable continuation is later required.

## 14. Stage J — cognitive-use feedback, accessibility, forgetting

Implement event recording for surfaced/inspected/exposed/followed/used/abandoned/resolved/insufficient.

Implement a documented simple baseline accessibility update using meaningful use + time, with strong cues able to bypass low ordinary accessibility.

Critical test:

```text
repeated SURFACED only
!= strength increase
```

Implement association strengthening only from justified events/evidence classes.

Proof forgetting without deletion.

## 15. Stage K — correction, suppression, purge, consolidation

### Correction

Create superseding revision with reason/support refs.

### Suppression

Immediate ordinary Recall exclusion; no auto-unsuppress.

### Purge

Trace and remove owned sources/memory/derivations/projections/object bytes according to shared-reference rules.

### Consolidation

Implement explicit `memory.consolidate` operation only.

Baseline capabilities:

- merge strongly redundant memory representations;
- form provenance-backed semantic abstraction from repeated Episodes;
- preserve contradictory evidence;
- maintain parent/support lineage;
- mark superseded ordinary recall representations without deleting evidence.

No wall-clock trigger.

## 16. Stage L — API and CLI completion

Implement all current endpoints/commands from `08-API.md`.

Use one semantic service path for HTTP and CLI; CLI must not duplicate business logic.

Generate/document JSON schemas/OpenAPI only if the chosen Axum ecosystem library reduces work; do not create a custom schema generator.

## 17. Stage M — portability and projection rebuild

Implement:

- coherent Subject Bundle export/import;
- content-hash verification;
- preserve/new identity import policy;
- Character Seed history;
- Memory/source/artifact export;
- serving projection rebuild after import;
- local installation backup documentation (not a backup framework).

## 18. Stage N — evaluation and documentation

Implement focused benchmark fixtures from `10-EVALUATION.md`:

- heterogeneous material;
- tool provenance;
- temporal/history;
- private associations;
- Work Cycle incremental recall;
- retrieved != reinforced;
- forgetting;
- correction/suppression/purge;
- consolidation;
- export/import.

Add performance metrics at meaningful boundaries.

Do not create a giant qualification matrix or permanent review ceremony.

## 19. Verification cadence

During implementation use the narrowest test that can falsify the current change.

At stage integration boundaries run:

```text
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
focused real PostgreSQL/LanceDB integration suite
```

Comprehensive expensive benchmarks run at algorithm/performance milestones, not after every edit.

## 20. Coding constraints

- Directly rewrite internal APIs while PRE_PRODUCTION.
- Delete obsolete implementation paths; do not bridge them.
- Prefer mature libraries for generic mechanics.
- Do not create interfaces solely for tests.
- Do not create generic providers when only one implementation is authorized.
- Preserve the real semantic seams in this package.
- Do not turn future MicroSystem names into current code scaffolding.
- Do not reduce Memory semantics to raw text chunks.
- Do not add an internal scheduler.

## 21. Completion condition

STOP when:

- all current Memory capabilities are implemented;
- required tests/benchmarks for claimed behavior pass;
- docs/config match current truth;
- no future cognition subsystem has been accidentally implemented or coupled;
- no observed current blocker remains.

Do not begin a second “stabilization/hardening” program automatically.
