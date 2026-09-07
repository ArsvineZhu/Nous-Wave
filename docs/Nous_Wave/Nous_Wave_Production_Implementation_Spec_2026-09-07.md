# Nous Wave Production Implementation Spec

**Date:** 2026-09-07  
**Status:** READY_FOR_EXECUTION  
**Repository:** `ArsvineZhu/Nous-Wave`  
**Observed baseline:** `d1e1d2611d786cc74e357abf32900babcbacbe83`  
**Execution posture:** PRE_PRODUCTION direct rewrite  
**Implementation language:** Rust 2024  
**Purpose:** replace the historical Memory Product Closure implementation with the first coherent Subject-cognition / Memory implementation derived from the 2026-09-07 semantic architecture and completed IR decisions.

---

# 0. Execution contract

This file is the implementation-decision handoff to the Coding Agent.

The executor may choose semantics-equivalent local coding details, names inside a private module, SQL index syntax, helper function decomposition and test fixture organization. The executor **must not** reopen or invent decisions about:

- semantic ownership;
- Authority vs projection vs runtime state;
- public cognitive I/O contract shape;
- database/vector/lexical/topology strategy;
- provider/capability semantics;
- memory formation semantics;
- entity identity ownership;
- Wave algorithm family;
- persistent state not described here;
- backward compatibility with the old pre-production implementation;
- new services/frameworks/generic plugin registries;
- autonomous scheduling policy.

If a true contradiction prevents implementation, report the smallest `PLAN_GAP` with the conflicting clauses and stop that branch. Do not hide the gap behind a generic abstraction.

## 0.1 Supersession

For this implementation wave, this Spec supersedes executable direction from:

- repository `AGENTS.md` line/current state declaring `CurrentWave = MEMORY_PRODUCT_CLOSURE`;
- `docs/Nous_Wave_Memory_Product_Closure_Spec_2026-09-06.md`;
- the abandoned `Nous_Wave_Cognitive_Rebase_Spec_Package_2026-09-07`;
- the IR package requirement that a serving benchmark must decide the architecture before implementation;
- historical Memoria/Memoria Next API, schema and package compatibility assumptions.

The semantic architecture in `Nous_Wave_Cognitive_Architecture_Baseline_2026-09-07.md` remains authoritative. This Spec freezes implementation questions that the baseline deliberately left open.

## 0.2 First repository action

Before substantive implementation, update root `AGENTS.md` so the Coding Agent cannot be redirected by stale Memory Product Closure text. The new current-wave statement must point to this Spec and preserve Library-first / JDD / PRE_PRODUCTION direct-rewrite rules. Do not add a governance framework, matrix, gate system or compatibility ledger.

## 0.3 No compatibility work

This repository has not entered a production compatibility regime. Therefore:

- reset/replace old database migrations rather than migrating historical development schemas;
- remove obsolete REST routes instead of retaining aliases;
- remove LanceDB-specific compatibility code rather than dual-writing;
- delete old process-local Work Cycle semantics where they conflict with Session runtime;
- do not add `legacy`, `v0`, fallback readers, dual-format decoders or bridge tables solely for current repository history.

Preserve reusable mechanics by adapting or moving code when cheaper than rewriting; preserve **behavioral value**, not historical shape.

---

# 1. Frozen implementation architecture

## 1.1 Semantic layers

The implementation must preserve these layers as distinct even if they live in one process:

```text
External canonical world/resources
        │ Host-owned current truth
        ▼
Observation + Artifact / Evidence layer
        │ what the Subject encountered
        ▼
Cognitive Authority
        │ Memory / Tags / Anchors / Associations / Resource awareness
        ▼
Semantic Runtime
        │ Session / ResidentSet / use state
        ▼
Serving Projections
        │ exact postings / Tantivy / USearch / Wave CSR / derived summaries
        ▼
ConsumerWorkingSet / ContextContribution
        │ bounded projection for one model/agent/consumer
        ▼
Disposable model cache
```

The following remain non-negotiable:

```text
Subject != Model
Artifact != Memory
ObservationOccurrence != Artifact
Tool result != Truth
Runtime remembered != durable memory formed
Retrieval != reinforcement
Entity identity != surface name
Association != embedding similarity
Resource awareness != duplicated resource contents
Serving projection != Cognitive Authority
```

## 1.2 Physical stack

Freeze this first production stack:

| Concern | Selected implementation | Role |
|---|---|---|
| Structured durable Authority | PostgreSQL 18.x + SQLx | canonical state, revisions, provenance, runtime durability, derivation ledger |
| Raw artifacts | existing BLAKE3 CAS over Apache OpenDAL | large/raw bytes, local filesystem first |
| Dense retrieval | USearch | rebuildable vector projection only |
| Lexical retrieval | Tantivy | rebuildable fielded lexical projection |
| Exact/admissible sets | PostgreSQL indexes + `roaring` in hot paths | exact refs, entity/tag/anchor postings, filters |
| Topology serving | immutable compact CSR using petgraph | Wave graph snapshot |
| Linear algebra | nalgebra | MGS/SVD/PCA-sized operations |
| CPU parallelism | Rayon | candidate and topology scoring |
| Atomic serving snapshot | arc-swap | in-process immutable generation publication |
| Optional local inference | FastEmbed where capability fits | provider implementation, not semantic Authority |

Remove LanceDB and Arrow from the production retrieval path in this rewrite. Do not introduce Qdrant, pgvector, Neo4j, Kuzu, Milvus or another graph/vector server in parallel.

## 1.3 Why this physical split is mandatory

Nous owns a custom cognitive addressing/ranking pipeline. Vector and lexical stores supply candidates; they do not own the retrieval semantics. Keeping PostgreSQL as Authority and USearch/Tantivy/Wave as rebuildable projections means:

- provider replacement cannot rewrite cognition;
- an index may be deleted/rebuilt without losing memory;
- exact identity remains independent of embeddings;
- topology can evolve without a graph database becoming Authority;
- no one storage engine is forced to represent raw media, vector spaces, history, runtime and graph behavior simultaneously.

---

# 2. Repository/module target

Do **not** create a new framework-shaped workspace. Reuse the current crate boundaries where they still describe useful ownership; rewrite their contents.

```text
apps/nous-wave
crates/core
crates/material
crates/memory-domain
crates/memory-retrieval
crates/memory-store
crates/memory-service
crates/object-store
```

## 2.1 `crates/core`

Own shared semantic primitives only:

- `SubjectId`, `MemoryId`, `MemoryRevisionId`, `ArtifactId`, `OccurrenceId`, `TagId`, `AnchorId`, `SessionId`, `DerivationId`, `ServingGenerationId`;
- opaque `EntityRef`, `ObjectRef`, `ResourceRef`;
- typed generic `CognitiveRef` enum for public/runtime addressing;
- time interval/value objects;
- `AuthorityClass`, `EpistemicClass`, `SourceClass`;
- `CapabilityDescriptor`, `CapabilityRequirement`;
- `ProducerSignature`, `EmbeddingSpaceSignature`;
- `CognitiveQuery`, typed cues, result types, degradation types;
- `ContextContribution`;
- error/status vocabulary.

It must not contain SQL, HTTP, vector DB, provider HTTP or object-store logic.

## 2.2 `crates/material`

Own source/evidence semantics:

- Artifact metadata;
- `ObservationOccurrence`;
- `SourceRegionRef` / `DerivedRegionRef`;
- `DerivedRepresentation`;
- coverage/readiness concepts;
- derivation input/output descriptors;
- source/provenance data types.

## 2.3 `crates/memory-domain`

Own durable cognition semantics:

- Memory object/revision;
- Specific / Integrative / Procedural class;
- evidence support and contradiction/supersession links;
- Tag object/revision;
- Anchor object/revision/support;
- association evidence;
- memory formation proposal/commit types;
- cognitive use event vocabulary;
- forgetting/suppression/purge semantics.

## 2.4 `crates/memory-retrieval`

Own all serving and retrieval computation:

- USearch projection adapter;
- Tantivy projection adapter;
- exact/posting candidate adapters;
- immutable `WaveGraphGeneration`;
- EPA basis and query observation;
- Residual Pyramid cue sensing;
- clean-room bounded Wave propagation;
- request-local Query River;
- LocalField / TransferField;
- candidate superset/consolidation;
- CandidateSemanticTrail;
- relative topology scoring;
- `WaveObservability`;
- optional rerank evidence integration;
- final ranking trace.

No cognitive Authority mutations happen here.

## 2.5 `crates/memory-store`

Own SQLx/PostgreSQL repositories and migrations:

- canonical tables below;
- transaction boundaries;
- revision/current-head commits;
- derivation attempt leases;
- serving-generation metadata;
- Session residency persistence;
- projection rebuild reads.

Do not leak SQL rows as public domain types.

## 2.6 `crates/memory-service`

Own application orchestration:

- observe artifact/event;
- immediate runtime admission;
- explicit memory formation/consolidation;
- capability/provider selection;
- derivation/backfill executor;
- query planning and service-level orchestration;
- resource resolution callbacks/contracts;
- materialization;
- use feedback;
- suppression/purge workflows;
- serving generation build/publish coordination.

The service coordinates semantic owners; it must not hide new domain meaning in arbitrary JSON.

## 2.7 `crates/object-store`

Keep/adapt the existing CAS mechanics:

- streaming BLAKE3 while writing;
- maximum byte bound enforced during stream;
- staging path;
- final atomic rename/publish;
- hash verification on read/stream completion;
- reference/purge race guard;
- unreferenced deletion.

Do not replace this with database BLOBs or a custom object-store abstraction.

## 2.8 `apps/nous-wave`

Own only:

- config loading;
- PostgreSQL/bootstrap startup;
- provider adapter construction;
- serving-generation loading;
- HTTP routes;
- CLI commands;
- process lifetime and graceful shutdown;
- event-driven derivation worker startup.

No domain algorithm should live in `main.rs` or `http.rs`.

---

# 3. Core identifiers and public reference model

Use UUIDv7 for Nous-owned logical/revision IDs. Do not encode semantic meaning in UUIDs.

Host-owned refs remain opaque strings with an explicit namespace where needed.

```rust
pub enum CognitiveRef {
    Memory(MemoryId),
    MemoryRevision(MemoryRevisionId),
    Artifact(ArtifactId),
    SourceRegion(SourceRegionId),
    DerivedRepresentation(DerivedRepresentationId),
    DerivedRegion(DerivedRegionId),
    Entity(EntityRef),
    Tag(TagId),
    Anchor(AnchorId),
    Resource(ResourceRef),
    ExternalObject(ObjectRef),
}
```

`EntityRef`, `ResourceRef`, and `ObjectRef` must be validated opaque values, not UUID aliases and not names.

Example canonical string forms may be:

```text
entity:host-person:01J...
resource:schedule:primary
object:messaging:message:...
```

The contents are opaque to retrieval except for exact equality and the declared namespace/type.

---

# 4. PostgreSQL Authority schema

This is the required logical schema. The Coding Agent may choose SQL enum vs constrained text where it does not change semantics. Prefer simple constrained text and ordinary indexes over schema metaprogramming.

## 4.1 Subjects

`subjects`

```text
subject_id uuid PK
created_at timestamptz NOT NULL
state_revision bigint NOT NULL DEFAULT 0
status text NOT NULL  -- active | suspended | purging
metadata jsonb NOT NULL DEFAULT '{}'
```

If Character Seed support remains in the app, store the seed as an Artifact/Occurrence with semantic source role `character_seed`; do not copy it into every memory row.

## 4.2 Artifacts and occurrences

`artifacts`

```text
artifact_id uuid PK
subject_id uuid NOT NULL FK subjects
content_hash text NOT NULL               -- BLAKE3 hex
byte_length bigint NOT NULL
media_type text NOT NULL
storage_key text NOT NULL                 -- CAS key; normally derived from hash
created_at timestamptz NOT NULL
metadata jsonb NOT NULL DEFAULT '{}'
UNIQUE(subject_id, content_hash)
```

Content deduplication is allowed here.

