# Mature Memory-System Design Transfer Audit

This file distinguishes research citation from actual design transfer.

Statuses:

- **PRESERVED** — semantic transfer is present in current code.
- **PARTIAL** — structure exists but key behavior is missing.
- **MISSING** — declared transfer is not implemented.
- **SUPERSEDED** — intentionally replaced by a different Nous design.
- **FUTURE** — recognized but not required by the current scope.

## 1. Letta / MemGPT — bounded context management

### Learned problem
Persistent cognitive state and the model's current context window are different layers.

### Nous adaptation
```text
Session
→ ResidentSet
→ ConsumerWorkingSet
→ ContextContribution
→ concrete consumer/model invocation
```

### Reject
- mutable prompt block as Subject Authority;
- one universal model context as cognition.

### Current state
- Session/ResidentSet: PRESERVED
- `ContextContribution` type: PRESERVED
- ConsumerWorkingSet: PRESERVED
- progressive materialization into context: PRESERVED

### Current verification
**IMPLEMENTED**

Verified behavior:
- `ConsumerWorkingSet` type and service operation;
- consumer identity/profile input without a persistent N×M activation table;
- budget-aware selection from ResidentSet + query results + resources;
- `ContextContribution` construction with provenance/materialization handles;
- no direct prompt-string composition in core.

Verified scenarios:
- same Session/ResidentSet + two different consumer profiles can yield different bounded working sets;
- projection alone does not reinforce memory;
- working set can contain refs/handles without fully materializing payloads.

---

## 2. Mem0 — distilled memory + hybrid evidence

### Learned problem
Memory should not equal transcript storage; exact/entity/lexical/semantic signals are complementary.

### Nous adaptation
- Artifact/Occurrence evidence;
- explicit/durable Memory formation;
- exact entity postings;
- Tantivy lexical;
- USearch dense;
- topology and resource lanes;
- evidence-family trace.

### Current state
**PRESERVED and corrected**

### Current verification
**IMPLEMENTED — correction, not redesign**

Verified corrections:
- field-dense lane ordering;
- evidence-family rank pollution;
- lexical-family truthfulness;
- dense candidates for derived/occurrence representations;
- modality/evidence coverage.

---

## 3. Graphiti / Zep — temporal/provenance memory

### Learned problem
Cognition needs episode/evidence lineage, valid time, observation time and non-destructive invalidation.

### Nous adaptation
- `ObservationOccurrence`;
- `occurred_at` / `observed_at`;
- `valid_from` / `valid_to`;
- revision lineage/current heads;
- supersession/contradiction;
- entity mention/binding revisions.

### Current state
**PRESERVED**

### Current verification
Verified preservation points:

Acceptance:
- contradictory/new revision never destroys original evidence;
- identity rebind preserves mention history;
- provider reinterpretation creates new derived representation identity/revision.

---

## 4. LangMem — hot cognition vs later formation/consolidation

### Learned problem
Immediate cognitive continuity and later durable memory operations are different workflows.

### Nous adaptation
```text
Observation
→ immediate Session/runtime admission

optional durable formation / interpretation / consolidation
→ separate operations
```

### Current state
**PRESERVED and corrected**

Observation commits source state and admits runtime references before optional formation and projection invalidation.

### Current verification
**IMPLEMENTED — P0**

Required order:
1. validate and record observation/evidence;
2. commit minimum durable occurrence/source state;
3. admit relevant ref(s) to Session/ResidentSet immediately;
4. return/immediately expose runtime continuity;
5. optional formation/derivation/consolidation happens separately or after runtime admission;
6. projection updates are targeted and must not delay semantic runtime admission when not required.

No internal scheduler.

---

## 5. A-MEM — dynamic structured memory evolution

### Learned problem
New experience can change current interpretation/topology of prior cognition without silently rewriting history.

### Nous adaptation
- Memory revisions;
- revisioned Tags/Anchors;
- AssociationEvidence;
- Integrative Memory;
- explicit reinterpretation/consolidation.

### Current state
**BOUNDED IMPLEMENTED**

Explicit provider proposals are validated and committed through Memory-owned formation/consolidation operations.

### Current verification
**BOUNDED IMPLEMENTED**

The bounded **consolidation proposal → validation → commit** path supports:
- creating Integrative Memory;
- proposing Tag/Anchor/Association changes;
- proposing new Memory revision only through explicit revision semantics;
- retaining original evidence/provenance.

