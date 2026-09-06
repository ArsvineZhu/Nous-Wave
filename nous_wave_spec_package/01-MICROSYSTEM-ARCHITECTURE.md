# 01 — MicroSystem Architecture

## 1. System definition

Nous Wave is a long-lived cognitive substrate that can be embedded beside or accessed by another AI/agent/application runtime.

It does not own generic agent behavior, tool execution, chat transport, browser automation, task scheduling or product UI.

```text
Host / Agent / Application
        │
        ├─ creates subject / supplies Character Seed
        ├─ submits cognitive material
        ├─ invokes recall/inspect/correction/etc.
        ├─ decides when offline-capable operations run
        └─ consumes cognitive projections/results
        │
        ▼
┌──────────────────────────┐
│        Nous Wave         │
│                          │
│ Subject Core             │
│ MicroSystem composition  │
│ Memory (current)         │
│ Future cognition systems │
└──────────────────────────┘
```

## 2. Why MicroSystems

The cognitive problem space contains owners with genuinely different semantics and research lifecycles:

- Memory and recollection;
- Self/Persona;
- Social World;
- Epistemic/Belief state;
- Goals/Commitments;
- Reflection;
- Diary;
- Dream/Simulation.

These must not be collapsed into one universal `MemoryRecord` or `CognitiveObject` JSON store.

At the same time they do not justify network microservices. The default topology is one process and one canonical database with clear internal ownership.

## 3. MicroSystem contract

A MicroSystem has:

```text
stable capability identity
semantic owner
owned durable state, if any
read/write operation contracts
explicit required and optional dependencies
readiness/status projection
configuration namespace
provenance/evidence behavior where applicable
```

It does not automatically have:

```text
separate process
separate database
separate HTTP server
background scheduler
plugin manifest
runtime dynamic loading
ABI compatibility layer
universal event bus
```

## 4. Composition model

Composition is static/startup-time for the current architecture.

The reference process creates Subject Core and enabled MicroSystems from configuration. Enabled systems register typed capabilities into the composition root.

The implementation SHOULD use ordinary Rust composition/types rather than building a generic DI framework.

A small typed readiness map is justified because hosts must be able to ask what cognition is available.

Example:

```text
subject.core       READY
memory             READY
persona            UNAVAILABLE
social             UNAVAILABLE
epistemic          UNAVAILABLE
reflection         UNAVAILABLE
```

## 5. Required versus optional

### Required

`subject.core`

It owns the minimum persistent identity necessary for a subject to exist.

### Current reference profile

`memory`

Memory is complete in this wave but architecturally optional. A host can create a Subject without Memory if it deliberately configures a minimal profile.

### Future optional systems

```text
persona
social
epistemic
goals
reflection
diary
simulation
```

No empty crate/package or fake implementation is created for these names.

## 6. Dependency behavior

A MicroSystem declares only real semantic dependencies.

Examples:

- Memory depends on Subject Core and shared Cognitive Material/Artifact facilities.
- A future Diary system will probably consume Memory and optionally Persona/Social/Epistemic context.
- A future Dream system may consume Memory association/recall but is not required by Memory.

When an operation requires an unavailable dependency:

```text
operation -> unavailable(required capability)
```

Do not invent fallback cognition.

The whole Nous Wave process may remain READY in a reduced capability profile.

## 7. No autonomous clock ownership

Nous Wave may read caller-provided or system wall-clock time as data, but does not turn time into autonomous intent.

Bad:

```text
03:00 -> automatically dream
midnight -> automatically diary
6h elapsed -> automatically consolidate
```

Correct:

```text
host scheduler/user/agent
        ↓
invoke consolidate / future diary / future simulation
        ↓
Nous Wave executes bounded cognitive operation
```

The current Memory service MAY run internal implementation work necessary to complete an explicitly requested operation or accepted ingest obligation, but that is execution mechanics, not autonomous cognitive scheduling policy.

## 8. Online and offline-capable operations

The classification is about caller latency, not autonomy.

### Online-oriented

- ingest small material;
- recall;
- inspect;
- continue a work cycle;
- expose/use feedback;
- direct correction/suppression queries.

### Offline-capable

- large-file derivation;
- embedding generation;
- index rebuild;
- memory consolidation;
- export/import;
- purge;
- future reflection/diary/simulation.

The caller decides when to invoke these and whether to wait synchronously.

## 9. Data ownership

Shared infrastructure exists only where multiple systems genuinely need identical mechanics:

```text
Subject identity / Character Seed
Cognitive Material provenance
Artifact repository
revision primitives
model-service contracts
operation/correlation identity
```

A future MicroSystem owns its semantic state rather than placing everything into Memory.

Memory does not own future Persona traits, Social relationships, or Epistemic beliefs simply because they can be remembered.

## 10. Process/deployment topology

### Primary local profile

```text
Nous Wave process (Rust)
    │
    ├─ Subject Core
    ├─ Memory MicroSystem
    ├─ process-local Working Cognitive State
    │
    ├─ PostgreSQL 18.x
    ├─ LanceDB embedded projection
    └─ OpenDAL -> local CAS directory

External model endpoint(s)
    ├─ embedding
    ├─ reranking
    └─ structured generation when needed
```

No Redis, Kafka, service mesh or container orchestrator is required.

### Future server profile

The same semantic service may use server-scale retrieval/object backends. Server deployment does not justify implementing them in the current wave.

## 11. Failure isolation

Failure should be scoped to real capability boundaries.

Examples:

- reranker unavailable: Recall can truthfully run without reranking when requested quality permits;
- embedding service unavailable: exact/lexical/temporal/association paths may remain usable, while ingest requiring new embeddings is degraded/pending;
- object repository unavailable: artifact-dependent operations fail; unrelated structured Memory reads may continue;
- future Diary unavailable: Memory remains fully functional.

Do not claim a capability is READY when its required dependency is unavailable.

## 12. Repository topology

Initial repository shape SHOULD follow real ownership, not future taxonomy scaffolding:

```text
nous-wave/
├─ Cargo.toml
├─ AGENTS.md
├─ README.md
├─ crates/
│  ├─ core/                 # Subject Core, shared identity/time/basic contracts
│  ├─ material/             # source/artifact/provenance semantics
│  ├─ object-store/         # OpenDAL-backed CAS mechanics
│  ├─ memory-domain/        # canonical Memory types/rules
│  ├─ memory-store/         # PostgreSQL repositories/migrations
│  ├─ memory-retrieval/     # planner, recall, association, LanceDB projection
│  └─ memory-service/       # Memory composition/public service operations
├─ apps/
│  └─ nous-wave/            # service/CLI composition root
├─ specs/
└─ tests/
```

Do not create `persona/`, `diary/`, `dream/` packages until implementation is actually authorized.

Package boundaries may be merged if implementation proves a split has no semantic/maintenance value; the semantic ownership boundaries remain.
