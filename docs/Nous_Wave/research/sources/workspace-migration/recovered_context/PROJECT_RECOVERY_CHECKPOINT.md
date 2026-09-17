# Nous Wave Project Recovery Checkpoint

**Checkpoint date:** 2026-09-16
**Nature:** recovered project memory for migration; not implementation authorization.

## 1. Current project identity and posture

Nous Wave is an independent, general-purpose **Subject cognition system**. The durable semantic boundary remains: external systems own their canonical state, while Nous Wave owns the Subject's cognition of and access to that state. Heptalogos is a consumer/integration pressure test, not the identity of Nous Wave itself.

Core semantic distinctions that remain important include: `Subject != Model`, `Subject != Agent`, `Agent != Persona`, `Artifact != Memory`, `Message != Memory`, tool output is evidence rather than automatic truth, retrieval/projection does not imply reinforcement, semantic cognitive runtime is distinct from model activation cache, and runtime eviction is distinct from forgetting or purge.

## 2. Current architecture direction (newer than the old Rust-only baseline)

The first-wave implementation direction converged to:

```text
Nous Core + official @nous-wave/client
TypeScript Cognitive Host + Rust Cognitive Kernel

Public client/API: Connect over HTTP
Host -> Kernel: standard gRPC over HTTP/2
Schema authority: Protobuf
Schema/code generation: Buf v2
Public namespace:  nous.wave.v1alpha1.*
Private namespace: nous.wave.kernel.v1alpha1.*
```

The TypeScript Cognitive Host owns product-level orchestration such as Focus, Projection, Steward, Managed Cognitive Context, model/capability routing, and configuration resolution. The Rust Cognitive Kernel owns Cognitive Authority mechanics, persistence, evidence/material, retrieval/serving mechanics, topology, and runtime persistence. No long-term provider SDKs should be embedded in the Kernel.

The public product boundary is the official Client/Core contract. A future GUI is a Presentation consumer and must not directly access database, retrieval index, or Kernel internals.

The older `00-DECISIONS.md` contains a historical `D12. Rust core — LOCKED` and modular-monolith assumptions. These are **superseded by the later Pre-Spec convergence** and must not be re-promoted to current truth merely because the older file is preserved.

## 3. Cognitive Runtime / Projection / Context

Recovered current direction:

- Subject / Agent / Model / Session / Consumer are distinct concepts.
- Semantic cognitive runtime is Subject-level and model/provider-independent in semantics.
- Active Cognition includes Focus, working knowledge/residency, and continuation across model invocations.
- Consumer-specific working sets are allowed over shared Subject cognition.
- Deterministic Projection is the baseline; model-assisted cognition is optional.
- Managed Cognitive Context controls the Nous-owned cognitive context lane while the consumer retains final `InvocationSpec`/behavior authority.
- Context application is based on an epoch/snapshot model such as `RESET(epoch snapshot) | APPEND(delta)` rather than silently treating prompt text as Authority.
- Steward is session-scoped/optional refinement after deterministic planning, proposal-first, validated against source refs/schema, and does not directly mutate Authority.
- LLM-enhanced behavior must degrade truthfully when LLM capability is absent.
- No hidden autonomous scheduler is required for maintenance; invocation timing belongs to host/user/external scheduler/agent runtime.

## 4. NousQL current design direction

NousQL is an ergonomic AI-facing **Cognitive Query language**, not a universal Nous shell. It compiles in TypeScript:

```text
NousQL text
-> parser
-> typed CognitiveQuery
-> normal public/private query path
```

The Rust Kernel should not parse the user-facing DSL. Mutation/lifecycle/ingestion operations remain typed Client API operations rather than NousQL commands.

Important v0.2 decisions preserved from the current workspace file:

- Natural-language semantic cue is a first-class Query Atom, e.g. `~"Rust lifetime errors caused by borrowed values escaping scope"`.
- `&&` is hard intersection.
- `+(...)` is soft preference/ranking influence.
- No implicit AND.
- `#school` is a semantic concept; `@tag("school")` is an exact persisted Tag. They must not be silently converted into one another.
- `@anchor(...)` addresses an exact Anchor and can be combined with explicit topology exploration.
- Association is better represented as an operation/intent when there is no stable Association reference type, rather than inventing a fake `@association(id)` object.
- Preferred selector surface stays compact: `@e`, `@tag`, `@anchor`, `@r`, `@object`, `@ref`.
- `occurred`, `observed`, and `valid` are distinct time axes and must not be collapsed into one ambiguous `within`.
- Public query semantics express cognitive need/effort (for example `$effort(deep)`) rather than provider/algorithm mechanics such as “use dense/rerank/wave”.
- Capability-gated vocabulary keeps the full language broad while only teaching each model the terms supported by the current runtime.
- Scope is explicit: branch-local attachments and group/global attachments are semantically different; canonicalization may reorder typed attachments but must not rewrite scope.
- Directional relations use explicit roles rather than positional guessing.

