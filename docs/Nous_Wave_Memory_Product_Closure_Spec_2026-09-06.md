# Nous Wave — Memory Product Closure & Portable Release Execution Spec

**Date:** 2026-09-06  
**Status:** ACTIVE / DECISION-COMPLETE  
**Repository:** `ArsvineZhu/Nous-Wave`  
**Baseline:** `master@b82246ee83922bb8e6f801425755885a7f97db92`  
**Compatibility epoch:** `PRE_PRODUCTION`  
**Executor role:** implementation only; do not reopen decided architecture unless this spec explicitly yields `PLAN_GAP`.

---

## 0. Mission

This wave converts the current substantial Memory implementation into a **functionally complete, empirically validated, portable Nous Wave Memory product**.

The wave is successful only when all of the following are true:

```text
Subject Core is independently usable
+ Memory is a real optional MicroSystem
+ Cognitive Material has product-grade write/read paths
+ Memory formation and association producers are real
+ VCP-derived retrieval principles execute in the production recall path
+ correction / suppression / forgetting / purge semantics are closed
+ export/import preserves required learned cognition
+ every public semantic capability has executable proof
+ core cognitive algorithms have ablation evidence
+ a source-less portable release package starts and works on a clean machine
```

This is **not** authorization to implement Persona, Social, Epistemic, Goals/Commitments, Reflection, Diary, Dream/Simulation, autonomous scheduling, generic agent orchestration, a plugin marketplace, or a universal cognition framework.

The product is not considered complete merely because `cargo test` compiles, an HTTP server starts, or a release archive can be created.

---

# 1. Authority and reading order

Before editing, read in this order:

1. root `AGENTS.md`;
2. Heptalogos Project Charter / current standing project posture;
3. JDD (`Justification-Driven Development`);
4. the current `nous_wave_spec_package/`;
5. this spec;
6. the current implementation and tests at the actual repository HEAD.

For conflicts:

```text
this active spec
> previous implementation plan details superseded here
> current implementation
> historical development decisions
```

Standing invariants remain in force unless this spec explicitly changes them:

```text
Subject != Model
Character Seed != Persona
Artifact != Memory
Message != Memory
Tool result != Truth
File format != semantic class
Retrieved != Reinforced
Forgetting != Suppression != Purge
Offline-capable != autonomously scheduled
MicroSystem != microservice/plugin marketplace
Serving Projection != Cognitive Authority
```

Current code has no preservation privilege. Rewrite current V1 schemas and internal APIs directly when required. Do **not** add compatibility migrations, legacy readers, aliases, deprecated payloads, fallback schemas, or dual behavior for development history.

---

# 2. JDD execution constraints

This wave is deliberately substantial because the missing behavior is already required. It must not expand into generalized machinery.

## Required

- complete the approved Memory semantics;
- use mature dependencies for generic mechanics;
- use real PostgreSQL / LanceDB / object bytes when those mechanics are the claim;
- use focused tests while editing and full verification at integration boundaries;
- delete obsolete paths rather than bridge them;
- update current-truth documentation when behavior changes.

## Forbidden unless a concrete blocker creates a `PLAN_GAP`

```text
new generic workflow framework
new DI framework
provider registry / plugin system
second vector database
new graph database
Redis / Kafka
scheduler / cron / autonomous maintenance loop
universal CognitiveObject ontology
compatibility layer for current development data
release qualification framework
permanent gate bureaucracy
mock-only production abstractions
future MicroSystem crates
```

A small domain type, helper, test fixture, release script, or dependency is not “overengineering” when it directly satisfies this spec.

---

# 3. Frozen product decisions

The executor MUST implement these decisions as written.

## D1 — Memory is a real optional MicroSystem

Global Memory capability availability and per-Subject Memory enablement are separate concepts.

```text
Global memory capability absent
→ subject.core can be READY
→ Cognitive Material / Artifact storage can still be used
→ all Memory-owned operations return UNAVAILABLE
→ memory readiness reports UNAVAILABLE, not DEGRADED
```

Per-Subject `memory_enabled=false` also makes Memory-owned operations unavailable for that Subject while leaving Subject Core and material storage valid.

`[microsystems] memory=false` MUST NOT merely mean “do not open LanceDB”.

LanceDB/model outages under an enabled Memory MicroSystem are degradation/readiness states, not MicroSystem absence.

No runtime plugin system is introduced. A compiled capability may still be operationally unavailable.

---

## D2 — Public content write authority is Cognitive Material, not direct source-less Memory insertion

There is no public `INSERT MEMORY` semantic.

Hosts write what the Subject encountered or was explicitly given as **Cognitive Material**:

```text
SourceRecord
→ immutable Artifact payload
→ optional Derivation
→ Memory formation
```

A caller that wants “remember this note” submits an explicit Human/Host material source with truthful classification/provenance. The system may deterministically form a Reference Memory from it.

This preserves:

```text
where did this come from?
what bytes/text were actually supplied?
what later interpretation formed the Memory?
```

Direct Memory operations remain limited to owned Memory semantics such as correction, suppression, consolidation and purge.

---

## D3 — Artifact payloads use one storage path: CAS only

Remove the unused inline payload dual path.

Delete from the current PRE_PRODUCTION schema/config:

```text
artifacts.inline_payload
object_store.inline_payload_max_bytes
```

All Artifact bytes, including small text/JSON, are immutable content-addressed object-store payloads.

PostgreSQL stores Artifact metadata, provenance and references. The object repository stores payload bytes.

```text
PostgreSQL = canonical structured cognitive state
CAS/Object Store = immutable payload authority
LanceDB = rebuildable retrieval projection
```

Do not preserve an inline fallback reader.

---

## D4 — Two public material input modes only

### JSON material ingest

`POST /v1/subjects/{subject}/material`

Supported content forms:

```text
text
json
existing_artifact reference
```

Remove public JSON `Vec<u8>`/binary-as-array as a normal file transport.

### Streaming upload

Add:

```text
POST /v1/subjects/{subject}/material/upload
Content-Type: multipart/form-data
```

Multipart contract:

```text
metadata: one bounded JSON part
content:  one binary stream part
```

Metadata uses the same canonical source envelope as normal ingest:

```text
api_version
source_kind
classification.origin
classification.semantic
classification.epistemic
media_type
scope
occurred?
observed_at?
actor?
invocation_id?
idempotency_key?
parent_sources[]
metadata
```

Bounds:

