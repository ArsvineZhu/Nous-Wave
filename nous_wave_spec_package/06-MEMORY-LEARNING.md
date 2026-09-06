# 06 — Memory Formation, Learning, Feedback, Forgetting, and Consolidation

## 1. Scope

This file defines learning that belongs to Memory itself.

It does not define Persona evolution, relationship evolution, belief revision, Diary authorship or Dream learning.

Future systems may consume Memory evidence/episodes and produce their own domain proposals.

## 2. Ingest pipeline

Accepted Cognitive Material proceeds through independently observable stages:

```text
SOURCE_RECORDED
ARTIFACT_STORED
DERIVATIONS_PENDING
DERIVATIONS_READY
MEMORY_FORMATION_PENDING
MEMORY_READY
PROJECTIONS_READY
```

The exact internal state representation may be simpler, but public status must not claim semantic readiness before required work completes.

## 3. Record first, derive second

When possible:

```text
record immutable source/artifact metadata
        ↓
create durable processing obligation/status
        ↓
derive/chunk/embed/extract
        ↓
commit Memory objects
        ↓
update serving projections
```

A crash between source recording and processing must not silently lose the material.

Use the narrowest durable-work mechanic justified by this requirement; do not introduce a general workflow platform.

## 4. Deterministic preprocessing

Deterministic transformations such as canonical text normalization, supported document segmentation and content hashing should be distinguishable from model inference.

Do not modify source content in place.

## 5. Model-assisted extraction

A generation model may propose:

- episode boundaries;
- titles/summaries;
- entities/concepts;
- semantic MemoryItems;
- relation candidates;
- classification refinement.

Each proposal is schema-validated and bound to source refs + model provenance.

The model does not receive authority to purge sources, rewrite history or modify future domains.

## 6. Episode formation

Episode boundaries follow coherent experienced activity, not transport packet/message count.

Useful signals can include:

- source grouping/conversation IDs supplied by host;
- time proximity;
- participant/entity continuity;
- explicit task/tool sequence boundaries;
- model-proposed semantic cohesion.

Do not require an LLM to create an Episode when deterministic host grouping already provides a sufficient boundary.

## 7. Long source handling

Large documents/tool outputs are preserved as source Artifacts and processed into bounded derivations/sections.

Memory formation may produce:

- document-section references;
- summary representations;
- semantic MemoryItems;
- entities/concepts;
- episodes describing the reading/tool-use event.

Avoid treating all chunks as equal memories.

## 8. Tool observation learning

Tool result content retains tool/invocation provenance.

If a model extracts a semantic claim from a tool result, the claim-like MemoryItem records that it is derived from a reported tool observation.

Do not erase the distinction between:

```text
tool reported X
X is certainly true
```

## 9. Association learning

Association evidence accumulates from real memory/cognitive usage.

Separate evidence classes so learning can later tune them independently.

At minimum retain:

- explicit/narrative source association;
- episodic co-membership;
- temporal adjacency;
- cognitive-use co-recall;
- followed association;
- host-explicit link.

Do not persist vector similarity as learned experiential evidence.

## 10. Cognitive-use feedback

Memory accepts feedback tied to a Work Cycle and recalled objects.

A useful event model records:

```text
event_id
cycle_id
object refs
kind
occurred_at
context/exposure refs when supplied
causation/previous-event refs when meaningful
```

The first implementation may use simple deterministic updates from these events; raw events remain durable enough to support future algorithm changes.

## 11. Avoid self-reinforcement loops

Forbidden:

```text
retrieve A
→ increment A.weight
→ A ranks higher
→ retrieve A more
→ increment again
```

Retrieval/surfacing alone produces no strengthening.

Repeated meaningful use may improve accessibility/association, but the rule must be tied to explicit inspected/followed/exposed/useful events.

## 12. Accessibility update

Initial accessibility can combine operationally interpretable signals such as:

- time since meaningful use;
- meaningful use frequency;
- explicit importance/retention hints from authorized host input;
- consolidation state;
- suppression state.

Do not equate accessibility with truth, sentiment, relevance or generic “importance”.

The exact function is a research seam. Use one documented baseline and preserve inputs/events for later replacement.

## 13. Forgetting

Forgetting is implemented as accessibility decay / latent-state transition, not deletion.

The baseline should support:

```text
recent/frequently useful -> easier ordinary activation
long-unused -> lower ordinary accessibility
strong exact cue -> still retrievable
explicit deep/max effort -> may recover low-accessibility material
```

Do not invent a human-neuroscience claim stronger than the operational model supports.

## 14. Consolidation

Memory-native consolidation reduces fragmentation and forms more stable memory representations while retaining provenance.

Possible actions:

- merge several strongly redundant MemoryItems into a consolidated object;
- create a semantic abstraction from multiple Episodes;
- connect recurring episode patterns;
- archive/mark lower-level representations as superseded for normal recall while keeping evidence/history;
- refresh association/accessibility summaries.

Consolidation outputs are new revisions/objects with parent/support refs.

## 15. Consolidation invocation

Memory exposes an explicit operation such as:

```text
consolidate(subject, scope?, target?, budget?)
```

It never schedules itself by wall-clock policy.

The host may call it after a session, overnight, after N events, on demand, or never.

## 16. Consolidation safety

Consolidation must not:

- destroy raw evidence;
- convert narrative/simulated sources into observed truth;
- collapse contradictory sources into one fictitious certainty;
- erase source-level disagreement;
- mutate future Persona/Social/Epistemic state.

## 17. Correction

Explicit correction operations have stronger semantics than passive learning.

A correction identifies the affected interpretation/memory and new evidence/reason, then creates a superseding revision/object relation.

Corrections may lower accessibility or recall priority of superseded wrong interpretations while preserving historical traceability.

## 18. Suppression

Suppression is caller-authorized and immediately respected by ordinary recall/projection generation.

Learning does not automatically unsuppress a suppressed object.

## 19. Purge

Purge removes owned data according to `09-PORTABILITY-OPERATIONS.md` and must invalidate/rebuild derived retrieval projections.

A future Memory cannot “relearn” purged content merely from a retained serving cache; purge covers those projections.

## 20. Background execution mechanics

Large derivation/consolidation/purge/export operations may run asynchronously after explicit caller request.

The implementation may use PostgreSQL-backed durable operation/work records because crash continuation is a concrete requirement.

Do not generalize this into a scheduler or autonomous cognition runtime.

## 21. Idempotency

Ingest and processing must tolerate retries.

Use stable source/invocation/idempotency identities where the caller provides them, plus content hashes where appropriate.

Retrying the same accepted source must not create duplicate canonical evidence/memory merely because a worker restarted.

## 22. Memory completion condition

Memory is not “implemented” until the complete flow is present:

```text
heterogeneous source
→ durable artifact/provenance
→ derivation/memory formation
→ projections
→ structured recall/work cycle
→ cognitive-use feedback
→ correction/forgetting/suppression/purge
→ export/import/rebuild
```
