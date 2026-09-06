# 04 — Memory MicroSystem Domain Model

## 1. Scope

Memory owns durable experience, recollection semantics and memory-native learning dynamics.

It does **not** own future Persona traits, Social relationships, Epistemic beliefs, Goals, Diary narratives or Dream world state merely because those concepts can be remembered.

Memory may retain cognitive material about any of them and expose typed references/episodes to their future owners.

## 2. Memory is not a bag of facts

The Memory MicroSystem represents several distinct layers:

```text
Cognitive Material / Evidence
        ↓
Episodes / Experience structure
        ↓
Memory objects and interpretations
        ↓
Associations / accessibility
        ↓
Recall serving projections
```

The implementation must preserve source lineage through these layers.

## 3. Core object kinds

The current Memory wave requires at least:

### `Episode`

A bounded experienced/observed occurrence or coherent slice of activity.

An Episode may be formed from:

- one source event;
- many conversation turns;
- a tool-use sequence;
- a document-reading event;
- mixed text/media/tool material.

Minimum semantics:

```text
episode_id
subject_id
scope
valid/occurrence time when known
observation/record time
source/artifact refs
participants/entity refs when known
title/summary representation as derivation, optional
revision/head metadata
```

### `MemoryItem`

A durable recallable cognitive representation that is not necessarily identical to the raw source.

Kinds may include:

```text
EPISODIC
SEMANTIC
PROCEDURAL
CONCEPTUAL
REFERENCE
```

Do not force every item into a fact triple.

A semantic/procedural item must retain provenance to supporting source/episode/derivation history.

### `EntityRef`

Memory needs stable references for known entities/concepts so retrieval and association can connect experiences.

The current Memory owner may maintain a lightweight entity catalog required for recollection. It must not pretend to be a future complete Social identity or world ontology.

### `RelationRef`

Typed relationships needed by Memory reconstruction/association may be represented, but future domain-owned relationships remain external owners.

## 4. Stable logical identity and revisions

Each durable cognitive object has stable logical identity and immutable revisions.

```text
Object
  object_id
  object_kind
  subject_id

Revision
  revision_id
  object_id
  payload
  created_at
  temporal fields
  provenance
  content digest

RevisionParent
  revision_id
  parent_revision_id
  relation

CurrentHead
  object_id
  revision_id
```

Most histories are linear. Multiple parents are used only when consolidation/synthesis genuinely merges prior objects.

## 5. World evolution versus correction

Do not encode all change as “edit”. Distinguish at least:

- source/event remains immutable;
- an interpretation can be corrected;
- a remembered state can genuinely change over world time;
- a consolidated representation can supersede lower-level memory without erasing its support;
- serving projections can be rebuilt without semantic revision.

## 6. Time semantics

Memory must support more than recency.

Relevant axes include:

```text
world/event valid time
observation/source time
knowledge/interpretation time when applicable
recorded/system time
```

Not every object needs all axes.

Approximate time is represented as ranges/precision rather than invented exact timestamps.

## 7. Scope

Every source/memory object belongs to an explicit scope meaningful to the host, at minimum subject-private by default.

The current implementation may support namespaced scope identifiers such as conversation, workspace, project or host-defined domain, but must not invent an enterprise RBAC system.

Scope is both a provenance and recall-admissibility dimension.

## 8. Raw source and memory representation coexist

Memory results may expose:

- original source excerpts/references;
- Episode reconstruction;
- learned/consolidated MemoryItems;
- entities/relations;
- supporting evidence.

The system must not discard raw material after forming a summary unless an explicit retention/purge policy authorizes it.

## 9. Multimodal memory

Memory is multimodal at the Artifact/provenance level even when the current semantic algorithms primarily operate on text derivations.

A memory may point to:

```text
image
image region
audio segment
video segment
document section
code/file region
```

The public model must not imply all memories are strings.

## 10. Durable derivations versus serving projections

### Durable derivations

Worth retaining because recomputation may be expensive or irreproducible:

- OCR/ASR/caption outputs;
- model interpretations;
- semantic summaries;
- embeddings tied to a model revision when policy retains them;
- extracted entities/episodes;
- consolidated memory representations.

### Serving projections

Rebuildable physical indexes/layouts:

- LanceDB index layout;
- FTS index structures;
- query-local graph CSR;
- process caches;
- rerank feature caches.

`Derived != disposable`.

## 11. Association evidence

Association is a memory-native learned relation that captures experiential/cognitive linkage beyond semantic similarity.

Association evidence may come from:

- explicit source relation;
- ordered/narrative co-occurrence;
- shared episode participation;
- temporal adjacency;
- repeated meaningful co-recall;
- follow-up traversal inside a Cognitive Work Cycle;
- explicit user/host linking.

Each association retains origin/evidence class rather than collapsing immediately into an unexplained universal weight.

## 12. Accessibility

Accessibility represents current ease/probability of activation under typical cue/context conditions.

It is not a truth score and not a static property of a memory independent of context.

The system may maintain memory-level accessibility state derived from recency, meaningful use, forgetting and consolidation, but actual recall difficulty is contextual:

```text
RequiredEffort = f(memory, cue, objective, cycle state, current accessibility)
```

## 13. Forgetting

Cognitive forgetting changes accessibility or serving priority while preserving the memory and its provenance.

A forgotten memory can still return under a strong direct cue or deeper recall effort.

Do not implement forgetting as deletion or expiration by default.

## 14. Suppression

Suppression is an intentional governance/host operation that excludes an object from ordinary recall.

It is distinct from natural accessibility decay.

Suppressed items remain inspectable through authorized direct operations unless purge policy says otherwise.

## 15. Purge

Purge is physical/semantic deletion according to authority and scope.

It must trace affected:

```text
source records
artifacts when exclusively owned and purge-authorized
derivations
memory objects/revisions
associations
embeddings/search projections
cycle/use records where required
```

Do not rely on a single `DELETE memory_row` statement.

## 16. Correction

Correction creates new interpretation/memory revisions with explicit provenance and supersession.

It does not rewrite immutable source evidence.

If a source itself was erroneous, retain the record that the source reported X and record the later correction that X was wrong.

## 17. Memory formation does not equal model inference

A model may propose:

- episode boundaries;
- entities;
- summaries;
- semantic memory items;
- associations.

Model output is a derivation/proposal that must pass deterministic schema/domain validation before canonical Memory mutation.

## 18. Required result typing

Recall output must not collapse into `[{text, score}]`.

A result can identify typed material such as:

```text
Episode
MemoryItem
EntityRef
RelationRef
Artifact / DocumentSection
ConversationSlice
ToolObservation
```

Each result retains source/provenance and epistemic classification sufficient for the caller to reason about it.

## 19. Memory readiness

Memory readiness should distinguish at least:

```text
READY
DEGRADED
UNAVAILABLE
FAILED
```

Examples:

- PostgreSQL unavailable -> Memory unavailable/failed;
- LanceDB unavailable but exact/lexical fallback is valid -> degraded;
- embedding service unavailable -> ingest/search partially degraded depending on requested route;
- object store unavailable -> artifact-backed operations degraded/failed while pure structured reads may remain possible.

## 20. Non-goals

Current Memory must not silently grow into:

- Persona ontology;
- relationship scoring;
- belief truth ownership;
- diary authoring policy;
- dream generation;
- autonomous agent scheduler;
- universal knowledge graph database;
- full document-management product.
