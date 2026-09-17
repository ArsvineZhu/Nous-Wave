# Nous Wave — Cognitive Runtime, Projection, Context Management, Configuration, and Language Architecture Notes

**Date:** 2026-09-09  
**Status:** DESIGN RECORD / RESEARCH DIRECTION  
**Repository context:** `ArsvineZhu/Nous-Wave`  
**Scope:** This document records the current design direction discussed after the corrective closure work began. It is **not** an implementation specification and does **not** claim these capabilities are already implemented.

---

# 1. Purpose

Nous Wave is evolving beyond a memory subsystem. Its natural scope is becoming a general cognitive runtime for a distributed Subject whose cognition may be consumed, maintained, projected, and acted upon by multiple models, agents, sessions, and host processes.

This design record captures the emerging direction around:

- Subject / Agent / Model / Session separation;
- Active Cognition and focus management;
- Session-scoped cognitive stewardship;
- shared cognitive projection across Memory, Persona, Relationship, and future MicroSystems;
- Assistive Cognition versus Managed Cognition;
- context compilation and cache-aware context epochs;
- model-assisted but non-model-dependent operation;
- maintenance / Dreaming semantics;
- configuration architecture;
- Agent-facing cognitive query language;
- internal model capability routing;
- and the likely move from a Rust-only codebase toward a heterogeneous Rust + TypeScript architecture.

The central design goal is:

> Nous Wave should manage cognition as a durable, structured, queryable, activatable, projectable system, while remaining usable under degraded conditions where advanced LLM assistance is unavailable.

---

# 2. Core Subject model

## 2.1 Subject is not an Agent

The Subject is a distributed cognitive identity.

A Subject may be enacted, represented, operated, or assisted by multiple models and agents at the same time.

Therefore:

```text
Subject != Model
Subject != Agent
Agent != Persona
Consumer Context != Subject Cognition
```

A single Subject may have:

```text
Expression Agent
Operator Agent
Research Agent
Planning Agent
Coding Agent
Session Steward
other future agents
```

These agents can have different:

- system prompts;
- roles;
- tool access;
- context budgets;
- cognitive needs;
- models;
- reasoning profiles;
- and projection policies.

The Subject's cognition must not be globally broadcast to every agent.

## 2.2 Subject cognition is shared; projection is consumer-specific

The durable cognitive state may contain:

```text
Memory
Persona / Self cognition
Relationship / Social cognition
Goals / Commitments
Epistemic cognition
Resources
Entities
Evidence
future MicroSystems
```

However, what one agent should receive is consumer-specific.

For example:

```text
Expression Agent
    may require:
        current Persona
        linguistic style
        relationship state
        current conversational focus
        relevant episodic memory

Operator Agent
    may require:
        current task
        permissions
        machine/project state
        procedural memory
        prior operational failures

    may not require:
        expressive Persona
```

A Persona state can be highly persistent at the Subject level while still being irrelevant to a particular consumer.

Therefore consumer projection is a semantic concern, not only a token-budget concern.

---

# 3. Active Cognition

## 3.1 Resident state must become richer than a flat ResidentSet

The current notion of a resident set is insufficient to express the different lifetimes and roles of activated cognition.

The emerging design requires a richer Active Cognition model.

A useful conceptual decomposition is:

```text
Subject-stable activation
    current self/persona state
    long-running commitments
    important relationship state

Focus-stable activation
    active topic
    current task
    current plan
    related memories/resources/entities

Working cognition
    facts currently needed for reasoning
    recent observations
    intermediate conclusions
    unresolved items

Ephemeral execution state
    one-off tool outputs
    temporary calculations
    traversal frontiers
    scratch state
```

These are not equivalent retention classes.

## 3.2 Activation should be multi-dimensional

A single scalar activation score is insufficient.

Useful independent axes include:

