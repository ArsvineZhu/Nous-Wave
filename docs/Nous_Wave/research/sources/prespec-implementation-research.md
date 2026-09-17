# Nous Wave Pre-Spec Implementation Research

**Status:** COMPLETE pre-Spec implementation research; not implementation authorization  
**Scope:** implementation convergence for the new Nous Wave architecture and the revised Nous Wave ↔ Heptalogos boundary  
**Working product shape:** Nous Core + official Client; Rust Cognitive Kernel + TypeScript Cognitive Host

## 0. Standing constraints

This record carries forward the recovered handoff and current repository authority.

- `Subject != Model`, `Subject != Agent`, `Agent != Persona`.
- Nous Wave owns Subject cognition; Heptalogos owns Subject agency / behavior authority.
- Cognitive Authority, runtime state, serving projection, and model cache remain separate.
- Retrieval/projection does not imply reinforcement.
- LLM assistance is optional and proposal-first.
- Managed Cognition controls the Nous-owned cognitive context lane; the consumer retains final model invocation / behavior authority.
- PRE_PRODUCTION permits direct replacement of obsolete APIs and repository structure; no compatibility bridge without a current obligation.
- Library-first / Anti-NIH: generic transport, schema, client generation, HTTP, parsing, model-provider, pagination, tracing, etc. should use mature implementations when they lower total trajectory cost.
- `Configurable != visible != editable`.
- No hidden autonomous scheduler is required for cognition maintenance.
- Future GUI is an external Presentation consumer. Nous Core must expose a complete management contract and an official Client; GUI must not directly access database/index/kernel internals.

## 1. Research method

Each implementation area is closed only far enough to remove material ambiguity before the execution Spec. For each area record:

1. semantic owner;
2. mechanics provider / mature dependency;
3. process and data boundary;
4. public vs private contract;
5. state and revision semantics;
6. failure/degradation behavior;
7. concrete interaction simulation;
8. decisions safe to freeze for the first implementation wave;
9. remaining OPEN items that do not block that wave.

The work is organized by executable responsibility rather than historical phases.

## 2. Current repository facts relevant to migration

- Current workspace is Rust-only and `apps/nous-wave` owns embedded PostgreSQL startup, CLI and hand-written Axum HTTP surface.
- Current `cognitive-runtime` already persists Session and ResidentSet semantics and validates cognitive references.
- Current `ConsumerWorkingSet` performs bounded item/text materialization and emits typed `ContextContribution`, but does not yet encode the richer consumer-role / projection-policy semantics now required.
- Current public HTTP routes already cover Subject, Session, Artifact, Observation, Memory, Query, Use Feedback, Resource, projection refresh and topology operations. These capabilities should be preserved semantically while the transport surface is replaced rather than duplicated indefinitely.
- Current architecture/decision documents still describe Rust-only/modular-monolith physical defaults and therefore must be rewritten as part of the first implementation slice when the new architecture is authorized.

---

## Stage 1 — Protocol, process topology, and Core/Client spine

_Status: CONVERGED for Pre-Spec._

### 1.1 Transport decision

**Freeze for first wave:**

```text
Public client/API protocol: Connect over HTTP
Host -> Kernel protocol: standard gRPC over HTTP/2
Schema authority: Protobuf
Rust Kernel RPC implementation: tonic 0.14.x stable line
TypeScript Host client/server: @connectrpc/connect-node
Browser consumer transport: @connectrpc/connect-web
Schema/codegen tool: Buf + Protobuf-ES + prost/tonic codegen
```

Reasoning:

- The public side needs browser-compatible generated clients and ordinary Node consumers without a hand-written REST DTO layer. Connect-Node can serve Connect, gRPC and gRPC-Web, while Connect-Web uses the same generated service descriptors in browsers.
- `@connectrpc/connect-node` also provides `createGrpcTransport()`, so the TS Host can call a standard Rust gRPC server without requiring the Rust side to implement Connect.
- Tonic has moved into the official gRPC/CNCF Rust project and remains the current released/stable implementation line (`0.14.x`). The new `grpc` Rust crate is still a `0.9` preview as of this research point. Therefore do not adopt the preview in the first wave; isolate tonic behind the Kernel transport adapter so a future migration is local.
- Do not use `connect-rust 0.9` on the critical path merely for protocol symmetry.

**Public and private namespaces are separate:**

```text
nous.wave.v1alpha1.*          # public Core contract
nous.wave.kernel.v1alpha1.*   # private Host<->Kernel contract
```

Public messages represent product/cognitive semantics. Private messages may represent kernel-oriented commands/results but still may not leak SQL, Tantivy, USearch, petgraph, SQLx, tonic or other provider objects.

### 1.2 Protobuf/Buf policy

Use one `proto/` workspace with Buf v2 configuration.

```text
buf lint      -> permanent, low-cost schema hygiene
buf generate  -> canonical code generation entrypoint
buf format    -> deterministic source formatting
buf breaking  -> NOT a standing PRE_PRODUCTION compatibility gate yet
```

`buf breaking` is useful once a real client compatibility obligation exists. During the current PRE_PRODUCTION epoch, it may be run intentionally when investigating an external consumer but must not forbid authorized direct schema rewrites.

Generated code is treated as generated mechanics:

- no hand edits;
- deterministic regeneration command;
- generated-file path excluded from source-size ownership rules where appropriate;
- application/domain code imports generated descriptors/messages only through explicit API/transport boundaries where practical;
- Rust domain owners do not become generated-Protobuf data models.

### 1.3 Core process topology

First-wave product composition:

```text
nous-core (Node/TypeScript)
    owns public endpoint and product-level cognitive orchestration
    spawns/supervises
        |
        v
nous-kernel (Rust)
    owns PostgreSQL/object repository/retrieval/index/Authority mechanics
    exposes private loopback gRPC only
```

Do not force one executable. Distribution may contain both runtime artifacts in one application bundle.

The Kernel is a sidecar process, not a microservice deployment unit. It has no independent public network identity and no browser-facing endpoint.

### 1.4 Kernel startup and discovery

Use Node's `child_process` and normal OS process semantics. Do not add a process supervisor framework.

Proposed handshake:

1. Host generates a random per-boot kernel credential with Node `crypto`.
2. Host spawns `nous-kernel` with:
   - explicit data/config paths;
   - loopback bind request `127.0.0.1:0`;
   - the bootstrap credential in inherited process environment (or an equivalent inherited secret channel if implementation evidence shows environment leakage is material for the current threat model).
3. Kernel binds an ephemeral loopback port.
4. Kernel writes one bounded machine-readable readiness record to a dedicated/defined startup output channel containing protocol revision + bound endpoint; ordinary logs remain structured and distinguishable.
5. Host creates a gRPC client and verifies standard gRPC health/readiness before publishing Nous Core READY.
6. Every private RPC carries the per-boot credential through gRPC metadata validated by a tonic interceptor.
7. Kernel exit causes the Core's Kernel readiness to become unavailable; the Core does not invent successful state.

The one-record bootstrap handshake is intentionally tiny. It solves endpoint discovery without a port-file lifecycle, custom socket framing, service-discovery daemon, or cross-platform named-pipe design.

### 1.5 Public server mechanics

Use Fastify only if/when the Core needs non-Connect HTTP endpoints such as artifact byte streaming. `@connectrpc/connect-fastify` is the preferred integration because Connect's own Node documentation supports this composition and recommends schema validation interceptors.

Do not build a parallel REST management API. Public management/query operations are RPCs. Raw large-object transfer may use ordinary HTTP streaming on the same Core server because giant byte payloads are a different data-plane problem.

### 1.6 Public Client shape

`@nous-wave/client` is the only supported programmatic consumer facade for Heptalogos, CLI and future GUI.

Internal composition:

```text
Protobuf-ES generated service descriptors
+ Connect transport
+ Nous ergonomic domain facade
+ cancellation/deadline plumbing
+ standard error decoding
+ artifact transfer helper
```

The facade must remain framework-neutral. A future React GUI may additionally use Connect-Query + TanStack Query instead of Nous implementing its own query cache.

The Client does not expose transport/provider objects as product semantics.

### 1.7 Stage-1 interaction simulation

```text
createNousClient()
  -> Connect transport to nous-core
  -> SystemService.GetStatus

nous-core
  -> verifies Kernel client is healthy
  -> returns product + cognitive readiness

memory/list request
  -> public MemoryService
  -> Host application handler
  -> private Kernel Memory query
  -> Rust owner validates Subject and reads Authority
  -> private response
  -> Host maps to public read model
  -> generated Client returns typed result
```

No caller sees SQLx rows, internal Rust enums, tonic Status internals, or index implementation identifiers except intentionally exposed diagnostic metadata.

### 1.8 Stage-1 decisions safe to freeze

- Core + official Client is the product shape.
- Public protocol is Protobuf/Connect; private Kernel boundary is Protobuf/gRPC.
- Tonic 0.14.x is the first-wave Rust RPC implementation behind an adapter.
- Connect-Node is the TS transport/server implementation.
- Buf owns schema lint/format/generate; compatibility enforcement is deferred until an obligation exists.
- Kernel is a supervised local sidecar with a tiny boot handshake and standard gRPC health verification.
- No parallel REST management surface.
- Large blob transfer is separated from ordinary control RPCs.

### 1.9 Non-blocking OPEN items

- Exact shipping packaging of Node runtime + Rust binary.
- Whether the startup credential remains environment-carried or moves to another inherited secret channel after a concrete threat/packaging probe.
- Stable public `v1` compatibility policy; first wave remains `v1alpha1`.
- Remote management/authentication. First-wave public binding remains local-only.
- GUI framework.

**Stage 1 status: CONVERGED for Pre-Spec.**

---

## Stage 2 — Public Management Contract and official Client

_Status: CONVERGED for Pre-Spec._

### 2.1 Contract design principle

The public API is not a single `ManagementService` and is not a GUI-specific CRUD surface. Public services follow cognitive semantic ownership; GUI administration, Heptalogos integration and CLI are different consumers of the same contracts.

Proposed first-wave public service set:

```text
SubjectService
CognitionService
MemoryService
MaterialService
ResourceService
TopologyService
SystemService
```

`ConfigurationService` enters when the configuration architecture is implemented; it should not be faked as a generic JSON map in the first contract.

### 2.2 Public validation

Use Protovalidate annotations for transport-level/public-message validation and `@connectrpc/validate` in the TypeScript Core server.

Rationale:

- Protovalidate is stable (v1+) and integrates directly with Protobuf/Connect TypeScript.
- It eliminates repeated public validation for basic string lengths, enum/presence/range and cross-field constraints that are genuinely contract-level.
- Official Protovalidate runtimes currently do not include Rust. Do not adopt an immature/unofficial Rust validator only for symmetry.
- Rust domain constructors/services continue to own semantic invariants and Authority validation. Public validation is early rejection, not Authority.

Examples appropriate for Protovalidate:

```text
page_size >= 0
bounded user text length
required subject id / memory id
non-empty labels
bounded depth/max_nodes
```

Examples that remain domain validation:

```text
Evidence belongs to Subject
current revision matches mutation fence
Memory class permits requested operation
purge lifecycle preconditions
association support is semantically legal
provider proposal is acceptable Authority input
```

### 2.3 Pagination

Every public collection RPC is paginated from its first implementation.

Use the mature `page_size` / `page_token` / `next_page_token` pattern.

Rules:

- `page_size` optional; service supplies a documented bounded default.
- negative size -> `INVALID_ARGUMENT`.
- over-maximum size -> clamp to maximum rather than fail.
- `page_token` opaque and bound to the normalized query/filter/order state that produced it.
- all request parameters except page size must match the call that produced the token.
- end-of-collection is represented only by an empty `next_page_token`.
- tokens carry continuation only; never authorization.

For Kernel-owned large collections, the private Kernel list RPC should own the stable cursor. The Host may pass the opaque token through or wrap it; it does not reinterpret database/index cursor internals.

Do not expose SQL `OFFSET` as the public pagination model.

### 2.4 Concurrency and mutable heads

Current Memory uses immutable revisions plus a mutable Memory head/status. GUI, Steward and Heptalogos may race, so the current public resource must expose an opaque `etag` and authoritative mutable heads need a monotonic state revision internally.

Important migration point: current `MemoryObject` exposes `current_revision_id` and `status`, but no monotonic head revision. Add a monotonic `head_revision`/equivalent Authority fence in the Kernel schema rather than deriving freshness only from `(current_revision_id,status)`, because suppress->restore can otherwise create an ABA state that looks identical to an old client.

Public behavior:

```text
GetMemory -> Memory { etag = "..." }

ReviseMemory(memory, etag)
SuppressMemory(name, etag)
RestoreMemory(name, etag)
PurgeMemory(name, etag)

etag mismatch -> ABORTED
```

ETag is an API concurrency token, not a cognitive revision identifier. `MemoryRevisionId` remains an immutable domain identity.

### 2.5 Memory public read models

Do not return the current Rust `MemoryView` byte-for-byte as the public schema. Define stable product read models.

Suggested shapes:

```text
MemorySummary
  name / memory_id
  memory_class
  status
  head_revision_no
  current_revision_id
  title
  bounded preview
  semantic_role
  created_at
  updated_at
  tag summary
  etag

Memory
  head fields above
  current_revision: MemoryRevision
  evidence: EvidenceLink[] (bounded/detail form)
  entities
  tags
  etag

MemoryRevision
  immutable revision identity
  revision_no
  parent_revision_id
  representation
  epistemic/freshness fields
  relation/supersession semantics
  provenance/evidence summary
  created_at
```

The list resource stays lightweight. Full evidence/source material is retrieved progressively.

### 2.6 Memory RPC semantics

First-wave public methods:

```text
ListMemories
GetMemory
GetMemoryRevision
ListMemoryRevisions

FormMemory
ReviseMemory
ConsolidateMemory
SuppressMemory
RestoreMemory
PurgeMemory
```

`ReviseMemory` remains a custom cognitive operation rather than a generic `UpdateMemory` because it creates an immutable revision, carries evidence and revision relation semantics, and does not mutate the existing revision in place.

`Suppress`, `Restore`, `Purge`, and `Consolidate` remain custom methods because they carry lifecycle/side-effect meaning. Do not encode them as writable `status` fields.

No generic `PATCH memory` API is required in the first wave.

### 2.7 Administrative browse vs cognitive search

Keep these two intents separate:

```text
ListMemories
  administrative collection browsing
  stable filters/order/page cursor

CognitionService.Query
  exact/lexical/semantic/topological/resource cognitive retrieval
```

Do not create a second free-form `filter`/SQL-like language for the GUI.

Initial `ListMemories` filter may include only stable administrative dimensions already owned by Memory, e.g.:

```text
MemoryClass[]
MemoryStatus[]
TagId[]
EntityRef[]
created/observed interval
```

A GUI search box requiring relevance should use `CognitiveQuery`, then resolve/display returned Memory identities.

### 2.8 Subject management surface

A future GUI needs subject selection and inspection, so public SubjectService should include:

```text
CreateSubject
GetSubject
ListSubjects
GetCharacterSeed
ReviseCharacterSeed
```

Heptalogos may supply its own logical Subject UUID at creation. Shared logical ID does not merge Authority domains.

`ListSubjects` is paginated from day one.

### 2.9 Session/Cognition management surface

First-wave existing runtime capabilities plus the new projection work should converge on:

```text
OpenSession
GetSession
ListSessions
CloseSession
Query
ReportUse
```

Focus and `BuildProjection` are added in Stage 3 rather than faked into the initial transport conversion.

Session list/read models expose cognitive continuity state, resident counts and revision/readiness metadata; they do not expose arbitrary Node/LLM/provider runtime objects.

### 2.10 Material and evidence management

Evidence inspection is a first-class GUI requirement.

Public metadata methods should allow following:

```text
Memory
 -> MemoryRevision
 -> EvidenceLink
 -> Occurrence / SourceRegion / DerivedRepresentation / DerivedRegion
 -> Artifact metadata
 -> raw/derived content materialization
```

The control API returns metadata and stable references. Large bytes are transferred through the Client's artifact streaming helper rather than embedded in ordinary unary messages.

Do not require `ListAllArtifacts` for the first GUI slice unless an actual browser screen needs it. Evidence traversal from cognitive objects is the first current consumer.

### 2.11 Resource management surface

Public ResourceService:

```text
ListResources
GetResource
RegisterResource / UpsertResource (choose one semantic name in Spec)
RemoveResource
```

Resource awareness remains a descriptor about an external Authority, not a copied external database.

The public read model includes coverage, authority class, query dimensions, modalities, freshness/liveness, access cost and readiness where available.

### 2.12 Topology management surface

Current Memory topology contains versioned Tags/Anchors and association evidence. The GUI should eventually inspect and manage these, but the first Spec must not invent lifecycle operations that the domain has not yet defined.

Safe first public surface:

```text
GetTag / ListTags / ListTagRevisions
GetAnchor / ListAnchors / ListAnchorRevisions
GetNeighborhood(center, bounded depth/max_nodes, relation filters)
CreateTag
CreateAnchor
CreateAssociation
```

Revision/revocation management for Tag/Anchor is deferred until its Authority semantics are explicitly designed. Do not expose direct row editing merely because the GUI would find it convenient.

`GetNeighborhood` is a bounded cognitive/topology read operation. Never expose `DumpGraph()` or petgraph storage layout.

### 2.13 System/readiness surface

SystemService must let GUI/CLI/Heptalogos discover actual capabilities rather than assume them:

```text
GetStatus
GetBuildInfo
GetCapabilities
GetProjectionStatus
```

Readiness must be component/family-specific, for example:

```text
memory.authority       READY
dense.retrieval        UNAVAILABLE
lexical.retrieval      READY
topology.retrieval     READY
steward                 DEGRADED
persona                 UNAVAILABLE
```

Serving/index status is semantic-family oriented (`exact`, `lexical`, `dense`, `topology`, etc.) while implementation IDs remain diagnostic metadata, not public behavioral switches.

### 2.14 Error model

Use Connect/gRPC canonical status codes. Do not preserve the current hand-written `{ error: string }` HTTP Problem as the new cross-language contract.

Mapping direction:

```text
invalid public/domain input      INVALID_ARGUMENT
missing Subject/Memory           NOT_FOUND
stale etag / concurrent head     ABORTED
operation precondition           FAILED_PRECONDITION
required capability absent       UNAVAILABLE
internal invariant/infrastructure INTERNAL
```

Attach typed details where they materially improve Client/GUI behavior (field violations, precondition info, resource identity, retry metadata). Do not proliferate Nous-specific error-class hierarchies when canonical codes + typed details suffice.

### 2.15 Client API design

`@nous-wave/client` wraps generated clients without duplicating schemas by hand.

Proposed ergonomic surface:

```text
nous.system.status()

nous.subjects.list(...)
nous.subjects.get(id)

nous.memories.list(...)
nous.memories.get(id)
nous.memories.revise(...)
nous.memories.suppress(...)

nous.cognition.query(...)
nous.sessions.open(...)

nous.material.readArtifact(...)
```

The package should also expose generated service descriptors in a clearly named advanced namespace so a future GUI can compose Connect-Query/TanStack Query without a second generated protocol package or copied endpoint definitions.

Do not make React a dependency of `@nous-wave/client`.

### 2.16 Client cancellation, deadlines and mutation uncertainty

Every Client call accepts `AbortSignal`/deadline options through Connect's normal transport mechanics.

Do not implement blanket automatic retries for authoritative mutations in the first wave.

Example:

```text
ReviseMemory sent
network disconnects before response
```

The Client reports an uncertain transport outcome. Caller refetches the Memory head/etag and reconciles. Blind retry could create a duplicate revision if the first commit succeeded.

Automatic retries may later be enabled only for demonstrably idempotent reads or operations with explicit request-id/dedup semantics.

### 2.17 Stage-2 GUI simulation

```text
GUI opens Subject
 -> ListMemories(page_size=50)
 -> MemorySummary[] + next_page_token

user opens memory M
 -> GetMemory(M)
 -> current revision + etag + evidence refs

user opens evidence
 -> MaterialService resolves evidence metadata
 -> Client streams Artifact/source bytes on demand

user edits representation
 -> ReviseMemory(base_etag=E8, new evidence/content)
 -> Kernel verifies M is still E8
 -> creates immutable revision 9
 -> moves head, increments head_revision
 -> invalidates affected projections
 -> returns Memory(etag=E9)

another stale GUI tab sends base_etag=E8
 -> ABORTED
 -> tab refetches before offering merge/retry UX
```

### 2.18 Stage-2 decisions safe to freeze

- Public services are semantic-owner based, not one generic management service.
- All collection RPCs are paginated from first implementation.
- Memory mutable heads receive a monotonic concurrency revision and public ETag.
- Memory revisions remain immutable; edit creates a new revision.
- Memory lifecycle operations remain explicit custom methods.
- GUI relevance search reuses CognitiveQuery instead of inventing a second filter DSL.
- Protovalidate is used on the TS public boundary; Rust domain validation remains authoritative.
- Client is generated-contract-backed, framework-neutral and exposes advanced descriptors for UI integrations.
- No automatic retry of authoritative mutations without idempotency evidence.

### 2.19 Non-blocking OPEN items

- Exact public resource-name string format (`subjects/{id}/memories/{id}` vs typed UUID fields) can be selected in the Spec; internal cognitive references remain strongly typed.
- Exact first-wave administrative ordering options for list APIs.
- Tag/Anchor revision mutation lifecycle.
- Remote/public authentication policy.
- Long-running operation framework for genuinely expensive future maintenance; no LRO machinery yet.

**Stage 2 status: CONVERGED for Pre-Spec.**

---

## Stage 3 — Session, Focus, Active Cognition, and Projection Runtime

_Status: CONVERGED for Pre-Spec._

### 3.1 Existing runtime state is a real asset, but its ownership needs to narrow