```text
metadata JSON <= 64 KiB
upload payload default max = 8 GiB
max is configurable and actually enforced while streaming
```

The server MUST NOT accept arbitrary server-local paths from HTTP clients.

CLI may accept a local path because the CLI is the client; it opens and streams that file through the same semantic service path.

---

## D5 — Streaming CAS write is bounded-memory

A large upload MUST NOT be materialized as one `Vec<u8>`.

Implement a streaming CAS path that:

1. receives chunks;
2. incrementally updates BLAKE3;
3. incrementally counts bytes and enforces the configured limit;
4. writes into a staging object/file;
5. resolves the final content hash only when the stream completes;
6. atomically commits/deduplicates the final CAS object;
7. commits SourceRecord/Artifact/reference metadata transactionally after valid payload completion.

Use existing local filesystem/OpenDAL mechanics. Do not create a generalized blob-provider abstraction.

A crash may leave an unreferenced CAS staging/object byte payload; it MUST NOT create a canonical Source/Artifact claim. Existing explicit unreferenced-byte cleanup mechanics may remove it later. Do not build a background garbage collector in this wave.

---

## D6 — Artifact read/provenance is a real public capability

Add HTTP and matching CLI semantic surfaces for:

```text
source show
artifact show metadata
artifact stream content
artifact lineage / derivations
memory history
memory episode membership
association evidence/debug view
suppression/readiness state
```

Required HTTP examples:

```text
GET /v1/subjects/{subject}/sources/{source}
GET /v1/subjects/{subject}/artifacts/{artifact}
GET /v1/subjects/{subject}/artifacts/{artifact}/content
GET /v1/subjects/{subject}/artifacts/{artifact}/lineage
GET /v1/subjects/{subject}/memory/{object}/history
GET /v1/subjects/{subject}/memory/{object}/episode-membership
GET /v1/subjects/{subject}/memory/{object}/associations
```

Artifact content is a streaming response with correct `Content-Type`, `Content-Length` and content hash/ETag metadata.

Byte-range serving is **not required** in this wave.

---

## D7 — Character Seed create semantics are corrected

Current `CreateSubject -> SeedContent::Artifact` cannot truthfully reference an Artifact already owned by a Subject that does not exist yet.

Change the contract:

```text
CreateSubject:
  Character Seed accepts inline text/bytes supplied as creation input

Existing Subject seed replacement:
  may accept inline content
  OR an existing Artifact owned by that Subject
```

Do not build pre-Subject Artifact staging infrastructure.

CLI `subject create --seed-file ...` reads the file client-side and submits creation content.

---

## D8 — Full Memory profile has a built-in local embedding implementation

A “complete local Memory product” cannot depend on a separately operated embedding HTTP service for its normal dense/residual recall path.

Adopt:

```text
fastembed = 6.0.2
```

for local ONNX embedding.

`fastembed` is a mechanics provider. Nous Wave retains model identity, preprocessing and cognitive semantics.

Keep exactly two embedding modes:

```text
local  # default for full portable profile
http   # existing external integration route
```

This is a concrete two-way composition, not a provider registry.

`none` is permitted only for minimal/degraded profiles where dense/residual readiness is truthfully unavailable.

Because FastEmbed inference is synchronous, call it through a bounded blocking execution path (`spawn_blocking` or equivalent); do not block Tokio request workers.

All stored/query embeddings MUST be finite and L2-normalized before retrieval math.

Model provenance must include at least:

```text
model identity
model revision / asset identity
preprocessing identity
embedding dimension
model/config digest
```

---

## D9 — Default embedding model is selected by a fixed, bounded bake-off; the executor does not ask the user

Candidate set is fixed to the models exposed by FastEmbed 6.0.2:

```text
BAAI/bge-m3
intfloat/multilingual-e5-base
intfloat/multilingual-e5-small
```

All have multilingual intent; the current product needs Chinese + English behavior.

Run the same Nous Wave retrieval fixture against all three. Record:

```text
Recall@10
MRR
nDCG@10
P50/P95 embedding latency
steady RSS while loaded
model asset size
```

Selection rule:

1. discard any candidate that fails the required Chinese/English retrieval correctness fixtures;
2. find the highest aggregate retrieval-quality candidate;
3. select the smallest/faster candidate whose aggregate Recall@10 and MRR are each within **3 percentage points** of the best candidate;
4. if no smaller candidate is within that band, select the best-quality candidate;
5. write the selected exact model/preprocessing identity into the product config and release manifest;
6. do not keep runtime “auto model selection”.

This is a one-time current-wave product decision produced by evidence, not a permanent qualification framework.

If all candidates fail a required correctness fixture, report `PLAN_GAP` with evidence.

---

## D10 — VCP is an algorithmic research source, not a code dependency

Nous Wave MUST independently implement the useful principles described by VCP TagMemo / RiverMemo, especially:

```text
full/broad cue observation rather than one tiny keyword only
query projection observation
Residual Pyramid / orthogonal weak-cue recovery
bounded competitive propagation
actual request-level graph flow
hub/specificity control
anti-backtracking
observability-gated structural contribution
Direct Anchor independent of sparse graph quality
```

Do **not** copy VCP source code, constants, file structure, protocol objects, production identifiers or configuration naming.

VCP currently declares `CC BY-NC-SA 4.0`; Nous Wave therefore uses a clean independent implementation of the published mathematical/algorithmic ideas and records attribution in research documentation.

Do not import VCP as a dependency.

---

# 4. VCP-derived query observation and residual retrieval

This section is normative.

## 4.1 Input semantics

`RecallCues.text` is a bounded **cognitive cue text**, not necessarily a search-engine keyword. A host may provide the relevant current context or composed cue text here.

Do not introduce a chat-message ontology into Nous Wave merely to mimic VCP “full context”.

Current text bound remains bounded; increase only if the existing 64 KiB bound is demonstrably insufficient for the intended host cue.

---

## 4.2 Query vector observation

For a text recall with local/external embedding available:

```text
q0 = L2-normalized embedding(cue_text)
```

Run the normal direct dense probe first.

Let the first probe return up to:

```text
probe_k = min(8, current effort.max_candidates)
```

Load the corresponding normalized candidate vectors from retained embedding state.

Compute an observation summary, not a new ontology:

```text
max cosine contact
mean positive cosine contact
normalized projection entropy across the probe
first dominant direction
explained energy ratio
residual energy ratio
```

Projection entropy is diagnostic/planning information. It is not a cognitive truth field and is not persisted as Subject state.