```text
Activation durability
    subject-stable
    session-stable
    focus-stable
    turn-local

Cognitive role
    self/persona
    task
    social
    epistemic
    procedural
    evidence
    resource
    execution

Consumer relevance
    required
    preferred
    available
    irrelevant

Activation strength
    foreground
    active
    resident
    dormant
```

Example:

```text
Current Persona State
    durability = subject-stable
    activation = active

Expression Agent
    relevance = required

Operator Agent
    relevance = irrelevant
```

This is fundamentally different from ranking all cognition by one global importance value.

---

# 4. Focus as a runtime primitive

A Session may contain one or more meaningful focus states.

A Focus represents the currently foregrounded task or topic and may include:

```text
objective
active references
working knowledge
unresolved items
continuation state
execution-local descendants
```

A Subject may simultaneously have multiple Sessions and therefore multiple Focus states:

```text
Session A
    Focus: novel writing

Session B
    Focus: Nous Wave architecture

Session C
    Focus: machine operations
```

Different agents may operate on different Sessions or Focuses concurrently.

A focus switch should not mean a full reset.

Expected semantics:

```text
foreground focus
    ↓ switch
suspended focus
    ↓
retain checkpoint / important working cognition

ephemeral execution details
    ↓
discardable
```

---

# 5. Session continuity and continuation state

Context window rollover must be distinct from Session continuity.

The following invariants should hold:

```text
Context rollover != Session reset
Context compression != Memory formation
Task handoff != durable Memory
History retrieval != reinforcement
Runtime eviction != cognitive forgetting
```

A Session should be able to survive:

- model replacement;
- provider replacement;
- context rollover;
- Steward replacement;
- process restart;
- consumer change.

Therefore the Session runtime should own explicit continuation state rather than relying on one model's prompt history.

A future concept may resemble:

```text
ContinuationCheckpoint
    session
    previous window
    current objective
    current focus
    active refs
    working knowledge
    unresolved items
    recent relevant tool outcomes
    continuation hints
    provenance
```

Its identity is:

```text
runtime-derived continuation state
```

not:

```text
durable learned Memory
```

---

# 6. Session Steward

## 6.1 Role

A Session Steward is a model-assisted cognitive maintainer scoped to one Session.

It is not:

- the Subject;
- the Memory system;
- cognitive Authority;
- or one global agent for the entire Subject.

Preferred shape:

```text
Subject
├─ Session A
│  └─ Steward A
├─ Session B
│  └─ Steward B
└─ Session C
   └─ Steward C
```

This avoids:

- cross-task contamination;
- huge global context;
- focus interference;
- unrelated working-state competition.

## 6.2 Steward workload

The Steward primarily maintains and projects active Session knowledge.

It may maintain or propose updates to:

```text
current focus
working knowledge
important current conclusions
active references
unresolved items
continuation state
```

It may answer factual questions from the Session's active knowledge projection.

A likely target model profile is:

```text
very long context
low latency
low cost
good factual retention
good structured output
low hallucination
moderate reasoning
```

Flash / Spark-class models are attractive here.

The Steward does not need frontier-level reasoning if its primary job is faithful projection and state maintenance.

## 6.3 Steward is not Authority

The Steward's model context is a derived projection.

It must not become the only copy of Session state.

Preferred relationship:

```text
Session Runtime
    = semantic/runtime state

Steward context
    = replaceable projection
```

The Steward may propose:

```text
ActiveStateDelta
FocusSuggestion
RecallRequest
ContinuationCheckpointProposal
FactualProjection
```

but the runtime validates and commits state transitions.

---

# 7. Cognitive Projection Runtime

## 7.1 Projection is shared infrastructure

Projection must not belong to Memory alone.

Future cognitive MicroSystems such as:

```text
Memory
Persona
Relationship
Goals
Epistemic
Resources
```

should all feed a shared projection layer.

Conceptual architecture:

```text
Subject Cognitive Systems
│
├─ Memory
├─ Persona
├─ Relationship / Social
├─ Goals / Commitments
├─ Epistemic
├─ Resources
└─ future MicroSystems
        │
        ▼
Cognitive Projection Runtime
        │
        ├─ deterministic selection
        ├─ retrieval / ranking
        ├─ active-state resolution
        ├─ optional Steward model
        ├─ projection synthesis
        └─ degradation
        │
        ▼
ConsumerWorkingSet
        │
        ▼
ContextContribution / Managed Context
```

This avoids building a separate projection agent for every MicroSystem.

## 7.2 Possible contributor contract

A conceptual internal contract may resemble:

```rust
trait CognitiveProjectionContributor {
    async fn contribute(
        &self,
        request: &ProjectionRequest,
        plan: &ProjectionPlan,
    ) -> Result<ProjectionContribution>;
}
```

Memory may contribute:

```text
relevant memories
procedural cognition
supporting evidence
```

Persona may contribute:

```text
current persona state
expression-relevant tendencies
current stance
```

Relationship may contribute:

```text
relationship state
social expectations
interaction-history abstraction
```

Individual MicroSystems should not decide the final physical prompt layout.

---

# 8. Assistive Cognition and Managed Cognition

## 8.1 Assistive Cognition Mode

Low-coupling mode:

```text
External Agent
    ↓
Nous Query Language / tool
    ↓
typed CognitiveQuery
    ↓
Nous
    ↓
retrieved cognition
```

Advantages:

- easy integration;
- agent-agnostic;
- compatible with Codex, OpenClaw, Claude Code, and other external agents;
- no requirement that Nous control the agent's full context lifecycle.

Limitation:

```text
existing agent context
+ old recalls
+ new recalls
+ tool history
```

can continue to accumulate.

Nous cannot remove stale material from a third-party agent's context if it only exposes retrieval tools.

## 8.2 Managed Cognition Mode

High-coupling mode:

```text
Nous / Host
    ↓
compose model-visible context
    ↓
invoke downstream agent/model
    ↓
observe output
    ↓
update Session cognition
    ↓
compose next context
```

Nous may control:

```text
retain
drop
replace
reorder
re-materialize
compress
roll over
```

model-visible interaction state.

This is explicit management of provider-visible context items such as:

```text
system/developer messages
user/assistant messages
tool calls
tool results
reasoning summaries where available
structured task state
retrieved cognition
```

It does not depend on access to private hidden chain-of-thought.

---

# 9. Context Compiler

Managed Cognition requires a physical Context Compiler.

Conceptual input:

```text
Cognitive Authority
Session Active Cognition
Conversation history
Tool history
Host instructions
Consumer profile
Provider constraints
```

Output:

```text
provider-specific model-visible context
```

The compiler may perform:

```text
deterministic selection
Steward-assisted projection
retrieval
deduplication
replacement
expiration
budgeting
cache-aware layout
```

Therefore:

```text
ContextContribution != final provider prompt
```

A later provider-specific compilation layer may produce:

```text
OpenAI Responses input
DeepSeek messages[]
Anthropic messages[]
Google content structures
other provider-specific representations
```

---

# 10. Cache-aware context management

## 10.1 Managed Context must not arbitrarily rewrite the prompt every turn

Provider prompt/KV caches often depend strongly on stable prefixes.

Therefore Managed Cognition should optimize both:

```text
cognitive quality
and
physical context stability
```

The system should avoid rewriting stable early context on every turn.

## 10.2 Stable / Epoch / Incremental layout

A promising context layout is:

```text
Stable Prefix
    system/developer core
    consumer identity
    stable Subject context
    stable tool definitions

Session Epoch Snapshot
    current focus
    active knowledge checkpoint
    important current conclusions

Incremental Tail
    recent turns
    tool results
    incremental cognition

Current Turn
```

Within one context epoch:

```text
Stable Prefix
+ Epoch Snapshot
+ append
+ append
+ append
```

At rollover:

```text
persist continuation
→ build new Session snapshot once
→ start new context epoch
→ accept one major cache miss
→ stabilize prefix again
```