The current Kernel already persists `cognitive_sessions`, `resident_refs`, and `cognitive_use_events`. `cognitive_sessions.state_revision` is monotonic, resident entries carry entry/use/hold metadata, and meaningful use can promote or admit resident references. This is useful runtime state, not throwaway scaffolding.

The new architecture should therefore preserve the following Kernel responsibilities:

```text
Kernel owns durable runtime mechanics:
  Session identity/existence
  Session open/close lifecycle record
  Session monotonic runtime revision
  Resident reference set
  reference validation against Subject
  meaningful-use events
  atomic compare-and-set runtime checkpoint persistence
```

The TypeScript Host should own the higher-variance cognitive policy:

```text
Host owns runtime cognition policy:
  Focus semantics
  which refs should enter/leave residency
  consumer-specific projection policy
  cognitive lane/context epochs
  Steward orchestration
  checkpoint payload meaning
```

This is an ownership refinement, not a rewrite of all Session storage into TypeScript.

### 3.2 Session is a Nous cognitive continuity scope

A `CognitiveSession` is explicitly not a chat conversation, Heptalogos `Reaction`, model thread, provider session, or process lifetime.

The practical relationship is:

```text
Heptalogos Conversation
    1 -> N Reactions

Nous CognitiveSession
    1 -> N Focuses over time

Reaction
    uses one CognitiveSession
    and normally one active Focus
```

A Host may choose a stable mapping from a conversation/account/user situation to a Session, but that mapping is consumer policy. Nous does not make `conversation_ref == session_id` an invariant.

Public Session state should remain intentionally small:

```text
CognitiveSession {
  session_id
  subject_id
  state: OPEN | CLOSED
  opened_at
  last_activity_at
  last_meaningful_use_at?
  runtime_revision
  etag
}
```

The current free-form `metadata jsonb` may remain an internal transition aid while PRE_PRODUCTION, but the public contract should not expose an unbounded arbitrary metadata bag as the primary extension mechanism.

### 3.3 One foreground Focus per Session in the first implementation

The first implementation should support:

```text
Session
  0..1 ACTIVE foreground Focus
  0..N SUSPENDED Focus
  N CLOSED historical Focus
```

Focus states:

```text
ACTIVE
SUSPENDED
CLOSED
```

This deliberately postpones multi-foreground attention arbitration. Truly independent simultaneous work should use different Sessions until evidence proves that one inference needs multiple independent foreground Focus owners.

Public operations:

```text
CreateFocus
GetFocus
ListFocuses
SwitchFocus
SuspendFocus
ResumeFocus
CloseFocus
```

`SwitchFocus` is a semantic operation, not shorthand for two unrelated writes. It must guarantee that a Session never commits two ACTIVE foreground Focuses.

### 3.4 Focus identity and state

Suggested semantic state:

```text
Focus {
  focus_id
  subject_id
  session_id
  state
  created_at
  last_activated_at?
  suspended_at?
  closed_at?
  runtime_revision
  checkpoint_revision
  descriptor
  etag
}
```

`descriptor` is a compact human/model-readable statement of the current cognitive task/topic, not an ontology. It can begin as bounded text plus source refs.

Do not add:

```text
importanceScore
attentionEnergy
salience0to1
urgencyFloat
```

without an operational contract. Ordering and retention policy should first use explicit states, recency, use events, caller requirements, and retrieval evidence rather than invented cognitive scales.

### 3.5 Focus mutation requires atomic Session runtime fencing

Public clients can race and the Host can restart. Focus state therefore cannot live only in a JavaScript object graph.

Required invariant:

```text
for each OPEN Session:
  count(Focus where state == ACTIVE) <= 1
```

The reliable state transition belongs behind the Kernel persistence boundary even though TypeScript owns the meaning of Focus.

The first-wave implementation should use a narrow compare-and-set runtime state primitive rather than a distributed lock or workflow engine:

```text
expected_session_runtime_revision
+ runtime mutation
-> new_session_runtime_revision
```

A switch can therefore be one Kernel transaction:

```text
verify Session revision R
checkpoint current active Focus if supplied
mark current ACTIVE -> SUSPENDED
mark target -> ACTIVE
advance Session runtime revision to R+1
commit
```

A stale caller gets `ABORTED` and refetches.

This reuses PostgreSQL transaction semantics already present in the Kernel and avoids introducing a new lock service, event bus, or workflow runtime.

### 3.6 A narrow Runtime Checkpoint Store is justified

Focus checkpoints, managed context epochs, and Steward continuation all need restart-safe runtime persistence. Encoding each fast-changing TypeScript payload as a new Rust table/schema would couple the Kernel to Host research churn.

A single narrow runtime checkpoint mechanism is justified by COST because it centralizes one responsibility: reliable versioned persistence for non-authoritative cognitive runtime payloads.

Proposed private Kernel envelope:

```text
RuntimeCheckpointRecord {
  subject_id
  session_id
  owner_kind
  owner_key
  schema_version
  revision
  payload_bytes
  created_at
  updated_at
}
```

Constraints:

```text
owner_kind is from a small Kernel-recognized set
payload is versioned Protobuf bytes
mutation is compare-and-set by revision
records are scoped to Subject + Session
payload size is bounded
```

It must not become:

```text
searchable Memory
Cognitive Authority
reinforcement evidence
universal JSON document database
public extension KV store
arbitrary plugin storage
```

Rust owns reliable storage/CAS; TypeScript owns payload semantics and schema.

### 3.7 FocusCheckpoint stores continuity, not a copy of cognition

A Focus checkpoint should primarily store references and minimal derived continuation state:

```text
FocusCheckpoint {
  focus_id
  checkpoint_revision
  descriptor
  resident_refs[]
  working_refs[]
  continuation_summary?
  source_runtime_revision
  producer
  created_at
}
```

The important rule is:

```text
checkpoint stores refs to Authority
not snapshots of Memory/Persona/Relationship Authority
```

On resume:

```text
load checkpoint
-> validate/resolve refs against current Authority
-> drop/reclassify invalid refs truthfully
-> re-run relevant retrieval/projection
-> create a new consumer context epoch when needed
```

This prevents a resumed Focus from restoring stale Persona/Memory copies.

`continuation_summary` is replaceable runtime interpretation. If model-produced, its producer metadata is retained, but it is not Memory and does not silently become truth.

### 3.8 ResidentSet is Session-wide; FocusWorkingSet is derived

Keep the current durable `ResidentSet` concept because it records cognitive material that remains cheaply available across nearby operations.

Refine the layering:

```text
Subject Cognitive Authority
        |
        v
Session ResidentSet
  durable/recoverable refs + use metadata
        |
        v
FocusWorkingSet
  derived subset for active Focus
        |
        v
Consumer Projection
  bounded projection for one consumer/invocation
```

`FocusWorkingSet` should not receive a second authoritative database table in the first wave. Its stable refs live in `FocusCheckpoint`; its live ranking/selection lives in Host memory and is recoverable from Authority + Session + checkpoint.

This avoids three competing truths:

```text
Memory copy
Active Cognition copy
Prompt copy
```

### 3.9 Existing ConsumerWorkingSet becomes a Kernel candidate/materialization primitive

The current Rust `ConsumerWorkingSet` implements useful mechanics: Subject/session validation, ref deduplication, modality filtering, byte/item budgets, materialization, and degradation reporting.

Its current `ConsumerProfile`, however, only knows:

```text
consumer_id
max_items
max_text_bytes
accepted_modalities
materialization_policy
```

It cannot express the future semantic difference between an expression agent, planner, evaluator, system operator, or GUI diagnostic consumer.

Therefore the implementation should narrow—not delete—the Rust concept:

```text
old final-sounding ConsumerWorkingSet
        ->
KernelContributionBatch / MaterializedCandidateBatch
```

The Kernel remains responsible for validated source/candidate material. The Host becomes responsible for consumer semantics and final cognitive projection.

A rename is preferred during the PRE_PRODUCTION rewrite if it removes the false ownership implication; no compatibility shim is required.

### 3.10 Consumer identity must be opaque and policy-bound

Nous Wave must support consumers other than Heptalogos. Therefore public protocol must not hard-code a global enum such as:

```text
EXPRESSION_AGENT
PLANNER_AGENT
OPERATOR_AGENT
```

Use an opaque stable consumer identity, for example:

```text
heptalogos.expression
heptalogos.planner
heptalogos.reviewer
nous.gui.memory-inspector
```

But a request must not gain privileges merely by self-declaring an ID or asking for `persona=required`.

The Host resolves the ID to a configured/registered `ConsumerPolicy`:

```text
ConsumerPolicy {
  consumer_id
  allowed contribution families
  default requirements
  materialization permissions
  budget ceilings
  sensitivity/authority constraints
}
```

The caller may provide bounded situation/query hints within that policy. Authorization and cognitive selection remain separate concerns.

First-wave composition can be a static typed map loaded from configuration. Do not build a plugin marketplace or policy engine.

### 3.11 ProjectionRequest

The public semantic request should be richer than `working_set()` but still transport-neutral:

```text
ProjectionRequest {
  subject_id
  session_id
  focus_id?
  consumer_id
  situation
  query?
  effort
  budget
  known_context_state?   // only when managed mode is later used
}
```

`Situation` contains current external refs / exact cues supplied by the consumer and does not copy Heptalogos ReactionWorkspace into Nous.

`effort` continues the existing semantic levels:

```text
light
normal
deep
maximum
```

It influences planning depth/cost, not a public engine preset.

### 3.12 Cognitive need requirements

Consumer policy needs a compact way to express which cognitive families matter. Use requirement strength rather than arbitrary weights:

```text
REQUIRED
PREFERRED
OPTIONAL
FORBIDDEN
```

Example resolved policy:

```text
heptalogos.expression:
  persona       REQUIRED when available profile requires it
  relationship PREFERRED
  memory        PREFERRED
  epistemic     OPTIONAL
  goals         OPTIONAL

machine.operator:
  persona       FORBIDDEN
  relationship  FORBIDDEN
  memory        OPTIONAL only for explicit diagnostic operation
```

The exact list of future families should not force empty future crates. The Host can define stable capability/contributor IDs while a missing contributor reports readiness truthfully.

### 3.13 Projection Contributor is a narrow internal seam, not an extension framework

The Host needs one common way to collect Memory now and Persona/Relationship later. A small internal interface is justified:

```text
ProjectionContributor {
  id
  readiness()
  contribute(ProjectionContext) -> ContributionBatch
}
```

First-wave composition is explicit/static:

```text
contributors = [memoryContributor, runtimeContributor]
```

Later systems are added intentionally.

Do not add discovery, dynamic loading, priorities marketplace, lifecycle framework, generic DI, or provider package protocol around this interface merely because the interface exists.

### 3.14 Contribution IR uses a common envelope with typed payloads

A common envelope is necessary for budgeting, provenance, evidence and invalidation:

```text
ContributionEnvelope {
  contribution_id
  family
  source_refs[]
  authority_class
  evidence_refs[]
  provenance
  freshness
  requirement_strength
  estimated_size
  materialization
}
```

The content itself should remain a TypeScript discriminated union owned by the contributor/renderer rather than a universal `CognitiveObject` JSON ontology.

For the current Memory contributor, a contribution can carry a typed Memory projection payload. Future Persona/Relationship payloads can differ.

After planning, each selected contribution is rendered into consumer-facing `ContextSegment` values.

### 3.15 Do not invent another universal relevance/importance scalar

Retrieval already has evidence-family-aware ranking. Projection planning should preserve that evidence rather than collapsing all cognitive families into one invented `importance: 0.0..1.0` score.

First-wave deterministic ordering should use an inspectable tuple such as:

```text
requirement tier
explicit/current-reference priority
retrieval order within evidence family
resident/focus state
freshness/use recency where semantically applicable
stable tie-breaker
```

A contributor may retain its own domain score with provenance, but the shared Projection IR does not require one universal cognitive number.

This aligns with the existing decision that a single fused score must not erase exact/entity/lexical/dense/topology evidence families.

### 3.16 Projection budget has hard transport/resource dimensions and optional token estimation

The current Kernel byte/item budget remains useful and deterministic.

First-wave budget:

```text
ProjectionBudget {
  max_items
  max_text_bytes
  max_estimated_tokens?
}
```

Rules:

- item and byte ceilings are always enforceable;
- token ceiling is enforced only when the active consumer/model path supplies a tokenizer/estimator contract;
- no single tokenizer is embedded into cognition semantics;
- provider/model-specific token counting belongs to the model/consumer adapter;
- absence of token estimation does not disable deterministic projection.

This avoids making a provider tokenizer part of Cognitive Authority while still allowing precise model-context budgeting when available.

### 3.17 Deterministic Projection Planner first

Before Steward/model synthesis, implement a deterministic planner that can produce a useful projection with zero model providers:

```text
resolve ConsumerPolicy
-> validate Session/Focus
-> incorporate exact Situation refs
-> run/consume CognitiveQuery as required
-> obtain Kernel materialized candidates
-> add ResidentSet / FocusWorkingSet refs
-> call available contributors
-> apply requirement and authorization filters
-> deduplicate by cognitive/source identity
-> enforce modality/materialization/budget
-> render selected contributions
-> return CognitiveProjection + trace
```

The trace should explain selection in compact machine-readable form without exposing internal algorithm implementation knobs.

### 3.18 CognitiveProjection public result

Suggested result:

```text
CognitiveProjection {
  projection_id
  subject_id
  session_id
  focus_id?
  consumer_id
  created_at
  source_runtime_revision
  segments[]
  selected_refs[]
  degradation[]
  trace_summary
}

ContextSegment {
  segment_id
  semantic_role
  media_type
  text?
  structured_content?   // only for explicitly standardized public payloads
  source_refs[]
  authority_class
  evidence_refs[]
  provenance
  freshness
  cache_stability_hint
}
```

`cache_stability_hint` is semantic (`IMMUTABLE`, `EPOCH_STABLE`, `DYNAMIC`) and not a provider-specific cache-control object. A downstream model adapter may map it to provider options.

Do not expose internal Tantivy/USearch/Wave/EPA scoring fields as the normal projection contract.

### 3.19 Projection is use selection, not reinforcement

A projection must preserve the existing use ladder:

```text
candidate
-> surfaced/selected for projection
-> exposed to downstream model/consumer
-> referenced/followed
-> useful downstream effect
-> explicit reinforcement
```

`BuildProjection` may record that refs were surfaced/selected, but it must not automatically mark them meaningfully used or reinforced.

The consumer calls `ReportUse` after the actual downstream stage. The current Kernel use-event store can remain the durable evidence mechanism while the public vocabulary is refined.

### 3.20 ReactionWorkspace remains entirely in Heptalogos

The following must never enter Nous Session/Focus checkpoints merely because they are cognitively adjacent:

```text
tool intermediate output owned by Reaction
partial behavior draft
review proposal
DecisionCommit candidate
EffectOperation state
external-dispatch uncertainty
```

Heptalogos owns these as agency/execution artifacts. Nous may observe committed or explicitly supplied material afterward, with provenance, but it does not become the workflow/checkpoint engine for Heptalogos.

### 3.21 Session/Focus recovery simulation

Normal path:

```text
OpenSession S (runtime rev 1)
CreateFocus A -> ACTIVE (rev 2)
query / project
meaningful use updates ResidentSet (rev 3...)
SwitchFocus A -> B:
  persist A checkpoint
  A SUSPENDED
  B ACTIVE
  commit Session rev N+1 atomically
```

Host crashes after commit but before response:

```text
client outcome uncertain
-> reconnect
-> GetSession/GetFocus
-> observe committed runtime revision
-> continue from Kernel truth
```

Host restarts later:

```text
load Session + active Focus + checkpoint refs
-> resolve refs against current Authority
-> rebuild FocusWorkingSet
-> invalidate/rebuild managed consumer epoch when source revisions changed
```

No old Persona/Memory copy is restored.

### 3.22 Focus switch race simulation

Two clients read Session revision 17.

```text
Client A: SwitchFocus(B, expected=17)
Client B: SwitchFocus(C, expected=17)
```

One transaction commits revision 18. The other receives `ABORTED`, refetches, and can decide whether another switch is still desired.

There is no distributed lock and no last-write-wins ambiguity.

### 3.23 Projection with zero model providers

```text
Session S ACTIVE
Focus F ACTIVE
consumer = heptalogos.expression
Steward unavailable
embedding unavailable
```

Planner can still use:

```text
exact refs
lexical retrieval
explicit topology
available resource descriptors
ResidentSet
Memory Authority
stored derived text already available
```

It returns a deterministic projection plus explicit degradation for missing channels. GUI/Client management remains unaffected.

### 3.24 Stage-3 decisions safe to freeze

- Session remains a durable/recoverable Nous cognitive continuity scope and is not a Conversation/Reaction/model thread.
- Current Rust Session/ResidentSet/use-event persistence remains Kernel mechanics.
- Focus semantics move to the TypeScript Host, with one ACTIVE foreground Focus per Session in the first wave.
- Focus state transitions and checkpoints commit through Kernel CAS/transactions so Host restart cannot create dual truth.
- Add one narrow versioned `RuntimeCheckpointStore` for non-authoritative Host runtime payloads; do not create one Rust schema per evolving TS runtime type.
- FocusCheckpoint stores Authority refs and replaceable continuation state, not copies of cognitive Authority.
- Session ResidentSet is durable; FocusWorkingSet is derived/checkpointed; consumer Projection is invocation-specific.
- Current Rust ConsumerWorkingSet narrows into Kernel candidate/materialization mechanics; Host owns final consumer-specific Projection policy.
- Consumer IDs are opaque and resolved to Host-owned policies; callers cannot self-grant cognitive access by declaring needs.
- Projection Contributor is a static narrow internal seam, not a plugin framework.
- Shared Contribution IR uses a common provenance/budget envelope with typed family-specific payloads; no universal CognitiveObject ontology.
- Deterministic Projection Planner is implemented before Steward/model synthesis.
- Projection selection does not equal meaningful use or reinforcement.

### 3.25 Stage-3 non-blocking OPEN items

- Exact Focus descriptor fields beyond bounded text + refs.
- Exact internal table layout for RuntimeCheckpointStore (`bytea` payload vs equivalent), to be decided in the implementation Spec.
- Whether public Focus mutations expose ETag directly or only Session runtime ETag; both can preserve CAS semantics.
- Exact ContextSegment structured-content repertoire; first wave may remain text + refs where sufficient.
- Provider/model-specific token estimators enter with Model Runtime research, not this stage.

**Stage 3 status: CONVERGED for Pre-Spec.**

---

## Stage 4 — Model Runtime, Steward, and Managed Cognitive Context

_Status: CONVERGED for Pre-Spec._

### 4.1 AI SDK 7 is the Host model substrate, not the cognitive architecture

Current official AI SDK documentation confirms that the SDK already provides the generic mechanics Nous needs:

```text
createProviderRegistry / customProvider
language/embedding/rerank provider abstraction
generateText / streamText
Output.object / JSON-schema-backed structured generation
middleware
provider-specific options
ModelMessage[]
AbortSignal / timeout / retry mechanics
```

Therefore Nous should not create another provider SDK, model HTTP client hierarchy, structured-output parser, or generic middleware framework.

The boundary is:

```text
AI SDK owns:
  provider/model transport mechanics
  provider normalization
  structured-output transport/validation mechanics
  streaming/tool protocol mechanics when used
  provider-specific options

Nous owns:
  cognitive operation roles
  which operation may use which model binding
  cognitive inputs/outputs
  proposal validation
  Authority commit boundaries
  readiness/degradation semantics
```

Do not adopt AI SDK's Agent abstractions as Nous's cognition architecture. `StewardEngine` is a bounded cognitive worker and can use `generateText(... Output.object(...))` directly.

### 4.2 One Host ModelRuntime; do not keep a parallel Rust provider layer

The new polyglot architecture should converge model/provider invocation in the TypeScript Host.

The current Rust code has provider/capability seams for embeddings/formation/derivation. Those semantics remain useful, but direct external model/provider transport should migrate toward Host orchestration rather than creating two independent provider stacks:

```text
TS Host ModelRuntime
  -> language / embedding / rerank / multimodal providers
  -> validated result
  -> private Kernel commit/update RPC

Rust Kernel
  -> owns pending derivation / producer / Authority / serving semantics
  -> does not need provider SDKs as a second control plane
```

This migration is an implementation-wave concern and should not be performed as an unrelated cleanup before the Host exists.

The benefit is substantial: one provider configuration surface, one secret/readiness model, one fallback policy, one observability adapter, and one place to handle fast-changing AI SDK/provider behavior.

### 4.3 AI SDK Provider Registry is the mechanics registry

Use the SDK's `createProviderRegistry()` and `customProvider()` for provider/model IDs, aliases, preconfigured settings and provider wrappers.

Nous adds only a semantic binding layer on top:

```text
CognitiveModelRole
  steward
  projection_synthesis
  memory_formation
  memory_consolidation
  text_interpretation
  image_interpretation
  speech_transcription
  text_embedding
  text_rerank
  future persona_maintenance
  ...

ModelRoleBinding {
  role
  model_ref
  required_capabilities
  timeout_profile
  privacy/route constraints where applicable
}
```

`model_ref` resolves through AI SDK provider mechanics. A concrete provider/model string is configuration, not architecture.

Do not build an automatic score-based model marketplace/router in the first wave.

### 4.4 Provider route policy: direct providers and AI Gateway are both mechanics options

AI SDK supports dedicated provider packages and AI Gateway. AI Gateway additionally provides provider/model fallback and routing mechanics.

Nous should not make a Vercel cloud service a semantic prerequisite for a local-first cognition system.

Recommended rule:

```text
AI Gateway may be configured as a provider route
and may supply mature provider/model fallback mechanics.

Dedicated provider packages remain valid routes.
```

If a configured route supplies mature fallback (for example AI Gateway), use it. For direct providers, first-wave Nous may simply expose one selected binding and truthful `UNAVAILABLE` rather than recreating a sophisticated cross-provider router.