---

## 4.3 Dominant direction

Build a dominant direction from the strongest direct probe neighborhood:

1. take probe candidates with cosine similarity within `0.08` of the best positive similarity;
2. weight each included vector by its positive cosine similarity;
3. sum and L2-normalize the weighted vector;
4. if the neighborhood is empty/degenerate, use the top valid candidate vector;
5. reject non-finite or near-zero directions.

For later residual levels, orthogonalize each new dominant direction against already accepted directions using **Modified Gram-Schmidt** before accepting it.

Reject a new direction when its post-orthogonalization norm is `< 1e-5`.

No SVD framework is required.

---

## 4.4 Residual Pyramid

Initialize:

```text
r0 = q0
E0 = ||q0||² = 1
```

At level `i`:

```text
di = accepted dominant direction
projection = dot(ri, di) * di
r(i+1) = ri - projection
energy_ratio = ||r(i+1)||² / E0
```

Normalize `r(i+1)` only for the next vector search; retain the pre-normalization energy ratio for stopping.

Search LanceDB with the residual vector and fuse novel results as a distinct candidate channel:

```text
residual_dense_1
residual_dense_2
residual_dense_3
```

Residual search NEVER writes permanent association evidence.

Default maximum residual levels by effort:

```text
LIGHT    0
NORMAL   1
DEEP     2
MAXIMUM  3
```

Stop early when any is true:

```text
energy_ratio < 0.08
no finite residual vector
no novel candidate object is produced
new dominant direction is degenerate/collinear
candidate budget is exhausted
```

A residual level may use at most the current effort candidate budget; do not create a second unbounded candidate pool.

---

## 4.5 Required residual trace

Extend `RecallEffortTrace` with bounded diagnostics sufficient to prove production use:

```text
direct_dense_queries
residual_rounds
residual_energy_ratios[]
residual_candidates_added
residual_stop_reason?
```

No raw high-dimensional vectors are returned through the public API.

---

# 5. Association formation — build the graph that the existing activation engine consumes

The current activation solver is retained and evolved; do not replace it with PPR or a graph database.

## 5.1 Allowed durable association evidence classes

Use only evidence grounded in a real current mechanism:

```text
explicit_source
episode
temporal_adjacency
meaningful_co_recall
followed
host_explicit
```

Dense/vector similarity alone MUST NOT create durable association edges.

---

## 5.2 Explicit source association

When a current Memory revision is supported by a SourceRecord, ensure a bounded source-memory association exists.

Represent graph connectivity through the existing heterogeneous `NodeRef` types.

Create both traversal directions where useful:

```text
Source -> Memory
Memory -> Source
```

Each edge retains `source_id` evidence.

Do not create a complete graph among every Memory object sharing a source. The Source node is the fan-out junction.

---

## 5.3 Episode association

Episode formation produces:

```text
Episode -> Member
Member -> Episode
```

with `evidence_class=episode`.

Do not create an all-pairs clique among episode members.

Primary Episode boundary in the current wave remains explicit host/source grouping (`episode_key` or canonical equivalent). A model may propose additional inferred Episodes when generation is configured, but model availability is not required for the deterministic host-grouped Episode path.

Do not invent episode boundaries solely from database insertion time.

---

## 5.4 Temporal adjacency

Temporal adjacency is formed only when there is a meaningful ordered context:

- same explicit Episode/group; or
- same explicit scope plus trustworthy `occurred`/`observed` ordering chosen by the current formation rule.

Use only consecutive neighbors after deterministic ordering.

Do not connect every item inside a time window.

Default directional support:

```text
forward chronological edge = 1.0
reverse traversal edge      = 0.35
```

These are implementation defaults, not human-memory laws. They may be tuned by the algorithm evaluation in this wave without a `PLAN_GAP`.

---

## 5.5 Cognitive-use associations

Preserve the current rule:

```text
SURFACED                  -> no strengthening
INSPECTED                 -> no strengthening by itself
EXPOSED_TO_CONTEXT        -> no strengthening by itself
FOLLOWED                  -> meaningful association evidence
REFERENCED_OR_ACTED_ON    -> meaningful use / bounded co-recall evidence
```

Do not convert retrieval frequency into importance.

---

## 5.6 Remove dead relation machinery if it remains producer-less

The current `memory_relations` / `NodeKind::Relation` path is not allowed to remain as a table/API concept with no real producer.

Decision for this closure:

```text
Episode membership + association_evidence
are the current relation/association mechanics.
```

Unless current code contains a real required producer not identified by this spec, remove `memory_relations` and `NodeKind::Relation` from the current Memory V1 schema/API and update consumers directly.

Do not add a relation-authoring framework merely to justify an existing table.

If execution discovers a real current semantic relation that cannot be represented by Episode membership or association evidence, stop that subtask as `PLAN_GAP` with the concrete use case.

---

# 6. Request-level activation flow, observability and Direct Anchor

Static graph structure must not manufacture relevance.

## 6.1 Actual flow recording

Extend the existing bounded activation traversal to calculate ephemeral request-level flow statistics while it propagates.

For every edge that actually propagates positive activation in the current recall, accumulate:

```text
evidence_id
propagated_mass
```

Do not persist request-level flow as canonical Subject state.

Bound any public/debug list to the strongest support paths/edges already inside the recall budget.

---

## 6.2 Activation observability Ω

Compute one request-level scalar `omega` in `[0,1]` from actual flow, not static degree alone.

Required factors:

```text
edge_sufficiency
emergent_support
positive_flow_entropy
completion_factor
```

Definitions:

```text
seed_count = max(1, number_of_distinct_activation_seeds)
flow_edges = number of edges with propagated_mass > 0

edge_sufficiency = 1 - exp(-flow_edges / (2 * seed_count))

emergent_support =
  clamp(non_seed_positive_activation_mass / max(total_positive_activation_mass, eps), 0, 1)

positive_flow_entropy =
  normalized Shannon entropy of positive propagated edge masses
  in [0,1]; 0 when fewer than 2 positive-flow edges

completion_factor =
  1.0 when the requested activation region exhausted naturally
  0.5 when a state/edge budget truncated an otherwise non-empty frontier
```

Then:

```text
omega = (edge_sufficiency
         * max(emergent_support, 1e-6)
         * max(positive_flow_entropy, 1e-6)
         * completion_factor) ^ 0.25
```

If there is no positive propagated edge, `omega = 0`.