`observation_occurrences`

```text
occurrence_id uuid PK
subject_id uuid NOT NULL
artifact_id uuid NULL                    -- may be NULL for ref-only external events
source_class text NOT NULL               -- message | tool | file | web | host_event | simulation | generated | import | ...
external_object_ref text NULL
occurred_at timestamptz NULL             -- when event happened in source/world if known
observed_at timestamptz NOT NULL          -- when Subject encountered it
conversation_ref text NULL
actor_entity_ref text NULL               -- already resolved by Host if known
context jsonb NOT NULL DEFAULT '{}'
created_at timestamptz NOT NULL
```

The same `artifact_id` may appear in arbitrarily many occurrences. Never collapse occurrences because raw bytes are equal.

## 4.3 Stable source coordinates

`source_regions`

```text
source_region_id uuid PK
subject_id uuid NOT NULL
artifact_id uuid NOT NULL
coordinate_kind text NOT NULL
coordinate jsonb NOT NULL
coordinate_hash text NOT NULL
parent_source_region_id uuid NULL
created_at timestamptz NOT NULL
UNIQUE(subject_id, artifact_id, coordinate_kind, coordinate_hash)
```

Allowed first-wave coordinate kinds:

```text
whole_artifact
byte_range        {start,end}
text_span         {encoding,start,end}
pdf_region        {page,bbox?}
image_bbox        {x,y,width,height,unit}
media_time        {stream,pts_start,pts_end,time_base}
json_pointer      {pointer}
structural_path   {format,path,text_start?,text_end?}
```

Do not use parser chunk ordinal as a source coordinate.

## 4.4 Producer identity and derived representations

`producer_signatures`

```text
producer_signature_id uuid PK
signature_hash text UNIQUE NOT NULL
provider_class text NOT NULL              -- local | http | process | deterministic-library
operation text NOT NULL                   -- text.embedding, image.interpretation, document.extract, ...
implementation text NOT NULL              -- adapter/library/model family
model_identity text NULL
model_revision text NULL
preprocessing_identity text NOT NULL
preprocessing_revision text NOT NULL
config_digest text NOT NULL
created_at timestamptz NOT NULL
metadata jsonb NOT NULL DEFAULT '{}'
```

`embedding_spaces`

```text
embedding_space_id uuid PK
space_hash text UNIQUE NOT NULL
model_identity text NOT NULL
weights_revision text NOT NULL
task text NOT NULL
input_representation text NOT NULL
preprocessing_identity text NOT NULL
preprocessing_revision text NOT NULL
dimension integer NOT NULL
normalization text NOT NULL                -- l2 | none | ...
output_semantics text NOT NULL             -- dense_similarity | sparse | late_interaction | ...
created_at timestamptz NOT NULL
```

Provider host/name is **not** automatically part of space compatibility. Two producers may declare the same space only when configuration explicitly proves identical weights/task/preprocessing/output semantics. Equal dimension is never sufficient.

`derived_representations`

```text
derived_representation_id uuid PK
subject_id uuid NOT NULL
source_region_id uuid NOT NULL
representation_kind text NOT NULL          -- extracted_text | ocr | transcript | image_description | scene_description | summary | ...
producer_signature_id uuid NOT NULL
revision integer NOT NULL
payload_text text NULL
payload_artifact_id uuid NULL              -- for large/binary derived output
quality jsonb NOT NULL DEFAULT '{}'
created_at timestamptz NOT NULL
supersedes uuid NULL
UNIQUE(subject_id, source_region_id, representation_kind, producer_signature_id, revision)
CHECK(payload_text IS NOT NULL OR payload_artifact_id IS NOT NULL)
```

Never overwrite a prior model interpretation. A new provider/model creates a distinct derived representation revision/identity.

`derived_regions`

```text
derived_region_id uuid PK
subject_id uuid NOT NULL
derived_representation_id uuid NOT NULL
coordinate_kind text NOT NULL             -- text_span | segment | bbox-in-derived | structural_path
coordinate jsonb NOT NULL
coordinate_hash text NOT NULL
parent_derived_region_id uuid NULL
created_at timestamptz NOT NULL
UNIQUE(derived_representation_id, coordinate_kind, coordinate_hash)
```

Derived coordinates are stable only relative to the immutable `derived_representation_id`.

## 4.5 Coverage / derivation work

`coverage_needs`

```text
coverage_need_id uuid PK
subject_id uuid NOT NULL
source_region_id uuid NOT NULL
representation_kind text NOT NULL
capability_operation text NOT NULL
requirement text NOT NULL                 -- required | preferred | opportunistic
state text NOT NULL                       -- missing | scheduled | ready | unavailable | failed
current_representation_id uuid NULL
updated_at timestamptz NOT NULL
UNIQUE(subject_id, source_region_id, representation_kind, capability_operation)
```

This replaces any aggregate `indexed=true` concept.

`derivations`

```text
derivation_id uuid PK
derivation_key text UNIQUE NOT NULL        -- deterministic hash
subject_id uuid NOT NULL
source_region_id uuid NOT NULL
representation_kind text NOT NULL
producer_signature_id uuid NOT NULL
state text NOT NULL                       -- pending | running | succeeded | failed | unavailable
successful_representation_id uuid NULL
created_at timestamptz NOT NULL
updated_at timestamptz NOT NULL
```

The deterministic key is a domain-separated hash of:

```text
source region immutable identity
+ source artifact hash / source representation revision
+ representation kind
+ ProducerSignature hash
+ derivation contract version
```

`derivation_attempts`

```text
attempt_id uuid PK
derivation_id uuid NOT NULL
attempt_no integer NOT NULL
state text NOT NULL                       -- running | succeeded | transient_failed | permanent_failed
lease_owner text NULL
lease_until timestamptz NULL
started_at timestamptz NOT NULL
finished_at timestamptz NULL
problem_code text NULL
problem_detail jsonb NOT NULL DEFAULT '{}'
UNIQUE(derivation_id, attempt_no)
```

Retry gets a new `attempt_id` but the same `derivation_id`. Provider replacement produces a new `derivation_id`.

## 4.6 Memory objects and revisions

`memory_objects`

```text
memory_id uuid PK
subject_id uuid NOT NULL
memory_class text NOT NULL                -- specific | integrative | procedural
current_revision_id uuid NOT NULL         -- deferrable FK to revision
created_at timestamptz NOT NULL
status text NOT NULL                      -- active | suppressed | purging
```

`memory_revisions`

```text
memory_revision_id uuid PK
memory_id uuid NOT NULL
subject_id uuid NOT NULL
revision_no integer NOT NULL
parent_revision_id uuid NULL
semantic_role text NOT NULL               -- episode | fact | preference | schema | procedure | failure_pattern | ...
title text NULL
representation_text text NOT NULL
attributes jsonb NOT NULL DEFAULT '{}'
epistemic_class text NOT NULL
confidence double precision NULL
occurred_at timestamptz NULL
observed_at timestamptz NOT NULL
valid_from timestamptz NULL
valid_to timestamptz NULL
created_at timestamptz NOT NULL
supersession_state text NOT NULL           -- current | superseded | contradicted | revoked
UNIQUE(memory_id, revision_no)
```

`representation_text` is the concise cognitive representation used by text retrieval/context projection. It is not the original source; evidence links preserve the source.

For historical/temporal cognition, preserve both source/world validity and Subject observation time. A new correction does not delete the old revision.

`memory_revision_evidence`

```text
memory_revision_id uuid NOT NULL
evidence_no integer NOT NULL
occurrence_id uuid NULL
source_region_id uuid NULL
derived_representation_id uuid NULL
derived_region_id uuid NULL
support_role text NOT NULL                 -- direct | corroborating | interpretation | contradiction | contextual
weight double precision NULL
PRIMARY KEY(memory_revision_id, evidence_no)
CHECK(exactly one evidence target family is populated as required by support_role)
```

A model-generated image description can support a memory only as `interpretation`; the raw source region remains independently addressable.

Revision commit invariant: inserting a new Memory revision, changing `memory_objects.current_revision_id`, and writing its immediate evidence/relation rows must occur in one transaction. `current_revision_id` must always point to a revision of the same Memory/Subject. There is no observable half-committed head revision.

`memory_revision_relations`

```text
from_revision_id uuid NOT NULL
to_revision_id uuid NOT NULL
relation text NOT NULL                    -- supersedes | contradicts | integrates | derived_from | proceduralizes
created_at timestamptz NOT NULL
PRIMARY KEY(from_revision_id,to_revision_id,relation)
```

## 4.7 Entity mentions and binding correction

`entity_mentions`

```text
mention_id uuid PK
subject_id uuid NOT NULL
occurrence_id uuid NULL
source_region_id uuid NULL
derived_region_id uuid NULL
surface text NOT NULL
semantic_role text NULL
created_at timestamptz NOT NULL
```

`entity_binding_revisions`

```text
binding_revision_id uuid PK
mention_id uuid NOT NULL
revision_no integer NOT NULL
entity_ref text NULL                      -- NULL means explicitly unbound/unknown
binding_state text NOT NULL               -- bound | unbound | disputed
host_resolution_ref text NULL
reason text NULL
created_at timestamptz NOT NULL
UNIQUE(mention_id, revision_no)
```

Current exact entity postings are derived from the latest binding revision. A mistaken merge is corrected by rebinding the affected mentions individually. Never rewrite raw surfaces and never use Union-Find as irreversible Authority.

## 4.8 Tags

`tags`

```text
tag_id uuid PK
subject_id uuid NOT NULL
current_revision_id uuid NOT NULL
created_at timestamptz NOT NULL
status text NOT NULL                      -- active | superseded | revoked
```

`tag_revisions`

```text
tag_revision_id uuid PK
tag_id uuid NOT NULL
revision_no integer NOT NULL
label text NOT NULL
description text NULL
kind_hint text NULL                       -- optional, not a universal taxonomy
origin text NOT NULL                      -- explicit | model_proposed | consolidation | import
producer_signature_id uuid NULL
created_at timestamptz NOT NULL
UNIQUE(tag_id, revision_no)
```

`memory_revision_tags`

```text
memory_revision_id uuid NOT NULL
tag_id uuid NOT NULL
role text NOT NULL                        -- explicit | inferred | structural | procedural | situational | ...
ordinal integer NULL                      -- only when real source/narrative order is known
order_provenance jsonb NULL               -- source offset / temporal / explicit structural evidence
provenance jsonb NOT NULL
PRIMARY KEY(memory_revision_id,tag_id,role)
```

Tag identity is not a label string. Tag label changes create revisions. Semantic similarity may propose a possible existing Tag but must not silently merge Tag identities.

## 4.9 Anchors

`anchors`

```text
anchor_id uuid PK
subject_id uuid NOT NULL
current_revision_id uuid NOT NULL
created_at timestamptz NOT NULL
status text NOT NULL                      -- active | superseded | revoked
```

`anchor_revisions`

```text
anchor_revision_id uuid PK
anchor_id uuid NOT NULL
revision_no integer NOT NULL
label text NULL
description text NOT NULL
origin text NOT NULL                      -- explicit | consolidation
producer_signature_id uuid NULL
created_at timestamptz NOT NULL
UNIQUE(anchor_id, revision_no)
```

`anchor_support`

```text
anchor_revision_id uuid NOT NULL
support_ref_kind text NOT NULL            -- memory_revision | tag | entity | resource | anchor | source_region
support_ref text NOT NULL
support_role text NOT NULL
provenance jsonb NOT NULL
PRIMARY KEY(anchor_revision_id,support_ref_kind,support_ref,support_role)
```

Anchor creation rules are defined in §10. Anchor identity is never an average vector.

## 4.10 Association evidence

`association_evidence`