## 5. Identity / Addressing convergence

The latest recovered identity model is:

```text
DisplayName / Alias
        -> discovery / first query
LexicalRef
        -> AI / NousQL exact continuation
Authority Identity
        -> internal storage / canonical ownership
```

Example:

```text
Alice
ent:amber-lotus-cello-river
UUIDv7 / Host EntityRef
```

Key rules:

- Name lookup is convenient discovery; LexicalRef is exact continuation.
- LexicalRef canonical syntax: `<type>:<word>-<word>-<word>-<word>`.
- Four random words are drawn from a frozen 4096-word vocabulary (48-bit candidate space), with central minting + database UNIQUE constraint + retry; probability alone is not the uniqueness authority.
- Body words are random/semantic-neutral; identifiers must not encode mutable meaning such as `alice-school-friend`.
- Lowercase ASCII and `-` are canonical.
- LexicalRef is never reused; purged objects leave tombstones.
- Nous-owned objects such as Memory/Tag/Anchor can map stable LexicalRef to internal UUIDv7 identity.
- Host-owned Entity remains Host-owned; Nous binds the LexicalRef to the Host canonical EntityRef rather than inventing a second canonical Entity identity. Resource follows the same ownership discipline.
- Content digests (SHA/BLAKE3) are for immutable payload fingerprinting, integrity, deduplication, signatures/config digests, etc.; ordinary cognitive object identity is not a content hash.
- Query canonicalization has a source form and bound form. Name resolution/binding happens in the Host before Kernel execution.

## 6. COST methodology to preserve

The project methodology is **COST Optimizes Software Trajectory**: preserve required semantics while minimizing expected total trajectory cost across implementation, verification, understanding, maintenance, rework/migration, operations, and context/token burden.

Important consequences:

- “smaller now” is not automatically cheaper overall.
- Mature dependencies/tools should replace repeated generic infrastructure when they reduce total cost without violating semantic/authority boundaries.
- Historical code, architecture, tests, or process have no automatic preservation privilege.
- Compatibility exists only for real current obligations.
- Process/verification mechanisms are justified by the uncertainty/risk they actually reduce, not by ceremony.
- Do not use hypothetical far-future needs to justify permanent present complexity.
- Executors implement decided semantics/boundaries and retain normal implementation judgment; major undecided architecture questions should be surfaced rather than hidden behind generic abstractions.

## 7. Known discovered project artifacts

The File Library search located at least:

- `Nous_Wave_Cognitive_Architecture_Baseline_2026-09-07.md`
- `Nous_Wave_Implementation_Research_Agenda_2026-09-07.md`
- `Nous_Wave_PreSpec_Implementation_Research.md`
- `00-DECISIONS.md`
- `COST.zh-CN.md`
- `Session Handoff.md` (handoff template)
- `审查执行完成Spec.txt` (two File Library objects with same filename)
- `收敛新架构实现方式.txt` (two File Library objects with same filename; one contains NousQL v0.2 design discussion, one contains Identity/Addressing convergence)

The current workspace references generated artifacts that were not physically mounted during this migration:

- `Session-Handoff-2026-09-09-20-47.md`
- `Nous_Wave_Cognitive_Runtime_Projection_Context_Language_Design_Record_2026-09-09.md`
- `NousQL_Language_Design_v0.2.md`
- `NousQL_LLM_Quick_Reference_v0.2.md`
- `NousQL_Identity_and_Addressing_Contract_v0.5.md`

These names are preserved so they can be re-found in an account export or old chat attachment history.

## 8. Migration caveat

This checkpoint is deliberately a **state-recovery document**, not a substitute for original source files. Where old and new architecture records disagree, prefer the newer Pre-Spec convergence and explicit later decisions. Do not treat historical files as current implementation authorization merely because they survive in the migration archive.