A small explicit ordered fallback list can be added later if real direct-provider operation requires it; it must remain a bounded role-binding policy, not a benchmark-driven autonomous router.

### 4.5 Model capability description is explicit configuration/evidence, not optimistic probing

Do not assume every provider/model supports every feature. A `ModelProfile` should record only capabilities the operator/provider adapter can substantiate:

```text
ModelProfile {
  model_ref
  input_modalities
  structured_output
  context_window?          // if known and operationally needed
  embedding_dimension?     // embedding models only
  rerank
  streaming
  provider_route
  readiness
}
```

Model capabilities may be sourced from the provider adapter/configuration and updated as provider evidence changes.

Do not make one generic `supportsEverything` boolean, and do not auto-call providers at startup merely to discover capabilities unless a concrete readiness requirement justifies the cost.

### 4.6 Internal structured model outputs use mature schema validation

Steward and projection synthesis require strongly structured outputs.

AI SDK `Output.object()` supports Zod/JSON Schema-compatible schemas and validates complete outputs. Use this rather than parsing fenced JSON or asking the model to obey ad-hoc textual formats.

For internal Host-only proposal schemas, a lightweight Zod schema can be the canonical TS definition and inferred type. It is acceptable that this differs from public Protovalidate because the responsibilities differ:

```text
Public wire contract    Protobuf + Protovalidate
Internal model proposal Zod/AI SDK Output.object
Rust Authority          Rust domain validation
```

Do not duplicate a public protobuf message merely so an LLM can output it directly. Model proposal schemas should be narrower than Authority mutation schemas.

### 4.7 Model structural validity is not semantic Authority validity

The pipeline is always:

```text
model output
-> AI SDK schema validation
-> Nous proposal semantic validation
-> referenced-object validation
-> owner-specific commit validation
-> Authority mutation (if authorized)
```

Examples of semantic checks beyond Zod:

```text
all proposed refs existed in the bounded candidate set
all refs belong to the Subject
base revisions still match
no proposal exceeds the operation's authority ceiling
no unsupported cognitive family is invented
no evidence reference is silently fabricated
```

A schema-valid LLM object remains a proposal.

### 4.8 Steward is Session-scoped but not a generic autonomous Agent

Implement:

```text
StewardEngine
```

not:

```text
ToolLoopAgent + arbitrary tools + autonomous loop
```

The first Steward has no arbitrary tool access. The Host supplies a bounded state package; the model returns a bounded structured proposal.

Trigger sources:

```text
observation admitted
turn/interaction boundary reported by consumer
focus switch
context pressure
explicit checkpoint request
explicit maintenance request
```

There is no autonomous wall-clock scheduler/self-wakeup loop in Nous Core.

### 4.9 Steward input is bounded cognitive state

Suggested input:

```text
StewardInput {
  subject_id
  session_id
  active_focus?
  session_runtime_revision
  recent_observation_refs[]
  resident_refs[]
  current_focus_working_refs[]
  new_query_candidates[]
  current_context_track_summary[]
  available_capabilities
  trigger
  budgets
}
```

Materialized text supplied to the model is selected by deterministic Host/Kernel logic before invocation. The Steward is not given unconstrained database access.

### 4.10 Steward output is a proposal batch

First-wave proposal surface should be intentionally small:

```text
StewardProposal {
  resident_admissions[]
  resident_releases[]
  focus_checkpoint_draft?
  query_suggestions[]
  context_suggestions[]
  maintenance_intents[]
}
```

Each ref-oriented field selects from input refs/candidate IDs wherever possible instead of allowing arbitrary UUID generation.

Examples:

```text
ResidentAdmissionSuggestion {
  candidate_id
  reason
  hold_hint?
}

FocusCheckpointDraft {
  summary
  selected_ref_ids[]
}

MaintenanceIntent {
  kind
  source_ref_ids[]
  reason
}
```

A MaintenanceIntent is routed to the relevant cognitive owner. It is not a Memory/Persona mutation itself.

### 4.11 Steward proposal application is deterministic and owner-bounded

After structured generation:

```text
validate candidate IDs
-> resolve to real CognitiveRefs
-> check Session revision / Focus state
-> accept/reject each runtime suggestion deterministically
-> persist accepted runtime changes through Kernel CAS
-> enqueue/execute explicitly requested owner maintenance operations only when authorized
```

If the Steward suggested an impossible ref, stale Focus, invalid hold, or disallowed operation, reject that item or the batch according to the owner contract. Do not repair semantic violations by guessing.

### 4.12 StewardGeneration is runtime lineage, not Subject identity

Record a replaceable generation descriptor:

```text
StewardGeneration {
  generation_id
  model_binding_revision
  prompt_program_revision
  proposal_schema_revision
  config_digest
  started_at
}
```

This supports:

```text
which Steward interpreted this checkpoint?
which model/config produced this suggestion?
```

Replacing the Steward generation does not create a new Subject and does not invalidate durable cognitive Authority by itself.

Model-produced Focus summaries/checkpoints retain producer lineage because they are derived runtime interpretations.

### 4.13 Deterministic Steward fallback is a real implementation path

When no eligible Steward model exists:

```text
observation/query refs
-> deterministic admission policy
-> existing meaningful-use rules
-> deterministic eviction/hold policy
-> deterministic Focus checkpoint from refs + bounded template
-> deterministic Projection Planner
```

Unavailable capabilities are reported in readiness. Session, Focus, Query, Memory, GUI administration and projection continue where their non-model mechanics are available.

Do not simulate a model with heuristic text pretending to have understood semantics. The fallback should be explicit and conservative.

### 4.14 Model-assisted Projection runs after deterministic selection

The first stage of Projection is always deterministic. Optional synthesis is a second stage:

```text
Contribution selection
-> bounded selected contributions
-> optional ProjectionSynthesizer model
-> synthesized ContextSegment(s)
-> source-ref validation
-> provenance attachment
-> final budget enforcement
```

The synthesizer may compress/reorganize selected material. It cannot introduce a new authoritative fact or source ref.

If synthesis fails or produces invalid refs/structure:

```text
fall back to deterministic rendering
```

This gives an actual `LLM-enhanced but not LLM-dependent` implementation.

### 4.15 Managed Cognition owns a cognitive context lane, not the final prompt

The consumer remains owner of its complete model invocation.

For Heptalogos:

```text
Heptalogos PromptProgram
  [system / product governance]
  [tools / behavior protocol]
  [Nous cognitive lane]
  [Reaction-specific material]
  [conversation/input tail]
```

Nous manages only `[Nous cognitive lane]`.

Nous cannot change Heptalogos behavior authority, tool policy, effect rules or final `InvocationSpec` by managing cognition.

### 4.16 Managed context is consumer-specific

The same Session may serve different consumers with materially different cognitive views. Therefore context state key is conceptually:

```text
(subject_id, session_id, focus_id?, consumer_id)
```

not simply `session_id`.

A planner, expression model and reviewer can each have a different context epoch over the same Session/Focus.

### 4.17 Managed context protocol uses RESET or APPEND, not arbitrary patch algebra

Avoid a complex general-purpose context diff language.

Use a simple two-operation contract:

```text
ContextPatchKind = RESET | APPEND
```

Request:

```text
BuildManagedContextRequest {
  ProjectionRequest
  known_cursor?
}
```

Response:

```text
ManagedContextPatch {
  track_id
  epoch_id
  base_revision?
  revision
  kind: RESET | APPEND
  segments[]
  selected_refs[]
  source_runtime_revision
  degradation[]
}
```

Semantics:

```text
RESET:
  consumer replaces the entire Nous cognitive lane
  with the supplied epoch snapshot.

APPEND:
  consumer keeps the current lane and appends
  the supplied immutable cognitive delta segments.
```

If existing context must be deleted/revised materially, start a new epoch and return `RESET`. Do not implement insert/delete/move/replace patch operations in the first wave.

### 4.18 CognitiveContextEpoch

An epoch represents a consumer-specific coherent cognitive snapshot:

```text
CognitiveContextEpoch {
  track_id
  epoch_id
  revision
  subject_id
  session_id
  focus_id?
  consumer_id
  consumer_policy_revision
  renderer_revision
  source_runtime_revision
  source_refs[]
  segment_digests[]
  created_at
}
```

Within one epoch, new independent material may append as deltas. Conditions that force a new epoch include:

```text
Focus switch
consumer policy revision
material source revision invalidates existing segment
major Persona/Relationship/Goal revision later
renderer semantic revision
budget/model context profile changes that require reshaping
context compaction/too many deltas
```

Correctness may trigger RESET immediately; cache hit rate never blocks it.

### 4.19 Managed context cursor is state synchronization, not Authority

The consumer sends a cursor from its currently applied cognitive lane. If it has no cursor or the cursor is stale/unknown, Nous returns `RESET`.

The cursor can be a typed structure (`track_id`, `epoch_id`, `revision`) or an opaque encoded equivalent. It is not a security token and does not grant cognition access.

A lost cursor should degrade to a full RESET, not require recovery machinery.

### 4.20 Context epoch persistence is intentionally lightweight

Exact append continuity across a Host crash is an optimization, not semantic Authority.

First-wave persistence should keep enough runtime metadata to diagnose/recover the active track if cheap, but it need not persist an unbounded duplicate of every rendered context string.

Valid first-wave recovery:

```text
Host restarts
-> recover Session/Focus + runtime checkpoint metadata
-> consumer sends old/unknown cursor
-> Host cannot prove append continuity
-> return RESET compiled from current Authority
```

This is correct and simpler than creating a context event store solely to preserve provider-cache continuity.

If later benchmarks show RESET-after-restart is expensive, persist bounded epoch snapshots/deltas in `RuntimeCheckpointStore` without changing public semantics.

### 4.21 Context Compiler internal pipeline

The TypeScript Host should implement a clear but non-framework-heavy compiler pipeline:

```text
ProjectionRequest
-> resolve ConsumerPolicy
-> build deterministic CognitiveProjection
-> optional synthesis
-> render ContextSegments
-> compare with consumer ContextTrack
-> choose APPEND or RESET
-> emit ManagedContextPatch
```

Suggested internal modules:

```text
cognition/projection/
  contributor.ts
  planner.ts
  renderer.ts

cognition/context/
  context-track.ts
  compiler.ts
```

Do not create a separate package for each module in the first wave.

### 4.22 Cache semantics remain semantic hints at the Nous boundary

AI SDK supports provider options at function, message and message-part levels, including provider-specific cache controls. This is useful but must remain below the semantic boundary.

Nous ContextSegments may expose only a provider-neutral hint:

```text
cache_stability:
  IMMUTABLE
  EPOCH_STABLE
  DYNAMIC
```

A model adapter may map this to provider-specific `providerOptions` when Nous invokes models internally.

When Heptalogos consumes Nous segments, its AIRuntime can make the equivalent mapping for its provider.

No Anthropic/OpenAI/Gemini cache-control object appears in the Nous public cognitive contract.

### 4.23 Cache-friendly ordering

Within a consumer cognitive lane, render the least volatile material first:

```text
stable Subject/Persona-like projection later when implemented
-> relationship/stable goal projection
-> Focus epoch snapshot
-> append-only recent cognitive deltas
```

But ordering is subordinate to semantic correctness and the consumer's prompt program.

Do not rebuild the entire cognitive lane merely because one new Memory was surfaced when an APPEND delta is semantically sufficient.

Do RESET when an old statement would become materially wrong/stale if retained.

### 4.24 Provider state/cache cannot become cognitive runtime

Provider conversation IDs, server-side threads, KV caches, prompt-cache IDs or proprietary continuation state may be used as disposable acceleration metadata later.

They cannot be the sole storage of:

```text
Session continuity
Focus
ResidentSet
CognitiveContextEpoch semantics
Memory
Persona
Relationship
Steward state required for recovery
```

If provider state disappears, Nous reconstructs cognition from its own Authority/runtime semantics.

### 4.25 Internal Model Runtime readiness

Expose per cognitive operation rather than one `llm_ready` boolean:

```text
steward                 READY | DEGRADED | UNAVAILABLE
projection_synthesis    READY | DEGRADED | UNAVAILABLE
memory_formation        ...
text_embedding          ...
text_rerank             ...
image_interpretation    ...
```

System/GUI status can aggregate these without exposing the concrete model as semantic identity. Diagnostics may include configured model/provider route and producer signature.

### 4.26 Producer signatures remain Kernel-side lineage for committed derived outputs

When a Host model result becomes a persisted derived representation, Memory revision input, embedding projection, or other producer-sensitive data, the Kernel records the existing producer/signature semantics.

The Host supplies a normalized producer descriptor including:

```text
operation
provider route class
model identity/revision when known
preprocessing identity/revision
config digest
```

The Kernel computes/validates its canonical producer signature representation.

This preserves existing projection invalidation semantics while removing provider HTTP mechanics from Rust.

### 4.27 Embedding/derivation migration pattern

Do not let the Kernel call back into the Host through reverse RPC merely to obtain model work.

Preferred control direction remains Host -> Kernel:

```text
observation commit
-> Kernel reports coverage/derivation needs
-> Host receives/queries pending needs at a current trigger
-> Host resolves ModelRuntime capability
-> generate embedding/interpretation
-> Host commits derived result to Kernel
-> Kernel publishes/invalidate serving projections
```

This can initially run synchronously or in bounded Host-triggered batches around writes/explicit maintenance. There is no need for a new autonomous scheduler.

If later throughput requires a durable worker protocol, build it from the already-explicit derivation/attempt Authority rather than adding a second generic task system.

### 4.28 Steward invocation simulation

```text
Heptalogos reports turn boundary
-> Host loads Session S, Focus F, current resident refs
-> deterministic query obtains candidate refs C1..C20
-> StewardEngine builds bounded ModelMessages
-> ModelRuntime resolves role `steward`
-> generateText(Output.object(StewardProposalSchema))
-> structured output selects C3,C7; drafts Focus summary; suggests consolidation intent
-> Host validates C3/C7 are real candidate refs
-> Kernel CAS admits C3/C7 / saves checkpoint
-> consolidation intent remains explicit pending owner operation
```

If generation fails:

```text
-> record steward readiness/failure
-> deterministic admission/checkpoint path
-> continue Session
```

### 4.29 Managed context simulation

First call:

```text
consumer has no cursor
-> BuildProjection
-> compile full cognitive lane
-> epoch E1 revision 1
-> RESET [segments A,B,C]
```

New Memory D becomes relevant without invalidating A/B/C:

```text
consumer sends E1:r1
-> compile delta D
-> E1:r2 APPEND [D]
```

Persona projection A later materially changes:

```text
old A cannot remain semantically valid
-> new epoch E2:r1
-> RESET [A2,B,C,D]
```

This keeps the public context synchronization model small and cache-friendly without sacrificing correctness.

### 4.30 Heptalogos invocation simulation

```text
Reaction owns current situation
-> Heptalogos Nous adapter requests ManagedContextPatch
-> applies RESET/APPEND only to typed `Nous cognitive lane`
-> Heptalogos PromptProgram combines:
     behavior/governance instructions
     tools
     Nous cognitive lane
     Reaction-specific context
     message tail
-> Heptalogos AIRuntime invokes behavior model
-> model output remains proposal
-> Heptalogos Behavior Authority spine decides/commits action
-> Heptalogos reports actual cognitive use back to Nous
```

Thus Managed Cognition improves continuity/caching but never becomes final behavior Prompt Authority.

### 4.31 Stage-4 decisions safe to freeze

- AI SDK 7 Core is the TypeScript model/provider substrate; Nous does not build a parallel generic provider framework.
- The TypeScript Host becomes the long-term owner of external model/provider invocation, including embedding/interpretation work; Kernel retains Authority/derivation/producer semantics.
- `createProviderRegistry/customProvider` handle provider/model mechanics; Nous adds only cognitive role bindings and readiness.
- AI Gateway is an optional mature route/fallback provider, not a semantic prerequisite.
- Internal model outputs use `Output.object` + mature schema validation; structural validity never grants Authority.
- Steward is a bounded Session-scoped `StewardEngine` using structured generation, not a general autonomous Agent loop.
- First-wave Steward has no arbitrary tool access and returns only bounded proposals over supplied refs/candidates.
- Deterministic Steward and deterministic Projection remain complete fallback paths.
- Model-assisted projection happens only after deterministic candidate selection and falls back to deterministic rendering on failure.
- Managed Cognition owns only the consumer's Nous cognitive context lane; final invocation/prompt remains consumer-owned.
- Managed context synchronization uses only `RESET` and `APPEND`; material revision/removal starts a new epoch.
- Context epochs are consumer-specific and provider-independent; provider cache controls remain adapter mechanics.
- Exact managed-context append continuity across Host restart is not a first-wave guarantee; safe RESET is sufficient.
- Provider-side threads/caches/state never become semantic cognitive runtime.
- Host -> Kernel remains the model-work control direction; do not add reverse Kernel -> Host RPC just for provider calls.

### 4.32 Stage-4 non-blocking OPEN items

- Concrete initial model/provider bindings for Steward and maintenance roles.
- Whether direct-provider ordered model fallback is needed beyond gateway/provider mechanics in the first product profile.
- Exact Zod major/pin selected at implementation time from current AI SDK compatibility.
- Exact token-estimator contract and model-specific context-window metadata.
- Whether bounded context epoch snapshots become persistent after performance evidence.
- Model access to tools for any future maintenance task; no tool access is authorized by this stage.

**Stage 4 status: CONVERGED for Pre-Spec.**

---

## Stage 5 — Configuration, persistence/recovery, artifact transport, and operational boundaries

_Status: CONVERGED for Pre-Spec._

### 5.1 Configuration has two bootstrap layers, not one global mutable object

Separate configuration by when it can exist:

```text
BootstrapConfig
  required before Core/Kernel can start

ResolvedRuntimeConfig
  typed cognitive/runtime/provider configuration
  available after bootstrap parsing/validation
```

Bootstrap fields should stay small:

```text
Core bind/listen policy
Kernel executable/path override if needed
data roots
managed vs external PostgreSQL bootstrap inputs
logging/bootstrap diagnostics
local Client discovery/auth bootstrap
config file location
```

Resolved runtime configuration covers:

```text
model/provider routes
CognitiveModelRole bindings
projection budgets
consumer policies
Steward settings
Session/ResidentSet limits
serving family enablement
artifact limits
maintenance operation limits
```

Do not pass one untyped `Config` object through every subsystem. Each owner receives a typed snapshot of the fields it owns.

### 5.2 First-wave config loading should be explicit TOML + explicit environment overrides

The current product already uses TOML and explicit environment override for PostgreSQL. Preserve the operator-facing format unless a concrete reason requires change.

For the TypeScript Host, use a maintained TOML parser such as `smol-toml` and the existing Host schema-validation choice rather than creating a parser or adopting a large config discovery framework.

Reasoning:

- `smol-toml` 1.8.x is current, dependency-light, actively maintained and used broadly;
- `c12` is capable and widely adopted, but its current v4 line is beta and its discovery/extends/watch/remote-layer machinery is not required by the first-wave product;
- Nous has an explicit config path and controlled data roots, so upward directory discovery and dynamic config execution would add ambiguity rather than value.

First-wave precedence:

```text
compiled typed defaults
< explicit nous.toml
< explicit environment-variable allowlist
< explicit CLI/bootstrap override where a real startup need exists
```

Do not auto-map arbitrary environment variables into config fields.

### 5.3 Do not implement config hot reload in the first wave

A configuration value being configurable does not imply live mutability.

First-wave rule:

```text
configuration change
-> restart Core
```

unless a specific runtime operation already has safe dynamic semantics.

This avoids:

```text
file watchers
config event bus
partial reconfiguration coordinator
rollback machinery
hot provider rebinding races
```

A later managed ConfigurationService can introduce revisioned activation when a current GUI/operator requirement justifies it.

### 5.4 Configuration visibility/editability metadata is semantic, even before GUI config editing

Keep the established principle:

```text
configuration existence
!= visibility
!= editability
```

Internally classify fields:

```text
ConfigDescriptor {
  key
  owner
  sensitivity
  visibility
  editability
  restart_required
}
```

`SystemService.GetEffectiveConfig` can return a redacted projection:

```text
key
safe effective value or redacted marker
source: DEFAULT | FILE | ENV | BOOTSTRAP_OVERRIDE
owner
visibility
editability
restart_required
```

Do not expose internal tuning parameters merely because the Core can enumerate them.

### 5.5 Secrets stay out of ordinary effective config

Provider API keys and future sensitive credentials must never appear in `GetEffectiveConfig` output or normal logs.

First-wave provider credentials can use the mature provider packages' environment/config inputs with explicit secret-valued fields redacted before diagnostics.

Do not build a full SecretService/keyring lifecycle solely for this architecture wave. A future GUI requirement to create/rotate credentials is a separate responsibility and should select a mature OS/secret-store mechanism then.

### 5.6 Data ownership by process

The new process boundary should be explicit:

```text
TypeScript Core owns:
  public endpoint
  Client contract implementation
  cognitive orchestration
  config resolution
  model/provider transport
  consumer context runtime

Rust Kernel owns:
  PostgreSQL lifecycle/access
  cognitive Authority schema
  object repository
  serving/index files
  runtime checkpoint persistence
  private RPC
```

The Host must never open the Kernel PostgreSQL directly for convenience.

The Kernel remains the only writer to cognitive Authority and serving ownership state.

### 5.7 Preserve managed and external PostgreSQL bootstrap modes

The current executable already supports managed embedded PostgreSQL and an external URL override. The new Kernel can retain this useful deployment distinction:

```text
managed:
  Kernel owns PostgreSQL child lifecycle and data root

external:
  Kernel connects to explicitly configured PostgreSQL
  and does not pretend to own the server process
```

The public Client/Host contract is identical in both modes.

Do not expose SQL connection strings or DB lifecycle to GUI domain APIs.