```text
association_evidence_id uuid PK
subject_id uuid NOT NULL
from_ref_kind text NOT NULL
from_ref text NOT NULL
to_ref_kind text NOT NULL
to_ref text NOT NULL
association_kind text NOT NULL            -- experiential | causal | temporal | relational | coactivation | procedural | structural
polarity text NOT NULL                    -- positive | negative
support_class text NOT NULL               -- host_explicit | memory_evidence | consolidation | meaningful_use | derived_structure
support_value double precision NOT NULL
occurrence_id uuid NULL
memory_revision_id uuid NULL
producer_signature_id uuid NULL
valid_from timestamptz NULL
valid_to timestamptz NULL
created_at timestamptz NOT NULL
revoked_at timestamptz NULL
```

Embedding similarity alone must never insert a permanent `association_evidence` row. It may only propose candidates for later explicit/model-supported association formation.

## 4.11 Resource awareness

`resources`

```text
subject_id uuid NOT NULL
resource_ref text NOT NULL
display_label text NULL
authority_class text NOT NULL             -- authoritative_current | reference | evidence_source
coverage jsonb NOT NULL
query_dimensions jsonb NOT NULL
modalities jsonb NOT NULL
freshness_policy jsonb NOT NULL
access_cost_class text NOT NULL
resolver_key text NOT NULL
readiness text NOT NULL                    -- ready | degraded | unavailable
updated_at timestamptz NOT NULL
PRIMARY KEY(subject_id, resource_ref)
```

Resource contents do not live in this table. `resolver_key` selects a host-provided resolver/callback at runtime.

## 4.12 Semantic runtime

`cognitive_sessions`

```text
session_id uuid PK
subject_id uuid NOT NULL
opened_at timestamptz NOT NULL
last_activity_at timestamptz NOT NULL
last_meaningful_use_at timestamptz NULL
closed_at timestamptz NULL
state_revision bigint NOT NULL DEFAULT 0
metadata jsonb NOT NULL DEFAULT '{}'
```

`resident_refs`

```text
session_id uuid NOT NULL
ref_kind text NOT NULL
ref_value text NOT NULL
entered_at timestamptz NOT NULL
entry_reason text NOT NULL                 -- observed | recalled | explicit_hold | resource_awareness
last_meaningful_use_at timestamptz NULL
hold_until timestamptz NULL
state text NOT NULL                        -- resident | provisional | evicted
metadata jsonb NOT NULL DEFAULT '{}'
PRIMARY KEY(session_id,ref_kind,ref_value)
```

Do not persist one authoritative floating-point activation score. Ranking/accessibility may derive numeric values from the event/state facts.

`cognitive_use_events`

```text
use_event_id uuid PK
subject_id uuid NOT NULL
session_id uuid NULL
ref_kind text NOT NULL
ref_value text NOT NULL
use_kind text NOT NULL                    -- surfaced | inspected | selected | referenced | acted_on | corroborated | corrected | pinned | rejected
consumer_ref text NULL
occurred_at timestamptz NOT NULL
context jsonb NOT NULL DEFAULT '{}'
```

Only `referenced`, `acted_on`, `corroborated`, explicit `pinned`, and an explicit correction/corroboration workflow may strengthen accessibility/association evidence. `surfaced`, `inspected` and mere context projection do not.

## 4.13 Serving generations

`serving_generations`

```text
generation_id uuid PK
subject_id uuid NOT NULL                 -- first wave serving generations are Subject-scoped
kind text NOT NULL                       -- lexical | dense | wave | epa_basis | postings
space_signature text NULL
input_generation bigint NOT NULL
artifact_location text NOT NULL
artifact_hash text NOT NULL
state text NOT NULL                      -- building | ready | retired | failed
built_at timestamptz NOT NULL
published_at timestamptz NULL
metadata jsonb NOT NULL DEFAULT '{}'
```

A separate small `serving_current` pointer table maps `(subject_id,kind,space_signature)` to one READY generation. Publication must be atomic. In process, use `ArcSwap<ServingSnapshot>` so an entire query holds immutable generation references until completion.

---

# 5. Artifact, document and multimodal derivation

## 5.1 Ingest invariant

The observation path is:

```text
bytes/ref/event
→ Artifact if bytes exist
→ ObservationOccurrence
→ immediate Session residency
→ capability/coverage accounting
→ optional derivation
→ optional durable memory formation later
```

Never:

```text
message/file
→ automatically one Memory row
```

## 5.2 Artifact upload

Reuse the current streaming CAS implementation. The service must commit in this order:

1. hold shared reference guard;
2. stream bytes to staging while hashing and enforcing byte bound;
3. atomically publish CAS object;
4. transactionally insert/find `artifacts` row;
5. insert `observation_occurrences` if this upload is being observed now;
6. release reference guard.

A failed DB commit may leave an unreferenced CAS object; bounded cleanup may remove it later. Never let purge race a just-written object before reference commit.

## 5.3 Document extraction

Library-first adapters:

- plain UTF-8/text/Markdown/JSON/code: Rust-native deterministic extraction;
- PDF/Office/complex document: optional Docling adapter as the preferred rich extractor;
- broad fallback extraction: optional Apache Tika adapter;
- do not reimplement PDF, OOXML or binary document codecs in Rust.

Docling/Tika are optional capabilities. Absence must not make base ingest fail.

For external tools/sidecars, define a narrow `DocumentExtractionProvider` operation that returns:

```text
representation kind
source-native region mappings where available
extracted text/structure
derived-region coordinates
producer signature
warnings/coverage
```

## 5.4 Image/audio/video

Raw media is always storable/addressable without a model.

When capabilities exist:

```text
image
  → metadata
  → OCR
  → global textual description
  → optional region descriptions
  → optional image embedding

audio
  → metadata
  → transcript
  → segment/speaker descriptions if available
  → optional audio embedding

video
  → metadata
  → audio/transcript path
  → scene/keyframe descriptions
  → temporal summaries
  → optional image/video embeddings
```

Use FFmpeg/ffprobe for container/stream/media mechanics rather than implementing codecs. Invoke external binaries directly without a shell, with bounded input/output, timeout and structured error mapping.

## 5.5 Persistent textual surrogate

If a multimodal interpretation succeeds, persist its text as a `DerivedRepresentation`. Later loss of image/audio/video capability must leave:

- raw media available;
- previous textual surrogate lexically retrievable;
- previous text embeddings usable only in their compatible embedding space;
- provenance showing the surrogate was a model interpretation.

A new interpreter creates another representation. Do not overwrite the prior one.

---

# 6. Capability/provider model

## 6.1 No universal provider registry

Implement a small startup-composed capability set, not a marketplace/plugin system.

```rust
struct CapabilityDescriptor {
    operation: CapabilityOperation,
    input_modalities: SmallVec<Modality>,
    output_kind: RepresentationKind,
    producer: ProducerSignature,
    limits: CapabilityLimits,
    readiness: CapabilityReadiness,
}
```

First-wave operations:

```text
text.embedding
text.rerank
text.interpretation
memory.formation.text
memory.consolidation.text
image.interpretation
image.embedding
speech.transcription
document.extraction
```

Only implement adapters actually configured/needed. The semantic enum may include these operations without creating one trait/service per possible future modality.

## 6.2 Requirement semantics

Every plan/operator request uses:

```text
required
preferred
optional
forbidden
```

Behavior:

- `required` unavailable → fail that requested operation truthfully;
- `preferred` unavailable → continue with explicit degradation;
- `optional` unavailable → omit channel and report when diagnostics requested;
- `forbidden` → planner must not invoke even if configured.

## 6.3 ProducerSignature vs EmbeddingSpaceSignature

Keep them separate in types and storage.

`ProducerSignature` answers reproducibility/provenance.  
`EmbeddingSpaceSignature` answers vector comparability.

A vector record stores both.

Never compare vectors unless `EmbeddingSpaceSignature` matches exactly.

## 6.4 Zero-model startup

Startup succeeds with no model providers. The system must still support:

- Artifact/Occurrence storage;
- exact refs;
- entity postings;
- lexical retrieval over existing text/derived text;
- explicit Tags/Anchors/Associations;
- deterministic Wave traversal over existing topology;
- Session residency;
- ResourceRef lookup;
- materialization/evidence inspection.

Automatic LLM memory formation, new model-generated Tags/Anchors and missing embeddings are unavailable, not emulated.

---
# 7. Observation, runtime admission and memory formation

## 7.1 Observation API semantics

`observe()` records what the Subject encountered. It does not assert that the input is true and does not require durable Memory formation.

Required input shape conceptually:

```rust
struct ObservationInput {
    subject: SubjectId,
    session: Option<SessionId>,
    occurrence: OccurrenceDescriptor,
    material: ObservationMaterial,
    entities: Vec<ResolvedEntityMention>,
    formation: FormationDirective,
    runtime: RuntimeDirective,
}
```

`ObservationMaterial` may be:

```text
InlineText
ArtifactRef
ExternalObjectRef
StructuredJson
ResourceAvailability
```

`FormationDirective`:

```text
none
consider_specific
explicit_specific
```

Meaning:

- `none`: store/observe/runtime only; no automatic durable Memory;
- `consider_specific`: if a configured memory-formation capability exists, create proposal(s); commit only after service validation below;
- `explicit_specific`: caller explicitly asks Nous to form a Specific Memory from supplied evidence/representation; no LLM required if the caller supplies representation and semantics.

The default API behavior is `none`. A Host profile may deliberately choose `consider_specific` for selected interaction classes, but that is Host policy, not hidden Nous behavior.

## 7.2 Immediate runtime admission

If a valid Session is supplied:

1. occurrence reference becomes resident immediately;
2. explicit entity/resource refs from the observation may become resident;
3. if a durable memory is formed synchronously, the new MemoryRef becomes resident;
4. no vector or LLM call is required before the next turn can access the observed event.

This is how Nous prevents the anti-pattern:

```text
new message → wait for durable memory/vector index → search it on next turn
```

## 7.3 Provider-assisted Specific Memory formation

The memory-formation provider receives bounded evidence and returns `MemoryFormationProposal[]`, not rows to insert directly.

Proposal:

```rust
struct MemoryFormationProposal {
    memory_class: Specific,               // first-wave provider formation only
    semantic_role: String,
    representation_text: String,
    title: Option<String>,
    evidence: Vec<EvidenceRef>,
    entity_refs: Vec<EntityRef>,           // must come from supplied resolved refs
    tag_proposals: Vec<TagProposal>,
    occurred_at: Option<DateTime>,
    valid_from: Option<DateTime>,
    valid_to: Option<DateTime>,
    epistemic_class: EpistemicClass,
    confidence: Option<f64>,
}
```

Service validation before commit:

- every evidence ref exists and belongs to the Subject;
- every `EntityRef` was supplied by Host/resolved context; provider cannot invent authoritative identity;
- proposal does not claim external current-state Authority;
- representation length and proposal count are bounded;
- `valid_to >= valid_from` when both exist;
- evidence class is compatible with claimed epistemic class;
- duplicate proposals within the same request are collapsed by normalized representation/evidence identity, not by semantic vector alone.

Valid proposals become cognitive Memory: they record what the Subject retained/understood, not world truth.

## 7.4 Explicit memory formation without LLM

Expose a typed operation allowing Host/user/other approved MicroSystem to commit a memory from explicit evidence:

```text
form_memory(
  class,
  semantic_role,
  representation,
  evidence_refs,
  entity_refs,
  temporal/epistemic metadata,
  optional explicit tags
)
```

This is required for zero-model operation and for high-confidence host-originated cognition.

## 7.5 Integrative and Procedural consolidation

Consolidation is **explicitly invoked**, never scheduled autonomously by Nous.

Input:

```rust
struct ConsolidationRequest {
    subject: SubjectId,
    source_memories: Vec<MemoryRevisionId>,
    target: ConsolidationTarget,       // Integrative | Procedural | TopologyOnly
    capability: CapabilityRequirement,
}
```

Rules:

- source revisions remain immutable;
- consolidation output must cite all supporting revisions/evidence;
- an Integrative memory captures a gist/schema/pattern that is not identical to any one source trace;
- a Procedural memory captures reusable action/failure/applicability knowledge;
- contradictions between sources are preserved in output support metadata rather than erased;
- source memories are not deleted or rewritten merely because an integrative memory exists;
- consolidation may propose Tags, Anchors and Associations, but those go through their own validators.