This is a bounded operational baseline. Evaluation may tune the constants/floor only if the fixed observability fixtures demonstrate a concrete problem. Do not add operator-facing knobs merely because they are mathematical constants.

---

## 6.3 Direct Anchor

Direct factual contact is independent of graph observability.

The direct score is formed only from direct query evidence channels such as:

```text
exact reference
lexical contact
dense direct contact
explicit entity/reference cue
temporal admissibility of that directly contacted candidate
```

A sparse graph or low `omega` MUST NOT reduce a strong direct candidate.

---

## 6.4 Association contribution

Association relevance exists only when request-induced activation actually reaches a candidate.

Normalize direct and association channel scores to `[0,1]` inside the ranking stage.

Use:

```text
effective_association = association_score * (0.35 + 0.65 * omega)

combined = 1 - (1 - direct_score) * (1 - effective_association)

if exact_reference:
    combined = 1.0
```

Consequences required by design:

- static graph beauty with no request-induced activation contributes nothing;
- low observability limits structural amplification but does not erase a real traversed association;
- direct anchors are never penalized by sparse graph evidence;
- final score remains bounded;
- vector similarity never becomes durable experiential association.

Optional configured reranking may reorder only the bounded candidate set; it may not create new canonical evidence or bypass exact-reference precedence.

---

## 6.5 Required activation trace

Expose bounded summary fields:

```text
activation_seed_count
actual_flow_edges
positive_flow_entropy
emergent_support_ratio
association_observability
activation_budget_truncated
```

Existing edge/state/hop counters remain.

---

# 7. Memory formation provenance and regeneration recipe

Current purge regeneration needs to know how a logical Memory was formed. String-prefix inference from `formation_key` is not sufficient product semantics.

## 7.1 Add narrow formation class

Add a current V1 field owned by Memory, for example:

```text
formation_class
formation_metadata jsonb
```

Allowed current classes:

```text
SOURCE_REFERENCE
HOST_EPISODE
MODEL_EXTRACT
DUPLICATE_CONSOLIDATION
EPISODE_ABSTRACTION
```

`formation_metadata` contains only bounded data required to rerun that exact formation rule, for example:

- `episode_key` for `HOST_EPISODE`;
- processor/model provenance reference for `MODEL_EXTRACT`;
- no duplicated full content.

Do not turn this into a general workflow recipe language.

Correction creates a new revision of the same logical object; it does not change the original formation class by default.

---

# 8. Purge closure

The existing recursive lineage purge direction is retained.

## 8.1 Regeneration outcome is explicit

All affected surviving logical objects resolve to exactly one of:

```text
Regenerated(new_revision)
Unsupported
```

`Unsupported` means remaining retained evidence no longer independently supports the logical object under its formation rule.

`Unsupported` MUST delete the logical Memory object and dependent derived state. It MUST NOT leave the object permanently in `regenerating` or convert a semantic invalidation into a failed processing obligation forever.

Transient infrastructure/model failure is different:

```text
provider/database temporarily unavailable
→ operation remains retryable/pending/degraded

formation ran successfully and found no support
→ Unsupported
→ delete object
```

---

## 8.2 Formation-specific support rules

### SOURCE_REFERENCE

If its owning source is purged, delete the Reference object.

If the logical reference legitimately has retained supporting sources, rebuild from retained source artifacts.

### HOST_EPISODE

Rerun the deterministic Episode grouping rule against retained sources using the stored group identity.

If no retained source belongs to the Episode, delete the Episode.

If one or more retained sources remain, regenerate membership and representation from those sources.

### MODEL_EXTRACT

Rerun the same model extraction semantic operation on retained evidence using current configured compatible processor semantics and retain provenance of the new derivation.

If the successful extraction result contains no supported replacement of the required Memory kind/identity, return `Unsupported`.

A missing/unavailable model is a transient dependency failure, not `Unsupported`.

### DUPLICATE_CONSOLIDATION

Re-evaluate the duplicate-equivalence rule over retained parents/evidence.

If fewer than two independently retained representations still satisfy the consolidation rule, return `Unsupported`.

### EPISODE_ABSTRACTION

Re-evaluate the repeated-Episode abstraction rule.

If the recurrence condition no longer holds after purge, return `Unsupported`.

---

## 8.3 No resurrection proof

After a completed purge, all ordinary and deep recall paths, association traversal, projection rebuild, restart, export/import and regeneration MUST be unable to recover content whose only support was purged.

This is a hard correctness requirement, not a best-effort test.

---

# 9. Portability V2 — preserve required learned cognition

Rewrite the current bundle format directly to schema version `2`. No V1 compatibility reader is required.

## 9.1 Bundle includes

```text
Subject metadata
Character Seed revision history
SourceRecords + parent links
Artifacts + hashes + bytes
Derivations + input/output lineage
Memory objects
Memory revisions/current heads/parents
formation class/metadata
Episode membership
suppression state
accessibility state
association evidence
cognitive-use events required by retained learning semantics
operation-independent model/provenance identities required to interpret retained state
```

## 9.2 Cognitive-use export rule

Do not export every ephemeral recall observation merely for completeness.

Export use events that are required to explain retained durable learning, especially events referenced by retained learned association evidence and meaningful-use semantics.

Accessibility aggregates themselves are exported exactly.

`SURFACED`/inspection-only history that has no retained semantic effect need not be portable.

## 9.3 Bundle excludes

```text
LanceDB tables/index files
process-local Cognitive Work Cycle frontier/cache
open HTTP sessions
worker leases
temporary staging bytes
transient processing attempt state
```

These are serving/runtime mechanics.

## 9.4 Export consistency

Export a coherent canonical Subject snapshot.

If a Subject has an object in semantic `regenerating` state or an in-flight destructive purge that prevents a coherent snapshot, export returns a truthful conflict/unavailable result. Do not serialize a half-regenerated cognitive state.

Projection lag alone does not block export.

## 9.5 Import

Import verifies all hashes before canonical commit.

For `preserve_identity=false`, remap every identity-bearing reference consistently, including:

```text
Subject
Source
Artifact
Seed revision
Derivation
Memory object/revision
Episode membership
Use event
Association evidence
```

After canonical import:

```text
rebuild/requeue embeddings as required
rebuild LanceDB projection
```

Import must not reset accessibility or suppression.

---

# 10. Configuration becomes truthful

The current example config contains fields not wired into runtime behavior. Fix this in this wave.

