# 05 — Recall, Cognitive Work Cycle, Association, and Effort

## 1. Recall is a cognitive operation

Recall reconstructs useful cognitive material under current intent and working context. It is not synonymous with vector search.

```text
Structured Recall Intent
+ Working Cognitive State
        ↓
Planner
        ↓
selected candidate channels
        ↓
progressive retrieval / association expansion
        ↓
ranked typed cognitive results
        ↓
inspect / follow / expose
        ↓
Cognitive Work Cycle evolves
```

## 2. Structured Recall Intent

Required public structure:

```text
RecallIntent
├─ target
├─ objective
├─ temporal perspective
├─ cues
├─ hard constraints
├─ result need
├─ requested effort
└─ cycle context/reference
```

Query text is optional but normally present.

## 3. Target

Targets may include:

- explicit object/source/artifact references;
- entity/concept references;
- events/episodes;
- relations;
- free semantic target description.

Caller-known IDs/semantics are preserved rather than forced back into natural-language query text.

## 4. Objective

Support at least:

```text
CURRENT
HISTORICAL
EVOLUTION
TRANSITION
EPISODE_RECONSTRUCTION
EXPLANATION
COMPARISON
VERIFICATION
ASSOCIATIVE_RECOLLECTION
```

Objectives guide planning but do not expose physical search-engine knobs.

## 5. Temporal perspective

The caller can express:

- event/world time range or point;
- observation time range;
- “what was known/remembered by time X” when supported by object history;
- approximate/open ranges.

Planner must not replace temporal validity with simple recency scoring.

## 6. Cues versus constraints

Cues seed activation/search; constraints define admissibility.

Examples:

```text
cue.entities = [Alice]
    means start from Alice

constraints.scope = project-X
    means exclude results outside project-X
```

Do not turn every tag/entity cue into an exact filter.

## 7. Requested Recall Effort

Public semantic levels:

```text
light
normal
deep
maximum
```

Meaning:

- `light`: credible quick recollection; little proactive broadening;
- `normal`: ordinary cognitive workload; limited broadening when evidence is insufficient;
- `deep`: actively explore additional semantic/temporal/association routes and conflicts;
- `maximum`: strongest bounded retrieval/verification permitted by configured resource budgets.

These are not aliases for fixed `top_k`, hop depth or ANN parameters.

## 8. Progressive planner

The planner starts with routes expected to have highest utility per cost and expands only as needed.

Typical examples:

```text
explicit ref -> exact lookup
rare identifier -> lexical/entity
known temporal episode -> temporal/entity
semantic recollection -> dense + lexical/sparse
associative recollection -> association + semantic seed
verification -> direct sources + contradictory/supporting evidence
```

Do not run every channel on every call.

## 9. Candidate channels

Current Memory should implement or compose:

- exact/reference;
- source/artifact lookup;
- entity/concept;
- lexical;
- dense semantic;
- temporal;
- episodic;
- relation;
- associative activation.

Sparse learned retrieval is an OPEN extension when a selected model/mechanic justifies it.

## 10. Fusion

Use rank-oriented fusion that tolerates score incompatibility across heterogeneous channels.

RRF is the recommended initial baseline unless implementation evidence shows a simpler/better fit.

Avoid hand-maintained weighted-score soup across unrelated score scales.

Direct deterministic matches may receive explicit precedence rather than being diluted into fusion.

## 11. Reranking

Rerank only a bounded candidate set when ambiguity justifies it.

Skip neural reranking for exact/current high-confidence results.

Model identity/revision is recorded in the effort trace/provenance when used.

## 12. Cognitive Work Cycle

A Work Cycle represents one bounded higher-level reasoning/problem context that may perform multiple recalls.

```text
begin_cycle(intent/context)
    ↓
recall
    ↓
inspect/follow
    ↓
refine intent
    ↓
continue recall
    ↓
close/abandon
```

Minimum identity: `cycle_id`.

The current Memory implementation does not turn Work Cycle into a generic workflow engine.

## 13. Working Cognitive State

Process-local state may include:

```text
active entities/concepts
prior intents
candidate pool/frontier
inspected objects
exposed objects
visited association nodes/edges
activation mass/scores
semantic neighborhoods
active temporal windows
exhausted/negative regions
unresolved targets
```

This state is short-lived and reconstructable from durable events only where such reconstruction is genuinely useful.

Do not persist every ephemeral ranking detail by default.

## 14. Incremental recollection

