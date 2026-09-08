# Sources and Transfer Boundary

This is a research/source ledger, not an implementation dependency list.

## Project authority and inherited research

Current repository authority:
- `ARCHITECTURE.md`
- `DECISIONS.md`
- active implementation handoff/spec
- `DESIGN_TRANSFER.md`

Engineering method:
- `JDD.md` — Justification-Driven Development.

Important inherited research:
- `Nous_Wave_Cognitive_Architecture_Baseline_2026-09-07.md`
- `Nous_Wave_Implementation_Research_Agenda_2026-09-07.md`
- the earlier `00-DECISIONS.md`

Their relevant current semantics are integrated into current authority documents. Historical research does not override the current Decision Ledger.

## VCP Toolbox

Research lineage: VCP Toolbox TagMemo / RiverMemo.

Reference snapshot previously studied:
`lioensky/VCPToolBox@b13428a53f2cb178ee3cb4b8e4aa65d564ffadc6`

Relevant references:
- `docs/TagMemo_Wave_Algorithm_Deep_Dive.md`
- `docs/RIVERMEMO_TOPOLOGY_V3.md`
- TagMemo implementation evidence
- RiverMemo Rust production-kernel evidence

License observed in research: CC BY-NC-SA 4.0.

### Boundary

VCP is a research/algorithmic reference only.

Nous Wave:
- independently implements algorithms in Rust;
- uses its own semantic types/state/protocol;
- does not copy or transliterate VCP source;
- keeps attribution/research lineage;
- does not use VCP as a runtime dependency.

Transferred problem families:
- weak/independent cue discovery;
- bounded competitive associative spreading;
- actual flow/provenance;
- local/transfer topology field reasoning.

Concrete VCP-derived algorithms are current DEFAULT implementations, not public protocol invariants.

## Letta / MemGPT

Transferred:
- persistent cognition and current model context are separate;
- bounded consumer-specific context projection.

Nous:
`Session -> ResidentSet -> ConsumerWorkingSet -> ContextContribution`.

Rejected:
- mutable prompt/context block as cognitive Authority.

## Mem0

Transferred:
- distilled Memory rather than raw-transcript-only;
- exact/entity/lexical/semantic signals are complementary.

Rejected:
- embedding-only retrieval;
- model-extracted entity identity as canonical identity;
- opaque fused score that erases evidence channel.

## Graphiti / Zep

Transferred:
- occurrences/episodes;
- provenance;
- valid time vs observation time;
- revision/supersession/contradiction without destructive overwrite.

Rejected:
- extracted graph identity as Host identity Authority.

## LangMem

Transferred:
- immediate/hot cognitive continuity and later formation/consolidation are separate workflows.

Rejected:
- internal autonomous consolidation schedule.

## A-MEM

Transferred:
- structured cognitive notes/topology;
- new experience may evolve current interpretation;
- historical evidence remains intact.

Rejected:
- silent in-place rewrite of historical Memory.

## Microsoft GraphRAG

Transferred:
- global/corpus-wide questions may need hierarchical/integrative derived summaries.

Nous:
- optional Resource synopsis / Integrative derived representation and planner seam.

Rejected:
- community-report generation as universal Subject memory organization.

## HippoRAG 2

Transferred:
- associative graph multi-hop retrieval is a first-class retrieval family.

Nous:
- topology channel + current Wave implementation.

Rejected:
- PPR as permanent production law.

## LightRAG

Transferred:
- vector and topology projections are complementary;
- serving projections should be incrementally/independently maintained.

Rejected:
- wholesale adoption of LightRAG storage topology.

## MemOS

Transferred:
- memory/cognition as managed first-class resource;
- provenance/version/lifecycle across heterogeneous forms;
- Authority/Runtime/Projection distinction.

Rejected:
- model-parametric state as Subject Authority.

## Current generic implementation dependencies

Current wave DEFAULT stack:
- PostgreSQL + SQLx — structured Authority mechanics;
- OpenDAL + BLAKE3 — object/CAS mechanics;
- Tantivy — lexical projection;
- USearch — dense projection;
- petgraph CSR — topology serving;
- roaring — dense integer posting sets where useful;
- nalgebra — small-matrix linear algebra;
- Rayon — CPU parallelism;
- ArcSwap — atomic in-process publication;
- Axum/Tokio — service runtime.

These are implementation DEFAULTS frozen for the current wave. The earlier research phase could not benchmark every alternative before implementation; do not describe them as universally proven optimal.