## 10.1 Keep and wire when real

```text
[microsystems].memory
[postgres]
[object_store].root
[object_store].max_upload_bytes
[retrieval].default_effort
[retrieval.budgets.*]
[association] bounded activation parameters that actually control the solver
[accessibility].use_decay
[processing] worker_concurrency / claim_batch / reconcile interval if the implementation actually uses them
[models.embedding]
[models.rerank]
[models.generation]
```

Do not expose every internal residual/observability constant in the public config during this wave. Keep algorithm baselines typed in code unless evaluation demonstrates an operator-facing tuning need.

## 10.2 Remove fake config

If a field is not used and this spec does not require it, remove it from current config/schema/docs rather than wiring machinery solely to preserve the field.

---

# 11. API and CLI completion

HTTP and CLI are presentations over the same service semantics.

## 11.1 Required HTTP capability set

At completion the current API must support:

```text
subject create/show/list
seed replace/history
material JSON ingest
material streaming upload
material status
source show
artifact metadata/content/lineage
derivation submit/read lineage as required
record tool observation
recall
cycle begin/recall/inspect/feedback/close
memory show/history
Episode membership inspection
association evidence inspection
memory correct
memory suppress/unsuppress
memory consolidate
memory purge
projection rebuild
bundle export/import
status/readiness
```

Do not expose LanceDB query syntax, vector tables, raw SQL or model-provider internals as the host contract.

## 11.2 Required CLI capability set

CLI must expose equivalent meaningful operations, including:

```text
subject create/show/list
material ingest --input ...
material upload --file ... --metadata ...
source show
artifact show
artifact get --output ...
artifact lineage
recall
cycle ...
memory show/history/correct/suppress/consolidate/purge
projection rebuild
bundle export/import
status
```

CLI file upload/read is streaming and does not read large files into one buffer.

---

# 12. Test policy for this closure

“Every feature is tested” means every **public semantic capability** has executable proof of its normal contract and its most important boundary/failure behavior.

It does not mean one test per function or TDD ceremony.

Tests MUST NOT introduce product abstractions only for mocks.

Use real mechanics where claimed.

---

# 13. Required test matrix

The executor may reorganize current test files by responsibility. The following scenarios are mandatory.

## 13.1 Subject Core

- create Subject with Character Seed;
- zero Memory objects is valid;
- replace Seed creates immutable history;
- stale seed revision conflict;
- restart preserves Seed history;
- `memory=false` global profile leaves Subject Core usable;
- Memory operation under global absence returns `UNAVAILABLE`;
- per-Subject `memory_enabled=false` leaves material storage usable and Memory unavailable.

## 13.2 Material and Artifact I/O

- short Unicode text ingest;
- long Markdown ingest;
- JSON/tool-result ingest;
- namespaced semantic class;
- same content with different semantic classification stays semantically distinct while CAS bytes deduplicate;
- idempotency key prevents duplicate SourceRecord;
- binary streaming upload;
- representative large streaming upload (at least 64 MiB in integration test);
- configured max upload rejects oversized stream before canonical Source commit;
- interrupted upload creates no canonical Source/Artifact claim;
- artifact metadata round-trip;
- artifact streamed bytes exactly match input hash/length;
- lineage/derivation inspection;
- unsupported binary Artifact remains retained even without parser.

## 13.3 Memory formation

- source creates deterministic Reference Memory when Memory enabled;
- one source can support multiple model-proposed Memory objects when generation configured fixture is available;
- explicit host Episode groups multiple sources;
- Episode membership is inspectable;
- explicit source and Episode association producers create bounded graph edges;
- temporal adjacency only connects deterministic consecutive eligible nodes;
- no dense-similarity edge is persisted.

## 13.4 Direct recall

- exact reference;
- lexical;
- dense;
- entity/reference cue;
- temporal admissibility;
- direct strong candidate remains available when association graph is sparse/unavailable.

## 13.5 Residual retrieval

- Normal effort executes at most one residual level;
- Deep/Maximum can execute additional bounded levels;
- residual energy decreases after non-degenerate projection;
- collinear direction terminates cleanly;
- no novel candidate terminates cleanly;
- residual candidates are traceable and do not create durable association evidence;
- hidden weak-cue fixture demonstrates material retrieval gain over ordinary dense-only baseline.

## 13.6 Association activation

- private semantically distant linked memory can be reached;
- high-degree hub does not absorb unbounded mass;
- immediate A→B→A oscillation remains suppressed;
- actual flow counts only traversed positive-flow edges;
- zero-flow graph yields `omega=0`;
- budget truncation lowers completion factor;
- static attractive topology with no query-induced path cannot manufacture a candidate;
- direct anchor is not penalized by low observability.

## 13.7 Cognitive Work Cycle

- second equivalent recall in the same cycle performs fewer index queries/reuses candidate state;
- inspected/exhausted behavior is respected;
- followed feedback advances association state;
- closed cycle cannot be reused;
- restart truthfully ends process-local frontier continuity.

## 13.8 Learning / forgetting

- 1000 `SURFACED` events do not increment meaningful uses;
- repeated meaningful use increments once per idempotent event ID;
- time lowers ordinary accessibility;
- stronger meaningful-use history raises accessibility;
- exact/strong cue can still recover low-accessibility retained Memory;
- forgetting does not delete source/revision history.

## 13.9 Correction / suppression

- correction creates new immutable revision/current head;
- previous revision remains historically readable;
- correction cannot silently mutate logical object kind/scope unless this spec explicitly permits it;
- suppression immediately removes ordinary recall;
- direct inspection still truthfully reports suppression/history;
- unsuppress restores eligibility without creating a new revision.

## 13.10 Purge

- source purge removes exclusively owned Artifact bytes;
- shared CAS bytes remain while another retained Artifact references the hash;
- recursive derivation descendants are removed;
- projection rows cannot resurrect purged Memory;
- mixed-source supported object regenerates with retained support only;
- unsupported model-derived object is deleted, not left permanently regenerating;
- Episode membership regenerates from retained sources;
- consolidated/abstracted object disappears when its rule no longer holds;
- repeat same operation ID is idempotent;
- reuse same operation ID with different request conflicts;
- restart/resume first-order purge mechanics work;
- post-purge deep/maximum recall proves no resurrection.

## 13.11 Portability

Before export create a Subject with:

```text
multiple sources
an Episode
meaningful uses
at least one learned association
a suppressed Memory
a corrected Memory history
```

