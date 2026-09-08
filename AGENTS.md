# AGENTS.md

Execution contract for Nous Wave.

## Project posture

```text
DevelopmentMode = RAPID_EVOLUTION
CompatibilityEpoch = PRE_PRODUCTION
Architecture = RollingWave
```

Repository history creates no compatibility obligation. Rewrite internal APIs, schemas, crate boundaries and migrations directly when the current architecture requires it. Do not add aliases, shims, fallback readers, dual paths or legacy layers without a real current compatibility obligation.

## Authority

Read in this order:

1. `docs/Nous_Wave/ARCHITECTURE.md`
2. `docs/Nous_Wave/DECISIONS.md`
3. active implementation handoff/spec
4. `docs/Nous_Wave/DESIGN_TRANSFER.md`
5. current implementation

`docs/Nous_Wave/SYSTEM.md` explains behavior but does not override architecture/decisions.

A DEFAULT selected for the active wave may be frozen for execution without becoming permanent architecture.

## Core invariants

```text
Subject != Model
Character Seed != evolved Persona
Artifact != Memory
ObservationOccurrence != Artifact
Message != Memory
Tool result != Truth
Retrieved != Reinforced
Forgetting != Suppression != Purge
Resource awareness != Resource contents
Serving projection != Cognitive Authority
MicroSystem != microservice/plugin marketplace
```

Subject Core and Cognitive Runtime are required owners. Memory is an optional cognitive MicroSystem enabled by the reference profile.

## Executor role

Implement the active design. Do not reopen decided architecture during execution.

If implementation reveals a material unresolved semantic/ownership/provider/failure decision, report a narrow `PLAN_GAP`. Do not hide uncertainty behind a registry, framework, policy engine, configuration layer or generic abstraction.

## Library-first

Use mature libraries and platform facilities for generic mechanics.

Current physical defaults include PostgreSQL/SQLx, OpenDAL, BLAKE3, Tantivy, USearch, petgraph, nalgebra, roaring, Rayon, ArcSwap, Axum and Tokio.

Dependency count is not a quality metric.

## Algorithm boundaries

Stable abstractions represent semantic problem families, not concrete algorithm names.

Current defaults include:
- EPA/Residual for semantic cue sensing;
- bounded competitive Wave for associative expansion.

These algorithms may be replaced behind narrow internal traits when justified. Do not create a generic algorithm marketplace or expose implementation-specific algorithm fields in public Cognitive Query semantics.

VCP is research lineage only. Do not copy/transliterate its source code.

## Architecture discipline

- Domain owners own semantics; mature libraries own generic mechanics.
- Memory must not own Subject Core or generic Cognitive Runtime.
- Serving projections never become Authority.
- Preserve immutable provenance and revision lineage.
- Keep classification axes orthogonal.
- Do not force arbitrary material into chat-message/text-chunk shapes.
- Do not create a universal `CognitiveObject` JSON ontology.
- Do not invent numeric cognitive scales without operational meaning.
- Do not add a scheduler/cron/self-wakeup loop.
- Do not create empty future MicroSystem crates.
- Do not silently drop a design transfer documented in `DESIGN_TRANSFER.md`.

## Repository engineering

Deterministic mechanical invariants should be automated.

Use:
- rustfmt;
- Clippy workspace lints;
- cargo-deny;
- cargo-shear;
- source-shape guard;
- Just as the task entrypoint.

Large source files are signals, not architecture definitions. A >1000-line Rust source file is blocked by repository policy unless explicitly excluded/generated. Refactor by semantic ownership and independent reasons to change, not arbitrary line slicing.

## Verification

During iteration use the narrowest useful check.

At meaningful integration/acceptance boundaries run:

```text
just verify
```

Tests protect semantic contracts and observed risks. TDD is optional.

## Completion

When the active implementation behavior, repository governance and acceptance proof are complete, remove obsolete execution-stage artifacts and STOP.