This transfers LangMem's hot/background distinction without implementing an internal scheduler, and A-MEM's evolving organization without in-place historical rewrite.

---

# 8. Tags, Anchors and topology formation

## 8.1 Tag creation and matching

Tag creation pathways:

1. explicit caller Tag;
2. provider-proposed Tag during memory formation/consolidation;
3. imported Tag with provenance.

When a proposal may correspond to an existing Tag:

1. exact normalized label/known alias match may reuse a Tag;
2. semantic search may produce *candidate existing Tags* only;
3. if a semantic decision provider is configured, it may return `same | distinct | uncertain`;
4. `same` reuses existing Tag, `distinct` creates a new Tag, `uncertain` creates a new Tag unless caller explicitly resolves it.

Never merge Tag identity solely because cosine similarity exceeds a threshold.

Tag embeddings are serving representations tied to an EmbeddingSpaceSignature and can be rebuilt independently.

## 8.2 Anchor creation

Anchor is a durable cognitive landmark, not a VCP “Direct Anchor” score.

Creation pathways:

### Explicit Anchor

A trusted caller may create an Anchor with one or more support refs. One support is sufficient because intent is explicit.

### Consolidation-derived Anchor

A provider may propose:

```text
label/description
support refs
why the supports converge
suggested associated Tags/Entities/Resources/Memories
```

Service accepts the proposal only when:

- at least two independent support roots exist, where independence means distinct ObservationOccurrence roots or distinct durable Memory lineages; OR
- the proposal is explicitly confirmed by the caller.

This is an implementation quality rule, not a claim that “two observations makes truth”. It only prevents one uncorroborated model output from silently minting a topology landmark.

## 8.3 Anchor evolution/revocation

- refinement of the same landmark → new `anchor_revision`;
- materially different landmark → new Anchor ID;
- false/obsolete landmark → current revision/status becomes revoked/superseded;
- old revisions and supporting evidence remain inspectable;
- serving Wave generation includes only current active Anchor revisions unless historical query explicitly asks otherwise.

## 8.4 Association formation

Permanent associations require evidence beyond embedding proximity.

Allowed sources:

- explicit Host/caller relation;
- source structure/temporal relation;
- memory consolidation proposal supported by evidence;
- repeated **meaningful-use** coactivation;
- deterministic relation derived from known cognitive structure.

For meaningful-use coactivation:

- only `referenced`, `acted_on`, `corroborated` events may contribute;
- multiple `surfaced`/`inspected` events contribute zero permanent support;
- aggregate repeated evidence with diminishing returns (`log1p` family) in the serving projection rather than linearly increasing Authority strength.

Negative/revoked association evidence remains recorded and can reduce/disable serving conductance.

---

# 9. Semantic runtime

## 9.1 Session != Work Cycle

Replace old process-bound “Cognitive Work Cycle = working memory” semantics.

```text
CognitiveSession
    owns long-lived semantic continuity

ResidentSet
    persistent/recoverable refs available to that Session

ConsumerWorkingSet
    ephemeral bounded projection for one consumer/invocation

CognitiveWorkCycle
    optional request-local/call-chain exploration state only
```

A process restart must not semantically end an open Session.

## 9.2 Resident admission

Refs enter ResidentSet via:

- new observation (`observed`);
- selected cold recall (`recalled`, initially provisional);
- explicit hold/pin (`explicit_hold`);
- useful ResourceRef awareness (`resource_awareness`).

## 9.3 Refresh and reinforcement

Resident refresh and durable learning are separate.

- query result generated → no refresh by itself;
- result merely inserted into model context → no durable strengthening;
- model/consumer explicitly references it downstream → update `last_meaningful_use_at`;
- acted-on/corroborated/pinned → may create adaptive evidence;
- Session keepalive/network traffic → no cognitive refresh.

## 9.4 Eviction

Eviction is a runtime capacity action. It never deletes Memory.

Implement a simple bounded policy, not a configurable policy engine:

1. held refs are protected until `hold_until`;
2. current observation refs receive short-term priority;
3. meaningful-use recency outranks mere entry recency;
4. provisional recalled refs are evicted before equivalently recent explicitly held/meaningfully used refs;
5. enforce configured resident-count and approximate materialized-text budget lazily at operation boundaries.

Do not persist a continuously updated activation float. A private derived score may be computed from the facts above when ordering eviction/working-set selection.

## 9.5 ConsumerWorkingSet

`ConsumerWorkingSet` is in-memory/request-local:

```rust
struct ConsumerWorkingSet {
    consumer: ConsumerRef,
    refs: Vec<WorkingRef>,
    contributions: Vec<ContextContribution>,
    omitted: Vec<OmissionReason>,
}
```

Two consumers may receive different working sets from the same ResidentSet.

## 9.6 ContextContribution

Nous does not own final Host prompt composition. Return Host-neutral contributions:

```rust
struct ContextContribution {
    reference: CognitiveRef,
    semantic_role: String,
    text: Option<String>,
    authority: AuthorityClass,
    freshness: FreshnessDescriptor,
    evidence: Vec<EvidenceHandle>,
    provenance: ProvenanceSummary,
    priority: ContributionPriority,
    estimated_tokens: Option<u32>,
    materialization: Option<MaterializationHandle>,
}
```

This transfers the useful part of Letta-style bounded persistent context without treating context blocks as cognition Authority.

---

# 10. Serving projections and generation publication

## 10.1 Rebuildable projections

First wave projections:

```text
ExactPostingsGeneration
LexicalGeneration       -- Tantivy
DenseGeneration         -- one per EmbeddingSpaceSignature in use
WaveGraphGeneration
EpaBasisGeneration      -- per compatible Tag embedding space
```

Projection files/indexes are replaceable and deletable. PostgreSQL Authority must be sufficient to reconstruct them, except model-derived vector payloads that require the original model/provider; those are represented by coverage/derivation state and may be regenerated only if capability is available.

## 10.2 Semantic invalidation categories

Adapt the useful Memoria Next mechanism, but define categories for Nous semantics:

```text
VisibleText
TemporalValidity
EvidenceSupport
EntityBinding
EntitySurface
TagMembership
TagRevision
AnchorRevision
AssociationEvidence
MemoryLifecycle
DerivedRepresentation
EmbeddingSpace
ResourceDescriptor
```

Map categories to projection rebuild/update roots. Examples:

```text
EntitySurface
  → lexical representation if current memory text includes the presentation
  → NOT entity identity posting unless binding changed

EntityBinding
  → exact entity postings
  → affected CandidateSemanticTrail / Wave nodes if entity nodes participate
  → contextual dense projection only if that representation actually embeds entity surface

EmbeddingSpace replacement
  → DenseGeneration + EpaBasisGeneration for that space
  → NOT raw evidence / Memory revisions / explicit topology

AssociationEvidence
  → WaveGraphGeneration only
```

Do not build a permanent governance matrix. Implement this as code-level dependency mapping with tests.

## 10.3 Generation build/publish

For each projection:

```text
read Authority snapshot/input generation
→ build in staging location
→ compute artifact hash/metadata
→ structural integrity check
→ commit serving_generations READY row
→ atomically swap current pointer
→ ArcSwap publish new ServingSnapshot
→ old generation becomes RETIRED after no in-process readers
→ later bounded GC
```

A query snapshots all needed generation `Arc`s at start. It must not observe half-old/half-new Wave or index state due to a concurrent rebuild.

## 10.4 Dense projection

Use USearch with one logical index per EmbeddingSpaceSignature + representation family where necessary.

Vector key maps to a compact internal `ServingDocId` (`u64`) with PostgreSQL/Tantivy metadata mapping.

Do not encode UUID directly into floating payloads.

USearch is a candidate generator. Store alongside each vector mapping:

```text
ServingDocId
CognitiveRef / revision
EmbeddingSpaceSignature
ProducerSignature
representation kind
source/derived region if relevant
```

When hard admissible filters exist, produce a RoaringBitmap/compact set and use filtered ANN if the selected USearch Rust API supports traversal-time filtering; otherwise oversample then filter with an explicit degraded/diagnostic flag. Do not silently return out-of-scope candidates.

## 10.5 Lexical projection

Use Tantivy fields sufficient for:

```text
subject/internal scope key
serving doc id
ref kind / ref id
representation text
title
entity refs (keyword field)
tag ids (keyword field)
anchor ids (keyword field)
source class
timestamps
```

Use fielded query construction; do not concatenate all metadata into one string.

Tokenizer configuration should support multilingual content. Use mature Tantivy tokenizers/analyzers or an established compatible tokenizer library if Chinese segmentation quality requires it; do not implement a tokenizer from scratch.

---

# 11. Cognitive Query protocol

## 11.1 Public AST

Implement `api_version = 1` typed/serializable query.

```rust
struct CognitiveQuery {
    api_version: u32,
    subject: SubjectId,
    session: Option<SessionId>,
    situation: SituationDescriptor,
    targets: Vec<QueryTarget>,
    cues: Vec<Cue>,
    constraints: QueryConstraints,
    exploration: ExplorationIntent,
    resources: ResourceIntent,
    result_need: ResultNeed,
    effort: CognitiveEffort,
    capabilities: CapabilityPolicy,
    diagnostics: DiagnosticsRequest,
}
```

### SituationDescriptor

Host-neutral contextual facts that help select resident/current cognition, e.g. consumer, current interaction refs, current external object refs. It is not a prompt string.

### QueryTarget

```text
AnyRelevantCognition
Memory
Evidence
EntityNeighborhood
AnchorNeighborhood
Resource
Exact(CognitiveRef)
```

### Cue

```rust
pub enum Cue {
    Text(TextCue),
    Entity(EntityCue),
    Object(ObjectCue),
    Artifact(ArtifactCue),
    MediaRegion(MediaRegionCue),
    Tag(TagCue),
    Anchor(AnchorCue),
    Temporal(TemporalCue),
    Relation(RelationCue),
    Example(ExampleCue),
    Resource(ResourceCue),
}
```

### QueryConstraints

Must cover first wave:

```text
source classes include/exclude
memory classes include/exclude
entity requirements
occurred/observed/valid time intervals
suppression/current-revision policy
authority/source class
modality/representation constraints
required evidence class
```

### ExplorationIntent

```text
none
bounded_associative
around_tag
around_anchor
explain_association
```

No graph algorithm name is public.

### CognitiveEffort

```text
light
normal
deep
maximum
```

Effort maps to internal bounded budgets. It never exposes HNSW/USearch/Wave knobs publicly.

## 11.2 Result contract

```rust
struct CognitiveQueryResult {
    query_id: Uuid,
    generation: QueryGenerationTrace,
    status: QueryStatus,                 // complete | degraded | partial
    results: Vec<CognitiveHit>,
    resource_actions: Vec<ResourceActionSuggestion>,
    degradation: Vec<Degradation>,
    diagnostics: Option<QueryDiagnostics>,
}
```

`CognitiveHit`:

```text
reference + revision
semantic role / memory class
compact representation
source/authority class
occurred/observed/valid time
entity refs
provenance/evidence handles
match evidence by family
Wave/topology explanation when used
materialization handles
contradictions/supersession state
```

Never return only `{text, score}`.

---

# 12. Query planner and execution pipeline

The planner is deterministic given query + capability readiness + serving snapshot. It does not call an LLM merely to decide which storage engine to use.

## 12.1 Stage 0 — validate and snapshot

1. validate subject/session/AST bounds;
2. resolve capability policy;
3. load `ServingSnapshot` through ArcSwap once;
4. record exact generation IDs in query trace;
5. compile hard constraints/admissible set.

## 12.2 Stage 1 — runtime addressing

Search ResidentSet first by:

- exact targets;
- exact entity/tag/anchor/resource refs;
- lightweight lexical match against already materialized resident representations when useful;
- current situation relations.

If result need is satisfied and exploration is `none`, light effort may return without cold vector/topology retrieval. “Satisfied” is based on target/evidence requirements, not a magic score threshold.

