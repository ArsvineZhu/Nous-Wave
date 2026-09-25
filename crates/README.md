# Rust Crate Owners

`crates/` contains Rust domain and mechanism owners used by Nous Kernel.

| Crate | Responsibility |
|---|---|
| `core` | Shared identities, typed references, query contracts, and error foundations. |
| `subject-core` | Subject identity, initialization, and Character Seed lineage. |
| `cognitive-runtime` | Session continuity, ResidentSet, query orchestration, Resources, checkpoints, and use feedback. |
| `material` / `material-service` | Artifact, ObservationOccurrence, derived representation, and materialization operations. |
| `memory-domain` / `memory-service` | Memory, revisions, evidence relations, lifecycle, Tags, Anchors, and Associations. |
| `memory-retrieval` / `serving` | Retrieval channels, ranking, and rebuildable Serving generations. |
| `authority-store` | PostgreSQL canonical schemas, transactions, and persistence access. |
| `object-store` | Raw and derived object byte storage. |
| `protocol` | Rust bindings for the Protobuf contracts in [`proto/`](../proto/). |

The [current implementation architecture](../docs/architecture/current-implementation.md) explains how these owners compose.