### 5.8 PRE_PRODUCTION schema evolution remains direct

The new runtime checkpoint tables, Memory head revision/ETag mechanics, and protocol support may rewrite the current development schema directly.

Do not create:

```text
legacy migration readers
old REST compatibility schema
dual Session formats
v0->v1 runtime adapters
```

unless the machine-readable/current project authority declares a real compatibility obligation.

Development data can be rebuilt for this architectural rebase.

### 5.9 Kernel process liveness uses inherited OS pipe semantics

The Core needs a cheap way to ensure an abruptly dead Node parent does not leave an indefinitely orphaned Kernel.

Use normal inherited process/pipe semantics:

```text
Node spawns Kernel with a dedicated inherited stdin/liveness pipe
Node keeps the write end open
Kernel watches for EOF
parent exits/crashes -> OS closes pipe -> Kernel begins graceful shutdown
```

Readiness/diagnostic output remains on a separate bounded stdout/stderr channel.

This is not a custom IPC protocol; it is an OS process-lifetime primitive implemented with Node `child_process` and Tokio I/O.

Do not add a local daemon/service supervisor merely for this first-wave parent-child topology.

### 5.10 Kernel terminal failure policy is simple in the first wave

If the Kernel exits unexpectedly:

```text
Core marks Kernel unavailable
public mutation/query operations fail UNAVAILABLE
System status remains able to report the terminal reason if Core is still alive
```

The first wave does not require an autonomous crash-loop supervisor.

A complete Core restart is a valid first-order recovery path. If later operation proves automatic restart materially valuable, add bounded restart policy around the same child-process adapter without changing cognitive semantics.

### 5.11 gRPC health is the post-spawn readiness proof

Startup stdout endpoint publication is only bootstrap discovery. It is not readiness Authority.

After endpoint discovery:

```text
Host creates private gRPC transport
-> standard gRPC health check
-> required Kernel subsystems report ready
-> Core public readiness transitions READY/DEGRADED
```

Use Tonic's standard health support rather than a custom ping service where possible.

Detailed cognitive capability readiness remains `SystemService.GetStatus/GetCapabilities`, because gRPC Health answers service availability, not all cognitive semantics.

### 5.12 Local public endpoint requires a bounded local-client security seam

Loopback binding reduces exposure but does not by itself make a local HTTP API trusted: browsers and unrelated local processes can target localhost.

First-wave public Core remains loopback-only and should require a local bearer credential on management/cognition requests.

A low-cost local discovery mechanism is sufficient:

```text
Core boot
-> create ephemeral local Client credential
-> publish endpoint + credential into a protected runtime discovery record
-> official Node/Desktop Client can resolve it when given the instance data root
```

Browser GUI hosted by the Core can use same-origin/session bootstrap later. An independently hosted browser GUI requires an explicit secure connection/bootstrap design and is not silently granted access.

Do not implement remote OAuth/OIDC/multi-user RBAC in this wave.

### 5.13 Public CORS/origin behavior is deny-by-default

The first-wave public endpoint:

```text
binds loopback
requires credential
has no wildcard CORS
```

If the future browser Presentation is served from the same Core origin, no broad cross-origin policy is required.

When a separate web origin becomes a real consumer, add exact configured origins and the corresponding authentication/CSRF model rather than enabling `*` preemptively.

### 5.14 Artifact control plane and byte data plane are separated

Protobuf/Connect is appropriate for artifact metadata and semantic operations, but large media should not be encoded as giant unary protobuf `bytes` fields.

Use:

```text
Control plane:
  Connect RPC
  artifact metadata / observation / materialization / evidence

Byte data plane:
  ordinary HTTP streaming at the Core
  private gRPC streaming Core <-> Kernel
```

This preserves one public product endpoint while using transport mechanics suited to each payload.

### 5.15 First-wave browser-compatible upload uses streaming multipart

Browser clients cannot rely on every RPC streaming cardinality or Connect-Query streaming support. Use a conventional streaming upload endpoint behind the official Client helper.

Recommended implementation:

```text
GUI/Client
  -> HTTP multipart stream
     metadata part + one file/content part
  -> Fastify + @fastify/multipart
  -> bounded stream forwarding
  -> private Tonic client-streaming RPC
  -> Kernel ObjectStore/OpenDAL
```

`@fastify/multipart` already provides streamed file handling and explicit file-size limits. Do not implement multipart parsing.

The Client exposes `material.upload(...)`; normal consumers do not construct multipart fields manually.

### 5.16 Upload limits are enforced at every material boundary

At minimum:

```text
public HTTP body/file limit
Host stream forwarding limit
Kernel declared max_upload_bytes
object-store write result validation
```

A limit failure returns a structured resource-exhaustion/invalid request result before an Authority observation is formed.

Do not accept an unbounded stream simply because OpenDAL can store it.

### 5.17 Download uses standard HTTP Range and OpenDAL range reads

OpenDAL 0.58.x has range-based read APIs and `BytesRange`. Use those capabilities to implement standard HTTP byte ranges through the Core.

Flow:

```text
Client GET artifact-content with optional Range
-> Core validates Subject/artifact access
-> private Kernel range-read/server-streaming RPC
-> OpenDAL Reader range
-> HTTP 200 or 206 with standard Content-Range/length headers
```

Do not create a Nous-specific seek/chunk protocol.

This directly supports media viewers, large documents and future GUI partial access.

### 5.18 No resumable upload protocol until evidence requires it

A failed multi-GB upload may eventually justify resumability, but the first wave should not invent:

```text
upload session state machine
chunk indexes
custom retransmission
dedicated resume manifest
```

If real product evidence requires resumable upload, adopt a mature protocol/library such as tus or an object-store-native multipart mechanism appropriate to the deployment.

The public `@nous-wave/client` can preserve a high-level upload API so this mechanic can change later.

### 5.19 Artifact byte transport does not become a second management API

The raw HTTP surface is narrow:

```text
upload artifact bytes
read artifact bytes/ranges
possibly derived payload bytes later
```

Memory, Session, Query, topology, configuration, status and cognitive mutations remain Connect RPC.

Do not reconstruct the old hand-written REST API beside Connect merely because Fastify is present.

### 5.20 Request correlation uses standard propagation, not a custom lineage protocol

Cross-process diagnostics need correlation, but a new telemetry framework is not required to begin.

Use:

```text
public RPC/request id
W3C trace context where available
Connect interceptor
private gRPC metadata
Rust tracing span fields
TypeScript structured logger context
```

Pino/Fastify logging and Rust `tracing` are mature mechanics. Full OpenTelemetry export can be added when there is a real collector/observability requirement.

Cognitive provenance/producer lineage remains separate from operational trace IDs.

### 5.21 RuntimeCheckpointStore recovery boundaries

Checkpoint persistence guarantees:

```text
committed Focus state is recoverable
committed checkpoint revision is recoverable
CAS conflicts are detectable
```

It does not guarantee:

```text
exact JavaScript heap restoration
provider stream continuation
exact prompt-cache continuation
in-flight model request resurrection
```

On Core crash during a model invocation, the invocation can be abandoned. After restart, reconstruct from Kernel runtime/Authority and retry only if the higher-level current operation still requires it.

This is appropriate because model generation is proposal work, not canonical Authority.

### 5.22 Authority mutation crash boundary remains in the Kernel

A Host-side operation may have uncertain transport outcome if the connection fails after the Kernel commits.

Therefore:

```text
Client mutation
-> Host
-> Kernel transaction
-> response
```

If Host/transport dies after commit and before the client sees the response, the caller refetches the resource head/ETag.

Do not create cross-process distributed transactions between Host model work and Kernel Authority. Model work finishes first; commit is a separate bounded Kernel transaction.

### 5.23 Serving projections remain rebuildable and family-scoped

The Core status API should expose semantic serving family readiness, while Kernel continues to own:

```text
exact
lexical
dense
topology
relevant synopsis/derived projections
```

Provider/model change invalidates only affected families based on producer/embedding-space identity.

Host restart alone does not invalidate Kernel serving projections.

### 5.24 Configuration/restart simulation

```text
operator changes steward model binding in nous.toml
-> restart Core
-> Host parses TOML
-> structural validation
-> resolves explicit env secret references/provider inputs
-> starts/connects Kernel
-> builds AI SDK Provider Registry
-> resolves ModelRoleBinding `steward`
-> System status reports new binding readiness
```

No watcher, dynamic rollback or hidden old binding remains.

### 5.25 Artifact GUI simulation

```text
GUI opens Memory evidence
-> GetMemory returns EvidenceRef
-> MaterialService resolves SourceRegion/Artifact metadata
-> Client requests artifact range
-> Core sends Range to Kernel stream RPC
-> OpenDAL returns selected bytes
-> GUI renders preview
```

Upload:

```text
GUI selects 4 GB video
-> @nous-wave/client material.upload()
-> multipart body streams; never buffers 4 GB in Node
-> Fastify enforces configured maximum
-> Host forwards chunks through private client-streaming gRPC
-> Kernel writes OpenDAL content stream
-> only after successful stored material does semantic Observation/Artifact commit complete according to owner operation
```

### 5.26 Parent crash simulation

```text
Core spawns Kernel
Kernel watches liveness pipe
Core process is killed
-> liveness pipe EOF
-> Kernel gracefully closes Authority pool / managed PostgreSQL according to existing lifecycle
```

If the machine kills both processes or power is lost, PostgreSQL/object-store crash semantics own recovery; Nous does not create a second journal merely to mimic them.

### 5.27 Stage-5 decisions safe to freeze

- Separate small BootstrapConfig from owner-specific resolved runtime configuration.
- First-wave Host config is explicit TOML + explicit environment/CLI overrides; use a mature TOML parser rather than a broad dynamic configuration framework.
- No config hot reload in the first wave; restart is the default activation boundary.
- `GetEffectiveConfig` is a redacted projection with source/owner/visibility/editability/restart metadata.
- Secrets are excluded/redacted; no full SecretService is created by this wave.
- Kernel alone owns PostgreSQL, object store, serving projections and runtime checkpoint persistence; Host never opens Kernel storage directly.
- Managed/external PostgreSQL modes remain useful physical deployment options.
- Host/Kernel child lifetime uses an inherited liveness pipe; no daemon/supervisor framework is introduced.
- Kernel unexpected exit is a truthful terminal/unavailable condition; whole-Core restart is sufficient first-wave recovery.
- Standard gRPC Health is used for post-spawn Kernel service readiness.
- Public Core remains loopback-only with a bounded local-client credential seam and deny-by-default cross-origin behavior.
- Connect/Protobuf remains the control plane; large artifact bytes use Fastify HTTP streaming + private Tonic streaming.
- Use `@fastify/multipart` for streaming upload and OpenDAL standard range reads for download.
- No resumable-upload protocol until real evidence requires it.
- Crash/recovery semantics reconstruct runtime from Authority/checkpoints; exact model/provider/cache continuation is not required.

### 5.28 Stage-5 non-blocking OPEN items

- Exact protected local discovery-record path and credential lifecycle on each platform.
- Whether the final browser GUI is served by Core, Desktop shell, or a separately hosted origin.
- Mature secret-store route when GUI credential management becomes current scope.
- Resumable large-upload protocol after product evidence.
- Full OpenTelemetry exporter/collector integration.
- Automatic bounded Kernel restart if operation later proves it worthwhile.
- Backup/export/restore product semantics; not required to define the new Host/Kernel boundary.

**Stage 5 status: CONVERGED for Pre-Spec.**

---

## Stage 6 — Consumer integration boundary: Heptalogos as a reference application

_Status: CONVERGED for Pre-Spec._

> **Authority correction:** Heptalogos is itself under active architectural refactoring. Its current contracts, package boundaries, state names, and invocation flow are **consumer-reference material only**. They may expose requirements and provide acceptance scenarios, but they MUST NOT define Nous Wave public protocol authority. Nous public contracts are derived from Nous cognition semantics, the independent Core+Client/GUI product requirement, and consumer-agnostic integration needs. When Heptalogos changes, its adapter changes first; Nous changes only when an independently justified cognition requirement changes.


### 6.1 Heptalogos/Nous boundary is a coordination boundary, not shared Authority

The integration must preserve two independently authoritative systems:

```text
Heptalogos
  Messaging Authority
  Subject lifecycle / agency Authority
  Reaction / behavior / commit / effect Authority

Nous Wave
  cognitive Subject profile
  cognitive material/evidence
  Memory and future Persona/Relationship/etc. Authority
  cognitive Session/Focus/runtime state
  cognitive serving projections
```

A shared `SubjectId` is an identity correlation, not a shared transaction, shared database, or permission to mutate another owner's canonical state.

Current Heptalogos documents are useful evidence that an application may separately own lifecycle/agency while excluding advanced cognition. That is a compatible consumer shape, not an interface authority. Nous therefore exposes consumer-agnostic cognition operations; a Heptalogos adapter maps its current concepts to those operations without importing Heptalogos state names or control flow into the Nous public schema.

### 6.2 Subject provisioning must be reconciled, never distributed-transactional

The first integration must not attempt:

```text
Heptalogos transaction
+ Nous transaction
= one distributed transaction
```

Instead:

```text
Heptalogos SubjectRecord exists canonically
        ↓
Nous adapter checks cognitive Subject
        ↓
GetSubject(subject_id)
  ├─ exists -> validate expected identity/binding
  └─ NOT_FOUND -> CreateSubject(subject_id, CharacterSeed/bootstrap cognition)
        ↓
if response is lost -> GetSubject and reconcile
```

No 2PC, Saga framework, cross-database rollback or shadow Subject lifecycle is justified.

For first-wave public API semantics, `CreateSubject` should accept a caller-supplied `SubjectId`. The caller may use an optional standard `request_id` for retry idempotency. A repeated call with the same request id returns the prior successful/equivalent result. If the Subject already exists independently of that request id, the client reads it and validates the expected binding rather than silently replacing it.

### 6.3 Character Seed ownership after the boundary split

The clean ownership is:

```text
Heptalogos product/bootstrap input
    may supply initial Character Seed
        ↓
Nous Subject Core
    owns retained Character Seed lineage as cognition
```

Heptalogos must not maintain an independently evolving canonical Persona/Seed copy after provisioning.

The basic Heptalogos Subject route remains allowed to operate without Nous. In that degraded/basic mode it uses only the minimal product/Subject identity and behavior constraints required by its current Subject contract; it does not fabricate the missing evolved cognitive state from an old seed copy.

Stopping the Heptalogos Subject also must not stop/delete the Nous cognitive Subject. Cognitive state remains independently inspectable/manageable through the Nous Client/GUI even when agency is STOPPED.

### 6.4 Any application consumer, including Heptalogos, consumes only the official Nous Client

Heptalogos must not:

```text
import apps/nous-core internals
import Rust crates
call private Kernel gRPC
open Nous PostgreSQL
inspect serving index files
reuse generated private kernel DTOs
```

Its integration dependency is only:

```text
@nous-wave/client
```

A Heptalogos adapter MicroSystem converts Heptalogos semantic data into public Nous Client operations and converts results into Heptalogos Context/Activity artifacts.

This keeps Nous independently usable by GUI/CLI/other consumers and prevents the two repositories from becoming one physical codebase again.

### 6.5 Heptalogos adapter ports are consumer-local design, not Nous protocol design

Do not expose the complete Nous management API as one giant Heptalogos capability.

The current Heptalogos reference application benefits from three narrow local ports:

```text
CognitiveObservationPort
  canonical fact/material -> cognition observation

CognitiveProjectionPort
  situation/consumer need -> CognitiveProjection

CognitiveFeedbackPort
  actual downstream use -> cognitive use feedback
```

All three may be implemented by one `NousCognitionProvider` adapter internally, but Reaction/Subject code depends only on the port it needs.

Whether these ports are optional Capability-level dependencies is a Heptalogos-local choice. Nous does not encode `Basic Subject`, `Reaction`, or Heptalogos readiness states in its public contract.

No GUI/admin operations are re-exported through these Heptalogos ports; GUI management talks to Nous Core through the official Nous Client directly.

### 6.6 Reference-application observation delivery must not redefine Nous observation semantics

A canonical inbound message is owned by Heptalogos Messaging first:

```text
MessageFact canonical commit
        ├─ existing Reaction WorkItem
        └─ cognitive-observation obligation when Nous integration is enabled
```

For the current Heptalogos reference flow, cognitive observation should not be a precondition for beginning its behavior path.

Reason:

- Nous can be degraded/unavailable while Basic Subject behavior remains usable;
- MessageFact is already the world/application Authority;
- the current message is available directly to the Reaction ContextProjection;
- forcing synchronous observation before Reaction would turn optional advanced cognition into a hidden hard dependency.

Therefore the first-wave Heptalogos integration should use the existing durable WorkItem mechanics for enabled observation delivery, but as a distinct obligation from Reaction behavior.

No new queue/workflow framework is created.

### 6.7 Observation retry exposes a current Kernel gap

Current Nous `record_observation()` always allocates a fresh `OccurrenceId` and inserts a new `observation_occurrences` row. `external_object_ref` is validated but is not an idempotency binding.

This is correct for repeated real-world encounters with the same external object, so `external_object_ref` itself must **not** be turned into a universal UNIQUE key.

Instead the public mutating request should adopt the mature request-identification pattern:

```text
RecordObservationRequest
  request_id?: UUID
  observation: ...
```

If `request_id` is present:

```text
same request_id + same accepted canonical request
-> return prior equivalent AcceptedObservation
-> no second Occurrence

same request_id + different canonical request
-> idempotency conflict
```

The Kernel persists a bounded request-id binding for the operation. The public Client may generate a UUID automatically where a safe mutation helper documents that behavior; durable callers such as Heptalogos should persist/derive the request id from their WorkItem attempt identity rather than generating a new one on every retry.

This uses the standard request-id/idempotency concept rather than creating an `observation_key` protocol unique to Nous.

### 6.8 A consumer may supply current Situation cues before durable observation arrives

A projection request cannot assume a separate asynchronous observation delivery has already completed.

The correct sequence is:

```text
MessageFact committed
-> Reaction builds SituationDescriptor from current canonical facts
-> optional CognitiveProjectionPort.BuildProjection(...)

in parallel / eventually:
MessageFact
-> CognitiveObservationPort.RecordObservation(...)
-> Nous durable ObservationOccurrence
```

`SituationDescriptor` therefore carries current host facts/opaque refs needed to interpret the immediate situation, while the actual canonical content remains owned by Heptalogos.

Nous may use the situation as a query cue without claiming it has already been durably internalized.

### 6.9 Session policy: do not define any host Conversation == Nous Session

Architecture remains:

```text
Conversation != CognitiveSession
Reaction != CognitiveSession
Reaction != Focus
```

For the current Heptalogos reference application, a simple adapter default may be:

```text
one long-lived Nous CognitiveSession
per Heptalogos installation/Subject integration binding
```

Focus then carries foreground task/topic continuity.

This is an integration DEFAULT, not a permanent semantic equivalence. Future multi-conversation/multi-agent consumers may open additional Sessions.

To make restart recovery generic, public Session metadata should expose a typed optional opaque `client_ref`/`owner_ref` rather than requiring consumers to search arbitrary JSON metadata. Heptalogos uses a stable value derived from its Installation identity and adapter role, lists open Sessions for that ref on reconnect, and resumes the current one or creates a new one.

Do not persist a duplicate copy of the entire Session state in Heptalogos.

### 6.10 Focus selection first wave

Do not add LLM topic segmentation merely to prove Focus exists.

First-wave deterministic policy:

```text
current open Session
  -> one foreground Focus
  -> explicit switch/suspend/resume operations exist
```

For the current single-conversation Subject Chat, the adapter reuses the foreground Focus unless the integration explicitly requests a switch. Focus metadata may carry the current conversation/activity refs as opaque context.

The Steward may later suggest Focus transitions, but it does not self-grant Focus Authority in the first implementation wave.

This gives a real Focus lifecycle without inventing an attention classifier before there is evidence for one.

### 6.11 Consumer integration uses a cognitive projection contribution, not final prompt ownership

A consumer may own its own context/prompt compilation. The current Heptalogos repository demonstrates one such design. Conceptually:

```text
Heptalogos base ContextProjection
        +
Nous CognitiveProjection facet
        ↓
Heptalogos PromptProgram
        ↓
InvocationSpec
        ↓
AIRuntime
```

The Nous projection is therefore a typed advanced-cognition facet/contribution.

It may contain:

```text
projection_id / generation
subject / session / focus
consumer role
typed contribution segments
source CognitiveRefs
authority/evidence/provenance summaries
freshness
budget/degradation information
managed-context patch metadata when requested
```

It must not contain Heptalogos `DecisionCommit`, tool authorization, effect permission, final system policy or a complete final InvocationSpec.

### 6.12 Consumer role is a stable Nous semantic input

Any consumer sends a typed/opaque stable consumer identity + bounded needs/profile semantics rather than arbitrary prompt prose claiming a role.

Examples:

```text
heptalogos.subject.primary
heptalogos.subject.expression
future heptalogos.planner
future heptalogos.operator
```

The request includes explicit `ProjectionNeeds` rather than relying only on the consumer string.

Example policy:

```text
subject.primary
  memory: strong
  persona: normal/strong when available
  relationship: normal when available
  goals/commitments: strong when available

subject.expression
  persona/style: strong when available
  relationship: bounded
  raw retrieval/evidence breadth: weak

operator
  persona: forbidden
  relationship: forbidden
  system/current-state resources: allowed as separately authorized
```

This keeps `Subject != Agent != Persona` operational rather than documentary.

### 6.13 Managed Cognition boundary with any consumer

Managed Cognition does not let Nous rewrite a consumer's whole model input or execution context.

Nous owns only a cognitive lane:

```text
Heptalogos InvocationSpec
  [system/product/behavior authority blocks]  <- Heptalogos
  [tools/capability rules]                    <- Heptalogos
  [Nous cognitive lane]                       <- Nous managed RESET/APPEND patches
  [Reaction/current messaging context]        <- Heptalogos
  [other consumer-specific content]           <- Heptalogos
```