## 12.3 Stage 2 — exact and structured lanes

Execute in parallel where useful:

- exact target/reference;
- EntityRef posting;
- Tag posting;
- Anchor posting;
- temporal/metadata structured filters;
- explicit ResourceRef candidates.

Exact identity candidates are never dependent on embeddings.

## 12.4 Stage 3 — lexical lane

If TextCue or derived textual cue exists and lexical capability is ready, Tantivy returns candidates with raw BM25 + rank evidence.

Lexical is a candidate expander, unlike Mem0 v3's semantic-only candidate pool. This is deliberate: exact names, error codes, quotations and rare strings must be recallable even when dense retrieval misses them.

## 12.5 Stage 4 — semantic cue observation

If a compatible text embedding capability is available:

1. embed the TextCue/context representation in query mode;
2. run direct dense candidate lane;
3. if active Tag embeddings + compatible EPA basis exist, run EPA;
4. run Residual Pyramid if effort allows and query contains enough semantic content;
5. produce denoised/alternative cue vectors and sensed Tags;
6. keep every generated cue's provenance; do not collapse them to “query embedding”.

If embedding unavailable, skip this stage explicitly and continue provider-free lanes.

## 12.6 Stage 5 — build SourceField and Wave query

Wave seeds may come from:

- explicit TagCue/AnchorCue/EntityCue;
- resident Tags/Anchors/entities relevant to current situation;
- exact target refs that are Wave nodes;
- Residual Pyramid sensed Tags;
- explicitly allowed relation cues;
- resource-awareness nodes when query is about where to look.

Each seed records:

```text
node
seed family
origin cue
weight before normalization
whether it is hop-0/direct
```

Seed weights are internal operational values. Normalize positive seed mass before propagation so adding more cue types does not create unbounded energy.

## 12.7 Stage 6 — Wave propagation

Run the algorithm specified in §13. Capture:

- node energy per hop;
- accumulated node potential;
- actual directed edge flow;
- seed/origin provenance;
- strongest parent/path hints;
- budget truncation/completeness.

This request-local result is `QueryRiver`.

## 12.8 Stage 7 — LocalField / TransferField

Use the same WaveGraph transition kernel and same normalized source seeds to compute two bounded scales (§13.7).

If compatible embeddings exist for active field nodes, create field vectors and run additional USearch candidate lanes. If not, field node postings/topology candidates still work.

## 12.9 Stage 8 — candidate superset

Union candidates from:

```text
runtime
exact refs
entity postings
lexical
direct dense
denoised/residual dense
local-field dense
transfer-field dense
Tag postings
Anchor postings
Wave Memory nodes
temporal structured lane
resource directory
```

Defaults for `normal` effort:

```text
exact/runtime: all within hard bound
entity/tag/anchor postings: 128 each
lexical: 64
direct dense: 64
residual/denoised dense: 64 total
local field dense: 64
transfer field dense: 64
topology memory nodes: 128
max union: 512
```

Light halves expensive semantic/topology lanes; deep roughly doubles union caps up to hard maxima; maximum may expand further but hard maximum union remains 2048. These are implementation defaults, not public semantics.

## 12.10 Stage 9 — evidence-family consolidation

Every candidate retains all source observations, but correlated lanes do not receive independent votes.

Evidence families:

```text
Exact
Runtime
Entity
Lexical
SemanticDense
TagDirect
AnchorDirect
Temporal
WaveField
Resource
LanguageRerank
```

Within `SemanticDense`, direct/residual/local-vector/transfer-vector ranks are correlated representations. Collapse them to:

- best family rank;
- variant coverage metadata;
- no repeated independent-vote bonus.

This adapts Memoria Next's useful correlation-suppression mechanics while rejecting RRF as the cognitive model itself.

## 12.11 Stage 10 — candidate semantic trail and topology

Build/lookup `CandidateSemanticTrail` (§14). Calculate field contact, river-edge/path contact, direction and direct-seed evidence only where observable.

## 12.12 Stage 11 — deterministic ranking

Use §15 scoring. Hard constraints are already enforced and cannot be undone by reranker/topology.

## 12.13 Stage 12 — optional language rerank

If requested/preferred and ready, send only a bounded top candidate set with compact representations/evidence context. Reranker returns an ordering signal.

Treat reranker as the `LanguageRerank` evidence family; it does not create candidates, identity, associations or truth. It cannot reintroduce excluded/suppressed candidates.

## 12.14 Stage 13 — result projection/runtime promotion

Return top `result_need.limit` hits with evidence and degradation.

Only hits actually selected for downstream use are eligible for provisional Session promotion. Merely being in an internal candidate pool never updates runtime or learning.

---
# 13. Clean-room Wave algorithm

This section is implementation-authoritative for the first production Wave engine.

It is a clean-room design informed by the observed behavior/problem decomposition of VCP TagMemo V9.1/V9.2 and RiverMemo Topology V3.1. VCPToolBox is distributed under CC BY-NC-SA 4.0; it is a research reference here, not a source-code dependency or source donor. Do not copy VCP source, constants, comments, class layout or serialized protocols.

## 13.1 WaveGraph node set

`WaveGraphGeneration` contains compact current serving nodes for:

```text
Memory              -- current active Memory logical object/current revision projection
Tag
Anchor
Entity               -- Host EntityRef mapped to a compact serving node
Resource
```

Artifacts/source regions are evidence, not default graph nodes. Add them to exact/evidence retrieval rather than exploding the Wave graph.

Each node has:

```rust
struct WaveNode {
    serving_id: u32,
    reference: CognitiveRef,
    node_kind: WaveNodeKind,
    embedding_key: Option<EmbeddingKey>,
    posting_key: Option<ServingDocId>,
    intrinsic_residual_gain: Option<f32>,
}
```

The generation owns a deterministic `CognitiveRef <-> serving_id` mapping.

## 13.2 Canonical relationships projected into Wave edges

Include current positive support from:

- `memory_revision_tags`;
- current entity bindings/mentions associated with current Memory revisions;
- active Anchor support links;
- active `association_evidence`;
- current Memory integration/procedural relations where they provide cognitive navigation value;
- ResourceRef links explicitly represented by cognition/Anchor/association evidence.

Do not create graph edges from:

- embedding similarity by itself;
- raw co-occurrence without an allowed structural/evidence derivation;
- every Artifact/Occurrence relationship;
- model proposals that were never committed to Authority.

## 13.3 Edge evidence aggregation

For a directed logical pair `(i,j)`, gather active evidence.

For each evidence item `e`:

```text
q_class(e):
  host_explicit       1.00
  memory_evidence     0.90
  derived_structure   0.75
  consolidation       0.70
  meaningful_use      0.50
```

These are first-wave implementation priors, not cognitive truth scales. Keep them private/internal.

Compress repetition:

```text
positive(i,j) = Σ q_class(e) * ln(1 + max(0, support_value_e))
negative(i,j) = Σ q_class(e) * ln(1 + max(0, support_value_e)) for negative/revoked evidence
raw(i,j)      = max(0, positive(i,j) - negative(i,j))
```

If `raw == 0`, omit the directed edge.

Do not linearly reward repeated coactivation forever.

## 13.4 Intrinsic residual gain

When endpoints have compatible embeddings, the projection builder may compute an intrinsic residual/specificity signal for Tag/Anchor nodes:

1. retrieve a bounded local semantic neighbor set for node vector `v`;
2. build an orthonormal basis of valid neighbor directions using Modified Gram-Schmidt;
3. project `v` onto the neighbor span;
4. compute:

```text
g_intrinsic = clamp(||v - P_neighbor(v)||² / max(||v||², ε), 0, 1)
```

Interpretation: high residual means the node carries semantic direction not well explained by nearby semantic nodes.

This signal may affect **bridge classification and diagnostics**, but must not create an edge or Association by itself.

If no compatible embeddings exist, omit this signal; Wave remains functional.

## 13.5 Hub correction and bounded row kernel

Compute pre-normalization target inflow:

```text
I_j = Σ_i raw(i,j)
```

Let `m` be the median positive target inflow in the generation.

For target `j`:

```text
relative_j = I_j / max(m, ε)
hub_penalty_j = clamp(relative_j^(-β), p_min, p_max)
```

First-wave defaults:

```text
β      = 0.35
p_min  = 0.35
p_max  = 1.25
```

Then:

```text
adjusted(i,j) = raw(i,j) * hub_penalty_j
```

### BridgeEdge classification

An edge may be marked `bridge` when it is supported by a committed experiential/causal/procedural/Anchor relation and either:

- endpoint semantic similarity is low while evidence support is positive; or
- endpoint intrinsic-residual signal indicates a cross-neighborhood link; or
- the explicit association evidence marks the relation as cross-domain/bridge-like.

The classification is a serving property derived from Authority evidence. It is not a new canonical relation type.

### Fixed outbound budget

For each source row:

```text
M = 0.90                    # total outbound conductance budget
R = 0.15                    # part reserved for bridge competition, only if bridges exist
main_mass = M - R if row has bridge edges else M
```

Every positive edge competes for `main_mass` proportional to `adjusted`.
Bridge edges additionally compete for `R` proportional to their adjusted mass.

Therefore:

```text
Σ_j K[i,j] <= M
```

Bridge edges never create mass outside the row budget.

Store the final kernel in immutable CSR. Keep logical edge diagnostics/provenance separately or in compact side arrays; do not store JSON on every hot edge.

## 13.6 Request-local bounded propagation / QueryRiver

### Initial state

Normalize positive seed weights so:

```text
Σ seed_energy = 1
```

Each propagation state is keyed by `(previous_node, current_node, origin_seed)` conceptually. Implementation may consolidate origin-seed detail when diagnostics are disabled, but must retain enough provenance to distinguish hop-0 vs emergent support.

State:

```text
node
previous node
energy
path_budget
origin type
hop
```

First-wave defaults:

```text
max_hops               = 4
max_states             = 4096
max_neighbors_per_node = 32
minimum_state_energy   = 1e-4
immediate_return       = 0.20
initial_path_budget    = 2.0
normal_edge_cost       = 1.0
bridge_edge_cost       = 0.0
fir_gamma              = 0.55
```

### Step

For state `s` at node `i`, for the strongest bounded outgoing kernel edges `(i,j)`:

```text
flow = s.energy * K[i,j]
```

If `j == s.previous_node`:

```text
flow *= immediate_return
```

Drop flow below `minimum_state_energy`.

Path budget:

```text
next_budget = s.path_budget - (bridge ? bridge_edge_cost : normal_edge_cost)
```

If `next_budget < 0` and edge is not bridge, do not expand that state further. Finite hop/state bounds apply regardless, so bridge paths cannot become unbounded.

Merge states that share `(previous,current)` in the next hop by summing energy and retaining strongest provenance hints. If next-hop state count exceeds `max_states`, retain the highest-energy states and record the discarded energy mass for observability completeness.

### FIR accumulation

Use normalized finite impulse weights:

```text
w_h = γ^h / Σ_{k=0..H} γ^k
```

Accumulate node potential:

```text
U_i += w_h * energy_i(h)
```

This prevents later hops from dominating merely because many paths exist.

### Actual edge flow

For every expansion that carries positive flow, accumulate request-local:

```text
F[i,j] += flow
```

Also record:

```text
max single-hop flow
minimum hop
bridge flag
immediate-return flag
```

`F` is **not persisted as global topology**. It belongs to the query/WorkCycle trace.

### QueryRiver output

```rust
struct QueryRiver {
    source_field: SparseField,
    node_potential: SparseField,
    edges: Vec<RiverEdgeFlow>,
    provenance: Vec<NodeWaveProvenance>,
    total_edge_flow: f64,
    discarded_state_mass: f64,
    generated_state_mass: f64,
    complete: bool,
}
```

Global WaveGraph answers “which cognitive routes exist”. QueryRiver answers “which routes actually carried this query's cognitive flow”.

## 13.7 Dual-scale LocalField and TransferField

Use the **same** immutable transition kernel `K` and normalized source vector `s`. Do not build separate graphs/retrievers.

