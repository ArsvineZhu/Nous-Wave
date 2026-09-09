# Nous Wave System Description

This document explains the current system.

## 1. Process composition

Nous Wave runs as a modular monolith.

A reference process composes:

```text
Subject Core
Cognitive Runtime
Material/Evidence
Memory MicroSystem (enabled in reference profile)
Authority Store
Object Store
Serving Projections
Provider adapters
HTTP/CLI host surface
```

Memory is not the application kernel. Subject Core and Cognitive Runtime are usable owners even when Memory is absent.

## 2. Subject creation

Create Subject with:

```text
Subject identity
Character Seed
basic configuration
```

The Character Seed is stored as immutable/provenanced source material associated with Subject initialization.

The seed is not converted into current Persona state and is not copied into every Memory.

No pre-existing Memory is required.

## 3. Observation flow

An Observation can reference text, files, tool results, rich media, external objects or other material.

Normal flow:

```text
receive
→ validate
→ persist raw Artifact / ObservationOccurrence as needed
→ admit useful refs to Session / ResidentSet
→ immediate runtime continuity exists
→ optional derivation / durable memory formation
→ targeted serving-projection update
```

A provider or index failure after the occurrence is recorded must not erase runtime awareness of the just-observed material.

Observation is not synonymous with durable Memory formation.

## 4. Authority and object storage

PostgreSQL stores canonical structured Authority:

- Subjects;
- material metadata/provenance;
- source/derived regions;
- Memory/revisions/evidence;
- Tags/Anchors/Associations;
- Session/runtime durable state;
- Resource awareness;
- derivation work;
- serving-generation metadata.

Large/raw bytes live in the OpenDAL/BLAKE3 content-addressed object repository.

Serving indexes never become Authority.

## 5. Material derivation

A source region can have multiple immutable derived representations, each carrying:

- representation kind;
- producer signature;
- provider/model identity where applicable;
- preprocessing identity;
- source-region provenance;
- revision/supersession relation.

Provider replacement produces a new interpretation; historical interpretation remains inspectable.

Persisted textual surrogates of rich media can later be used by weaker runtimes.

## 6. Cognitive Runtime

### Session
Long-lived Subject continuity scope.

### ResidentSet
References currently available to the Session.

### ConsumerWorkingSet
Ephemeral bounded projection for one consumer invocation.

Working-set construction considers:
- ResidentSet;
- current query results;
- consumer capabilities;
- context budget;
- modality/materialization policy.

### ContextContribution
Structured evidence/context items contributed by Nous. The surrounding Host may perform final model prompt/context composition.

### Work Cycle
Optional call-chain-local reuse of exploration state. It is not a top-level durable cognitive owner.

## 7. Cognitive Query

The caller submits semantic intent, not physical engine commands.

The planner can combine:

- runtime/resident refs;
- exact refs;
- exact entity;
- lexical;
- dense;
- Tags/Anchors;
- associative topology;
- external authoritative Resources;
- optional reranking/materialization.

All evidence channels remain identifiable in diagnostics/ranking trace.

`CognitiveEffort` controls willingness to spend retrieval work.

## 8. Algorithm families

### Semantic cue sensing
Current implementation: EPA/Residual-derived weak-cue discovery.

Stable contract: implementation-neutral weighted cues + provenance.

### Associative expansion
Current implementation: bounded competitive Wave.

Stable contract: topology snapshot + weighted seeds + budget -> node potential + actual flow/provenance.

Implementations are replaceable behind narrow internal Rust traits. They are not public protocol identities.

## 9. Serving generations

Serving families are independent:

```text
lexical
dense per embedding space
topology
exact/postings where externalized
optional synopsis
```

A generation is immutable after publish and records:
- source Authority watermark/revision;
- implementation identity/revision;
- config digest;
- embedding-space identity when relevant;
- durable artifact manifest/checksum.

Build in staging, validate, atomically publish, then update current pointer.

Process restart reopens current compatible artifacts instead of rebuilding unconditionally.

## 10. Projection invalidation

Authority mutations produce explicit narrow invalidation.

Examples:

```text
Memory text revision -> lexical + affected dense
association update    -> topology
entity rebind         -> exact/entity + topology
embedding replacement -> affected dense space only
Tag label change      -> tag lexical/dense + topology as applicable
```

No universal rebuild is required.

## 11. Memory

Memory has:
- stable logical identity;
- immutable revisions;
- current head;
- evidence;
- semantic/epistemic/time semantics.

Memory classes currently include:
- Specific/Episodic;
- Integrative;
- Procedural/Experience.

Retrieved objects are not reinforced automatically.

## 12. Tags, Anchors and Associations

They are explicit cognitive topology objects/evidence.

Embedding similarity can suggest candidates but does not create permanent association by itself.

Anchor revisions/support/revocation remain explicit.

## 13. Consolidation and learning

Consolidation is explicitly invoked.

It may:
- create Integrative Memory;
- propose/commit Tags/Anchors/Associations;
- propose explicit Memory revisions.

Provider-assisted proposals are validated before commit.

No internal night/cron schedule exists.

## 14. Resources

External systems remain authoritative for their live/current state.

Nous stores Resource awareness and routes queries to Host-provided resolvers.

Global/corpus-wide queries may use a ready Resource synopsis/integrative derived representation when available.

## 15. Materialization

Query results may carry handles instead of eagerly copying large material.

Materialization resolves exact:
- Artifact;
- SourceRegion;
- DerivedRepresentation;
- DerivedRegion;
- Resource/external object through Host callbacks.

Bounded/partial reads are used when coordinates and backend support them.

## 16. Failure and degradation

Required capability unavailable:
- fail that operation truthfully.

Optional capability unavailable:
- continue with available lanes;
- report degradation.

Zero-model startup still supports:
- exact refs;
- entity postings;
- lexical/structured access where available;
- explicit topology;
- raw evidence/materialization;
- Session/ResidentSet;
- Resource access.