Then prove:

- export schema V2 contains required state;
- hash tamper fails import before canonical commit;
- import with new Subject identity remaps all retained references;
- accessibility counts/timestamps/hints preserve required semantics;
- suppression remains;
- Episode membership remains;
- learned association remains;
- correction history/current head remains;
- projection is rebuilt rather than copied;
- expected recall targets remain in required Top-K;
- purge on the imported Subject still correctly follows remapped lineage;
- export rejects incoherent regenerating/destructive state.

## 13.12 Status/config

- global Memory absent -> `UNAVAILABLE`;
- enabled Memory with missing optional dense projection/model -> truthful `DEGRADED` capability detail;
- wired budgets actually change bounded work;
- removed config fields are rejected/absent rather than silently accepted as fake controls.

---

# 14. Self-contained real PostgreSQL/LanceDB testing

The current critical integration tests are ignored because they require a manually supplied PostgreSQL 18 URL. This is insufficient for product closure.

Adopt:

```text
postgresql_embedded = 0.21.0
PostgreSQL = 18.6
```

for local test/runtime mechanics.

Use a tiny shared test helper that starts a disposable PostgreSQL 18.6 instance on an ephemeral port and gives the test an isolated data/database directory.

The helper is test/runtime composition, not a database abstraction.

Critical PostgreSQL/LanceDB integration tests MUST no longer be hidden behind `#[ignore]` as the normal path.

Do not add Docker/Testcontainers solely for this purpose.

At meaningful integration boundaries run:

```text
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

and the algorithm evaluation described below.

---

# 15. Cognitive algorithm evaluation and ablation

Unit correctness is not sufficient to justify the VCP-derived algorithm code.

Implement one focused evaluation executable/test target or benchmark module; do not create a permanent qualification framework.

## 15.1 Ablations

Run the same fixed fixture under:

```text
A0 = lexical/direct deterministic only
A1 = A0 + ordinary dense
A2 = A1 + residual retrieval
A3 = A2 + experiential association activation
A4 = A3 + request-flow observability gating + Direct Anchor behavior
```

Report:

```text
Recall@10
Precision@10 where ground truth is defined
MRR
nDCG@10
hit rate
hub false-positive rate
repeated-candidate ratio
candidates examined
index queries
association edges visited
P50/P95 recall latency on the executing machine
```

Performance numbers must include machine/CPU/RAM identity and are evidence, not universal claims.

---

## 15.2 Mandatory private/synthetic fixture families

Use invented names/facts to minimize pretrained-model prior leakage.

### Hidden weak cue

Construct contexts with one dominant theme and one materially important weaker orthogonal cue.

Required result:

```text
A2 must improve hidden-cue Recall@10 over A1 by at least 10 percentage points
OR recover every specifically designated weak-cue target that A1 misses
```

Overall nDCG@10 on the broader fixture must not regress by more than 0.03 relative to A1.

If residual retrieval cannot demonstrate its intended effect after bounded parameter tuning, remove it and report `PLAN_GAP` rather than retaining decorative algorithm code.

### Semantically distant experience

Create experiential links whose target text is intentionally not a close paraphrase of the current cue.

Required result:

```text
A3 must recover at least 80% of designated experiential targets in Top-10
and must outperform A2 on this fixture.
```

### Attractive hub trap

Create a high-degree generic node/region connected to many unrelated memories and a lower-degree private path to the actual target.

Required result:

```text
A3/A4 target remains Top-10
hub false-positive contamination is lower than an ungated/uncontrolled activation baseline
```

### Sparse graph direct fact

Create a direct lexical/dense fact with almost no association structure.

Required result:

```text
A4 returns the direct fact at no worse rank than A3 direct scoring would provide
```

Low Ω must not erase the fact.

---

## 15.3 Parameter tuning authorization

The executor may tune only the numeric defaults explicitly owned by the algorithms in this spec when a fixed evaluation fixture demonstrates a concrete failure.

Do not change semantic rules to game metrics.

Do not add runtime auto-tuning.

After tuning, freeze the selected constants and record them with the evaluation result.

---

# 16. Stage execution plan

Stages are dependency ordered. Do not start Portable Release before the Memory Product Gate passes.

---

## C0 — Authority/current-truth reset

### Work

- verify actual HEAD before edits;
- update root `AGENTS.md` current wave to this authorized closure;
- update the affected spec package decisions so current normative docs no longer describe incomplete/fake interfaces as completed;
- update `README.md` to describe the current in-progress closure honestly;
- remove stale “complete” claims that are not yet proven;
- record VCP as research attribution, not code lineage.

### Files likely affected

```text
AGENTS.md
README.md
nous_wave_spec_package/00-DECISIONS.md
01-MICROSYSTEM-ARCHITECTURE.md
03-COGNITIVE-MATERIAL.md
05-RECALL-ASSOCIATION.md
06-MEMORY-LEARNING.md
07-PERSISTENCE-PROJECTIONS.md
08-API.md
09-PORTABILITY-OPERATIONS.md
10-EVALUATION.md
12-IMPLEMENTATION-PLAN.md
RESEARCH.md
config.example.toml
```

Do not write historical narrative into current truth docs.

### Exit

Docs describe the target being implemented and contain no contradiction with this spec.

---

## C1 — Memory availability + canonical content I/O

### C1.1 MicroSystem availability

Implement D1 across runtime status and every Memory-owned operation.

### C1.2 CAS-only schema

Rewrite current V1 migration/schema and object code to remove inline payload.

### C1.3 Streaming write

Implement D4/D5.

### C1.4 Direct read/provenance

Implement D6.

### C1.5 Character Seed correction

Implement D7.

### C1.6 Config truth

Wire/remove fields per Section 10.

### Proof

Run Subject + Material/Artifact + status/config focused tests, then workspace fmt/clippy/tests.

### Exit

A clean Subject can be created with Memory absent, and a Memory-enabled Subject can ingest/read a 64 MiB binary file without a whole-payload API buffer.

---

## C2 — Local embedding + VCP-derived recall algorithm

### C2.1 Local embedding

Integrate FastEmbed 6.0.2 in the existing inference/model owner. Do not create a new provider crate.

### C2.2 Fixed model bake-off

Run D9 and freeze the selected local default.

### C2.3 Dense normalization/provenance

Ensure canonical retained embedding metadata and LanceDB projection use normalized finite vectors with model identity.

### C2.4 Query observation

Implement Section 4 observation summary.

### C2.5 Residual Pyramid

Add a narrow numeric module, preferably under `memory-retrieval`, for modified Gram-Schmidt and residual planning. It owns no persistence.

### C2.6 Association producers

Implement Section 5.

### C2.7 Actual flow and Ω

Extend the existing activation code; do not create a second graph algorithm.

### C2.8 Unified scoring

Implement Section 6.4 and traces.

### Proof

Run focused pure algorithm tests, real LanceDB dense/residual tests and initial ablations.

### Exit

Production `recall()` at Normal effort demonstrably executes local dense + at most one residual round when eligible; association flow is generated from real evidence; trace proves which work happened.

---

## C3 — Memory semantic closure

### C3.1 Formation recipe

Implement Section 7.

### C3.2 Purge unsupported deletion

Implement Section 8.

### C3.3 Consolidation support re-evaluation

Make purge semantics correct for duplicate consolidation and Episode abstraction.

### C3.4 Remove dead relation path

Execute D6.5 cleanup if no current required producer exists.

### C3.5 Portability V2

Implement Section 9.

### C3.6 API/CLI parity

Close Section 11 surfaces.

### Proof

Run correction/suppression/purge/portability/API/CLI focused suites, then workspace checks.

### Exit

There is no known Memory semantic operation whose documented result differs from runtime behavior.

---

## C4 — Product validation

This is the **Memory Product Gate**.

### Work

- remove/replace ignored critical integration tests;
- run every mandatory scenario in Section 13;
- run Section 15 ablations;
- run a real end-to-end fresh Subject scenario;
- run restart and import into a second clean instance;
- update docs/config to measured current truth;
- remove algorithm branches proven useless by the fixed evaluation rather than carrying them forward.

### Mandatory end-to-end scenario

```text
start Nous Wave
→ create Subject + Character Seed
→ ingest Unicode text
→ ingest JSON tool observation
→ stream-upload binary/document
→ process/derive
→ form Reference + Episode/associations
→ direct recall
→ residual recall
→ associative recall
→ begin Work Cycle
→ continue/inspect/feedback
→ correction
→ suppression/unsuppression
→ meaningful-use/accessibility change
→ mixed-source purge
→ restart
→ verify no resurrection
→ export bundle V2
→ start second clean instance
→ import + rebuild projection
→ verify retained learned cognition
```

### Memory Product Gate PASS requires

```text
fmt PASS
clippy -D warnings PASS
workspace tests PASS
real PostgreSQL 18.6 tests PASS
real LanceDB tests PASS
material streaming tests PASS
all required semantic scenarios PASS
ablation requirements PASS
no ignored critical integration test
no fake readiness claim
no fake config surface
no known PLAN_GAP
```

Only after this gate passes may C5 start.

---

# 17. Portable release decisions

Portable release is explicitly authorized **after C4 PASS**.

Primary qualified release target for this wave:

```text
Windows x86_64
```

Other platforms may remain buildable, but MUST be reported `NOT_RUN` until their source-less package is actually executed on that platform.

Portable means:

```text
no Rust toolchain required
no Cargo required
no repository/source tree required
no separately installed PostgreSQL required
no separately operated embedding service required
```

Internet is not required for normal use after package extraction.

---

## D11 — Managed PostgreSQL for portable profile

Adopt:

```text
postgresql_embedded = 0.21.0
PostgreSQL 18.6
```

The portable profile uses a private managed PostgreSQL instance under the package data root.

Use PostgreSQL Embedded’s native ephemeral-port behavior (`port=0`) and persistent `data_dir`; do not implement a custom port allocator.

Keep external PostgreSQL URL mode for server/developer deployment.

Exactly two modes:

```text
managed
external
```

No database-provider registry.

Portable release build enables the crate’s bundled PostgreSQL archive support so first start does not download PostgreSQL.

Pin the exact PostgreSQL asset/version used by the release.

---

## D12 — Portable path model

Default portable root is resolved from the executable directory, not the caller’s current working directory.

Default layout:

```text
Nous-Wave/
  nous-wave.exe
  config.toml
  models/
  licenses/
  data/
    postgres/
    objects/
    lancedb/
  logs/
