# 07 — Persistence, Object Storage, Retrieval Projections, and Model Services

## 1. Storage ownership

Nous Wave distinguishes:

```text
Canonical structured Authority      -> PostgreSQL
Raw/large immutable content         -> OpenDAL-backed object repository
Local retrieval serving projection  -> LanceDB OSS embedded
Process working state/cache         -> memory
External inference mechanics        -> model services/providers
```

No serving projection becomes cognitive Authority.

## 2. PostgreSQL baseline

Use PostgreSQL 18.x; current spec-time stable patch is 18.6.

Use normal relational modeling for semantic owners rather than storing the entire system as opaque JSON blobs.

JSON/JSONB remains appropriate for bounded extensible metadata/proposal payloads when relational columns provide no semantic/query value.

## 3. Schema families

Current implementation requires table families equivalent to:

### Subject Core

```text
subjects
character_seed_revisions
microsystem_state/readiness projection where durable state is justified
```

### Cognitive Material

```text
source_records
artifacts
artifact_sources
derivations
derivation_inputs
derivation_outputs
```

### Memory

```text
memory_objects
memory_revisions
memory_revision_parents
memory_current_heads
episodes
episode_members
memory_entities
association_evidence
association_state
accessibility_state
suppression_state
```

### Work Cycle / use feedback

```text
cognitive_cycles        # only durable cycle metadata/events, not full ephemeral frontier
cognitive_use_events
```

### Operations

```text
processing_obligations / durable operations
purge/export/import operation state where needed
projection_outbox/obligations if needed for crash-safe projection update
```

Exact table decomposition may vary when it makes ownership clearer; do not create one table per noun mechanically.

## 4. Host/process mutation model

Memory service is responsible for canonical transaction boundaries.

Long model/network/file work occurs outside database transactions:

```text
read/pin source + revision
→ perform external work
→ validate current assumptions
→ commit result/provenance
```

## 5. Temporal data

Use PostgreSQL temporal/range facilities where they simplify real historical semantics.

Do not add every possible timestamp column to every table.

Event/valid ranges, observation time and recorded time are modeled per object semantics.

## 6. Revision storage

Small cognitive state revisions store complete immutable snapshots by default.

Large artifacts remain content-addressed and referenced by hash/object locator.

Do not implement application patch chains as the primary history mechanism.

## 7. Object repository

### Keying

Use cryptographic content digest over raw bytes as stable content identity. BLAKE3 is recommended for object CAS if no ecosystem constraint requires SHA-256; external protocol digests may separately use SHA-256 where interoperability requires it.

Do not duplicate identical raw bytes per source when policy allows content deduplication.

### OpenDAL

Use Apache OpenDAL to own local filesystem/S3-compatible mechanics.

The Memory domain sees an ArtifactRepository/CAS semantic boundary, not raw storage-provider objects.

## 8. Inline content policy

Small text/JSON may be stored inline in PostgreSQL when bounded size makes this simpler and faster.

Define a configuration threshold; large content goes to the object repository.

The domain API exposes content/reference uniformly enough that caller semantics do not depend on this physical choice.

## 9. LanceDB projection

LanceDB is used for local embedded retrieval projection:

- dense vectors;
- searchable derived text/metadata as useful;
- multi-vector experiments when later authorized;
- local filesystem/object-backed operation.

Projection rows reference canonical PostgreSQL identities/revisions.

LanceDB data may be deleted and rebuilt from canonical/retained derivation state.

## 10. Lexical search

Do not force lexical retrieval through a vector database if PostgreSQL FTS/trigram/exact indexes provide a simpler and strong local path.

The initial architecture may use PostgreSQL for exact/entity/lexical/temporal candidate generation and LanceDB for dense retrieval.

This is a capability split, not a “one database must do everything” goal.

## 11. Dense representations

Embedding records retain:

```text
source/object revision identity
model identity
model revision
preprocessing identity
vector dimension/representation kind
created_at
```

If embeddings are stored primarily in LanceDB, keep enough durable derivation/projection metadata in PostgreSQL to determine validity and rebuild obligations.

Whether raw vectors themselves are retained as durable model artifacts outside LanceDB is configurable based on recomputation cost/model availability; do not silently claim they are disposable.

## 12. Model-service contracts

Core talks to model providers through narrow roles:

```text
EmbeddingService
RerankService
StructuredGenerationService
```

They may be local Python/vLLM-style workers, hosted APIs or host-provided services.

Nous Wave owns:

- request semantics;
- schema validation;
- model/revision provenance;
- budgets/timeouts;
- result admissibility.

It does not own model training/inference server mechanics.

## 13. Inference is optional by operation

A raw Artifact can be accepted without all model services READY.

Operations that truly require a missing model service become pending/degraded/unavailable rather than fabricating outputs.

Recall may still use non-neural routes when semantically sufficient.

## 14. Projection update consistency

Canonical commit precedes serving projection update.

If projection update can be lost across crash, retain a durable projection obligation/outbox in PostgreSQL or an equivalently simple canonical marker.

Do not build Kafka/CDC for this requirement.

## 15. Projection rebuild

Provide deterministic rebuild commands/operations for:

- LanceDB dense/search projection;
- lexical/derived index state when separately materialized;
- association hot projection;
- cached representations.

Rebuild does not alter canonical Memory revisions.

## 16. Association serving graph

Durable association evidence/state lives in PostgreSQL.

For recall, load a bounded relevant neighborhood into compact Rust adjacency/CSR-like structures and keep active Subject neighborhoods hot when useful.

Do not introduce a graph database in the current wave.

## 17. Process Working Set

Keep high-value process-local data such as:

- current cycle frontiers;
- recently active entities;
- hot association neighborhoods;
- validated representation cache.

Canonical state remains reconstructable after restart.

## 18. Configuration

Behavior-affecting parameters are typed/configurable even if not all are exposed in a GUI.

Examples:

- local DB/object/index paths;
- inline artifact threshold;
- model endpoints/identities;
- recall budgets;
- association budgets;
- accessibility baseline constants;
- derivation retention policy.

Do not expose provider-specific ANN knobs in the normal cognitive API.

## 19. No speculative infrastructure

Current wave does not add:

- Redis;
- Kafka;
- dedicated graph DB;
- Kubernetes manifests as architecture requirement;
- HA replication controller;
- provider marketplace;
- generic object-storage abstraction beyond OpenDAL adapter semantics already justified.