Define a bounded restart iteration:

```text
x_0 = s
x_{t+1} = normalize_L1((1 - α)s + α Kᵀ x_t)
```

Stop after a fixed bounded iteration count; do not solve an unbounded/global steady state.

First-wave defaults:

```text
LocalField:
  α_local = 0.35
  iterations = 4

TransferField:
  α_transfer = 0.72
  iterations = 8
```

Interpretation:

- LocalField remains near direct semantic/cognitive neighborhood;
- TransferField grants more mass to structurally supported bridges and longer associations.

Return both as sparse node fields normalized to `[0,1]` by maximum node value for scoring, while retaining L1-normalized values internally where mass semantics matter.

## 13.8 Field vectors

If active Wave nodes have compatible embeddings in the same EmbeddingSpaceSignature, construct:

```text
v_field = normalize(Σ_i field(i) * embedding(i))
```

Build separate LocalField and TransferField query vectors. Never combine vectors from different spaces.

Field-vector KNN is only a candidate lane. The node field itself remains the topology evidence.

---

# 14. EPA and Residual Pyramid cue sensing

## 14.1 EPA basis generation

EPA is a serving projection over active Tag embeddings in one EmbeddingSpaceSignature.

Disable EPA for a space with fewer than 8 valid Tag vectors.

For a large Tag set, do not run an O(N²) pairwise clustering algorithm. Use a bounded deterministic representative-selection procedure:

1. L2-normalize Tag vectors and compute streaming mean;
2. create a deterministic set of random projection directions seeded from the EmbeddingSpaceSignature hash;
3. bucket vectors along those projections to obtain density summaries;
4. retain bounded high-density representatives plus high-residual/outlier representatives so rare directions are not erased;
5. cap the representative matrix at 256 rows;
6. center representatives by the weighted mean;
7. use nalgebra SVD to obtain at most 64 orthogonal basis axes;
8. retain axes while singular-energy ratio remains >= `0.01` and hard basis cap not exceeded.

This procedure is an algorithmic part of Nous, not a generic clustering framework. Do not add a distributed training pipeline.

Persist EPA generation metadata, mean, basis vectors, singular energies and embedding-space identity in the `EpaBasisGeneration` artifact.

## 14.2 Query EPA observation

Given compatible query vector `q`:

```text
q_c = q - mean
p_k = <q_c, basis_k>
energy_k = p_k² / Σ p²
H = -Σ energy_k ln energy_k / ln(K)
focus = 1 - H
```

Dominant axes are those whose energy is at least `0.05`.

Cross-axis resonance for dominant axis pairs:

```text
r_ab = sqrt(energy_a * energy_b)
```

Keep pairs with `r_ab >= 0.15` for diagnostics/query-complexity assessment.

EPA does **not** directly mutate Memory, Tags or Associations.

## 14.3 Residual Pyramid

Run when:

- a compatible dense Tag index exists;
- query vector is valid;
- effort is not `light` with a trivially satisfied query;
- residual sensing is not forbidden by capability policy.

First-wave defaults:

```text
max_levels = 3
Tag top_k per level = 12
stop when residual energy/original energy < 0.10
```

At level `l`:

1. search active Tag vectors with current residual `r_l`;
2. sort candidate Tags by similarity descending, stable TagId tie-break;
3. Modified Gram-Schmidt the candidate vectors, rejecting near-collinear directions (`norm < 1e-6`);
4. project `r_l` onto the orthonormal span;
5. compute `r_{l+1} = r_l - projection`;
6. compute level explained energy against the **original query energy**;
7. emit sensed Tag cue evidence with coefficient magnitude and level provenance;
8. continue until energy cutoff, no novel basis direction, or max levels.

For orthonormal basis `u_i`, Tag independent contribution is:

```text
c_i = |<r_l, u_i>| / max(||q||, ε)
```

Sensed Tag seed weight:

```text
seed_i = max(0, cosine(r_l, tag_i)) * c_i
```

Normalize sensed Tag seed mass before adding it to SourceField.

EPA may adjust **how much residual work is attempted**:

- high entropy or meaningful multi-axis resonance → allow full configured levels;
- highly focused query → stop after fewer levels if each level adds negligible explained energy.

Do not use EPA “logic depth” as a personality/cognition truth score.

## 14.4 Denoised query vector

For dense candidate generation, construct a denoised query vector only from explicitly observed components:

```text
v_denoised = normalize(q + λ_tag * Σ sensed_tag_weight * tag_vector)
```

First-wave `λ_tag = 0.25`, internally configurable.

This vector is an additional semantic candidate lane, not a replacement for the original query and not persisted as Memory.

---

# 15. Candidate topology and unified ranking

## 15.1 CandidateSemanticTrail

For each current Memory revision, Wave projection may materialize:

```rust
struct CandidateSemanticTrail {
    memory: MemoryId,
    nodes: Vec<TrailNode>,
    order: TrailOrder,
    provenance: TrailProvenance,
}
```

`TrailOrder`:

```text
Ordered
Unordered
Unavailable
```

Only declare `Ordered` when a real order exists from:

- source-region offsets/order;
- event temporal sequence;
- explicit narrative/structural order;
- an explicitly ordered procedural representation.

Do **not** invent order from Tag ID, embedding similarity or retrieval rank.

For `memory_revision_tags`, add nullable `ordinal` and `order_provenance` fields or equivalent normalized support in the rewritten schema so source-derived ordering can be preserved when known.

## 15.2 Field contact

Let candidate node weights `c_i` sum to 1. For an unordered candidate, use equal weights unless explicit support weights exist.

Let Source/Local/Transfer fields be max-normalized maps `S,L,T`.

```text
contact(field, C) = Σ_i c_i * field(i)
```

First-wave field contact:

```text
FieldContact =
  0.45 * contact(S,C)
+ 0.35 * contact(L,C)
+ 0.20 * contact(T,C)
```

If one field is unavailable, renormalize only across the **available requested fields** and report degradation; do not substitute a different meaning silently.

## 15.3 Actual river-edge/path contact

Only available for `Ordered` trails with at least two Wave nodes.

For adjacent candidate pair `(a,b)`:

```text
forward = normalized query river flow F[a,b]
reverse = normalized query river flow F[b,a]
pair_match = max(forward, reverse_credit * reverse)
```

First-wave `reverse_credit = 0.35`.

Use up to the four strongest matched candidate edges to avoid long trails being rewarded/penalized purely for length:

```text
EdgeContact = mean(top_4(pair_match))
```

No matched edge → 0.

`DirectionAgreement` is the forward fraction of matched flow:

```text
forward_sum / max(forward_sum + reverse_sum, ε)
```

It is diagnostic and may contribute to structural score below.

## 15.4 Structural score

If both field and edge observations exist:

```text
StructuralScore = sqrt(FieldContact * EdgeContact) * (0.75 + 0.25 * DirectionAgreement)
```

If trail is unordered/unavailable, `StructuralScore = 0`; the candidate remains eligible through base/field/direct evidence.

Absence of topology observability is not negative evidence.

## 15.5 DirectSeedEvidence

This is the clean-room replacement for the VCP term “Direct Anchor”. It must never be named `Anchor` in Nous public/domain semantics.

Let `D` be hop-0 SourceField seed values and candidate node weights `c_i`:

```text
DirectSeedEvidence = Σ_i c_i * D(i)
```

It means the candidate directly touches the original query seeds before propagation.

## 15.6 WaveObservability Ω

`WaveObservability` is query-level and candidate-independent.

Compute:

### Propagated activity

With initial seed mass normalized to 1:

```text
A = clamp(total_edge_flow / max(max_hops, 1), 0, 1)
```

### Emergence

```text
N = emergent_node_potential / max(total_node_potential, ε)
```

where “emergent” excludes hop-0 seed nodes.

### Flow entropy

For positive actual edge flows normalized to probabilities `p_e`:

```text
H = -Σ p_e ln p_e / ln(edge_count)       if edge_count >= 2
H = 0                                    otherwise
```

### Completeness

Track generated and discarded state mass when hard state truncation occurs:

```text
C = 1 - discarded_state_mass / max(generated_state_mass, ε)
```

Clamp `[0,1]`. If Wave could not run at all, `C=0` and Ω=0.

### Observable functional

```text
Ω = (A * N * H * C)^(1/4)
```

A collapsed, seed-only, single-edge or heavily truncated river therefore cannot authorize a large structural innovation bonus.

Do not expose Ω as “intelligence”, “confidence in truth” or a universal cognitive scalar. It is a query-local topology-observability gate.

## 15.7 BaseRankScore

Do not raw-sum cosine, BM25, time score and graph energy.

Convert each independent evidence family to rank utility after correlated variants are consolidated:

```text
u_f(candidate) = w_f / (60 + rank_f)      rank starts at 1
```

First-wave weights:

```text
Exact              4.0
Runtime            2.0
Entity             2.5
Lexical            1.5
SemanticDense      1.5
TagDirect          1.5
AnchorDirect       1.5
Temporal           1.0
TopologyDiscovery  1.0
Resource           2.0
LanguageRerank     1.0
```

Normalize by the maximum possible utility of the evidence families actually enabled for the query:

```text
BaseRankScore = Σ u_f / Σ (w_f / 61)
```

Clamp `[0,1]`.

Important:

- direct/residual/local-vector/transfer-vector variants produce one `SemanticDense` family rank, not four votes;
- multiple chunk hits for the same Memory revision collapse before ranking;
- exact target hits required by the query may be pinned ahead of non-exact candidates rather than relying on weight;
- hard constraints are never ranking signals.

This is a robust score-normalization mechanic, not the cognitive theory of relevance.

## 15.8 Conditional topology innovation

Topology should reward information that adds structure beyond what base relevance already explains.

For candidate `c`, find peers with:

```text
|BaseRankScore(peer) - BaseRankScore(c)| <= 0.05
```

If at least 5 peers exist:

```text
baseline_struct = median(StructuralScore(peers))
TopologyInnovation(c) = max(0, StructuralScore(c) - baseline_struct)
```

Otherwise `TopologyInnovation = 0` for this term. DirectSeedEvidence and FieldContact still work.

This keeps topology as conditional positive information rather than a universal unrelated graph bonus.

## 15.9 Final deterministic score

Before optional language rerank, compute:

```text
FinalScore =
    BaseRankScore
  + 0.20 * FieldContact
  + 0.20 * Ω^0.75 * TopologyInnovation
  + 0.10 * DirectSeedEvidence
```

Each additive term is non-negative and bounded by its coefficient because its input is `[0,1]`.

Do not treat FinalScore as a probability. Preserve its component trace.

After optional rerank, add/update the `LanguageRerank` family rank and recompute BaseRankScore + the same topology terms. Do not replace the deterministic trace with the reranker score.

## 15.10 Result diversity

After ranking, apply a bounded result-dedup/diversity pass:

- exact duplicate Memory revision → one result;
- same source region / near-identical representation → keep stronger evidence item and attach alternate provenance;
- when dense vectors are available, use a simple MMR/residual diversity pass over the final small list; use mature vector math, not a second retrieval framework;
- do not discard a lower-similarity result if it is the only carrier of required exact/entity/temporal evidence.

This is where useful VCP/Memory-system “avoid repetitive context” behavior lands.

---

# 16. Resource-aware current-state queries

External current-state Authority must not be replaced by remembered values.

## 16.1 Planner rule

If query asks for current authoritative state and a READY ResourceRef declares ownership of that state:

1. return remembered cognition only as historical/context support;
2. emit/execute Resource resolver access according to Host integration contract;
3. mark resource result as current Authority evidence;
4. do not upgrade an old remembered claim to current truth because it ranks well.

Example:

```text
“What meetings do I have tomorrow?”

Schedule Resource READY
→ query Schedule
→ current Schedule result is authoritative
→ resident/memory may add context (“this meeting was discussed last week”)
```

## 16.2 Progressive disclosure

Resource resolver contract should support staged operations where the Host can provide them:

```text
describe coverage
query synopsis/index
query records/regions
materialize exact evidence
```

Nous stores resource awareness and returned evidence/occurrences when explicitly observed; it does not mirror the full resource by default.

Host integration uses a narrow resolver interface, not a generic plugin framework:

```rust
#[async_trait]
pub trait ResourceResolver: Send + Sync {
    async fn describe(&self, resource: &ResourceRef) -> Result<ResourceDescriptor>;
    async fn query(&self, resource: &ResourceRef, request: ResourceQuery) -> Result<ResourceQueryResult>;
    async fn materialize(&self, handle: &ResourceMaterializationHandle) -> Result<ResourceMaterial>;
}
```

Resolvers are wired at application startup by `resolver_key`. A query can use a resolver only for a ResourceRef whose descriptor declares the required authority/query dimension.

---

# 17. Derivation/backfill execution

## 17.1 Event-driven worker, not autonomous scheduler

A process-local worker may execute **already-created** derivation obligations and resume pending work after restart. This is execution mechanics, not policy scheduling.

Nous core must not invent:

- nightly consolidation;
- periodic memory reflection;
- autonomous diary/dream cycles;
- arbitrary reprocessing because “it may help”.

Work is created only by explicit ingest/query/formation/configuration/management actions that establish a CoverageNeed.

## 17.2 Claim/lease

Use PostgreSQL transaction + `FOR UPDATE SKIP LOCKED` or equivalent SQLx pattern to claim a bounded batch of pending/expired-running derivations.

On claim:

- create new `derivation_attempts` row;
- set lease owner/deadline;
- mark derivation running.

Provider call runs outside long DB transaction.

On success, transactionally:

- insert immutable DerivedRepresentation;
- mark attempt succeeded;
- mark derivation succeeded/current rep;
- update CoverageNeed READY;
- enqueue/mark projection updates required by the new representation.

Transient failure retains the same deterministic derivation, creates a later attempt when explicitly retried/worker retry policy permits. Permanent unsupported capability → `unavailable` until capability/config generation changes.

## 17.3 Retry

Use bounded exponential retry for transport/transient provider errors only. Reuse a mature retry helper (e.g. `backon`) if adding it reduces custom mechanics.

Do not retry:

- invalid provider schema;
- unsupported modality;
- deterministic input validation failure;
- missing source artifact;
- incompatible embedding dimension/signature.

Persist problem codes so restart does not forget why work failed.

---
# 18. Public HTTP/CLI surface

Keep the service Host-neutral and versioned. The exact Axum router decomposition is private; the following semantic operations must exist.

## 18.1 Subject/session

```text
POST /v1/subjects
POST /v1/subjects/{subject_id}/sessions
POST /v1/subjects/{subject_id}/sessions/{session_id}/close
GET  /v1/subjects/{subject_id}/sessions/{session_id}
```

Session creation returns semantic runtime readiness, not model cache state.

## 18.2 Artifacts/observations

```text
POST /v1/subjects/{subject_id}/artifacts
POST /v1/subjects/{subject_id}/observations
GET  /v1/subjects/{subject_id}/artifacts/{artifact_id}
POST /v1/subjects/{subject_id}/materialize
```

Artifact upload is streaming multipart/body with configured max bytes.

Observation may refer to an existing ArtifactRef instead of re-uploading bytes.

Example:

```json
{
  "api_version": 1,
  "session_id": "...",
  "occurrence": {
    "source_class": "message",
    "external_object_ref": "object:messaging:message:abc",
    "occurred_at": "2026-09-07T09:00:00Z",
    "observed_at": "2026-09-07T09:00:01Z"
  },
  "material": {
    "kind": "inline_text",
    "media_type": "text/plain",
    "text": "I prefer Ethiopian coffee lately."
  },
  "entities": [],
  "formation": "consider_specific",
  "runtime": { "admit": true }
}
```

## 18.3 Memory formation/consolidation

```text
POST /v1/subjects/{subject_id}/memories/form
POST /v1/subjects/{subject_id}/memories/consolidate
GET  /v1/subjects/{subject_id}/memories/{memory_id}
GET  /v1/subjects/{subject_id}/memories/{memory_id}/revisions
```

No generic `POST /memory` accepting arbitrary untyped JSON.

Correction/lifecycle operations:

```text
POST /v1/subjects/{subject_id}/memories/{memory_id}/revise
POST /v1/subjects/{subject_id}/memories/{memory_id}/suppress
POST /v1/subjects/{subject_id}/memories/{memory_id}/restore
DELETE /v1/subjects/{subject_id}/memories/{memory_id}   -- explicit purge semantics, not soft delete
```

Topology/identity operations required by the explicit creation/correction paths:

```text
POST /v1/subjects/{subject_id}/tags
POST /v1/subjects/{subject_id}/anchors
POST /v1/subjects/{subject_id}/associations
POST /v1/subjects/{subject_id}/entity-bindings/rebind
```

These remain typed domain operations; do not expose a generic graph-node/edge CRUD API.

## 18.4 Cognitive query

```text
POST /v1/subjects/{subject_id}/query
```

Example:

```json
{
  "api_version": 1,
  "session": "...",
  "situation": {
    "consumer": "subject-chat",
    "current_objects": ["object:messaging:conversation:xyz"]
  },
  "targets": [{"kind":"memory"}],
  "cues": [
    {"kind":"entity","entity_ref":"entity:host-person:alice"},
    {"kind":"text","text":"what coffee does she like?"}
  ],
  "constraints": {
    "memory_classes": ["specific","integrative"],
    "include_suppressed": false
  },
  "exploration": {"kind":"bounded_associative"},
  "resources": {"current_authority":"prefer"},
  "result_need": {
    "limit": 12,
    "need_evidence": true,
    "need_materialization_handles": true
  },
  "effort": "normal",
  "capabilities": {
    "text_embedding": "preferred",
    "text_rerank": "optional"
  },
  "diagnostics": "summary"
}
```

Example result excerpt:

```json
{
  "status": "complete",
  "generation": {
    "lexical": "...",
    "dense": ["..."],
    "wave": "..."
  },
  "results": [{
    "ref": {"kind":"memory","id":"..."},
    "revision": "...",
    "memory_class": "specific",
    "representation": "Alice said she currently prefers Ethiopian coffee.",
    "temporal": {
      "occurred_at": "...",
      "observed_at": "...",
      "valid_from": null,
      "valid_to": null
    },
    "match": {
      "families": ["entity","semantic_dense","wave_field"],
      "base_rank_score": 0.73,
      "field_contact": 0.62,
      "topology_innovation": 0.18,
      "wave_observability": 0.51,
      "direct_seed_evidence": 0.44,
      "final_score": 0.92
    },
    "evidence": [{"kind":"source_region","ref":"..."}],
    "materialization": [{"handle":"...","level":"evidence_region"}]
  }],
  "degradation": []
}
```

Scores are diagnostics/ranking values, not probabilities.

## 18.5 Use feedback

```text
POST /v1/subjects/{subject_id}/use
```

Example:

```json
{
  "api_version": 1,
  "session_id": "...",
  "consumer": "subject-chat",
  "events": [
    {"ref":{"kind":"memory","id":"..."},"use_kind":"referenced"}
  ]
}
```

This is the only normal path by which downstream meaningful use can refresh/strengthen cognition. Query execution must not synthesize `referenced` events.

## 18.6 Resources

```text
PUT    /v1/subjects/{subject_id}/resources/{resource_ref}
DELETE /v1/subjects/{subject_id}/resources/{resource_ref}
GET    /v1/subjects/{subject_id}/resources
```

These manage awareness descriptors, not external resource contents.

## 18.7 Internal/management projection operations

CLI and/or bounded management endpoints:

```text
nous-wave projection status
nous-wave projection rebuild --kind lexical|dense|wave|epa --subject ...
nous-wave derivation run-pending --limit N
nous-wave derivation retry --id ...
nous-wave inspect memory ...
nous-wave inspect artifact ...
nous-wave inspect session ...
```

Do not expose raw SQL/USearch/Tantivy knobs as public cognitive APIs.

---

# 19. Configuration

Configuration should be expressive enough to select providers and operational budgets, but do not expose every internal constant as user-facing configuration.

Required groups:

```toml
[database]
# existing PostgreSQL/embedded PostgreSQL mechanics

[object_store]
backend = "fs"
root = "..."
max_upload_bytes = ...

[retrieval]
resident_limit = ...
max_query_results = 100

[retrieval.lexical]
enabled = true
path = "..."

[retrieval.dense]
enabled = true
path = "..."

[retrieval.wave]
enabled = true
path = "..."

[providers.<name>]
# typed provider adapter config; exact schemas may vary by adapter

[capabilities]
# mapping operation → selected provider where multiple are configured
```

Internal Wave constants in §13/§15 belong in one typed internal `WaveConfig` with sane defaults and may be exposed only in an advanced/developer section if there is a real need. Do not turn them into dozens of mandatory user settings.

Provider secrets belong in environment/secret references, never persisted in ProducerSignature metadata.

---

# 20. Failure and degradation semantics

## 20.1 Query degradation

Examples:

```text
text_embedding_unavailable
embedding_space_not_ready
lexical_generation_unavailable
wave_generation_unavailable
epa_basis_unavailable
multimodal_interpretation_unavailable
resource_unavailable
rerank_unavailable
partial_derivation_coverage
wave_state_truncated
```

A missing optional lane does not turn the whole query into an error.

A required capability/authority missing does.

## 20.2 Projection corruption/loss

If USearch/Tantivy/Wave artifact fails integrity/open:

- mark generation failed/unavailable;
- continue with remaining provider-free/available lanes when query permits;
- never reinterpret projection absence as Memory absence;
- rebuild from Authority when possible.

## 20.3 Provider disappearance

Existing DerivedRepresentations remain readable. Existing vector projections remain usable only if their space/index is present. New derivations requiring the missing provider become unavailable/pending; system startup remains healthy.

## 20.4 External current Authority unavailable

If query requires current authoritative resource and resolver is unavailable:

- return error/partial according to query requirement;
- remembered historical values may be returned only with explicit `historical_or_stale` status;
- never silently answer as current truth.

## 20.5 Query budget truncation

Return partial/degraded trace with `wave_state_truncated` and lowered `WaveObservability` via discarded mass. Do not pretend unexplored topology is exhausted.

---

# 21. Forgetting, suppression, correction and purge

## 21.1 Forgetting/accessibility

Implement forgetting as retrieval/runtime accessibility policy, not deletion.

First wave accessibility derives lazily from:

```text
meaningful-use recency
explicit hold/pin
memory class/semantic role
suppression state
optional bounded adaptive familiarity evidence
```

Do not add a universal “importance” model unless supplied explicitly or produced by a defined owner.

## 21.2 Suppression

Suppressed Memory:

- remains in Authority/history;
- excluded from ordinary current recall/projections;
- inspectable through explicit management/historical operations;
- does not require CAS deletion.

## 21.3 Correction/supersession

A correction produces:

- new Memory revision or new Memory with explicit `contradicts/supersedes` relation as semantically appropriate;
- old revision valid interval closure when justified;
- new observation/evidence provenance;
- projection invalidation.

Never mutate old evidence text to make history look consistent.

## 21.4 Purge

Purge is authoritative deletion and must explicitly traverse:

```text
Memory/Revision refs
Evidence links
Derived representations exclusively owned by purge scope
entity/tag/anchor/association support that becomes invalid
serving projections
raw artifact only when no retained reference remains
runtime resident refs
```

Reuse existing reference lock/CAS deletion mechanics. Purge must not delete a shared content-addressed object while another retained Artifact row references it.

---

# 22. Mature-system design transfer — implementation obligations

The research is not merely commentary. The following transfers are required in code.

