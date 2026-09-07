# Sources and transfer boundary

This is a research/source ledger, not an implementation dependency list.

## Project authority / inherited research

- `Nous_Wave_Cognitive_Architecture_Baseline_2026-09-07.md` — semantic architecture authority.
- `Nous_Wave_Implementation_Research_Agenda_2026-09-07.md` — implementation questions and scenarios; superseded where this package makes a decision.
- `JDD.md` — engineering method: preserve semantics, minimize unjustified machinery, Library-first, no PRE_PRODUCTION compatibility ritual.
- `ArsvineZhu/Nous-Wave@d1e1d2611d786cc74e357abf32900babcbacbe83` — existing implementation evidence/reusable mechanics only.
- `ArsvineZhu/memoria-next`, branch `codex/memoria-next-foundation` — engineering-mechanics evidence; algorithmic authority explicitly rejected.

## Primary algorithm research lineage: VCP Toolbox

Evidence snapshot observed at `lioensky/VCPToolBox@b13428a53f2cb178ee3cb4b8e4aa65d564ffadc6`.

- TagMemo Wave V9.1/V9.2 deep dive:  
  https://github.com/lioensky/VCPToolBox/blob/b13428a53f2cb178ee3cb4b8e4aa65d564ffadc6/docs/TagMemo_Wave_Algorithm_Deep_Dive.md
- RiverMemo Topology V3.1:  
  https://github.com/lioensky/VCPToolBox/blob/b13428a53f2cb178ee3cb4b8e4aa65d564ffadc6/docs/RIVERMEMO_TOPOLOGY_V3.md
- TagMemo implementation evidence:  
  https://github.com/lioensky/VCPToolBox/blob/b13428a53f2cb178ee3cb4b8e4aa65d564ffadc6/TagMemoEngine.js
- RiverMemo Rust production-kernel evidence:  
  https://github.com/lioensky/VCPToolBox/blob/b13428a53f2cb178ee3cb4b8e4aa65d564ffadc6/rust-vexus-lite/src/rivermemo_topology_v3.rs
- License: CC BY-NC-SA 4.0:  
  https://github.com/lioensky/VCPToolBox/blob/b13428a53f2cb178ee3cb4b8e4aa65d564ffadc6/LICENSE

### License rule

VCP is a research/algorithmic reference, **not** a code dependency or source donor. Do not copy or transliterate VCP implementation source. Implement the specified behavior independently in Rust using Nous Wave's own types, state model and mathematical description. Keep attribution in research/documentation. This package is not legal advice; if future distribution strategy needs a formal copyright analysis, obtain one separately.

## Mature memory-system research transferred into Nous Wave

### Letta

- Memory blocks / context-management concept: https://www.letta.com/blog/memory-blocks/
- Architecture reference: https://github.com/letta-ai/skills/blob/main/letta/letta-api-client/memory-architecture.md

Transferred idea: persistent state can be intentionally projected into bounded model context.  
Nous implementation: `ResidentSet` + ephemeral `ConsumerWorkingSet` + typed `ContextContribution`.  
Rejected: context window or a mutable prompt block as Subject cognitive Authority.

### Mem0

- v3 migration / ADD-only extraction + hybrid retrieval + entity linking: https://docs.mem0.ai/platform/features/graph-memory
- core flow: https://github.com/mem0ai/mem0/blob/main/docs/core-concepts/how-it-works.mdx

Transferred idea: semantic, lexical and entity evidence are complementary; memory should be distilled rather than raw-transcript-only.  
Nous implementation: independent entity postings, Tantivy lexical candidates, USearch semantic candidates, memory formation from evidence.  
Rejected: semantic-only candidate pool; extracted entity identity as canonical identity; one fused score hiding evidence channels.

### Graphiti / Zep

- entity-edge temporal/provenance model: https://github.com/getzep/graphiti/blob/main/graphiti_core/edges.py
- edge resolution/contradiction logic: https://github.com/getzep/graphiti/blob/main/graphiti_core/utils/maintenance/edge_operations.py

Transferred idea: episodes/occurrences, evidence attribution, valid-time vs learned/observed-time, non-destructive invalidation.  
Nous implementation: `ObservationOccurrence`, `occurred_at`, `observed_at`, `valid_from`, `valid_to`, revision lineage, contradiction/supersession.  
Rejected: LLM-extracted graph entity as Host identity Authority.

### LangMem

- hot path: https://langchain-ai.github.io/langmem/hot_path_quickstart/
- background path: https://langchain-ai.github.io/langmem/background_quickstart/

Transferred idea: immediate/hot cognitive use and later consolidation are different workflows.  
Nous implementation: immediate Session residency plus explicitly invoked durable formation/consolidation/derivation.  
Rejected: an internal autonomous consolidation schedule.

### A-MEM

- paper: https://arxiv.org/abs/2502.12110
- implementation reference: https://github.com/agiresearch/A-mem

Transferred idea: structured cognitive notes, dynamic Tags/links, new experience may change current interpretation of old material.  
Nous implementation: revisioned memory/Tag/Anchor structures and derived associations; new revisions/integrative memories may reinterpret old evidence.  
Rejected: silent in-place rewriting of historical memory representation.

### Microsoft GraphRAG

- query overview: https://microsoft.github.io/graphrag/query/overview/
- global search: https://microsoft.github.io/graphrag/query/global_search/

Transferred idea: hierarchical derived summaries are useful for corpus-wide/global questions.  
Nous implementation: optional Resource synopsis / integrative derived representations for large reference resources.  
Rejected: community-report generation as default Subject runtime or universal memory organization.

### HippoRAG 2

- paper lineage: https://arxiv.org/abs/2502.14802

Transferred idea: graph-based associative multi-hop retrieval is a credible retrieval class.  
Nous implementation: topology is a first-class candidate/addressing channel.  
Rejected: PPR as the primary production law; VCP-derived bounded Wave mechanics are selected instead.

### LightRAG

- paper: https://arxiv.org/abs/2410.05779

Transferred idea: graph/topological and vector representations can be incrementally maintained as complementary indexes.  
Nous implementation: canonical Authority plus separately rebuildable lexical/vector/Wave generations.  
Rejected: adopting its storage topology wholesale.

### MemOS

- paper: https://arxiv.org/abs/2507.03724

Transferred idea: memory should be treated as a managed first-class resource with provenance/versioning across heterogeneous forms.  
Nous implementation: explicit Authority/projection/runtime layers, revisions, producer signatures and capability coverage.  
Rejected: model-parametric state as Nous Subject Authority.

## Library-first implementation dependencies selected by this Spec

The Coding Agent must use mature libraries for generic mechanics rather than reproducing them:

- PostgreSQL + SQLx — canonical structured Authority.
- Apache OpenDAL + BLAKE3 — raw artifact CAS/provider mechanics.
- USearch — dense vector serving projection.
- Tantivy — lexical serving projection.
- `roaring` — admissible candidate/posting bitmaps where dense integer IDs are used.
- `petgraph::csr` or equivalent petgraph CSR primitive — immutable Wave adjacency snapshots.
- `nalgebra` — small-matrix linear algebra (MGS/SVD/PCA-related mechanics where applicable).
- Rayon — CPU-parallel candidate/topology scoring.
- `arc-swap` — in-process atomic immutable ServingGeneration publication.
- FastEmbed — optional local model mechanics; concrete model identity remains configuration, not semantic architecture.

Docling/Tika/FFmpeg are integration candidates for document/media derivation and are capability-gated; the Rust core must not reimplement document codecs/parsers or media codecs.
