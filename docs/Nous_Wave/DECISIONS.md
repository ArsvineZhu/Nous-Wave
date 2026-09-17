# Current Decision Ledger

This ledger prevents implementation defaults from masquerading as permanent cognitive theory.

## Status vocabulary

- **LOCKED** — current semantic/architectural baseline. Reopen only with concrete contradictory evidence.
- **DERIVED** — follows strongly from locked semantics.
- **DEFAULT** — selected implementation route for now; replaceable when real evidence justifies replacement.
- **OPEN** — deliberately unfrozen.
- **FUTURE** — recognized area, not authorized for internal implementation now.

A separate execution state applies during a coding wave:

- **FROZEN_FOR_CURRENT_WAVE** — the Coding Agent must implement the chosen route and must not reopen it during execution.

A decision may therefore be:

```text
ArchitectureStatus = DEFAULT
ExecutionStatus = FROZEN_FOR_CURRENT_WAVE
```

This is not equivalent to `LOCKED`.

## Semantic decisions

### D1 — Project identity — LOCKED
Nous Wave is an independent general-purpose Subject cognition system.

### D2 — Subject initialization — LOCKED
`Subject identity + Character Seed + basic configuration` is sufficient.

### D3 — Character Seed lineage — LOCKED
Seed is retained with provenance/history and remains distinct from evolved Persona/Self cognition.

### D4 — MicroSystem architecture — LOCKED
Semantic owner + typed operations + explicit dependencies + readiness. Static MicroSystem composition remains; the authorized Architecture Rebase adopts a TypeScript Host supervising a Rust Kernel process.

### D5 — Required core vs optional cognition — LOCKED
Subject Core and Cognitive Runtime are required. Memory is optional and enabled in the reference profile.

### D6 — No internal scheduler — LOCKED

### D7 — Current feature scope — LOCKED
Fully implement Memory. Other cognitive MicroSystems remain FUTURE boundaries only.

### D8 — Cognitive material is source-agnostic — LOCKED

### D9 — classification axes remain orthogonal — LOCKED
Origin/source, semantic class, epistemic class and representation/media must not be conflated.

### D10 — Artifact != cognitive Memory — LOCKED

### D11 — tool/system output is evidence, not automatic truth — LOCKED

### D12 — TypeScript Cognitive Host + Rust Cognitive Kernel — LOCKED for current implementation wave
The 2026-09-16 Architecture Rebase authorization supersedes the Rust-only physical direction. Rust retains Authority, retrieval, serving and reliable persistence; TypeScript owns public Core/Client orchestration, Focus/Projection/Context, model invocation and NousQL. Execution details and pending acceptance are in [IMPLEMENTATION_SPEC.md](IMPLEMENTATION_SPEC.md). This decision describes the target; migration is in progress.

### D13 — PostgreSQL 18.x structured Authority — DEFAULT
Earlier documentation called this LOCKED. Reclassify physical store selection as DEFAULT because it is implementation architecture, not cognitive theory.

### D14 — OpenDAL-backed content-addressed object repository — DEFAULT

### D15 — local serving stack: Tantivy + USearch + exact postings + petgraph CSR — DEFAULT
The previous research phase did not benchmark all alternatives due budget. This is a direct adoption, not a proven universal optimum.

### D16 — immutable revisions/current heads/provenance — LOCKED

### D17 — direct current-state read vs recollection — LOCKED

### D18 — typed Cognitive Query intent — LOCKED

### D19 — semantic effort levels — LOCKED
`light / normal / deep / maximum`.

### D20 — Cognitive Work Cycle — DERIVED LOCAL WORKFLOW
Correct current interpretation:
- request-local/call-chain exploration continuation;
- not a top-level Subject runtime structure;
- no requirement for durable cycle state;
- implementation may omit it until repeated-recall cost justifies it, but must preserve the internal seam.

This supersedes the earlier interpretation that treated Work Cycle as a top-level persistent semantic owner.

### D21 — Retrieved != reinforced — LOCKED

### D22 — forgetting/accessibility != suppression != purge — LOCKED

### D23 — association != semantic similarity — LOCKED

### D24 — heterogeneous association projection — DERIVED

### D25 — VCP/Memoria are algorithmic research lineages, not code authority — LOCKED

### D26 — Associative expansion problem contract — LOCKED
The system requires bounded topology-based associative expansion with inspectable flow/provenance.

Current implementation:
- VCP-derived bounded competitive Wave — DEFAULT.

Known alternate:
- PPR/diffusion family — OPEN alternative.

Do not build a generic algorithm marketplace.

### D27 — semantic cue sensing problem contract — DERIVED
Free-form cues may require discovery of weak/independent semantic directions.

Current implementation:
- EPA + Residual Pyramid — DEFAULT.

Do not expose EPA in public protocol.

### D28 — Candidate ranking is evidence-family aware — LOCKED
No single fused score may erase exact/entity/lexical/dense/topology/resource provenance.

Current deterministic ranking implementation is DEFAULT.

### D29 — concrete model/provider identities — OPEN

### D30 — server-scale retrieval backend — OPEN

### D31 — consumer-specific cognition — LOCKED
ConsumerWorkingSet is a real semantic boundary.

### D32 — ContextContribution — LOCKED
Nous contributes context/evidence; Host may own final context composition.

### D33 — Resource Awareness — LOCKED

### D34 — progressive materialization/evidence access — LOCKED

### D35 — projection families are independently invalidatable/rebuildable — LOCKED
Exact, lexical, dense, topology and relevant derived/synopsis projections must not share one universal stale bit or unconditional full rebuild.

### D36 — serving engine implementation IDs are recorded — DERIVED
A serving generation records:
- family;
- implementation ID;
- implementation revision;
- config digest;
- producer/embedding-space identity where relevant.

### D37 — external research transfer is architectural input, not decorative citation — LOCKED
A transferred idea is either:
- implemented/adapted;
- explicitly superseded/rejected with rationale;
- FUTURE/OPEN.

It must not silently disappear between research, design transfer and code.