Do not implement autonomous/background scheduling.
Do not implement a generic knowledge-graph maintenance framework.

---

## 6. Microsoft GraphRAG — global / hierarchical derived understanding

### Learned problem
Corpus-wide/global questions cannot always be answered by local top-k chunks.

### Nous adaptation
Optional Resource synopsis / integrative derived representations with provenance.

### Current state
**MINIMAL CAPABILITY SEAM IMPLEMENTED**

### Current verification
**MINIMAL CAPABILITY SEAM, not full GraphRAG**

Current capability:
- `ResourceSynopsis` / equivalent derived representation remains first-class;
- a Resource resolver can advertise synopsis/global-summary capability;
- query planner can prefer a ready synopsis for global/corpus-wide intent;
- materialization/provenance is preserved.

Not required:
- universal community detection;
- GraphRAG community report pipeline over every Memory;
- new graph infrastructure.

---

## 7. HippoRAG 2 — graph associative retrieval

### Learned problem
Associative multi-hop topology retrieval is a credible first-class retrieval family.

### Nous adaptation
Topology is a first-class candidate/addressing channel.

### Current state
**PRESERVED / superseded implementation law**

PPR itself is not required. VCP-derived bounded Wave is the current default implementation.

### Current verification
Bounded Wave retrieval preserves actual flow, field refinement and the
`AssociativeExpansion` problem-level seam.

---

## 8. LightRAG — complementary, incrementally maintained serving projections

### Learned problem
Vector and graph/topology views are complementary and can be independently maintained.

### Nous adaptation
Authority + exact/lexical/dense/topology serving generations.

### Current state
**INDEPENDENT PERSISTED GENERATIONS IMPLEMENTED**

Each serving family has independent durable generations and projection watermarks.

### Current verification
**IMPLEMENTED — P0**

Current invalidation/update ownership:

```text
ChangeSet
├── exact
├── lexical
├── dense(space)
├── topology
└── synopsis/derived where relevant
```

Each projection:
- owns its own current generation;
- can rebuild/publish independently;
- records generation identity and source Authority revision/watermark;
- is immutable once published;
- never becomes Authority.

Do not create a generic event bus.

---

## 9. MemOS — memory lifecycle as first-class managed resource

### Learned problem
Memory/cognition must retain representation identity, provenance, lifecycle and versioning across heterogeneous forms.

### Nous adaptation
- Authority / Runtime / Projection separation;
- DerivedRepresentation;
- ProducerSignature;
- EmbeddingSpaceSignature;
- revision/provenance;
- coverage/readiness.

### Current state
**PRESERVED with durable serving lifecycle**

### Current verification
Serving generations are staged, checksummed, persisted, reopened and atomically selected.

---

## 10. VCP Toolbox — cognitive retrieval algorithms

### Learned problem
- dominant semantic directions can hide weak cues;
- bounded associative spreading must control hubs/backtracking/budget;
- actual traversed flow is useful retrieval evidence;
- local and transferred topology fields can improve ranking.

### Nous adaptation
- EPA/Residual cue sensing;
- Wave;
- Query River actual flow;
- LocalField/TransferField;
- CandidateSemanticTrail/observability.

### Current state
**PRESERVED with current-wave correctness fixes**

### Current verification
**IMPLEMENTED**

Important: VCP is research lineage only. Independent Rust implementation. No source copying/transliteration.

Algorithm architecture:
```text
SemanticCueSensing trait
    default = EpaResidualCueSensing

AssociativeExpansion trait
    default = BoundedWaveExpansion
```

No public protocol fields named after implementation algorithms.

---

## 11. Transfer completion matrix

| Transfer | Current state |
|---|---|
| Letta ConsumerWorkingSet/ContextContribution | IMPLEMENTED |
| Mem0 hybrid evidence correctness | IMPLEMENTED |
| Graphiti temporal/provenance | PRESERVED |
| LangMem immediate runtime vs later formation | IMPLEMENTED |
| A-MEM bounded consolidation evolution | BOUNDED IMPLEMENTED |
| GraphRAG global synopsis seam | IMPLEMENTED MINIMAL |
| HippoRAG associative topology class | PRESERVED |
| LightRAG independent incremental projections | IMPLEMENTED |
| MemOS lifecycle/provenance | IMPLEMENTED |
| VCP algorithm family | IMPLEMENTED DEFAULT + replaceable problem-level seams |