This is preferable to constantly rewriting the entire prompt.

## 10.3 Semantic correctness outranks cache preservation

Cache is a physical optimization.

If cognition or authoritative instructions materially change:

```text
semantic invalidation
    > cache preservation
```

A new context generation/epoch should be published even if cache reuse is lost.

---

# 11. Context stability as an independent projection property

A useful future projection property may be:

```text
Stable
EpochStable
Incremental
Ephemeral
```

For example:

```text
Host system core
    Stable

current Persona projection
    EpochStable

current task objective
    EpochStable

recent tool output
    Incremental

temporary retrieval result
    Ephemeral
```

This is independent of:

```text
importance
activation
consumer relevance
```

These axes should not be collapsed into one score.

---

# 12. LLM-assisted projection without LLM dependency

A central architectural invariant:

> Intelligence is an enhancement layer, not an availability prerequisite.

Nous must remain functional without an LLM.

A useful conceptual degradation stack is:

```text
Level 0 — Deterministic Projection
    exact
    lexical
    dense
    topology
    typed ranking
    bounded projection

Level 1 — Model-assisted Projection
    deterministic candidate generation
    ↓
    fast Steward
    ↓
    select / organize / compress

Level 2 — Managed Cognition
    Session active state
    + all cognitive MicroSystems
    + Steward
    + Context Compiler
    ↓
    directly managed consumer context
```

Loss of the advanced model layer may reduce:

```text
semantic compression quality
proactive recall
cross-subsystem integration quality
LLM synthesis
```

but must not remove:

```text
stored cognition accessibility
exact lookup
basic retrieval
Subject isolation
provenance
Authority correctness
```

Even the degraded mode should be richer than a bare vector database because it retains structured cognition and semantic types.

---

# 13. Internal model capability architecture

Nous may use several internal models.

Do not encode the architecture as a fixed number of named Agents.

Prefer capabilities:

```text
memory_formation
memory_revision
consolidation
topology_annotation
anchor_synthesis
association_synthesis
persona_synthesis
relationship_synthesis
session_stewardship
factual_projection
projection_compression
context_projection
material_interpretation
```

A model may implement multiple capabilities.

Examples:

```text
CheapLongContextModel
    session_stewardship
    factual_projection

StrongReasoningModel
    consolidation
    persona_synthesis
    relationship_synthesis

VisionModel
    material_interpretation
```

This allows model routing to evolve independently from semantic ownership.

---

# 14. Memory maintenance and Dreaming

## 14.1 No mandatory autonomous scheduler

Automatic cognitive maintenance should not require a hidden periodic scheduler.

Preferred triggers include:

```text
explicit public API
write-time maintenance
query-time lazy maintenance
Host-triggered maintenance
```

Expensive LLM-based operations require a configured model capability.

## 14.2 Maintenance output remains proposal-first

LLM output is never Authority merely because it was generated by a model.

The pattern remains:

```text
model proposal
→ semantic validation
→ commit
```

This applies to:

```text
Memory formation
Memory revision
Integrative cognition
Tag proposal
Anchor proposal
Association proposal
Persona evolution
Relationship evolution
```

---

# 15. Automatic Tags, Anchors, and topology synthesis

Nous may use smaller models for:

```text
Tag proposal
Anchor proposal
Association proposal
semantic grouping
topic abstraction
```

These operations need not use the same model as deep consolidation.

Topology assistance is therefore a model capability, not a permanent autonomous agent.

---

# 16. Agent-facing cognitive query language

## 16.1 Agent interface should not be raw structured API only

Internally, code should use typed structures.

Externally, an Agent should have a compact cognitive query language.

Example conceptual commands:

```text
recall memory
for operator
focus current
about "postgres serving generation"
prefer procedural, episodic
fresh current
effort deep
limit 12
```

or:

```text
recall cognition
for expression
focus conversation
include persona current
about "Alice"
prefer social, episodic
effort normal
```

or concise forms such as:

```text
@nous recall why we rejected global persona injection
@nous focus current
@nous inspect active knowledge
@nous find evidence for serving generation design
```

## 16.2 Query language compiles to typed AST

The language must not become the internal storage/query protocol.

Preferred flow:

```text
Agent Query Language
    ↓ parse
typed CognitiveQuery / RuntimeOperation AST
    ↓ planner
Retrieval / Active Cognition / Resources
```

The internal system should use structured fields for:

```text
consumer
focus
task
cognitive domains
freshness
effort
evidence requirements
source scope
```

## 16.3 Retrieval planning may use multiple probe families

A high-level query may compile into:

```text
ExactProbe
PrecisionLexicalProbe
RecallLexicalProbe
SemanticProbe
TopologySeeds
ResourceProbe
Freshness constraints
Time constraints
```

The system may learn from search syntax patterns such as boosted terms or QDF-style freshness hints, but should not copy unstable literal syntaxes such as:

```text
+(term)
--QDF=5
```

as core internal APIs.

The semantic idea matters more than the string syntax.

---

# 17. Configuration architecture

## 17.1 Configurable is not the same as user-configurable

A core engineering principle:

> First make runtime policy uniformly configurable; later decide which configuration is exposed to users.

This is consistent with Heptalogos configuration philosophy.

Many values will exist internally:

```text
projection candidate limits
active cognition decay
session resident budgets
Steward token budgets
Steward context size
projection output budget
retrieval breadth
dense candidate limits
topology rounds
focus retention policy
continuation rollover threshold
context epoch threshold
cache rewrite threshold
Persona projection policy
Relationship projection policy
memory formation thresholds
consolidation evidence limits
provider timeout
provider concurrency
```

These must not be scattered as unexplained hard-coded constants.

## 17.2 Configuration layers

A likely semantic hierarchy:

```text
System defaults
    ↓
Host configuration
    ↓
Subject overrides
    ↓
Session overrides
    ↓
Consumer overrides
    ↓
Resolved Configuration
```

The exact supported layers may differ by subsystem.

The important principle is that effective values are resolved before execution.

## 17.3 Typed resolved configuration

Expected future concepts:

```text
ResolvedProjectionConfig
ResolvedRetrievalConfig
ResolvedMemoryConfig
ResolvedProviderConfig
ResolvedContextConfig
```

These are immutable snapshots after defaults and overrides have been applied.

Avoid a generic universal configuration framework if mature config/serde tooling already covers generic mechanics.

Nous should own the typed schema and semantics, not reinvent generic configuration infrastructure.

## 17.4 Configuration observability

The system should be able to explain effective configuration.

Example future CLI/API concept:

```text
nous config explain cognition.projection
```

Possible output:

```text
effective mode: assisted
source: session override
fallback: deterministic

steward:
    capability: factual_projection
    provider: provider-x
    model: flash-x

budget:
    candidates: 256
    output_tokens: 6000
```

This is especially important for:

- Coding Agents;
- Heptalogos management UI;
- debugging;
- deployment;
- reproducibility.

## 17.5 Semantic config versus implementation config

Public/product configuration should prefer semantic policy:

```text
projection.effort = normal
```

Implementation-specific algorithm configuration may still exist internally:

```text
retrieval.algorithms.wave.*
```

but should not leak into public cognitive protocol fields unless there is a real semantic need.

---

# 18. Graceful degradation

A possible degradation chain:

```text
Managed + Steward available
    ↓ fail
Managed deterministic projection
    ↓
Assistive typed retrieval
    ↓ dense unavailable
lexical / exact / topology
    ↓ provider unavailable
stored cognition + exact/basic retrieval
```

Not all levels are behaviorally equivalent.

Degradation may legitimately lose intelligence, but not core Authority correctness.

This principle should apply across:

```text
Memory
Projection
Persona
Relationship
Managed Context
internal model capabilities
```

---

# 19. Programming language architecture