Nous may return:

```text
RESET cognitive epoch snapshot
APPEND cognitive delta
```

for its lane.

Heptalogos owns application of the patch and the final InvocationSpec. If it cannot support Managed Cognition, it can consume the same projection in Assistive mode as ordinary typed context.

### 6.14 Use feedback must describe actual downstream use

Current Nous runtime already distinguishes `Surfaced`, `Inspected`, `Selected`, `Referenced`, `ActedOn`, `Corroborated`, `Corrected`, `Pinned`, `Rejected` and only treats stronger kinds as meaningful resident use.

For the new public semantics, `SELECTED` and `EXPOSED` must be distinct rather than renamed into one another.

Recommended semantic ladder:

```text
SURFACED
  a candidate was returned/available to projection planning

SELECTED
  Nous selected the candidate into a CognitiveProjection
  (Nous can record this internally; consumer need not report it)

EXPOSED
  consumer actually inserted/materialized it into a model-visible or human-visible context

REFERENCED
  downstream model/human/logic explicitly referred to it

ACTED_ON
  committed downstream behavior/operation explicitly depended on it

CORROBORATED / CORRECTED / PINNED / REJECTED
  explicit semantic feedback
```

`SURFACED`, `SELECTED`, and `EXPOSED` remain non-reinforcing by default. A consumer must not report `REFERENCED` merely because a contribution was present in its context. If it cannot prove use, it stops at `EXPOSED`.

### 6.15 Feedback failure cannot roll back behavior Authority

Once Heptalogos has committed a `DecisionCommit`:

```text
Nous ReportUse fails
```

must not invalidate or roll back the decision.

Feedback is a follow-up cognitive observation/use signal.

Implementation policy:

- lightweight non-meaningful feedback may be best-effort;
- meaningful feedback may be sent through an independent existing WorkItem when product evidence justifies reliable delivery;
- every mutating feedback request can use standard `request_id` idempotency;
- no new feedback queue or distributed transaction is created.

### 6.16 Failure simulation: Nous unavailable before Reaction

```text
MessageFact committed
-> Reaction WorkItem starts
-> CognitiveProjection capability is UNAVAILABLE
-> Heptalogos ContextProjection records advanced-cognition degradation
-> subject.primary still runs with basic ContextProjection
-> BehaviorIntent / Review / DecisionCommit continue normally
```

This demonstrates that one current application can remain operational with cognition degraded; it does not make that Heptalogos readiness rule part of Nous semantics.

The separate cognitive-observation WorkItem remains retryable/blocked according to its own dependency state; it does not block the behavior WorkItem.

### 6.17 Failure simulation: Nous fails after returning a projection

```text
BuildProjection returns projection P
-> Heptalogos records P's projection/generation/ref evidence in Reaction workspace
-> Nous Core later dies
-> Heptalogos primary invocation may still use already materialized P if the Reaction's own freshness/fence remains valid
-> behavior Review still uses Heptalogos authority fences
```

A Nous crash does not retroactively erase the fact that projection P was an input artifact.

However no later claim is made that P is the newest cognition. A later Reaction must re-query when Nous returns.

### 6.18 Failure simulation: Message observation response lost after Kernel commit

```text
Heptalogos CognitiveObservation WorkItem
-> RecordObservation(request_id=R)
-> Kernel commits Occurrence
-> response lost
-> WorkItem retries R
-> request-id binding returns prior/equivalent AcceptedObservation
```

No duplicate Occurrence and no false rollback.

This scenario is a mandatory integration acceptance test because the current implementation does not yet provide it.

### 6.19 Failure simulation: Heptalogos restarts

```text
Heptalogos restarts
-> SubjectId/DesiredState recovered from Heptalogos Authority
-> Nous adapter reconnects through @nous-wave/client
-> GetSubject(shared SubjectId)
-> find/resume open cognitive Session by stable client_ref
-> obtain current Focus/checkpoint/epoch from Nous
-> Reaction processing resumes from Heptalogos WorkItem/commit Authority
```

No model context itself is treated as required recovery state.

### 6.20 Failure simulation: Nous Core restarts while Kernel state survives

```text
Nous Host dies
-> Kernel follows configured parent-liveness lifecycle and exits cleanly/fail-stops
-> Nous Core restart
-> Kernel restarts on same Authority store
-> Host restores Session/Focus/checkpoint semantic runtime
-> model/provider registry is rebuilt
-> context cache/KV state is not required
```

Heptalogos sees temporary Capability unavailability/degradation, not a new Subject identity.

### 6.21 Management boundary remains directly Nous-owned

Heptalogos Management must not become the only way to administer cognition.

Future GUI path remains:

```text
GUI
-> @nous-wave/client
-> Nous Core public Management/Cognition contracts
```

This allows cognition inspection/editing even when Heptalogos is stopped or absent.

Heptalogos may project a small readiness/link surface for operator convenience, but it must not proxy every Memory/Persona/Artifact operation and thereby recreate a second management API.

### 6.22 Stage-6 decisions safe to freeze in Nous Wave

- External identity correlation may reuse a stable Subject UUID, but Nous does not depend on a Heptalogos-specific Subject contract and correlation never merges Authority domains.
- Heptalogos Subject lifecycle remains Heptalogos-owned; Nous cognitive Subject remains independently manageable.
- Subject provisioning is reconcile/create/read, never distributed transaction.
- Heptalogos consumes only `@nous-wave/client`; private Kernel API is never a Heptalogos dependency.
- Nous exposes consumer-agnostic Observation, Projection and Feedback semantics. A Heptalogos adapter may map them into narrow local ports; those local port names/shapes are not Nous protocol authority.
- Enabled canonical observation delivery is a separate durable obligation from Reaction behavior and must not block Basic Subject latency/readiness.
- Public mutating operations that require safe retry use standard optional `request_id`; RecordObservation requires it for the Heptalogos durable path.
- `external_object_ref` is not abused as a universal idempotency key.
- Conversation, Reaction, CognitiveSession and Focus remain distinct.
- A Heptalogos adapter may initially use one long-lived Session and deterministic foreground Focus; this is consumer-local policy, not Nous architecture identity.
- Nous returns cognitive projections/context patches only. Every consumer retains ownership of its final invocation/execution/behavior authority, whatever those concepts are called in that application.
- Managed Cognition controls only the Nous cognitive lane.
- Public/use-event semantics distinguish `SELECTED` (Nous chose it) from `EXPOSED` (consumer actually presented it); neither exposure nor selection alone is reinforcement.
- Feedback failure never rolls back committed Heptalogos behavior.
- Nous GUI management remains directly Nous-owned, not proxied through Heptalogos.

### 6.23 Stage-6 non-blocking OPEN items

- Exact shipping owner that starts `nous-core` in a combined Heptalogos distribution. The semantic rule is only that Heptalogos talks to the public Client, never Kernel/internal Host code.
- Future multi-conversation Session policy.
- Steward-assisted topic/Focus splitting policy.
- Whether future advanced product profiles make CognitiveProjection a hard readiness dependency; current Basic Subject does not.
- Reliable delivery policy for each feedback class after real quality evidence.
- Cross-product backup/restore coordination; does not block the interaction boundary.

**Stage 6 status: CONVERGED for Pre-Spec after consumer-authority correction.**

---

## Stage 7 — Concrete implementation topology, protocol files, and migration path

_Status: CONVERGED for Pre-Spec._

### 7.1 Stage-7 objective

This stage converts the converged semantic design into a concrete repository and code-migration topology. The target is not a fresh rewrite of the cognition engine. The target is to preserve the already-correct Rust Authority/retrieval/material/serving implementation while moving high-change orchestration, model/provider mechanics, public product API, and Client ownership to TypeScript.

The controlling implementation rule is:

```text
move ownership at an explicit seam
!=
rewrite working domain mechanics merely because the process boundary changed
```

### 7.2 Polyglot workspace root

The repository should become one product workspace with two language toolchains, not two loosely coordinated repositories inside one repository.

Recommended root additions:

```text
package.json
pnpm-workspace.yaml
tsconfig.json
proto/
buf.yaml
buf.gen.yaml
```

Existing Cargo workspace remains authoritative for Rust crates.

The first-wave TypeScript workspace should remain deliberately small:

```text
apps/nous-core
packages/client
packages/protocol-ts
```

Do **not** introduce Nx, Turborepo, Rush, Lage, or another workspace orchestrator. There are too few JS projects to justify a second build graph. `pnpm -r`, normal package scripts, and the existing root `justfile` are enough.

`just` remains the human/Agent task entrypoint and orchestrates both ecosystems.

### 7.3 TypeScript toolchain

First-wave baseline:

```text
Node.js      24 LTS
TypeScript   7.x
pnpm         11.x
format       Prettier
lint         Oxlint
unit/test    Vitest
```

Why this route:

- Node 24 is LTS today; Node 26 remains Current until its October 2026 LTS transition.
- TypeScript 7 is production-ready and uses the normal `typescript` package.
- `typescript-eslint`'s currently declared support range stops below TypeScript 7, so it should not be made the primary TS7 gate merely because it is familiar.
- Oxlint's type-aware path is built around typescript-go / TypeScript 7 and is already used successfully in Heptalogos.
- Biome's documented TS language support currently trails the TS7 line, so adopting it now would add another migration immediately after the architecture rebase.

This is not a permanent rejection of ESLint or Biome; it is a current evidence-based routing decision.

### 7.4 Root verification composition

The existing Rust `just verify` should evolve rather than be replaced.

Conceptual task graph:

```text
just fmt
  -> cargo fmt
  -> pnpm format
  -> buf format -w

just check
  -> cargo check workspace
  -> pnpm typecheck
  -> buf lint

just lint
  -> cargo clippy
  -> pnpm lint

just test
  -> cargo test
  -> pnpm test

just generate
  -> buf generate
  -> generated-source cleanliness check

just verify
  -> format check
  -> code generation check
  -> Rust check/lint/test/deny/shear/source-shape
  -> TS typecheck/lint/test
  -> Buf lint
```

Do not introduce a bespoke repository task runner.

### 7.5 Protobuf source tree

Keep public and private namespaces separate from day one.

Recommended layout:

```text
proto/
  buf.yaml
  public/
    nous/wave/v1alpha1/
      common.proto
      subject.proto
      cognition.proto
      session.proto
      memory.proto
      material.proto
      resource.proto
      topology.proto
      system.proto
  kernel/
    nous/wave/kernel/v1alpha1/
      common.proto
      subject.proto
      runtime.proto
      memory.proto
      material.proto
      query.proto
      resource.proto
      topology.proto
      serving.proto
      checkpoint.proto
```

Physical directory names `public/` and `kernel/` are organization only. Protobuf package identity is the canonical namespace.

Avoid one giant `nous.proto`: it would recreate the current large-module problem at the protocol level.

Avoid dozens of tiny files based on individual RPC methods. Split by semantic owner.

### 7.6 Public contract shape

Public services should be grouped by domain owner, approximately:

```text
SubjectService
CognitionService
MemoryService
MaterialService
ResourceService
TopologyService
SystemService
```

`Session` and `Focus` may remain under `CognitionService` initially unless concrete client ergonomics prove a separate `SessionService` useful. Service count is not a quality metric.

The public schema must model stable semantics, not current Rust structs. In particular:

```text
NO SQL row shapes
NO Tantivy/USearch implementation fields
NO Rust enum naming leakage when semantics differ
NO internal serving artifact path
NO provider SDK object
NO model prompt cache representation
```

### 7.7 Private Kernel contract shape

Kernel RPCs should map to coarse application/domain operations, not repository-function mirroring.

Good:

```text
RecordObservation
GetCognitiveSource
RunCognitiveQuery
CommitMemoryFormationProposal
ReviseMemory
Build/PrepareServingProjection
Get/PutRuntimeCheckpoint
```

Bad:

```text
InsertMemoryRow
SelectMemoryRevisionRows
SearchTantivySegment
CallUsearch
SetResidentRefSqlState
```

This keeps Tonic and Protobuf at the application adapter boundary.

### 7.8 Code generation route

Use one Buf-controlled generation workflow across languages.

TypeScript:

```text
Protobuf-ES 2.x
-> packages/protocol-ts
```

Public and private TS descriptors can live in this generated package but must be exported through separate subpaths, for example:

```text
@nous-wave/protocol/public
@nous-wave/protocol/kernel
```

`@nous-wave/client` depends only on the public subpath.

`apps/nous-core` may depend on both.

Rust private generation:

```text
protoc-gen-prost 0.5.x
protoc-gen-tonic 0.5.x
Tonic / Prost 0.14.x family
```

This route is specifically designed for common Buf tooling in a polyglot repository. `protoc-gen-tonic` is ordered after Prost generation.

Generate private Rust protocol code under the Kernel transport module, for example:

```text
apps/nous-kernel/src/transport/gen/
```

or an equivalent generated subtree.

Do not create a new reusable Rust protocol crate until there is a second actual Rust consumer. The Kernel is currently the only Rust transport consumer, so another crate would be structure without ownership value.

Generated code is clearly marked and excluded from source-shape limits where appropriate.

### 7.9 Why Buf, not independent build.rs + TS generation

Using `tonic-build` from `build.rs` is mature for Rust-only repositories, but this repository is deliberately polyglot. Buf provides one schema graph, linting, formatting, plugin configuration and reproducible generation for both Rust and TypeScript.

Therefore:

```text
Buf owns protocol generation orchestration.
Prost/Tonic own Rust codegen mechanics.
Protobuf-ES owns TS codegen mechanics.
```

No custom generator wrapper is required beyond a thin repository command.

### 7.10 Rust application migration: `apps/nous-wave` -> `apps/nous-kernel`

The current Rust application is not discarded. It contains the correct physical composition substrate.

Current responsibilities include:

```text
PostgreSQL bootstrap/connect
AuthorityStore migration
ObjectStore
SubjectCoreService
CognitiveRuntimeService
MaterialService
ServingService
MemoryService
query planning/execution coordination
public Axum HTTP
CLI
provider configuration
```

Target split:

```text
KEEP IN RUST KERNEL
-------------------
PostgreSQL/bootstrap mechanics
AuthorityStore
ObjectStore
Subject Core authoritative persistence
Cognitive Runtime durable mechanics
Material admission/materialization
Memory Authority
retrieval/query execution
serving projections/indexes
reference validation
runtime checkpoints
private gRPC
health/readiness
minimal recovery/admin command surface

MOVE TO TS HOST
---------------
public product API
normal CLI client behavior
consumer-specific Projection policy
Focus semantics
Steward
ModelRuntime/provider SDKs
model-assisted cognitive formation orchestration
managed cognitive context
public configuration resolution/presentation
```

### 7.11 Preserve the current Rust query engine

The current `NousRuntime.query()` already has a strong architecture:

```text
validate CognitiveQuery
-> QueryPlan
-> serving.prepare(...)
-> cognitive_runtime.query_with_plan(...)
-> Memory contributor
-> degradation merge
```

This remains Kernel logic.

Do not recreate retrieval planning in TypeScript.

TS owns intent compilation and optional model-derived query material, but Rust remains the owner of deterministic retrieval execution and evidence-family-aware ranking.

### 7.12 Query-time dense embeddings without reverse RPC

Moving provider/model mechanics to the Host creates one apparent problem: current dense query paths may need a query embedding.

Do **not** solve this with Kernel -> Host reverse RPC.

Use a two-step Host-orchestrated request:

```text
Public CognitiveQuery
-> Host compiles typed query
-> Host determines optional model materialization needs
-> ModelRuntime creates query embedding when configured
-> embedding is labeled with EmbeddingSpaceSignature / producer identity
-> Host sends private Kernel QueryRequest
     + semantic query AST
     + optional DenseCueMaterialization
-> Kernel validates compatible embedding space
-> Kernel runs exact / lexical / dense / topology / resource families
```

If embedding is unavailable:

```text
exact/lexical/topology/resource still run
+ dense capability degradation is reported truthfully
```

The public API never exposes a raw vector requirement.

The Kernel never calls an external model provider itself.

### 7.13 Index-building embeddings follow the same one-way rule

Background/self-scheduled index embedding is not introduced.

When committed/derived material needs embedding:

```text
Kernel records a bounded derivation/coverage need
-> Host explicitly requests/observes pending need
-> Host ModelRuntime generates embedding
-> Host commits derived representation + producer/space identity
-> Kernel invalidates/publishes affected serving generation
```

The existing `coverage_needs`, derivation and producer-signature concepts can continue to carry this semantic boundary.

### 7.14 Observation + model-assisted memory formation split

Current Rust `NousRuntime.observe()` does two logically different things:

```text
1. durable observation admission
2. optional Memory formation orchestration
```

Target behavior:

`FormationDirective::None`

```text
Host -> Kernel RecordObservation
-> durable observation only
```

`FormationDirective::ExplicitSpecific`

This is deterministic explicit user/Host intent and may remain one Kernel-owned operation after the observation is committed, provided evidence and transactional ordering remain clear.

`FormationDirective::ConsiderSpecific`

```text
Host -> Kernel RecordObservation
-> commit Occurrence first
-> Host obtains bounded source material
-> Host ModelRuntime proposes Specific Memory
-> Host -> Kernel CommitMemoryFormationProposal
-> Kernel validates proposal/evidence/entity scope
-> Memory revision commits or fails independently
```

A provider failure must never invalidate the Observation that already occurred.

The current Rust `MemoryFormationProvider` route becomes obsolete after Host parity and should be removed rather than retained as a second provider path.

### 7.15 Public Observation idempotency: narrow implementation

Stage 6 identified a real cross-system replay problem. The lowest-trajectory solution is **not** a universal idempotency framework yet.

First wave adds a bounded owner-native binding only for Observation admission:

```text
observation_request_bindings
----------------------------
request_id UUID PRIMARY KEY
subject_id UUID NOT NULL
request_digest TEXT NOT NULL
occurrence_id UUID NOT NULL
created_at timestamptz NOT NULL
```

Possible scope refinement if multiple public callers require namespacing:

```text
PRIMARY KEY (caller_namespace, request_id)
```

but do not add caller identity machinery solely for this table if Core is loopback single-principal in the first wave.

Processing:

```text
request_id absent
-> normal non-idempotent API semantics permitted for ordinary one-shot clients

request_id present and not seen
-> validate request
-> commit Observation + request binding atomically

same request_id + same canonical request digest
-> reconstruct/return equivalent AcceptedObservation

same request_id + different digest
-> ALREADY_EXISTS / ABORTED-style conflict with structured detail
```

Do not use `external_object_ref` as the key. An external object can legitimately be observed more than once.

Do not build a generic `IdempotencyService` until a second domain owner proves the reusable semantic/mechanical pattern.

### 7.16 Other mutation retry policy

Not every mutation needs durable request-id infrastructure in the first wave.

Subject creation can use caller-supplied `SubjectId` plus `GetSubject` reconciliation.

Memory edits use head `etag` / expected revision. If network outcome is unknown:

```text
refetch current head
-> compare revision / expected semantic outcome
-> reconcile
```

Suppress/Restore/Purge similarly expose authoritative state after reconnect rather than authorizing blind automatic retry.

This keeps retry semantics operation-specific and truthful.

### 7.17 Public Memory read models

Public Memory schema should distinguish list/detail/history projections.

Example semantic separation:

```text
MemorySummary
  memory_id
  memory_class
  status
  head_revision_no / etag
  title / bounded preview
  created_at
  updated/head time
  compact topology summary

Memory
  object/head identity
  full current revision semantic fields
  evidence refs
  entities
  tags
  provenance metadata

MemoryRevision
  immutable historical revision
```

`ListMemories` must not materialize all evidence/source content.

This keeps GUI browsing proportional to the list operation.

### 7.18 Memory mutation concurrency implementation

Current Memory object has `current_revision_id`, and each immutable MemoryRevision has `revision_no`, but the public update request currently lacks an expected head fence.

Add a stable mutable-head concurrency token. The simplest domain-native token is a monotonic `head_revision`/revision number paired with Memory identity; the public representation also exposes an opaque `etag` derived from the authoritative head identity.

Mutation:

```text
ReviseMemory(memory_id, expected_etag, ...)
-> transaction locks/revalidates Memory head
-> mismatch => ABORTED conflict
-> insert immutable revision
-> move current head
-> update mutable-head token
-> invalidate affected serving families
```

Status transitions (`Suppress`, `Restore`, `Purge`) also fence the mutable Memory object state.

Do not force FieldMask onto operations where semantic revision creation is the real operation.

### 7.19 Material/blob data plane

Control metadata remains Protobuf/Connect.

Large byte payloads use normal HTTP streaming through the Host.

Recommended Host mechanics:

```text
Fastify 5
@connectrpc/connect-fastify
@fastify/multipart
Node stream / Web Streams adapters
```

Kernel internal byte movement uses Tonic streaming only where the Host must proxy a stream into/out of the Rust-owned ObjectStore.

For reads, public artifact metadata can return a stable Core content endpoint/handle, and Client hides the HTTP details.

Standard HTTP Range is used for bounded reads/download resumes where supported.

Do not duplicate entire artifacts into protobuf unary messages.

### 7.20 `@nous-wave/client` source structure

Recommended package shape:

```text
packages/client/
  src/
    create-client.ts
    cognition.ts
    memory.ts
    material.ts
    subject.ts
    resource.ts
    topology.ts
    system.ts
    errors.ts
    transport.ts
```

These files are ergonomic wrappers/facades, not a second domain layer.

Generated descriptors remain in `packages/protocol-ts`.

Client responsibilities:

```text
Connect transport construction
public service clients
credential/deadline/interceptor setup
AbortSignal propagation
artifact upload/download convenience
ConnectError -> stable NousClientError mapping where valuable
small pagination helpers
```

Client must not:

```text
reimplement Memory validation
reconstruct Authority rules
contain GUI state/cache
know Kernel endpoints
choose retrieval algorithms
```

### 7.21 Public Connect server organization

`apps/nous-core` implements public services as thin application-facing handlers.

Recommended source tree:

```text
apps/nous-core/src/
  bootstrap/
    main.ts
    kernel-process.ts
    readiness.ts
  kernel/
    client.ts
    auth.ts
    mapping/
  api/
    routes.ts
    validation.ts
    errors.ts
  cognition/
    session.ts
    focus.ts
  projection/
    planner.ts
    contributors.ts
    render.ts
    types.ts
  context/
    epoch.ts
    patch.ts
  steward/
    engine.ts
    prompts.ts
    proposals.ts
  models/
    runtime.ts
    roles.ts
    middleware.ts
  material/
    http.ts
  config/
    load.ts
    schema.ts
    effective.ts
```

This is a starting physical organization, not a promise that each directory becomes a package.

### 7.22 Kernel process bootstrap

`nous-core` is the normal product entrypoint.

Boot flow:

```text
1. load explicit bootstrap config
2. choose/spawn `nous-kernel` executable
3. generate per-boot random kernel credential
4. create inherited liveness channel
5. kernel starts private loopback gRPC on ephemeral port
6. kernel emits exactly one bounded structured readiness record
7. Host validates process identity/protocol version/credential relationship
8. Host checks standard gRPC Health
9. Host constructs private generated client
10. Host starts public Connect/Fastify endpoint
```

Use Node `child_process`, standard pipes, gRPC Health, loopback TCP and normal process signals.

Do not add service discovery, a supervisor daemon or a custom binary IPC protocol.

### 7.23 Parent death and shutdown

Kernel must not accidentally survive as an orphan normal product process when its owning Core dies.

First-wave mechanism:

```text
inherited liveness pipe/socket
EOF => bounded Kernel shutdown
```

Normal Host shutdown:

```text
stop public admission
-> cancel/settle Host request scope
-> ask Kernel for graceful shutdown if needed
-> close child liveness channel
-> bounded wait
-> terminate on timeout
```

The exact timeout is configuration, not a semantic constant.

The Kernel Authority database remains durable; process death is not state deletion.

### 7.24 Kernel authentication boundary

Private gRPC is loopback-only and protected by a per-boot high-entropy bearer/token value generated by Core.

Use standard gRPC metadata and interceptor mechanics.

Do not invent message-level encryption or a new authentication protocol for loopback first-wave transport.

Kernel rejects requests missing/mismatching the boot credential before domain dispatch.

### 7.25 Public Core access boundary

First wave public Core is loopback by default.

It still requires a bounded local-access credential seam so that a future GUI/Heptalogos process is not implicitly trusted merely because it is local.

However, do not build remote multi-user auth, OAuth, RBAC, sessions or a policy engine without a current remote/multi-principal requirement.

CORS remains deny-by-default; browser GUI development explicitly admits configured local origins.

### 7.26 Configuration split

Kernel configuration becomes physical/mechanics-oriented:

```text
database mode/location
object store root
serving roots
retrieval physical bounds
private bind/bootstrap parameters
```

Host configuration owns:

```text
public bind/access
model/provider credentials/aliases
Steward role
Projection budgets/policy
Context epoch policy
Focus policy
maintenance/model behavior
```

The current Rust `RuntimeProviderConfig` should disappear after provider parity; otherwise it would encode a second Provider configuration system.

### 7.27 Configuration parser route

The first wave needs explicit TOML, environment overrides and typed validation. It does not need discovery/extends/watch/remote layers.

Therefore:

```text
smol-toml (or equally small stable TOML parser selected at implementation refresh)
+ TypeScript schema validation
+ explicit environment mapping
```

is preferable to a large configuration framework.

If exact package evidence changes before implementation, refresh the parser choice; do not change the semantic config model.

No hot reload in the first wave.

### 7.28 Runtime checkpoints

Add one Kernel-owned mechanics table/service for TS runtime state only when Focus/Managed Context arrives, not in the initial protocol-spine commit if there is no consumer yet.

Conceptual storage:

```text
runtime_checkpoints
  subject_id
  session_id
  checkpoint_kind
  checkpoint_key
  schema_version
  revision
  payload_bytes
  created_at
  updated_at

UNIQUE(subject_id, session_id, checkpoint_kind, checkpoint_key)
```

Mutation is CAS on `revision`.

`payload_bytes` is a versioned Protobuf payload owned by Host semantics.

The table is explicitly excluded from cognitive query, Memory formation and reinforcement.

### 7.29 Focus public/private split

Public Focus API exposes semantic lifecycle:

```text
CreateFocus
GetFocus
ListFocuses
ActivateFocus / SwitchFocus
SuspendFocus
ResumeFocus
CloseFocus
```

The Host owns focus state transitions and policy.

Durability uses checkpoint CAS through Kernel.

Do not make Kernel understand topic segmentation, focus titles, model continuation logic or projection policy beyond opaque/versioned storage validation.

### 7.30 Projection internal contracts

Define a Host-internal typed IR before any LLM synthesis:

```text
ProjectionRequest
ConsumerDescriptor
ProjectionNeeds
SituationDescriptor
ContributionEnvelope<T>
ProjectionPlan
CognitiveProjection
```

Contributor payloads remain typed discriminated unions; do not collapse all cognitive domains into one string field.

Shared envelope contains only genuinely cross-domain semantics:

```text
reference/source identity
authority class
evidence/provenance
freshness
priority/relevance
estimated cost/tokens
materialization handle
```

Memory-specific payload remains Memory-shaped. Future Persona remains Persona-shaped.

### 7.31 Deterministic Projection baseline

Before Steward integration, the projection path must close using deterministic mechanics:

```text
ProjectionRequest
-> map ConsumerDescriptor + ProjectionNeeds
-> Kernel CognitiveQuery
-> deterministic candidate/contribution retrieval
-> contributor transforms
-> budget/dedup/priority planner
-> deterministic renderer
-> CognitiveProjection
```

This is the acceptance baseline.

Only after it works does Steward add optional synthesis/selection assistance.

### 7.32 Steward insertion point

Steward is **not** on the Authority write path and not the only Projection planner.

```text
deterministic plan
-> optional Steward refinement/synthesis
-> validate output against source refs / allowed proposal schema
-> final CognitiveProjection
```

For maintenance:

```text
Steward proposal
-> owner validation
-> explicit authoritative operation
```

No direct SQL/Authority mutation from ModelRuntime.

### 7.33 Managed Cognitive Context implementation

Keep the protocol intentionally small:

```text
CognitiveContextPatch =
  RESET(epoch snapshot)
  | APPEND(delta)
```

No general JSON Patch, CRDT, operational transform or arbitrary diff language.

Host tracks:

```text
epoch id
snapshot digest
append sequence
source projection refs
consumer identity
```

Consumers such as Heptalogos may apply these patches to their own prompt-cache/context representation. They remain the owner of final InvocationSpec.

### 7.34 NousQL implementation placement

Public typed CognitiveQuery remains the stable API.

NousQL is an ergonomic string language compiled only in TypeScript:

```text
NousQL text
-> parser
-> typed CognitiveQuery
-> normal public/private query path
```

Do not teach the Rust Kernel a user-facing DSL.

Parser choice should be a mature parser toolkit selected when grammar work begins; do not block Core/Client rebase on NousQL.

### 7.35 GUI management does not require a GUI implementation now

The implementation Spec should prove GUI compatibility through Client-driven acceptance tests rather than build a GUI.

A headless test consumer must be able to:

```text
list Memory pages
search/query
get Memory detail
read history
follow Evidence to source material
revise with concurrency fence
suppress
restore
purge
inspect projection/readiness status
```

If these work through `@nous-wave/client`, a future GUI can remain pure Presentation.

### 7.36 Current Rust public API removal strategy

During one working branch, old Axum routes may temporarily coexist while parity is being implemented. This is implementation scaffolding only.

At capability closure:

```text
old normal public Axum API removed
old public CLI direct-Rust dispatch removed or converted to Client consumer
no compatibility shim
no dual routing
```

The PRE_PRODUCTION posture explicitly permits this direct replacement.

Kernel can retain a minimal recovery/admin CLI that does not masquerade as the normal product API.

### 7.37 Migration sequence by executable capability

Do not create a separate “cleanup phase.” Repository truth changes travel with the capability that invalidates the old truth.

Recommended executable sequence:

```text
A. Polyglot protocol/process spine
   - Node/TS workspace
   - Buf/proto generation
   - Tonic private Kernel endpoint
   - TS Core supervision
   - Health/readiness
   - architecture/decision/governance truth updated in same slice

B. Public Core + official Client parity
   - public Connect services
   - existing Subject/Session/Observation/Memory/Query/Resource/Material/Topology
   - artifact streaming
   - GUI-management minimum
   - Observation request_id idempotency

C. Remove obsolete public Rust surface
   - Axum public routes
   - direct normal Rust CLI dispatch
   - Rust provider configuration/path after parity

D. Cognitive Host runtime
   - Focus
   - Projection IR/planner
   - deterministic Projection
   - RuntimeCheckpointStore
   - Managed Cognitive Context

E. Model-assisted cognition
   - one TS ModelRuntime
   - query embeddings / derivation materialization
   - Steward
   - model-assisted Memory formation/maintenance
   - deterministic degradation remains intact

F. Reference-consumer integration (Heptalogos is one current probe)
   - separate repository implementation using the public @nous-wave/client contract
   - consumer-local Observation/Projection/Feedback adapter
   - prove that consumer integration requires no Heptalogos-specific fields in Nous public protocol
```

Capabilities B/C may be implemented closely together, but C is a closure condition of the public API migration, not an independent historical milestone.

### 7.38 Cross-repository Spec boundary

Do **not** make one giant Coding-Agent Spec that mutates both Nous Wave and Heptalogos simultaneously.

Reason:

- each repository has separate governance, verification and Authority;
- the Nous public contract must exist before Heptalogos can integrate against it;
- cross-repo partial execution would make failure/review boundaries ambiguous.

Recommended planning artifacts:

```text
Nous Wave Architecture Rebase Implementation Spec
    owns A-E

Heptalogos Nous Cognition Integration Spec (consumer-side, non-authoritative for Nous protocol)
    owns F
    begins only after the required @nous-wave/client contract exists
```

The present Pre-Spec research remains the shared architecture rationale for both.

### 7.39 Documentation truth updates that travel with implementation

The first Nous implementation slice must update at least:

```text
docs/Nous_Wave/ARCHITECTURE.md
  - remove physical modular-monolith claim
  - distinguish Cognitive Host / Kernel / Client
  - preserve semantic MicroSystem model

docs/Nous_Wave/DECISIONS.md
  - supersede D12 Rust-only LOCKED direction
  - separate semantic architecture from physical polyglot DEFAULT
  - revise D4 physical static-composition wording while preserving MicroSystem semantics
AGENTS.md
  - add polyglot verification/toolchain rules
  - Generated Protocol boundary
  - Host/Kernel owner discipline
README.md
  - describe Core+Client product shape
DESIGN_TRANSFER.md
  - mark transferred items as implemented/superseded/FUTURE rather than silently disappearing
```

Do not preserve the old statements as “legacy compatibility notes.” Git history already preserves them.

### 7.40 Stage-7 decisions safe to freeze

- One repository, Cargo + small pnpm workspace; no Nx/Turborepo.
- Node 24 LTS / TypeScript 7 / pnpm 11 first-wave TS baseline, refreshed to exact patch at implementation start.
- TS gates: `tsc --noEmit`, Oxlint, Prettier, Vitest; no forced TS7-incompatible lint stack.
- Buf v2 owns cross-language Protocol generation.
- Public and Kernel Protobuf namespaces are separate `v1alpha1` contracts.
- Protobuf-ES 2.x for TS; Prost/Tonic 0.14 family for private Rust.
- Tonic generated types remain inside Kernel transport/application adapters.
- Existing Rust query/retrieval/Authority/material/serving mechanics are preserved.
- No Kernel -> Host reverse RPC for model work.
- Host produces query/model-derived material and submits it to Kernel with producer/signature metadata.
- Model-assisted Observation formation becomes a post-observation proposal/commit flow.
- Observation gets narrow standard `request_id` idempotency; no generic idempotency framework yet.
- Current public Axum surface is removed after Core/Client parity; no compatibility dual path at closure.
- RuntimeCheckpointStore is added only with its first Focus/context consumer.
- Deterministic Projection closes before Steward.
- One Nous Spec should implement the Nous architecture rebase; Heptalogos integration is a later separate repo Spec against the public Client contract.

### 7.41 Stage-7 implementation-refresh items

The Coding Spec may refresh exact package patch versions immediately before execution without reopening architecture:

```text
Node 24.x exact patch
pnpm 11.x exact patch
TypeScript 7.x exact patch
Oxlint / Prettier / Vitest exact patch
Buf CLI exact patch
Protobuf-ES / Connect-ES exact patch
Tonic / Prost exact compatible patch
Fastify / connect-fastify / multipart exact patch
```

Such refresh is a dependency-pin task, not an architecture-design task.

**Stage 7 status: CONVERGED for Pre-Spec.**

---

---

## Stage 8 — Spec readiness, acceptance matrix, and final decision closure

_Status: CONVERGED for Pre-Spec._

### 8.1 What the implementation Spec is allowed to treat as Authority

The Nous Wave implementation Spec is authored from this order:

```text
Nous Wave cognitive invariants / current architecture intent
-> this Pre-Spec convergence record
-> current Nous repository implementation as migration evidence
-> mature dependency/standard contracts
-> external consumer requirements (GUI, Heptalogos, future clients) as pressure tests
```

Heptalogos is **not** in the authority chain for Nous protocol shape. Its repository may be cited as a current application example, but the Spec must not copy its state names, Reaction lifecycle, Prompt structures, Service registry, or package boundaries into `nous.wave.v1alpha1` unless the same field is independently justified as general cognition semantics.

A simple review rule is mandatory:

```text
If this field/method exists only because current Heptalogos happens to need it,
can the requirement be expressed through an existing consumer-agnostic Nous concept?

YES -> keep the generic concept; adapt Heptalogos locally.
NO  -> only then consider a new Nous semantic contract.
```

### 8.2 Final product boundary before Spec

The first implementation wave now has a sufficiently concrete product shape:

```text
Nous Core
  = TypeScript Cognitive Host
  + supervised Rust Cognitive Kernel

@nous-wave/client
  = only normal programmatic public entry

Future GUI / CLI / Heptalogos / other apps
  = public Client consumers

Rust Kernel private gRPC
  = implementation boundary, never public product API
```

Cognitive ownership:

```text
Kernel:
  Authority / durable cognitive semantics
  Memory revisions/evidence/topology
  material/object repository
  deterministic retrieval/ranking
  serving/indexes
  durable Session/ResidentSet/use-event mechanics
  runtime checkpoint CAS mechanics

Host:
  public API implementation
  consumer policy
  Focus semantics
  Projection IR/planning/rendering
  Managed Context epochs
  Steward
  model/provider runtime
  configuration resolution/presentation
  Kernel supervision
```

This split is strong enough that neither language is treated as the architecture. Semantic ownership remains the architecture.

### 8.3 Public protocol may now be drafted without current-application leakage

The initial `nous.wave.v1alpha1` service families are ready to be specified:

```text
SubjectService
CognitionService
MemoryService
MaterialService
ResourceService
TopologyService
SystemService
```

The Spec may merge or split a service only for coherent semantic ownership/client ergonomics; it must not create one service per source file or per consumer.

The first public schema MUST NOT contain identifiers such as:

```text
ReactionId
DecisionCommitId
CommunicationCommitId
ConversationMailbox
Heptalogos Activity
Heptalogos WorkItem
PromptProgram
InvocationSpec
```

Opaque external refs and a generic `SituationDescriptor` are sufficient for current consumers.

### 8.4 Public operation catalogue ready for the Spec

The Spec should write exact protobuf requests/responses for at least the following product operations.

```text
SubjectService
  CreateSubject
  GetSubject
  ListSubjects
  GetCharacterSeed
  ReviseCharacterSeed

CognitionService
  OpenSession
  GetSession
  ListSessions
  CloseSession
  CreateFocus
  GetFocus
  ListFocuses
  SwitchFocus / ActivateFocus (select one exact name)
  SuspendFocus
  ResumeFocus
  CloseFocus
  Query
  BuildProjection
  BuildManagedContext
  ReportUse
  RecordObservation (or MaterialService if ownership analysis places it there)

MemoryService
  ListMemories
  GetMemory
  GetMemoryRevision
  ListMemoryRevisions
  FormMemory
  ReviseMemory
  ConsolidateMemory
  SuppressMemory
  RestoreMemory
  PurgeMemory

MaterialService
  GetArtifact metadata
  GetOccurrence
  GetSourceRegion
  GetDerivedRepresentation
  MaterializeEvidence / resolve metadata
  narrow byte-transfer handles/helpers through Core HTTP data plane

ResourceService
  ListResources
  GetResource
  Register/UpsertResource
  RemoveResource

TopologyService
  List/Get Tag
  List Tag revisions
  List/Get Anchor
  List Anchor revisions
  GetNeighborhood
  current explicit CreateTag/CreateAnchor/CreateAssociation operations

SystemService
  GetStatus
  GetCapabilities
  GetProjectionStatus
  GetBuildInfo
  GetEffectiveConfig (redacted/read-only first wave)
```

Exact placement of `RecordObservation` is a naming/ownership detail for the Spec; its semantics are already closed: Observation is distinct from Memory formation and supports optional request-id idempotency.

### 8.5 Public resource methods follow mature API conventions

Use resource-oriented API conventions only where they match the domain. The project does not need to imitate Google APIs mechanically, but the established patterns solve generic client/GUI problems correctly:

```text
Get/List resource reads
page_size/page_token pagination
opaque etag for concurrency
optional request_id for retry-safe mutations where needed
custom methods for semantic lifecycle operations
```

This specifically means:

```text
ReviseMemory      custom method: creates immutable semantic revision
SuppressMemory    custom method
RestoreMemory     custom method
PurgeMemory       custom method
ConsolidateMemory custom method
```

Do not contort these into generic `Update`/`Delete` merely to look REST-like.

### 8.6 Manual GUI correction becomes evidence, not an evidence bypass

GUI management creates a real requirement not fully represented by the current `ReviseMemoryInput`: an authorized human/operator must be able to correct cognition even when they do not already have a convenient existing `EvidenceRef` to attach.

The implementation MUST preserve the invariant:

```text
Memory revision has provenance/evidence
```

It MUST NOT solve GUI editing by making evidence optional on authoritative revisions.

First-wave semantic route:

```text
explicit management Form/Revise request
-> record an immutable explicit-management/Host observation
   containing the supplied correction/formation statement and optional reason
-> bind consumer/principal metadata available at the Core boundary
-> use that new ObservationOccurrence as direct evidence
-> create the Memory revision in the same Kernel-owned authoritative operation
-> return the Memory plus the created evidence ref
```

Where the caller supplies existing evidence refs, those refs are validated and retained in addition to or instead of the explicit-management evidence according to the exact operation contract.

The public Client hides storage choreography. GUI code never writes `observation_occurrences` or Memory evidence rows.

Do not introduce a generic audit framework here; reuse Nous material/evidence semantics because the explicit correction is itself cognitive provenance.

### 8.7 Concurrency closure

Every mutable authoritative head that can be edited by GUI/Steward/other Clients needs an observable concurrency fence.

For first-wave Memory:

```text
immutable MemoryRevisionId / revision_no
+
mutable Memory head_revision
+
public opaque etag
```

Operations that can change the current head/status take the expected ETag.

```text
match    -> commit
mismatch -> ABORTED + refetch
```

This prevents ABA problems such as suppress -> restore returning to a superficially identical state.

Focus/Session runtime mutations use Session runtime CAS/revision independently; do not reuse Memory ETags as a universal concurrency system.

### 8.8 Idempotency closure

`request_id` is operation-local, optional, and guarantees idempotency when present.

Mandatory first current consumer:

```text
RecordObservation(request_id)
```

because durable external delivery can retry after an unknown network outcome.

Required proof:

```text
same request_id + same canonical request
  -> same/equivalent success
  -> one Occurrence only

same request_id + different canonical request
  -> conflict

different request_id + same external_object_ref
  -> two Occurrences are allowed
```

Do not create a universal idempotency service before another owner proves a reusable need.

### 8.9 Final use/learning vocabulary

The semantic ladder is now closed as:

```text
CANDIDATE
  retrieval/planner candidate; may be transient

SURFACED
  returned/available at a query or contribution boundary

SELECTED
  Nous selected it into a CognitiveProjection

EXPOSED
  consumer actually presented/materialized it to the downstream model/human/logic

REFERENCED
  downstream processing explicitly referred to/followed it

ACTED_ON
  a committed downstream action/behavior materially depended on it

CORROBORATED / CORRECTED / PINNED / REJECTED
  explicit semantic feedback

EXPLICIT_REINFORCEMENT
  only an explicit owner operation if/when such a cognitive contract exists
```

`SURFACED`, `SELECTED`, and `EXPOSED` are not meaningful reinforcement by themselves.

Implementation consequence:

- Nous may record `SELECTED` itself when building a Projection;
- public consumer feedback begins with `EXPOSED` when the consumer can prove actual exposure;
- a consumer never upgrades to `REFERENCED` merely because content existed in a prompt/context.

The existing `UseKind::Selected` should therefore not be deleted; add/clarify `Exposed` and update `meaningful()` semantics directly during PRE_PRODUCTION rewrite.

### 8.10 Failure/degradation matrix for the Spec

The Spec should encode the following outcomes explicitly rather than leave them to implementation improvisation.

