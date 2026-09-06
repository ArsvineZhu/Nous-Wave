# AGENTS.md

Execution contract for Nous Wave.

## Project posture

```text
DevelopmentMode = RAPID_EVOLUTION
CompatibilityEpoch = PRE_PRODUCTION
CurrentWave = SUBJECT_CORE + COMPLETE_MEMORY_MICROSYSTEM
```

Existing code, tests, schemas, names, packages and development history have no preservation privilege. Rewrite directly when the current spec requires it. Do not add compatibility shims, aliases, fallback readers or dual paths without an explicit compatibility obligation.

## Executor role

You implement the active spec/plan. You are not the architecture research owner.

If execution reveals a material unresolved semantic/ownership/provider/failure decision, report `PLAN_GAP`. Do not hide uncertainty behind configuration, registries, generic interfaces or frameworks.

## Core invariants

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
```

## Current scope

Implement Subject Core and the complete Memory MicroSystem.

Do not implement Persona, Social, Epistemic, Goals/Commitments, Reflection, Diary or Dream/Simulation internals. Preserve only the documented cross-MicroSystem boundaries.

## Library-first

For generic mechanics, prefer adopted dependencies and standard facilities before custom code.

Current routes include:

- PostgreSQL + SQLx for canonical structured persistence;
- LanceDB embedded for local dense retrieval projection;
- Apache OpenDAL for object-storage mechanics;
- Axum/Tokio for service runtime;
- mature model-service HTTP clients for inference boundaries.

Do not reimplement them merely to reduce dependency count.

A narrow domain-owned processing reconciler is authorized; a generic workflow/job framework is not.

## Architecture discipline

- Domain owners own semantics; storage/index/model libraries own mechanics.
- Serving projections never become cognitive Authority.
- Preserve immutable source/provenance and revision lineage.
- Keep file/source classification axes orthogonal.
- Do not force arbitrary source material into chat-message or text-chunk shapes.
- Do not create one universal `CognitiveObject` JSON ontology.
- Do not quantify cognitive concepts without operational meaning.
- Do not add a scheduler, cron, wakeup policy or autonomous agent loop.
- Do not create empty future MicroSystem crates.

## Performance discipline

Optimize in this order:

```text
avoid unnecessary work
→ route to the correct small candidate space
→ reuse Cognitive Work Cycle state
→ improve candidate quality
→ improve data structures/indexes
→ add hot caches
→ systems-level optimization
```

Do not add distributed infrastructure to solve an unmeasured local problem.

## Tests

Tests protect meaningful contracts and observed algorithmic risks. TDD is optional.

Do not create product architecture solely for testability. Use real PostgreSQL/LanceDB integration where those mechanics are the claim being tested.

Run focused verification during iteration and the full workspace checks at meaningful stage boundaries.

## Completion

When the authorized Memory implementation and required proof are complete, STOP. Do not automatically start a second cleanup, hardening, compatibility, release or future-MicroSystem pass.