## 19.1 Rust-only should no longer be the default assumption

The current repository is Rust-only, but the system now contains two classes of workloads with different language needs.

Rust remains highly appropriate for:

```text
PostgreSQL Authority
transactional invariants
object/material storage
Tantivy lexical serving
USearch dense serving
topology serving
high-throughput retrieval
EPA / Residual / Wave kernels
deterministic fallback
existing Memory commit semantics
```

Newer cognitive/control-plane capabilities are different:

```text
Session Steward
Cognitive Projection orchestration
Managed Context
provider/model routing
Agent Query Language
configuration resolution
Persona orchestration
Relationship orchestration
Dreaming/consolidation orchestration
```

These are:

- high-change;
- JSON/schema heavy;
- model API heavy;
- integration heavy;
- less performance-sensitive;
- frequently modified by Coding Agents.

## 19.2 Preferred emerging split

The current leading direction is:

```text
Nous Wave
=
Rust Cognitive Kernel
+
TypeScript Cognitive Host
```

Conceptually:

```text
Heptalogos / External Agents
        │
        ▼
TypeScript Cognitive Host / Control Plane
        │
        ├─ Model capability routing
        ├─ Provider adapters
        ├─ Session Steward orchestration
        ├─ Cognitive Projection Runtime
        ├─ Managed Context compiler
        ├─ Agent Query Language
        ├─ context epoch/cache policy
        ├─ config resolution
        ├─ Persona orchestration
        ├─ Relationship orchestration
        └─ Dreaming orchestration
        │
        ▼ typed boundary
Rust Cognitive Kernel / Data Plane
        │
        ├─ Authority Store
        ├─ PostgreSQL transaction semantics
        ├─ object/material storage
        ├─ Serving generations
        ├─ Tantivy
        ├─ USearch
        ├─ topology structures
        ├─ EPA / Residual / Wave
        ├─ candidate generation
        ├─ deterministic fallback
        └─ existing Memory invariants
```

## 19.3 Existing correct Rust should not be rewritten without cause

The current Memory, Authority, Serving, Retrieval, Object Store, and Material data-plane work should remain in Rust unless a concrete reason appears.

The language shift is primarily about changing the default for **new components**, not rewriting working code for stylistic consistency.

Principle:

```text
Existing correct Rust stays Rust.
New components are no longer Rust-by-default.
```

## 19.4 Language selection should follow ownership and workload

Do not use:

```text
important = Rust
unimportant = TypeScript
```

Instead evaluate:

```text
performance characteristics
failure semantics
data ownership
change frequency
library ecosystem
development friction
```

Persona and Relationship may be semantically important while still being natural TypeScript orchestration layers.

If later they contain genuinely hot kernels, those kernels can be moved into Rust independently.

---

# 20. Coding Agent development velocity as an architecture variable

Nous Wave is developed heavily through Coding Agents.

Rust's compile-time guarantees are valuable in domains where:

```text
transactional correctness
memory safety
concurrency correctness
native index performance
```

justify the cost.

For rapidly changing areas such as:

```text
provider integration
projection policy
prompt/context composition
query language
configuration
model routing
```

TypeScript may significantly reduce iteration friction.

This is not a concession to weak tools.

It is a legitimate total-engineering-cost consideration.

---

# 21. Python role

Current preferred role:

```text
Production runtime:
    Rust
    TypeScript

Research / experimentation:
    Python
```

Python remains appropriate for:

```text
algorithm research
evaluation
benchmarks
data analysis
notebooks
prototype model experiments
```

Do not introduce Python as a third production runtime by default.

If a future essential ML capability is only available in Python, isolate it as a specific worker rather than expanding runtime complexity preemptively.

---

# 22. Runtime and distribution topology

## 22.1 One product does not require one process or one language

Distinguish:

```text
source language
compilation unit
runtime process
distribution artifact
user-facing product
```

Nous Wave may be:

```text
Rust + TypeScript source
two runtime processes
one installer / application bundle
one user-visible product
```