| Failure | Required truthful result |
|---|---|
| Steward model unavailable | deterministic Projection/Session/GUI remain usable; steward readiness unavailable/degraded |
| embedding model unavailable | dense family unavailable/degraded; exact/lexical/topology continue where available |
| projection synthesis invalid/fails | deterministic renderer used; no invented refs |
| Host model invocation dies before proposal | no Authority mutation |
| Host dies after Kernel mutation commit but before response | caller refetches resource/session; no distributed rollback |
| Core dies | Kernel detects parent-liveness loss and shuts down; restart reconstructs from durable state |
| Kernel dies unexpectedly | Core reports Kernel UNAVAILABLE; no fabricated successful operations |
| managed-context cursor unknown after restart | return RESET from current cognition, not recovery failure |
| stale Memory ETag | ABORTED/conflict; no last-write-wins |
| stale Session runtime revision | ABORTED; only one Focus transition commits |
| duplicate Observation request_id | return prior/equivalent success, no duplicate occurrence |
| model-assisted Memory formation fails after Observation | Observation remains committed; Memory formation failure is separate |
| GUI/Client absent | Nous Core cognition remains headless and operational |
| external consumer (e.g. Heptalogos) absent/stopped | Nous cognitive Authority remains independently manageable |

### 8.11 Acceptance suite: protocol/process spine

The implementation Spec should name executable scenario IDs so the Coding Agent knows what proves closure.

```text
NW-PROC-001 Core starts Kernel, discovers endpoint, validates gRPC Health, then publishes READY.
NW-PROC-002 private Kernel RPC without boot credential is rejected before domain dispatch.
NW-PROC-003 Core death closes liveness channel and Kernel performs bounded shutdown.
NW-PROC-004 unexpected Kernel exit projects public UNAVAILABLE; Core does not fake readiness.
NW-PROC-005 @nous-wave/client can use public Core without any private Kernel address/type.
NW-PROC-006 generated public/private protocol trees are reproducible and clean after `buf generate`.
```

### 8.12 Acceptance suite: GUI-management completeness without GUI

```text
NW-MGMT-001 Client lists Memory with bounded page size and opaque next_page_token.
NW-MGMT-002 changing non-page arguments while reusing a page_token is rejected.
NW-MGMT-003 Client opens Memory detail and traverses revision -> evidence -> source metadata.
NW-MGMT-004 Client streams a bounded/range Artifact payload without whole-file buffering in Core.
NW-MGMT-005 explicit human revision records explicit-management evidence and a new immutable MemoryRevision.
NW-MGMT-006 stale ETag revision is rejected with ABORTED; winning revision remains intact.
NW-MGMT-007 Suppress and Restore are explicit fenced domain operations.
NW-MGMT-008 Purge follows existing purge semantics and does not become raw DELETE-row behavior.
NW-MGMT-009 all above operations remain available with zero LLM/provider configuration except capabilities that intrinsically need a model.
NW-MGMT-010 GUI-equivalent Client never touches SQL/index/Kernel transport internals.
```

This is the actual proof of the Core+Client GUI requirement. Building React/Tauri/Electron is not required for this wave.

### 8.13 Acceptance suite: Observation, retrieval, and model separation

```text
NW-OBS-001 request-id retry after simulated lost response creates one Occurrence.
NW-OBS-002 same external object with different request ids can create distinct real encounters.
NW-OBS-003 Observation commits before optional model-assisted Memory formation.
NW-OBS-004 formation-model failure leaves Observation/evidence intact.

NW-QUERY-001 typed CognitiveQuery produces deterministic results with no LLM.
NW-QUERY-002 unavailable dense embedding produces explicit dense degradation, not total query failure.
NW-QUERY-003 Host-supplied dense cue is rejected when embedding-space signature is incompatible.
NW-QUERY-004 Kernel retrieval preserves evidence-family diagnostics without exposing engine query syntax publicly.
```

### 8.14 Acceptance suite: Session, Focus, Projection, and context

```text
NW-COG-001 one Session cannot commit two ACTIVE foreground Focuses.
NW-COG-002 two concurrent Focus switches from the same runtime revision yield one success + one ABORTED.
NW-COG-003 Focus resume resolves current Authority refs rather than restoring stale copied cognition.
NW-COG-004 deterministic BuildProjection works with Steward/model runtime unavailable.
NW-COG-005 Projection records SELECTED but does not make it meaningful reinforcement.
NW-COG-006 consumer ReportUse(EXPOSED) remains non-reinforcing.
NW-COG-007 model synthesis cannot introduce a ref outside the selected/allowed source set.
NW-COG-008 invalid synthesis falls back to deterministic rendering.
NW-CTX-001 first managed-context call returns RESET.
NW-CTX-002 semantically additive cognition may return APPEND.
NW-CTX-003 stale/revised source that invalidates an old segment forces a new-epoch RESET.
NW-CTX-004 Host restart may safely RESET instead of reproducing exact provider-cache continuation.
```

### 8.15 Acceptance suite: generic consumer boundary

Heptalogos can be used as one current integration probe, but the decisive test is consumer neutrality.

```text
NW-CONS-001 a minimal standalone test consumer can Observe -> Query/Project -> ReportUse using only @nous-wave/client.
NW-CONS-002 no public proto field/message/service name contains Heptalogos-specific execution concepts.
NW-CONS-003 a current Heptalogos adapter can map its own facts/situation into the public Nous contract without private API access.
NW-CONS-004 changing the Heptalogos local Reaction/Prompt/Service design does not require a Nous protocol change unless a cognition semantic requirement changes.
NW-CONS-005 Nous GUI/CLI management works while Heptalogos is not running.
```

`NW-CONS-004` should be a review assertion rather than a brittle source-code test; the Spec should explicitly ask reviewers to inspect for leaked consumer terms.

### 8.16 Repository/architecture closure scenarios

```text
NW-REPO-001 root `just verify` covers Rust + TS + Buf and succeeds from a clean checkout.
NW-REPO-002 generated protocol output is deterministic and not manually edited.
NW-REPO-003 Rust-only/modular-monolith physical claims are removed from current Architecture/Decisions.
NW-REPO-004 new AGENTS rules identify Host/Kernel/public/private ownership and library-first mechanics.
NW-REPO-005 after parity, old normal public Axum management routes are removed.
NW-REPO-006 after provider parity, the obsolete parallel Rust external-provider route/config is removed.
NW-REPO-007 no compatibility shim/legacy route remains solely because the previous branch had it.
```

### 8.17 Executable capability DAG for the future Spec

The Spec should preserve dependency order without turning it into bureaucratic historical phases.

```text
A  Protocol + process spine
|  - polyglot root
|  - Buf/proto
|  - Tonic private Kernel
|  - Core supervision/readiness
|  - architecture/governance truth changes
v
B  Public Core + Client management parity
|  - Subject/Session/Observation/Memory/Query/Resource/Material/Topology/System
|  - pagination / errors / ETags
|  - request-id Observation
|  - artifact streaming
|  - headless GUI acceptance
v
C  Remove superseded public/provider surface
|  - old public Axum
|  - normal direct Rust CLI dispatch
|  - duplicate Rust external-provider path after parity
v
D  Cognitive Host runtime
|  - Focus
|  - RuntimeCheckpointStore
|  - Projection IR + deterministic planner
|  - Managed Context RESET/APPEND
v
E  Model-assisted cognition
   - Host ModelRuntime
   - query embeddings / derivation production
   - Steward
   - model-assisted formation/synthesis
   - deterministic fallback acceptance
```

The Heptalogos repository is **not** a dependency node required to close A-E. Consumer integration is a separate later proof against the finished public contract.

### 8.18 Exact Spec-authoring structure

The next artifact should be one decision-complete Nous Wave implementation Spec organized roughly as:

```text
1. Purpose / repository baseline / non-goals
2. Locked architecture decisions
3. Current-state -> target-state ownership map
4. Dependency/version refresh table
5. Public protocol specification
6. Private Kernel protocol specification
7. Repository/toolchain changes
8. Kernel transport/process adaptation
9. Core bootstrap/supervision
10. Public Client implementation
11. Management/read model and byte-data-plane implementation
12. Observation idempotency and Memory concurrency/provenance changes
13. Focus/runtime checkpoint implementation
14. Projection/Context implementation
15. ModelRuntime/Steward migration
16. Obsolete-surface deletion
17. Documentation/governance updates
18. Acceptance scenarios and exact verification commands
19. Stop conditions / PLAN_GAP rules
```

The Spec should include concrete paths/types/RPC names and transaction/failure semantics. It should not repeat this research as prose or reopen architecture choices during execution.

### 8.19 Decision classification at Pre-Spec closure

#### LOCKED semantic/product decisions

```text
Nous Wave is an independent Subject cognition system.
Nous cognition is distinct from host/application agency.
Core + official Client is the independent headless product shape.
GUI is external Presentation and never owns cognitive Authority.
Public contract is consumer-agnostic; Heptalogos is reference material, not interface Authority.
Cognitive Authority/runtime/projection/model-cache remain distinct.
Memory revisions/evidence/provenance remain authoritative Kernel semantics.
Retrieved/selected/exposed != reinforced.
Model output is proposal-first and cannot directly mutate Authority.
Final consumer invocation/behavior authority remains outside Nous.
Session is cognitive continuity, not Conversation/model thread.
One foreground Focus per Session is the first-wave semantic constraint.
Projection is consumer-specific and deterministic without LLMs.
Managed Cognition owns only a Nous context lane using RESET/APPEND.
No hidden autonomous scheduler is part of Core cognition.
```

#### DEFAULT physical decisions, frozen for the first implementation wave

```text
Rust Cognitive Kernel + TypeScript Cognitive Host.
Public Protobuf/Connect; private Protobuf/gRPC.
Tonic 0.14.x stable released line behind transport adapter.
Connect-ES/connect-node/connect-web for TypeScript/public transport.
Buf + Protobuf-ES + Prost/Tonic generation.
Node 24 LTS + TypeScript 7 + pnpm 11, exact patches refreshed at execution start.
Fastify only for Core HTTP needs; @fastify/multipart for streaming upload.
AI SDK 7 Core for model/provider mechanics.
explicit TOML + explicit env overrides; no hot reload.
loopback public/private bindings with bounded local credentials.
parent-child Core/Kernel topology with inherited liveness channel.
```

#### OPEN / not blocking the first Spec

```text
GUI framework and Desktop/Web packaging.
remote management/authentication/multi-user policy.
Node 26 transition after it becomes the chosen LTS support baseline.
new grpc-rust preview successor to Tonic after stable migration evidence.
resumable upload protocol.
long-running operation framework.
automatic Kernel restart policy.
full OpenTelemetry exporter topology.
managed configuration mutation/hot reload.
concrete Steward/provider/model bindings.
multi-foreground Focus.
future Persona/Relationship/Epistemic/Goals schemas.
server-scale retrieval backend.
cross-product backup/restore coordination.
```

These OPEN items must not be converted into placeholder frameworks in the first Spec.

### 8.20 Material PLAN_GAP triggers during implementation

The Coding Agent should stop and report a narrow `PLAN_GAP` only if implementation evidence contradicts a material decision such as:

```text
Connect-Node cannot interoperate with the chosen standard private gRPC route as assumed.
Tonic 0.14 cannot support a required current streaming/health/auth behavior without unsafe/custom protocol work.
Current Memory Authority transaction structure cannot add a correct head fence without changing domain semantics.
Atomic explicit-management evidence + Memory revision cannot be implemented within Kernel ownership without a new semantic decision.
Current retrieval API cannot accept Host-produced dense cues while preserving embedding-space validation.
RuntimeCheckpoint CAS cannot preserve single-active-Focus invariant without moving more Focus mechanics into Kernel semantics.
Artifact streaming requires unavoidable whole-payload buffering in a chosen dependency path.
```

Do **not** report PLAN_GAP for ordinary implementation choices such as file splitting, function naming, error mapping details, test helper structure, or exact patch-version refresh when the chosen dependency family still satisfies the contract.

### 8.21 Dependency maturity checks to carry into Spec refresh

The research has rechecked the critical maturity-sensitive route:

- Tonic `0.14.6` is the current released Tonic line during this research and explicitly remains the recommended released branch while upstream prepares breaking changes; its documented feature set includes streaming, metadata/authentication and health-check support.
- Request identification, pagination, ETags and custom methods use long-standing approved API patterns rather than Nous-specific protocols.
- AI SDK provider/model/structured-output mechanics remain an implementation substrate, not cognition Authority.

The Spec should refresh exact versions at its start, but must not silently jump to a preview generation merely because it is newer.

### 8.22 Final Pre-Spec conclusion

No remaining architecture ambiguity requires another macro-design round before writing the Nous Wave implementation Spec.

The unresolved items are either:

```text
exact dependency patch pins,
small naming choices,
consumer-local adapter choices,
or explicitly FUTURE product capabilities.
```

The implementation trajectory is therefore ready to change from research mode to decision-complete Spec authoring.

Most importantly, the boundary is now stable against both sides evolving independently:

```text
Nous public contract
  <- defined by cognition semantics and independent management needs

Heptalogos / GUI / CLI / future applications
  -> consume that contract through @nous-wave/client
  -> adapt their own execution/product concepts locally
```

This prevents the next Heptalogos refactor from becoming a forced Nous protocol rewrite, while still allowing Heptalogos to remain a valuable real-world integration/acceptance application.

**Stage 8 status: CONVERGED. Pre-Spec implementation research is complete and ready for Spec authoring.**

---

## Research references used for generic mechanics

These references inform generic mechanics only; Nous cognitive semantics remain project-owned.

- Tonic released documentation: https://docs.rs/tonic/latest/
- Google AIP-121 Resource-oriented design: https://google.aip.dev/121
- Google AIP-130 Methods: https://google.aip.dev/130
- Google AIP-134 Update / ETags: https://google.aip.dev/134
- Google AIP-136 Custom methods: https://google.aip.dev/136
- Google AIP-155 Request identification: https://google.aip.dev/155
- Google AIP-158 Pagination: https://google.aip.dev/158
- Buf generation documentation: https://buf.build/docs/generate/
- ConnectRPC documentation: https://connectrpc.com/
- Vercel AI SDK documentation: https://ai-sdk.dev/docs

### 8.1 Final pre-Spec review method

Before declaring this research complete, replay the proposed architecture against every currently important seam:

```text
Subject provisioning
Artifact persistence
Observation admission
Session continuity
Focus continuation
Cognitive Query
Resource awareness
Memory formation/revision/lifecycle
Projection
Managed cognitive context
Steward/model assistance
GUI management
Heptalogos Reaction/Behavior Authority
process crash/restart
model/provider loss
concurrent administration
```

A design is not Spec-ready if any of these still requires an undefined reverse dependency, shared transaction, hidden scheduler, duplicated Authority, or generic mechanism invented only to bridge the new architecture.

### 8.2 Late correction: Artifact upload must not secretly equal Observation

The current public `POST /artifacts` path calls `ingest_stream()`, and `ingest_stream()` persists the Artifact then calls `record_observation()`. This means the transport operation named as Artifact upload implicitly creates a Subject encounter.

That is too coupled for the new management contract because the architecture already distinguishes:

```text
Artifact != ObservationOccurrence
persist != observe
```

Target public semantics:

```text
Artifact byte persistence
  -> separate data-plane upload operation
  -> returns Artifact identity/metadata

RecordObservation
  -> may reference the existing Artifact
  -> creates ObservationOccurrence
```

Client ergonomics may provide:

```text
nous.material.uploadArtifact(...)
nous.cognition.recordObservation({ artifact_ref })

nous.cognition.observeFile(...)
  # convenience composition that performs the two calls
```

The convenience method does not collapse the underlying Authority semantics.

Failure case is truthful:

```text
Artifact upload succeeds
Observation commit fails
-> Artifact exists without an Occurrence
-> caller may retry RecordObservation
-> no fabricated rollback of persisted bytes
```

The object repository may later perform owner-governed orphan cleanup/purge; this does not justify a distributed upload+observation transaction.

### 8.3 Late correction: in-process `ResourceResolver` cannot cross the new process boundary

Current `CognitiveRuntimeService` owns an in-memory Rust map of `ResourceResolver` trait objects. Query routing can call those resolvers directly. The shipped `NousRuntime` currently constructs `CognitiveRuntimeService::new(...)` without a concrete resolver, so the trait is a seam rather than a required current provider.

After Host/Kernel separation, preserving this exact abstraction would force one of two bad outcomes:

```text
Kernel -> Host callback RPC
or
move arbitrary external resource providers into Rust Kernel
```

Neither is justified.

The semantic requirement is only Resource Awareness and progressive access, not an in-process callback trait.

First-wave correction:

```text
Kernel owns
  ResourceDescriptor Authority
  resource refs / coverage / query dimensions / modality / freshness / cost/readiness
  local cognitive association with resources
  bounded ResourceActionSuggestion generation

Host/consumer owns
  actual external resource access when a real resolver exists
```

Remove `resolver_key` from stable public cognitive semantics. If a Core-owned resolver is later implemented, its binding is Host configuration/runtime state, not cognitive Resource identity.

`CognitiveQueryResult.resource_actions` remains useful as an **action request/suggestion**:

```text
resource_ref
requested dimensions
reason
current-authority requirement
bounded evidence/materialization expectation
```

It should not imply that Kernel has already queried the external source.

For a `CurrentAuthorityNeed::Required` query with no authoritative current data available:

```text
local cognition results may still be returned
query status = PARTIAL
required ResourceActionSuggestion(s) explain what is missing
```

Do not make missing caller-owned external authority look like a Kernel infrastructure failure.

A later concrete Host-side resolver/provider can satisfy the action and submit resulting Evidence/Observation or Situation material through normal contracts. Do not design a generic callback/provider framework until that consumer exists.

### 8.4 Public v1alpha1 service inventory to freeze in the Spec

The first Architecture Rebase Spec should define enough public surface for both official Client use and future GUI management.

#### SubjectService

```text
CreateSubject
GetSubject
ListSubjects
GetCharacterSeed
ReviseCharacterSeed
```

No Subject purge/delete operation is added merely because a GUI might someday want one. Subject-wide lifecycle deletion has broader data-owner semantics and remains separate.

#### CognitionService

```text
OpenSession
GetSession
ListSessions
CloseSession

CreateFocus              # arrives with Host runtime capability
GetFocus
ListFocuses
ActivateFocus/SwitchFocus
SuspendFocus
ResumeFocus
CloseFocus

RecordObservation
Query
BuildProjection           # arrives with Projection capability
ReportUse
```

Focus/Projection methods may enter in a later capability commit within the same Spec, but the namespace/ownership is fixed now.

#### MemoryService

```text
ListMemories
GetMemory
ListMemoryRevisions
GetMemoryRevision
FormMemory
ReviseMemory
ConsolidateMemory
SuppressMemory
RestoreMemory
PurgeMemory
```

`ReviseMemory` is semantic revision creation, not generic PATCH.

#### MaterialService / data plane

Control plane:

```text
GetArtifact
ListArtifacts             # bounded management projection
GetOccurrence
GetSourceRegion
GetDerivedRepresentation
MaterializeEvidence metadata/request
```

Byte plane:

```text
UploadArtifact HTTP stream
DownloadArtifact HTTP stream / Range
Materialize/download bounded evidence stream where bytes are large
```

Do not encode a multi-GB artifact in a unary protobuf message.

#### ResourceService

```text
ListResources
GetResource
Register/UpdateResourceDescriptor
RemoveResourceDescriptor
```

No generic executable resolver registration in v1alpha1.

#### TopologyService

At minimum preserve current capability without exposing engine internals:

```text
CreateTag
GetTag/ListTags as required for GUI
CreateAnchor
GetAnchor/ListAnchors as required for GUI
CreateAssociation
GetNeighborhood(bounded)
RebindEntity
```

Do not expose `DumpGraph` or petgraph/CSR storage.

#### SystemService

```text
GetStatus
GetCapabilities
GetEffectiveConfig
GetProjectionStatus
```

Mutation of configuration can remain absent in the first Core/Client closure if no current GUI editing requirement is authorized. Readable effective configuration is required for management transparency.

### 8.5 Proto conventions

Freeze the following low-cost rules:

```text
UUID-like semantic IDs -> string fields with validation
Instant -> google.protobuf.Timestamp
Duration -> google.protobuf.Duration where semantically required
optional scalar presence -> proto3 optional where absence matters
ordinary partial update -> google.protobuf.FieldMask only where update semantics are actually ordinary
opaque metadata -> google.protobuf.Struct only for explicitly non-core metadata
```

Do not use `google.protobuf.Any` as a universal escape hatch.

Enums reserve `*_UNSPECIFIED = 0` unless a specific protobuf convention makes another route materially better.

Core semantic fields stay typed even when current Rust storage uses JSONB.

### 8.6 Pagination contract

List APIs use standard:

```text
page_size
page_token
-> items
-> next_page_token
```

The token is opaque to the Client. Keyset/cursor mechanics may live in Kernel storage adapters; the public API does not expose offset arithmetic as a stability promise.

A token is valid only with the same effective filter/sort request it was issued for.

No custom generalized pagination library is required: generated protobuf types + bounded cursor encode/decode + owner query mechanics are enough.

### 8.7 Error contract

Use Connect/gRPC canonical status codes, with richer typed details where the GUI/Client benefits.

Mapping families:

```text
INVALID_ARGUMENT       malformed/invalid domain request
NOT_FOUND              missing Subject/Memory/Artifact/etc.
ALREADY_EXISTS         request-id identity collision where appropriate
ABORTED                stale mutable-head revision/etag conflict
FAILED_PRECONDITION    state/lifecycle/readiness prevents operation
UNAVAILABLE            temporary required capability/provider/kernel unavailable
RESOURCE_EXHAUSTED     explicit bounded resource limit
INTERNAL               uncategorized infrastructure defect
```

Use Google richer error detail messages where useful for:

```text
field violations
precondition violations
resource identity
retry metadata
```

Do not recreate the current free-form `{error: string}` HTTP Problem shape in the new public contract.

### 8.8 Client error semantics