```

The package may be moved as a directory; all default relative paths remain inside the package root.

`--data-dir` / explicit config may override data location.

The application must work from a path containing spaces and Unicode characters.

---

## D13 — Local embedding assets are bundled in the release package

The model selected in C2 is copied into `models/` with:

```text
exact model identity
asset manifest/hash
license
preprocessing identity
```

Portable runtime loads only the bundled/canonical model path by default and does not silently download a replacement.

A missing/corrupt model yields truthful Memory degradation/failure details; it does not fall back to another model automatically.

---

## D14 — Release contents are source-less

The release archive does not contain the repository source tree, Cargo workspace, tests or development specs.

It contains only runtime-required assets and operator-facing usage/license information.

Suggested layout:

```text
Nous-Wave-0.1.0-windows-x86_64/
├─ nous-wave.exe
├─ config.toml
├─ models/
│  └─ <selected-embedding-model>/...
├─ licenses/
│  ├─ THIRD-PARTY-LICENSES.*
│  ├─ PostgreSQL-LICENSE
│  └─ model-license...
├─ README.txt or README.md
└─ VERSION.json
```

`data/` and `logs/` may be created on first run rather than shipped populated.

Do not ship test databases.

---

# 18. C5 — Portable runtime and package assembly

Start only after C4 PASS.

## C5.1 Managed DB runtime

Integrate PostgreSQL Embedded into application startup for `postgres.mode=managed`.

Startup sequence:

```text
resolve portable root
→ prepare persistent PostgreSQL data/install dirs
→ setup/start PostgreSQL 18.6
→ create nous_wave database if absent
→ connect SQLx
→ migrate current V1 schema
→ open CAS
→ open LanceDB if Memory enabled
→ load bundled local embedding if configured
→ expose readiness
→ serve/execute CLI command
```

On graceful process shutdown, stop the managed PostgreSQL instance after closing application DB work.

First-order crash recovery relies on PostgreSQL’s own durable mechanics and normal restart. Do not build recovery-of-recovery.

## C5.2 Release build profile

A narrow build feature/profile such as `portable-release` is authorized to enable bundled PostgreSQL assets and release-only path defaults.

Do not fork product semantics by build feature.

## C5.3 Release assembly

Provide one simple release assembly script, preferably PowerShell for the Windows primary target, that:

- builds release binary;
- copies required native runtime files emitted by dependencies;
- copies selected embedding model assets;
- generates/copies third-party license notices using a mature existing tool where practical;
- writes exact version/commit/dependency/model metadata;
- creates the source-less directory and ZIP archive.

Do not create a release orchestration framework.

---

# 19. C6 — Source-less release qualification

The package is not qualified by running it inside the repository.

Test the extracted release in a clean Windows environment/VM/Sandbox with no:

```text
Rust
Cargo
repository checkout
system PostgreSQL
external embedding server
```

Use a writable Unicode/space-containing path such as:

```text
C:\Temp\Nous Wave 测试\
```

## Required release tests

### R1 — start/source-less

- unpack ZIP;
- run `nous-wave.exe status` / startup;
- managed PostgreSQL initializes locally;
- local embedding model loads;
- Subject Core/Memory readiness is truthful.

### R2 — fresh functional slice

From an empty data directory:

- create Subject;
- ingest text;
- stream-upload a file;
- process;
- recall expected content;
- stop and restart;
- recall again.

### R3 — portable cognition

- create meaningful-use/association state;
- export bundle;
- initialize a second clean data root;
- import/rebuild;
- verify required learned behavior;
- perform purge and verify no resurrection.

### R4 — relocation

Stop the application, move the whole portable directory to another path, restart and verify existing data remains usable.

If managed PostgreSQL itself embeds absolute data paths that prevent safe directory relocation, document that data root is not relocatable after initialization and qualify **binary/package portability** instead; do not hide the limitation. This concrete behavior is discovered by the release test, not solved with speculative path-rewrite machinery.

---

# 20. Release claim rules

Only claim what was actually executed.

Example final status:

```text
Memory Product              PASS
Windows x86_64 portable     PASS
Linux x86_64 portable       NOT_RUN
macOS portable              NOT_RUN
```

Do not turn one release qualification into a permanent multi-platform gate system.

---

# 21. Likely implementation map

This is guidance, not a requirement to create every listed file.

```text
apps/nous-wave/src/main.rs
  config/runtime profile/managed PostgreSQL composition/CLI

