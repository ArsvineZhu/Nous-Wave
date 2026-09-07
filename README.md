# Nous Wave

Nous Wave is a pre-production cognitive substrate for a persistent Subject. The
current implementation authority is the [2026-09-07 Production Implementation
Spec](docs/Nous_Wave/Nous_Wave_Production_Implementation_Spec_2026-09-07.md),
with semantic context in the [system description](docs/Nous_Wave/Nous_Wave_System_Description_2026-09-07.md).

The first wave separates evidence, Cognitive Authority, Session runtime, and
rebuildable serving projections:

- PostgreSQL + SQLx stores revisions, provenance, occurrences, runtime state,
  derivation work, and resource awareness.
- OpenDAL/BLAKE3 CAS stores raw artifacts.
- Tantivy, USearch, exact postings, and petgraph CSR are serving projections;
  they are never cognitive Authority.
- The typed `CognitiveQuery` protocol combines runtime, exact, lexical, dense,
  entity, resource, and bounded clean-room Wave evidence.

Zero-model startup is supported. Explicit memory formation, evidence inspection,
session residency, typed Tags/Anchors/Associations, and deterministic topology
remain available when inference providers are absent. Current external facts
remain owned by Host-provided Resource resolvers.

## Development

Copy [config.example.toml](config.example.toml) to `config.toml`. The default
managed profile uses PostgreSQL Embedded 18.6 and stores data below the executable
directory; `NOUS_WAVE_POSTGRES_URL` selects an external PostgreSQL URL.

```text
cargo run -p nous-wave -- --config config.toml status
cargo run -p nous-wave -- --config config.toml serve
```

The service binds to loopback unless an authenticated transport policy is added
by a later Host integration. The public HTTP surface is versioned under
`/v1/subjects/...`; CLI and HTTP operations use typed JSON rather than a generic
memory payload.

Verification at meaningful boundaries:

```text
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

VCP TagMemo/RiverMemo is research lineage for the independently implemented
bounded Wave, residual cue, actual-flow, and observability mechanics. VCP source
code is not copied and is not a runtime dependency.