`@nous-wave/client` may provide a small stable wrapper that exposes:

```text
canonical code
message
structured details
cause / ConnectError when useful
```

It must not create dozens of handwritten exception subclasses mirroring each RPC.

GUI code should branch on canonical status + typed details, not parse English strings.

### 8.9 Public request validation

At the TypeScript public boundary:

```text
Protobuf schema
+ Protovalidate annotations/interceptor where they express transport/API shape constraints
```

remain appropriate mature mechanics.

But transport validation is not cognitive Authority validation.

Kernel domain owners still revalidate:

```text
Subject scope
reference ownership
Memory evidence semantics
revision fences
producer/embedding-space compatibility
purge/suppression lifecycle
```

No unofficial Rust validation library is required merely to duplicate Protovalidate annotations.

### 8.10 Readiness model

Core readiness should not be one boolean.

Public System status includes bounded component/capability state, such as:

```text
kernel             READY / UNAVAILABLE / FAILED
subject_core       READY / ...
cognitive_runtime  READY / ...
memory             READY / UNAVAILABLE / DISABLED
lexical            READY / STALE / BUILDING / UNAVAILABLE
dense              READY / STALE / BUILDING / UNAVAILABLE
topology           READY / STALE / BUILDING / UNAVAILABLE
model.steward       READY / NOT_CONFIGURED / UNAVAILABLE
model.embedding     READY / NOT_CONFIGURED / UNAVAILABLE
```

Public normal API readiness may be DEGRADED while management/lexical Memory remains usable.

A GUI can therefore present truthful partial capability instead of a binary “server online” badge.

### 8.11 Projection status is separate from cognitive Authority status

Serving status projection should expose:

```text
family
state
current generation identity
implementation id/revision
producer/embedding-space identity where relevant
last publication/build time
staleness/degradation reason
```

Do not expose filesystem index paths or internal segment structures.

Deleting/rebuilding a serving generation never deletes Memory/Authority.

### 8.12 Core startup acceptance scenario

Mandatory process-level proof:

```text
start nous-core
-> Core loads bootstrap config
-> spawns nous-kernel
-> Kernel binds ephemeral loopback gRPC
-> readiness handshake + standard Health succeeds
-> Core starts public Connect endpoint
-> @nous-wave/client GetStatus succeeds
```

Verify the Client cannot access private Kernel RPC without the per-boot credential.

### 8.13 Kernel parent-death acceptance

```text
start Core + Kernel
-> kill Core ungracefully
-> liveness channel closes
-> Kernel exits/fail-stops within bounded policy
-> durable PostgreSQL/objects remain valid
-> restart Core
-> new Kernel process opens same Authority
-> public Client reads prior Memory/Session state
```

This is more valuable than testing a custom supervisor abstraction.

### 8.14 No-LLM GUI-management acceptance

Run with no external model credentials and dense capability absent if necessary.

The official Client must still complete:

```text
create/get Subject
upload Artifact
record Observation
form explicit Memory
list Memory pages
get Memory detail
list/get revisions
follow Evidence to Artifact/source
revise with correct etag
reject stale etag
suppress
restore
query exact/lexical/topology as available
inspect degradation/status
purge
```

This is the minimum proof that GUI management does not depend on Steward/LLM.

### 8.15 Artifact/Observation separation acceptance

```text
upload Artifact A
-> Artifact exists
-> no ObservationOccurrence exists solely because bytes were persisted

RecordObservation(request_id=R, artifact=A)
-> Occurrence O created

retry same R + same canonical input
-> O returned/equivalent result
-> no O2

retry same R + different Artifact/occurrence input
-> request-id conflict
```

This test closes two current semantic gaps at once.

### 8.16 Memory concurrency acceptance

```text
Client A GetMemory -> etag E8
Client B GetMemory -> etag E8

B ReviseMemory(expected=E8)
-> revision 9 / etag E9

A ReviseMemory(expected=E8)
-> ABORTED
-> no revision 10 from stale edit

A refetch
-> sees E9
```

Repeat equivalent fencing for status-changing operations where concurrent mutation matters.

### 8.17 Retrieval degradation acceptance

Model embedding provider absent:

```text
CognitiveQuery asks for normal optional semantics
-> exact/lexical/topology lanes run
-> dense lane reports unavailable/degraded
-> valid results still returned
```

If query explicitly marks embedding/dense capability REQUIRED:

```text
-> truthful required-capability failure/partial semantics according to typed query contract
-> no fake zero vector
-> no silent lexical substitution labeled as dense
```

### 8.18 Host-supplied query embedding acceptance

With a configured embedding ModelRuntime:

```text
public text cue
-> Host obtains embedding + EmbeddingSpaceSignature
-> private query includes DenseCueMaterialization
-> Kernel validates space compatibility
-> dense serving lane participates
-> generation/evidence diagnostics record correct lane/space
```

Mismatched space:

```text
-> dense material rejected/degraded
-> no comparison across incompatible vector spaces
```

### 8.19 Model-assisted formation acceptance

```text
RecordObservation commits O
-> model formation role invoked
-> proposal references O / allowed source material
-> Kernel validates proposal
-> Memory revision commits
```

Provider failure after O:

```text
Observation remains durable
no Memory is fabricated
operation reports model-assisted formation failure/degradation
retry may later propose from the same authoritative O
```

### 8.20 Resource-awareness acceptance

With no external resolver provider:

```text
register ResourceDescriptor R
-> Query requests/prefer current authority
-> local cognition executes
-> result contains bounded ResourceActionSuggestion for R
-> status truthfully reflects unresolved current-authority need
-> Kernel makes no callback/network call
```

This proves Resource Awareness survives the process split without inventing reverse RPC.

### 8.21 Deterministic Projection acceptance

Before Steward is enabled:

```text
Session + active Focus
-> BuildProjection(consumer, needs, situation, budget)
-> Kernel query/source materialization
-> typed contributions
-> deterministic planning/dedup/budget
-> CognitiveProjection
```

Assert:

```text
same authoritative inputs + same policy revision
-> deterministic selection/ordering within defined tie behavior

Projection creation
-> does not by itself create meaningful-use/reinforcement event
```

### 8.22 Consumer-role differentiation acceptance

Same Subject/Session/Focus:

```text
consumer = heptalogos.subject.primary
needs memory + relationship/persona future weights

consumer = heptalogos.subject.expression
needs stronger persona/language context, weaker goals
```

First wave with Memory only must still prove distinct policies can produce different allowed projections without duplicating Authority.

Do not require future Persona implementation to test the consumer boundary; use Memory/evidence roles available now.

### 8.23 Focus continuation acceptance

```text
Session S
Focus A active
-> projection/use
-> suspend A -> checkpoint revision 3
Focus B active
-> work
-> suspend B
resume A
-> CAS read checkpoint 3
-> resolve current Authority refs again
-> create/reuse valid Context Epoch
```

If a referenced Memory changed while A was suspended, resume must observe the new authoritative head according to reference semantics rather than restore a stale full copy.

### 8.24 Checkpoint concurrency acceptance

Two Host operations attempt to update the same Focus checkpoint revision:

```text
writer 1 expected=4 -> commits 5
writer 2 expected=4 -> conflict
```

No last-write-wins runtime checkpoint corruption.

This is mechanics correctness; it does not make checkpoint payload cognitive Authority.

### 8.25 Managed Cognitive Context acceptance

Consumer begins with:

```text
RESET(epoch 10, snapshot)
```

then receives:

```text
APPEND(delta 1)
APPEND(delta 2)
```

Material change / Focus switch:

```text
RESET(epoch 11, new snapshot)
```

Verify:

```text
sequence monotonic within epoch
old-epoch APPEND rejected/ignored by contract
consumer final InvocationSpec remains consumer-owned
no arbitrary JSON patch operations exist
```

### 8.26 Steward acceptance

With Steward configured:

```text
deterministic candidate plan
-> Steward proposal/synthesis
-> output schema validation
-> all cited/refined cognitive sources validated
-> final Projection
```

With Steward unavailable:

```text
same request
-> deterministic Projection path succeeds
-> degradation records Steward absence
```

Steward direct mutation attempts are structurally impossible because ModelRuntime returns typed proposals only.

### 8.27 Heptalogos full interaction acceptance

Cross-repository later integration proof:

```text
1. canonical MessageFact commits in Heptalogos
2. Reaction obligation exists independently
3. cognitive observation obligation calls RecordObservation(request_id)
4. duplicate WorkItem retry produces no duplicate Occurrence
5. Reaction obtains Nous CognitiveProjection when available
6. Heptalogos composes final ContextProjection / PromptProgram
7. subject.primary produces BehaviorIntent proposal
8. deterministic Review / DecisionCommit remain Heptalogos-owned
9. reply/silence processing follows Heptalogos spine
10. ReportUse records Exposed/Referenced/ActedOn as actually observed
```

Assert explicit non-events:

```text
MessageFact did not become Memory automatically
retrieval did not reinforce automatically
CognitiveProjection did not become DecisionCommit
Nous did not create EffectOperation
Heptalogos did not mutate Memory tables
```

### 8.28 Heptalogos degradation acceptance

Kill/disable Nous Core before Reaction projection.

Expected:

```text
Nous capability unavailable/degraded
Heptalogos Basic Subject ContextProjection still forms
subject.primary can continue under Basic profile
Behavior Authority remains usable
cognitive observation WorkItem remains retryable independently
```

This preserves existing Heptalogos Subject Base semantics.

### 8.29 Feedback failure acceptance

After Heptalogos DecisionCommit:

```text
ReportUse fails / Nous unavailable
-> DecisionCommit remains canonical
-> outbound/behavior path is not rolled back
-> feedback may be retried only according to its own delivery semantics
```

No cross-system transaction.

### 8.30 Client/browser transport acceptance

Without building a GUI, prove a browser-compatible generated transport against public Core where environment permits, or at minimum a Connect-Web conformance/example harness.

The contract must support a later web/Tauri/Electron UI without a second backend API.

Do not treat a future GUI framework selection as a Core blocker.

### 8.31 Source-less / packaging boundary

The first architecture rebase does not need to claim final installers, but the Spec must avoid a topology that makes packaging impossible.

Required future-facing seams:

```text
Core knows Kernel executable path through bootstrap config/packaging resolver
Kernel does not assume Cargo workspace layout at runtime
TS generated code is normal build output
Rust generated protocol code is compiled into Kernel
PostgreSQL/object-store roots remain explicit
```

Actual source-less Windows/macOS/Linux product qualification remains a later property gate unless the implementation wave explicitly claims it.

### 8.32 Cross-platform process mechanics

Use standard Node/Rust process behavior only in the first wave:

```text
loopback TCP
stdio/inherited liveness
process signals/termination
explicit executable path
```

Avoid Unix-domain-only semantics in the architectural core so Windows remains first-class.

If later evidence makes named pipes/UDS worthwhile, transport adaptation remains local.

### 8.33 Performance risk analysis

The new process boundary adds serialization/RPC overhead, but it is deliberately placed around coarse operations:

```text
query
projection source fetch/materialization
Memory mutation
Observation admission
checkpoint
```

It is not placed inside:

```text
candidate scoring loops
Wave iteration
Tantivy lookup internals
USearch ANN loop
petgraph traversal inner loops
SQL row-by-row calls
```

Therefore transport overhead should remain dominated by cognitive/model/database work.

The first Spec should include simple process-level latency/throughput characterization for representative query/list operations, but not create a benchmark bureaucracy or hard production SLO without evidence.

### 8.34 Memory GUI performance constraints

Management interfaces should avoid accidental N+1 behavior.

Examples:

```text
ListMemories returns summaries in one bounded owner query
Memory detail returns current head + bounded relation/evidence summary
full evidence materialization is explicit
ListMemoryRevisions paginates/history reads independently
Neighborhood is bounded by max nodes/depth
```

Do not build a GraphQL-style general query system to solve GUI overfetching.

### 8.35 Public API introspection/documentation

Protobuf descriptors are the machine-readable API source.

Use Buf/Protobuf documentation tooling where useful; do not manually duplicate every message field into separate hand-maintained OpenAPI schemas.

If an HTTP artifact endpoint is not Protobuf RPC, document that narrow data-plane route explicitly and keep the Client wrapper authoritative for normal callers.

### 8.36 No automatic OpenAPI duplicate for Connect

The GUI/Client requirement does not require a parallel REST/OpenAPI API.

If a third-party integration later has a concrete REST-only need, derive/proxy it from the canonical application contract rather than maintain independent semantic handlers.

First wave ships one public RPC contract plus the narrow byte-stream HTTP plane.

### 8.37 What the Nous Wave implementation Spec must contain

The Spec should be decision-complete on:

```text
repository tree changes
exact public/private proto services/messages needed for current capabilities
code generation commands and generated ownership
Core/Kernel boot/health/auth handshake
Rust module migration mapping
public Client API surface
Observation idempotency
Artifact/Observation split
Memory pagination/concurrency/error semantics
resource resolver correction
Focus/Projection/Context checkpoint schemas
ModelRuntime role bindings and provider mechanics boundary
Steward proposal boundary and deterministic fallback
configuration ownership split
removal of obsolete Axum/public provider paths
verification/acceptance scenarios
knowledge-document updates
```

It should **not** merely say “use Connect/Tonic/AI SDK” without file paths, responsibility mapping and closure scenarios.

### 8.38 What the Nous Wave implementation Spec must NOT authorize

```text
GUI implementation
remote multi-user management
OAuth/OIDC/RBAC platform
scheduler/cron/self-wakeup
universal background job framework
new DI/container framework
generic plugin marketplace
Kernel -> Host callback RPC
second model-provider implementation in Rust
parallel REST API
compatibility shim for current Axum routes
multi-foreground Focus
automatic model benchmark/router
generic CRDT/diff language
generic idempotency platform
GraphQL/general query layer
server-scale distributed retrieval backend
Persona/Relationship concrete implementation unless separately researched/authorized
```

These exclusions are trajectory controls, not claims that the features can never exist.

### 8.39 Spec decomposition decision

One **Nous Wave Architecture Rebase Implementation Spec** is appropriate if it is written as a dependency DAG of executable capabilities A-E, not as one giant undifferentiated checklist.

It may authorize multiple commits/PR slices, but all are governed by one current architecture rebase because the protocol/process boundary and Host cognition runtime are tightly coupled design changes.

Heptalogos integration should be a separate subsequent Spec in the Heptalogos repository. The Nous Spec must leave a stable alpha Client contract and integration acceptance fixture for it.

This prevents cross-repo partial completion from turning into a third de facto architecture.

### 8.40 Implementation stop points / PLAN_GAP triggers

The Coding Agent should stop and report a narrow `PLAN_GAP` only if execution proves one of these assumptions false:

```text
Connect-Node cannot reliably call the selected stable Tonic gRPC surface
Buf codegen cannot produce the required TS/Rust outputs without unsupported tooling
Node child-process/liveness route cannot provide bounded Kernel ownership on a supported platform
existing Rust retrieval requires synchronous external/model callback in a way the planned Host-orchestrated materialization cannot replace
Memory head concurrency cannot be fenced without changing a deeper semantic invariant
artifact streaming cannot remain bounded without a different data-plane protocol
```

Normal API naming, file placement or adapter details are not PLAN_GAPs if they preserve the frozen owner/boundary decisions.

### 8.41 Architecture decisions now LOCKABLE / safe to freeze for execution

```text
PRODUCT
  Nous Wave = independent Subject cognition system
  Core + official Client is required product shape
  GUI is external Presentation

BOUNDARY
  Nous owns cognition
  Heptalogos owns agency / behavior Authority
  shared SubjectId is correlation, not shared transaction/Authority

PROCESS
  TS Cognitive Host + Rust Cognitive Kernel
  Core normally supervises Kernel sidecar
  no native-addon whole-Kernel embedding

PROTOCOL
  Protobuf + Buf
  public Connect
  private standard gRPC/Tonic
  public/private v1alpha1 namespaces

CLIENT
  @nous-wave/client is official programmatic entry
  Heptalogos and future GUI consume it

KERNEL
  Authority / SQL / object store / retrieval / serving / evidence / runtime persistence
  no external model/provider SDK long-term

HOST
  public API / Focus / Projection / Steward / Managed Cognitive Context / ModelRuntime / config resolution

MODEL
  one TS model-provider substrate
  AI SDK Core mechanics
  proposal-first
  deterministic fallback

RUNTIME
  current Rust Session/Resident/use mechanics retained
  Focus policy in TS
  narrow versioned CAS RuntimeCheckpointStore when consumed

MANAGEMENT
  GUI-complete Memory administration through public Client
  page-token lists
  Memory etag/revision fence
  explicit semantic operations

MATERIAL
  Artifact persistence separated from Observation
  byte data plane uses standard streaming

OBSERVATION
  durable request_id idempotency for cross-system retry
  external_object_ref is not idempotency identity

RESOURCE
  Resource awareness remains Authority
  in-process Rust resolver registry is not preserved across process boundary
  unresolved external current authority is expressed as ResourceActionSuggestion/partial cognition

INTEGRATION
  canonical Heptalogos MessageFact precedes Nous Observation
  Observation obligation does not block Basic Reaction
  Projection is a Context facet
  feedback follows actual use and cannot roll back behavior
```

### 8.42 Decisions intentionally still OPEN after research

These do not block the architecture rebase Spec:

```text
exact future GUI framework (React/Tauri/Electron/etc.)
remote auth/principal model
multi-user / remote management
resumable multi-GB upload protocol beyond normal HTTP range/streaming
future Core-side Resource resolver provider architecture
multi-conversation Session policy
multi-foreground Focus
Steward-assisted automatic Focus segmentation
automatic model routing/benchmarking
long-running-operation framework
live push/event streaming for GUI
generic configuration mutation API
stable v1 compatibility epoch
server-scale/distributed retrieval backend
concrete Persona / Relationship / Epistemic / Goals domain models
cross-product coordinated backup/restore
final product installer/service packaging
```

OPEN means intentionally unneeded for the next implementation, not forgotten.

### 8.43 Research-derived changes to current repository truth

The upcoming Spec must explicitly supersede these current physical claims:

```text
DECISIONS D12: Rust runtime/core implementation LOCKED
-> superseded by polyglot Core physical architecture

ARCHITECTURE / D4 physical modular-monolith/static-composition statement
-> semantic MicroSystem model retained
-> physical process architecture becomes Host + Kernel
```

It must **not** discard the valid semantic content around Subject Core, Cognitive Runtime, Memory ownership, provenance, query intent, retrieval evidence, and projection separation.

This is an architecture evolution, not a repository reboot.

### 8.44 Final migration safety principle

At every migration point, ask:

```text
Is this code being moved because its semantic owner changed,
or rewritten merely because the language/process boundary changed?
```

If the second answer is true, preserve it behind the new adapter unless concrete evidence requires a rewrite.

This applies especially to the already-corrected retrieval algorithms, Authority schema semantics, Memory revision model and serving projection machinery.

### 8.45 Final end-to-end architecture simulation

Normal Heptalogos message path after both Specs:

```text
Heptalogos Driver / Subject Chat
-> canonical MessageFact
-> independent Reaction WorkItem
-> independent CognitiveObservation WorkItem

CognitiveObservation WorkItem
-> @nous-wave/client RecordObservation(request_id)
-> Nous Core
-> private Kernel gRPC
-> durable ObservationOccurrence
-> runtime admission / serving invalidation
-> response / idempotent retry safety

Reaction
-> base Heptalogos situation
-> @nous-wave/client BuildProjection
-> Host Focus/Projection planner
-> optional ModelRuntime/Steward
-> private Kernel Query/materialization
-> CognitiveProjection
-> Heptalogos ContextProjection
-> PromptProgram / InvocationSpec
-> Heptalogos AIRuntime
-> BehaviorIntent
-> deterministic Review
-> DecisionCommit
-> CommunicationCommit / Expression / Effect path as applicable

post-use
-> @nous-wave/client ReportUse
-> cognitive use event / residency updates
-> no retroactive behavior transaction
```

Normal GUI path:

```text
GUI
-> @nous-wave/client
-> Nous Core public Connect
-> Memory/Subject/Cognition/Material/System service
-> private Kernel gRPC when Authority/data mechanics are required
-> structured result/error/status
-> GUI presentation/cache
```

No layer in this flow requires the GUI or Heptalogos to understand PostgreSQL, Tantivy, USearch, Tonic, SQLx, model-provider SDK internals, or Rust domain objects.

### 8.46 Pre-Spec completion verdict

All architecture-significant implementation areas required for the next Nous Wave Spec now have:

```text
semantic owner
mechanics route
process boundary
state placement
failure owner
degradation behavior
migration direction
acceptance scenario
explicit non-goals
```

The remaining OPEN items are either future product capabilities or exact dependency pins that can be refreshed immediately before Coding execution.

There is no remaining known architecture-level ambiguity that requires another broad research cycle before drafting the Nous Wave Architecture Rebase Implementation Spec.

**Stage 8 status: CONVERGED.**

---

## 9. Final status

```text
Stage 1 Protocol / Core-Client spine                CONVERGED
Stage 2 Management Contract / official Client      CONVERGED
Stage 3 Session / Focus / Projection                CONVERGED
Stage 4 Model Runtime / Steward / Context           CONVERGED
Stage 5 Config / recovery / transport / operations  CONVERGED
Stage 6 Heptalogos integration                      CONVERGED
Stage 7 implementation topology / migration         CONVERGED
Stage 8 Spec readiness / acceptance                 CONVERGED
```

**Pre-Spec implementation research is COMPLETE.**

Recommended next artifact:

```text
Nous Wave Architecture Rebase Implementation Spec
```

After its public `@nous-wave/client` alpha contract and Core capability are implemented and verified:

```text
Heptalogos Nous Cognition Integration Spec
```

No further broad architecture exploration is required before the first Spec unless new contradictory evidence or a new product requirement changes one of the frozen boundaries.