| Research lineage | Learned mechanism | Required Nous implementation | Explicit non-transfer |
|---|---|---|---|
| Letta | persistent bounded context units / external memory | `ResidentSet`, `ConsumerWorkingSet`, `ContextContribution`, progressive materialization | context block/prompt text as cognitive Authority |
| Mem0 v3 | semantic + lexical + entity signals; distilled facts | independent lexical lane, dense lane, exact EntityRef postings, Specific Memory representation | semantic-only candidate pool; extracted entity = canonical identity; hidden fused score |
| Graphiti/Zep | episodic provenance + valid/invalid time + non-destructive contradiction | `ObservationOccurrence`, observed/world times, Memory validity, revision/supersession/contradiction | LLM graph entity as identity Authority |
| LangMem | hot path vs later/background memory work | immediate runtime admission + explicit durable formation/consolidation/derivation | internal cron/autonomous consolidation schedule |
| A-MEM | structured notes, dynamic links, evolving interpretation | revisioned memory representation, Tags/Anchors/Associations, consolidation-derived evolution | silent in-place rewrite of historical memories |
| GraphRAG | hierarchical derived summaries for global corpus questions | optional Resource synopsis/integrative representation pipeline | default Subject runtime communities; mandatory LLM graph extraction |
| HippoRAG | associative graph retrieval is useful | topology as first-class query addressing and diagnostic alternate baseline | PPR as primary production law |
| LightRAG | incremental graph + vector complementarity | independent rebuildable serving generations with semantic dependency invalidation | wholesale multi-store architecture |
| MemOS | heterogeneous memory should have lifecycle/provenance/versioning | explicit Authority/projection/runtime layers, producer signatures, versions/coverage | parametric/model state as Subject Authority |
| VCP TagMemo/RiverMemo | residual cue sensing, bounded conserved propagation, query actual-flow river, dual scales, topology observability | §§13–15 clean-room Rust implementation | VCP code, SQLite/N-API shape, VCP “Direct Anchor” terminology, CC-NC-SA source reuse |

---

# 23. Direct rewrite execution sequence

This sequence is ordered to keep the repository buildable at meaningful boundaries. It is not a ceremony; combine adjacent steps when practical as long as semantic dependencies are respected.

## T0 — Replace stale execution authority

- update root `AGENTS.md` current wave;
- add this Spec + System Description under `docs/` or the repository's chosen current-doc path;
- mark/delete the 2026-09-06 closure Spec as historical/superseded so normal Agent context does not select it;
- no compatibility note beyond one clear supersession statement.

## T1 — Dependency and workspace reset

- remove LanceDB/Arrow dependencies and code paths;
- add USearch, Tantivy, petgraph, nalgebra, roaring, Rayon, arc-swap;
- retain SQLx/PostgreSQL, OpenDAL/BLAKE3, FastEmbed where used;
- use current compatible stable dependency versions, with one workspace declaration each;
- no dependency qualification bureaucracy.

## T2 — Core/domain type reset

Implement core IDs/refs, capability signatures, CognitiveQuery/Result, material and memory-domain types. Delete old types whose semantics conflict; do not keep aliases.

## T3 — Fresh PostgreSQL schema

Replace old migrations with the §4 schema. Since PRE_PRODUCTION, development DB reset is acceptable. Implement repository transaction APIs and revision-head invariants.

## T4 — Material/CAS/Occurrence path

Adapt existing CAS mechanics; implement Artifact vs Occurrence and SourceRegion/DerivedRegion; ensure same bytes can create multiple occurrences.

## T5 — Session Runtime

Implement CognitiveSession/ResidentSet/use events and restart recovery before rebuilding recall. New observations must be immediately resident without index round-trip.

## T6 — Capability + derivation path

Implement ProducerSignature/EmbeddingSpaceSignature, provider descriptors, coverage needs, derivations/attempts, persistent textual surrogate path, event-driven worker.

## T7 — Lexical + dense serving generations

Implement Tantivy and USearch adapters, compact ServingDocId mapping, build/publish/ArcSwap snapshot, exact/entity/tag/anchor postings. No Wave yet.

## T8 — Memory formation/consolidation + Tag/Anchor Authority

Implement explicit/provider-assisted formation validators, immutable revisions, Tags/Anchors/Associations, identity rebind semantics.

## T9 — WaveGraph generation

Build canonical relationship projection, edge evidence compression, hub correction, bounded row kernel, CSR generation and publication.

## T10 — EPA + Residual Pyramid + QueryRiver

Implement §13–14 clean-room algorithms in Rust, with deterministic fixtures and traces.

## T11 — CognitiveQuery planner / candidate superset

Replace old `recall` as the primary public retrieval contract with `/v1/.../query`, while implementing exact/runtime/entity/lexical/dense/Tag/Anchor/Wave/resource lanes and evidence-family consolidation.

## T12 — Relative topology + final ranking

Implement CandidateSemanticTrail, field contact, actual river-edge contact, Ω, conditional topology innovation, DirectSeedEvidence, deterministic scoring and optional reranker evidence.

## T13 — Materialization / resource / correction / purge

Complete evidence access, external current-Authority handling, correction/supersession, suppression and purge.

## T14 — API/CLI/config cleanup

Delete obsolete routes/config entries/LanceDB/model-role assumptions/process-cycle APIs. Document the actual v1 protocol and configuration.

## T15 — Focused semantic verification and stop

Run the verification below. Fix real failures. When required behavior is implemented and the repository is clean/current, stop. Do not add speculative second backends or governance machinery.

---

# 24. Verification requirements

Testing is proof of this selected architecture, not an architecture-selection gate.

## 24.1 Required semantic tests

### Artifact/Occurrence

- same bytes observed twice → one Artifact, two Occurrences;
- same bytes in different social/time context preserve both occurrences;
- source region stable after a different parser/derived representation is created.

### Entity

- nickname/surface changes do not break EntityRef recall;
- two accounts bound to same Host EntityRef retrieve same neighborhood;
- mistaken merge can split/rebind selected mentions without raw evidence rewrite.

### Memory

- observation with formation `none` creates no Memory;
- explicit formation works with zero model provider;
- provider proposal cannot invent EntityRef;
- integrative memory cites source revisions and leaves them intact;
- correction closes/supersedes applicability without deleting history.

### Runtime

- new observation visible immediately in same Session before vector indexing;
- Session resident refs survive process restart;
- consumer A/B can receive different working sets;
- projection/surfacing does not refresh meaningful-use time;
- `referenced` does;
- eviction leaves durable Memory intact.

### Capability/degradation

- no providers: exact/entity/lexical/topology/runtime/resource paths still work;
- text embedding only: dense works, image interpretation unavailable;
- remove multimodal provider after prior image interpretation: persisted textual surrogate still retrieved;
- replace embedding model: incompatible vectors never mixed; Authority unchanged.

### Wave

Deterministic synthetic fixtures must cover:

- hub suppression: generic high-degree node cannot absorb unbounded mass;
- fixed row mass: sum outbound conductance <= configured budget;
- immediate return suppression;
- bridge edge competes within reserve rather than creating extra mass;
- finite hop/state termination;
- actual edge-flow trace contains only edges that carried flow;
- state truncation lowers completeness/Ω;
- source-only/no-emergence query gives low/zero topology observability;
- ordered trail can receive path/direction evidence;
- unordered trail receives no fabricated path score;
- lack of topology evidence never applies a negative penalty to base relevance.

### Residual

- strong first semantic direction does not erase an orthogonal weaker cue;
- residual energy monotonically does not increase beyond numerical tolerance;
- collinear Tag candidates do not create duplicate basis directions;
- energy cutoff terminates.

### Ranking

- semantic variants do not each count as independent evidence-family votes;
- exact target/hard constraint cannot be displaced by out-of-scope high-vector candidate;
- topology bonus is bounded and non-negative;
- reranker cannot reintroduce suppressed/out-of-scope candidate;
- evidence trace can explain why top candidates were found.

### Resource Authority

- current Schedule-like query uses READY external resource rather than remembered old value;
- resource failure yields explicit current-state degradation instead of stale claim presented as current truth.

### Derivation

- crash after claim but before result → lease expiry/retry works;
- retry changes AttemptId, not DerivationId;
- provider replacement creates new DerivationId/representation and preserves old one;
- duplicate worker claim is prevented by lease/transaction semantics.

### Generation coherence

- query beginning before projection publication finishes entirely on old snapshot;
- later query uses new snapshot;
- no query mixes vector space/generation identities.

## 24.2 Performance sanity checks

There is no benchmark-selection gate, but implementation must avoid obvious regression:

- no O(total memories) scan in normal query path;
- no O(total graph edges) request-local graph rebuild;
- no N×M persistent consumer activation table;
- Wave traversal bounded by configured states/neighbors/hops;
- candidate final scoring parallelizable with Rayon;
- raw file upload/read is streaming;
- provider calls never hold long PostgreSQL transactions;
- projection build occurs off the current immutable generation.

## 24.3 Focused commands

The Coding Agent should add ordinary workspace commands/tests appropriate to the Rust project. Do not create dozens of permanent gates. At minimum the final state must pass:

```text
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

If clippy exposes unrelated pre-existing noise during the rewrite, fix real local issues rather than adding suppressions/gate wrappers.

---

# 25. Explicit deletions / forbidden carry-over

The final implementation must not contain these historical concepts unless they have been transformed into the new semantics:

- `identity_key = name:{label}` identity Authority;
- model-proposed entity strings as Entity identity;
- `MemoryKind::Reference` used to classify reference material as Memory;
- raw Artifact row owning occurrence-specific social/epistemic context;
- one aggregate `indexed` boolean;
- fixed `embedding/rerank/generation` model-role assumption as capability architecture;
- one embedding per Memory revision assumption;
- process ID as Subject working-memory identity;
- old Work Cycle as long-lived runtime;
- LanceDB table name/model hash as architecture contract;
- raw-sum heterogeneous retrieval scores;
- plain multi-lane RRF where correlated semantic variants vote independently;
- PPR/diffusion as selected primary topology algorithm;
- VCP `Direct Anchor` name in Nous semantics;
- compatibility aliases for old REST routes/schema/config.

---

# 26. First-wave non-goals / preserved seams

Do not implement in this wave:

- Persona/Self ontology;
- Social/Relationship cognition owner beyond entity/association seams needed by Memory;
- Goals/Commitments owner;
- Diary or Dream/Synthetic Reality system;
- parametric memory/model-weight modification;
- model KV cache persistence as cognition;
- autonomous consolidation/reflection scheduler;
- generic MicroSystem plugin marketplace;
- multiple vector backends;
- graph database;
- distributed retrieval cluster;
- full GraphRAG-style community extraction for every memory;
- speculative audio/video-native vector pipeline when no configured capability requires it.

Preserve typed seams so those systems can later consume/produce CognitiveRefs, observations, memories, resources, Tags/Anchors/Associations and ContextContributions without rewriting Memory Authority.

---

# 27. Completion criteria

This production rewrite is complete when all of the following are true:

1. stale Memory Product Closure execution authority no longer controls normal Agent context;
2. old schema/API compatibility baggage is removed;
3. PostgreSQL Authority reflects §4;
4. Artifact/Occurrence/Region/DerivedRepresentation semantics are implemented;
5. Session runtime is first-class and restart-recoverable;
6. explicit/provider-assisted memory formation and consolidation preserve evidence/revision semantics;
7. Host EntityRef + rebind/split behavior is implemented;
8. Tags/Anchors/Associations are durable cognitive topology objects/evidence, not flattened labels;
9. capability/producer/embedding-space/coverage semantics are implemented and zero-model startup works;
10. USearch/Tantivy/exact postings/Wave CSR are rebuildable serving generations;
11. the clean-room EPA/Residual/Wave/QueryRiver/dual-field/topology/ranking pipeline is implemented as specified;
12. CognitiveQuery/Result is the primary public retrieval protocol;
13. resource-aware current Authority behavior is explicit;
14. meaningful-use feedback is distinct from retrieval exposure;
15. focused semantic verification passes;
16. documentation describes the system that actually exists;
17. no second backend/framework/generic abstraction was added “for future flexibility” without current use.

At that point, stop. Later performance tuning and empirical evaluation may change **implementation defaults** when evidence exists, but do not preemptively build alternatives now.