Later recalls in the same cycle reuse prior work where semantics remain valid.

Goals:

- avoid repeated embedding/index work;
- continue graph expansion from useful frontier;
- avoid resurfacing exhausted candidates;
- preserve activated entities/context;
- reduce context waste;
- improve multi-hop recall.

This is not a naive query-string cache.

## 15. Association projection

Memory maintains a heterogeneous **Derived Association Projection** optimized for recollection.

Possible node categories include:

- Episode;
- MemoryItem;
- Entity/Concept;
- Source/Artifact references where useful.

Edges retain type/source evidence such as:

```text
explicit relation
ordered/narrative co-occurrence
shared episode
meaningful co-recall
followed association
usage-learned link
temporal adjacency
```

Semantic vector similarity alone must not create permanent experiential association.

## 16. Competitive spreading activation

The primary initial algorithm is bounded competitive activation inspired by the VCP/Memoria lineage but specified by invariants rather than legacy code.

Core properties:

1. directed association evidence can preserve narrative/causal order;
2. each active node has a bounded outgoing activation budget;
3. neighbors compete for that budget;
4. generic high-inbound hubs are corrected/suppressed;
5. immediate backtracking is penalized, not necessarily forbidden;
6. propagation has finite work/hop/state budgets;
7. activation retains path/provenance support;
8. multiple independent seeds may reinforce support without creating unbounded mass.

A conceptual transition:

```text
conductance(i,j)
    = bounded_compressed_evidence
      * edge_type_factor
      * target_specificity
      * contextual_factor

transition_mass(i,j)
    = outbound_budget(i)
      * normalized_conductance(i,j)
      * backtrack_factor
```

Do not freeze historical VCP constants as cognitive law.

## 17. Association algorithm seam

Keep the propagation implementation replaceable behind a narrow typed Memory-internal contract because algorithm evolution is an explicitly known research seam.

The seam must express real association/activation inputs/outputs, not a universal plugin API.

PPR/diffusion is a known challenger/secondary method. It is not required as a fixed stage of the first production pipeline unless the implementation plan explicitly adds it for a demonstrated reason.

## 18. Accessibility integration

Recall ranking may consider accessibility after factual/temporal admissibility.

Accessibility affects likelihood/ease of recollection, not whether a source/history is true.

Strong explicit references can bypass low ordinary accessibility.

## 19. Diversity and inhibition

Avoid returning near-duplicate branches/candidates simply because one dense neighborhood dominates.

Initial implementation may use bounded per-source/per-episode dedup and MMR-like diversity mechanics.

Full lateral inhibition remains OPEN research and is not required solely because cognitive-science papers use the term.

## 20. Typed result

Each result contains at least:

```text
object/reference identity
typed kind
content/projection appropriate to request
source/provenance refs
temporal fields
origin/semantic/epistemic classification
ranking/support explanation in structured form
accessibility/retrieval metadata where useful
```

Do not expose provider-specific ANN internals in the normal AI-facing result.

## 21. Inspect and follow

Recall returns bounded summaries/results. `inspect` obtains more evidence/detail for selected objects.

`follow` or a new recall intent may seed from an association/object discovered in the cycle.

These operations generate meaningful cognitive-use events.

## 22. Exposure and use events

Record stages separately:

```text
SURFACED
INSPECTED
EXPOSED_TO_CONTEXT
FOLLOWED
REFERENCED_OR_ACTED_ON
ABANDONED
SUPPRESSED
CYCLE_RESOLVED
CYCLE_INSUFFICIENT
```

The host should explicitly report context exposure/outcome where it knows the fact.

## 23. Reinforcement rule

Never strengthen an association or memory merely because the retrieval engine returned it.

Meaningful reinforcement may use:

- explicit follow;
- repeated useful co-recall;
- selected/exposed context that leads to a subsequent useful operation;
- explicit caller feedback.

Keep the raw events so future learning algorithms can be changed without losing history.

## 24. Recall effort trace

Record a vector/trace rather than inventing one opaque effort score:

```text
recall rounds
candidate channels invoked
index queries
candidates examined
repeated candidates
temporal expansions
association nodes activated
edges visited
max graph depth
reranker calls
evidence inspections
model-assisted calls
CPU time
wall time
```

Separate logical effort from physical cost.

## 25. Required-effort research metric

For evaluation, estimate the minimum requested effort that retrieves a target under a defined cue/context/usefulness criterion.

This supports operational study of accessibility/forgetting without claiming the number is an intrinsic property of the memory.