apps/nous-wave/src/http.rs
  streaming upload/download + inspection routes

crates/object-store/src/lib.rs
  CAS streaming/staging/hash commit

crates/memory-domain/src/recall.rs
  trace/schema additions only

crates/memory-retrieval/src/dense.rs
  normalized dense mechanics

crates/memory-retrieval/src/association.rs
  actual flow + observability inputs

crates/memory-retrieval/src/residual.rs   # justified new narrow numeric module
  MGS + residual planning/math; no persistence

crates/memory-service/src/models.rs
  local FastEmbed + external HTTP composition/provenance

crates/memory-service/src/material.rs
  JSON material + streamed Artifact registration/read lineage

crates/memory-service/src/processing.rs
  formation producers/embedding obligations/regeneration

crates/memory-service/src/recall.rs
  query observation/residual channels/fusion/direct+association scoring

crates/memory-service/src/cycles.rs
  learned association integration remains here

crates/memory-service/src/memory.rs
  formation class/current memory semantics

crates/memory-service/src/purge.rs
  Regenerated | Unsupported closure

crates/memory-service/src/bundle.rs
  bundle V2 complete cognitive state

crates/memory-service/src/operations.rs
  consolidation re-evaluation

crates/memory-store/migrations/*
  rewrite current V1 schema directly
```

Do not split crates merely to make the tree look layered.

---

# 22. Commit strategy

Do not optimize for a fixed number of commits.

Prefer reviewable semantic units roughly aligned with:

```text
1 current-truth/spec reset
2 memory availability + schema cleanup
3 streaming material/artifact I/O
4 local embedding
5 residual retrieval
6 association formation + flow observability
7 purge/formation closure
8 portability V2 + API parity
9 tests/evaluation closure
10 portable runtime/package
11 source-less qualification/doc truth
```

If several closely coupled changes are safer in one commit, keep them together. Do not create ceremonial micro-commits.

---

# 23. PLAN_GAP rules

Do not ask the architect/user for ordinary implementation choices already decided here.

Report `PLAN_GAP` only when execution proves one of these is required:

- a new durable cognitive distinction not represented by current Subject/Material/Memory semantics;
- a new semantic owner;
- a third provider role rather than the decided local/http composition;
- a new compatibility obligation;
- a materially different failure/recovery model;
- all fixed local embedding candidates fail required product correctness;
- the residual algorithm cannot demonstrate its intended weak-cue effect without changing its semantic design;
- a real current relation cannot be represented by Episode membership/association evidence after dead relation machinery removal;
- portable release is blocked by an unavoidable native/runtime constraint that requires architecture change rather than packaging work.

A library API inconvenience, compiler error, test setup issue, file layout choice or numeric tuning inside authorized bounds is not a `PLAN_GAP`.

---

# 24. Final completion state

This wave ends only when both gates are resolved.

## Gate A — Memory Product

Must be `PASS` before release work begins.

Required:

```text
semantic closure complete
VCP-derived algorithms active and empirically justified
all public content paths work
all mandatory semantic tests pass
no critical ignored test
portability V2 works
current docs/config truthful
```

## Gate B — Portable Release

Required before claiming a usable standalone release:

```text
source-less package assembled
clean-machine startup passes
managed PostgreSQL works
bundled local embedding works
fresh functional slice passes
restart passes
export/import passes
relocation behavior is known and truthfully documented
```

After Gate B is green, update README/current docs with the actual verified release status and **STOP**.

Do not automatically begin Persona, Diary, Dream, Social, Epistemic, a second hardening program, generalized release infrastructure, or compatibility work.

---

# 25. Executor launch instruction

Implement this spec against the actual current HEAD.

Start by reading the required authorities and inspecting the existing implementation rather than assuming the baseline is unchanged. Preserve working mechanisms that already satisfy the new contract; rewrite incomplete or contradictory mechanisms directly.

Use mature libraries for generic mechanics, specifically the already adopted PostgreSQL/SQLx, LanceDB, OpenDAL/Axum/Tokio stack plus the newly authorized FastEmbed and PostgreSQL Embedded routes. Do not reimplement model runtimes or PostgreSQL process management.

During execution use focused verification; at C1/C2/C3/C4/C5/C6 boundaries run the strongest proof required by that boundary. Do not hide failing critical tests behind `#[ignore]`.

When Gate A and Gate B are complete and current documentation matches measured truth, stop.