There is no strong reason to require one physical executable if doing so harms architecture.

## 22.2 Preferred runtime candidate

Current leading candidate:

```text
nous-host
    TypeScript

nous-kernel
    Rust
```

with a coarse-grained typed local boundary.

Advantages:

```text
clear ownership
independent failure boundaries
easy degradation
kernel can operate without model host
simpler debugging
stronger process isolation
```

This also aligns with the principle that advanced intelligence must not be required for basic cognition availability.

## 22.3 Same-process alternative

A second candidate is TypeScript calling Rust through a native addon boundary such as N-API / napi-rs.

Potential advantage:

```text
very low call overhead
natural TS API surface
single process
```

Potential cost:

```text
Node/JS runtime lifecycle
Tokio/native lifecycle coupling
crash coupling
native packaging complexity
```

This remains a candidate, not a selected design.

## 22.4 RPC candidate

A typed local RPC boundary such as ConnectRPC / Protobuf is a strong candidate for qualification.

Desired properties:

```text
single schema source
generated Rust and TypeScript types
coarse-grained calls
local transport
stable service boundary
easy external tooling
```

The final boundary technology remains undecided.

---

# 23. Language boundary should reinforce degradation

A desirable failure topology:

```text
TypeScript Cognitive Host
    unavailable
        ↓

Managed Cognition
Steward
LLM Dreaming
advanced orchestration
    unavailable

Rust Cognitive Kernel
    still provides:
        exact lookup
        lexical/dense/topology retrieval
        stored cognition
        deterministic projection
        Authority operations
```

This is a major architectural advantage of keeping the kernel independently operable.

---

# 24. High-level language policy

Current provisional default:

| Domain | Preferred language |
|---|---|
| PostgreSQL Authority / transactions | Rust |
| Object / Material I/O | Rust |
| Lexical / dense / topology serving | Rust |
| EPA / Residual / Wave | Rust |
| deterministic fallback retrieval | Rust |
| existing Memory domain/commit semantics | Rust |
| model/provider orchestration | TypeScript |
| Session Steward | TypeScript |
| Cognitive Projection high-level orchestration | TypeScript |
| Managed Context | TypeScript |
| Agent Query Language | TypeScript |
| capability routing | TypeScript |
| new Persona / Relationship orchestration | TypeScript-first, Rust where justified |
| research / benchmark / notebook work | Python |
| future performance kernels | Rust |

This is a provisional architecture direction, not a permanent language law.

---

# 25. Emerging full architecture

The current design direction can be summarized as:

```text
                           Subject
                              │
              ┌───────────────┴────────────────┐
              │                                │
              ▼                                ▼
      Cognitive Authority               Model Capabilities
              │                                │
     Memory / Persona                   formation
     Relationship / Goals               consolidation
     Resources / Evidence               topology
     Epistemic cognition                projection
              │                         stewardship
              │
              ▼
          Session Runtime
              │
       ┌──────┼────────────┐
       │      │            │
       ▼      ▼            ▼
     Focus  Active      Working
            Cognition   Knowledge
       │      │            │
       └──────┴──────┬─────┘
                     │
              Session Steward
                     │
                     ▼
         Cognitive Projection Runtime
                     │
          ┌──────────┴──────────┐
          │                     │
          ▼                     ▼
 Assistive Cognition      Managed Cognition
          │                     │
 Agent Query Language      Context Compiler
          │                     │
          ▼                     ▼
 External Agent          Consumer Context
                                │
                                ▼
                              Model
```

Possible physical implementation:

```text
TypeScript Cognitive Host
    ├─ Steward
    ├─ Projection
    ├─ Managed Context
    ├─ model/provider orchestration
    ├─ Agent Query Language
    ├─ Persona/Relationship orchestration
    └─ config resolution
            │
            ▼
      typed local boundary
            │
            ▼
Rust Cognitive Kernel
    ├─ Authority
    ├─ retrieval
    ├─ serving
    ├─ storage
    ├─ deterministic fallback
    └─ performance-sensitive kernels
```

---

# 26. Important invariants to preserve

The following should guide future design:

```text
Subject != Agent
Subject != Model
Agent != Persona

Memory != Session state
Memory != Context
Context rollover != forgetting
Retrieved != reinforced

Steward != Authority
Steward context != Active Cognition Authority
Projection != Cognitive Authority

ConsumerWorkingSet is role-sensitive,
not only budget-sensitive.

Persona is not globally injected.
Cognition is projected according to consumer role.

LLM assistance is optional.
Basic cognition must degrade gracefully.

Model output is proposal before validation.
Dreaming is not mandatory background scheduling.

Configurable != user-configurable.
Meaningful runtime policy should have a typed config owner.

Cache is a physical optimization.
Semantic correctness outranks cache preservation.

Existing correct Rust should not be rewritten without reason.
New code should choose language by ownership and workload.
```

---

# 27. Deferred design questions

These topics require dedicated research before implementation is locked:

1. Exact Active Cognition data model.
2. Focus lifecycle and multi-focus semantics.
3. Whether Session owns exactly one Steward at a time or a replaceable Steward generation.
4. ContinuationCheckpoint schema and persistence semantics.
5. How much Session state is Authority versus runtime-derived state.
6. Exact contributor contract for Memory, Persona, Relationship, Goals, and Epistemic cognition.
7. How ProjectionPlan relates to current CognitiveQuery / QueryPlan.
8. Steward model protocol and structured outputs.
9. Context epoch lifecycle and cache metrics.
10. Provider-specific context compilation.
11. Assistive/Managed mode API shape.
12. Agent Query Language grammar.
13. Query language privilege/authorization relationship to ConsumerProfile.
14. Dreaming/maintenance public APIs.
15. Lazy query-time maintenance limits.
16. Model capability routing configuration.
17. Product versus internal configuration exposure.
18. Rust ↔ TypeScript boundary technology:
    - ConnectRPC / Protobuf;
    - N-API / napi-rs;
    - another coarse-grained IPC option.
19. Packaging and distribution topology.
20. Whether the current Rust app becomes `nous-kernel` or remains the top-level executable in degraded mode.
21. Future Persona/Relationship ownership split between TS orchestration and Rust kernel.
22. Benchmarking the quality/latency/cache tradeoff of Managed Cognition.

---

# 28. Recommended next research wave

After the current corrective closure is complete, a dedicated architecture research wave should evaluate:

```text
Active Cognition
Focus
Session Steward
Cognitive Projection Runtime
ContinuationCheckpoint
Assistive vs Managed Cognition
Context Compiler
cache-aware context epochs
Agent Query Language
model capability routing
Dreaming / maintenance invocation
configuration architecture
Rust / TypeScript runtime boundary
```

This research should produce architecture decisions before implementing Persona or Relationship at scale.

The next wave should not assume:

```text
Rust-only
one executable
one agent per Subject
one global Persona projection
Memory-only projection
LLM-required operation
append-only external-agent context
```

Those assumptions are now explicitly open for reconsideration.

---

# 29. Current directional conclusion

Nous Wave is trending toward a heterogeneous cognitive runtime with:

```text
durable Subject cognition
+
Session-scoped active cognition
+
consumer-specific projection
+
optional Session Steward
+
LLM-assisted cognitive maintenance
+
Managed Context for deeply integrated agents
+
typed Agent Query Language for loosely integrated agents
+
graceful degradation to deterministic retrieval
+
configuration-first runtime policy
+
Rust kernel + TypeScript cognitive host
```

This direction preserves the strongest parts of the current system—Authority, provenance, retrieval, serving, and deterministic fallback—while creating room for the much more dynamic model-facing cognitive orchestration required by Heptalogos.

The key design principle is:

> Nous Wave should own cognition and its projection, not merely store memories or append retrieved chunks to model prompts.
