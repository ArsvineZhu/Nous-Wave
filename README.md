# Nous Wave

Nous Wave is a pre-production Subject cognition system.

It maintains long-lived cognitive state across model calls, processes, providers and interaction surfaces while keeping external systems authoritative for their own live/current facts.

## Architecture

Current authority:
- [`docs/Nous_Wave/ARCHITECTURE.md`](docs/Nous_Wave/ARCHITECTURE.md)
- [`docs/Nous_Wave/DECISIONS.md`](docs/Nous_Wave/DECISIONS.md)
- [`docs/Nous_Wave/DESIGN_TRANSFER.md`](docs/Nous_Wave/DESIGN_TRANSFER.md)

System walkthrough:
- [`docs/Nous_Wave/SYSTEM.md`](docs/Nous_Wave/SYSTEM.md)

Research/source lineage:
- [`docs/Nous_Wave/SOURCES.md`](docs/Nous_Wave/SOURCES.md)

Core ownership:

```text
Subject Core             required
Cognitive Runtime        required
Memory MicroSystem       optional; enabled in reference profile
```

Serving projections are rebuildable mechanics and never cognitive Authority.

## Development

Pinned Rust toolchain: `rust-toolchain.toml`.

Install repository development commands once:

```text
cargo install just cargo-deny cargo-shear --locked
```

Common commands:

```text
just fmt
just check
just lint
just test
just verify
```

`just verify` includes formatting, workspace checking, Clippy, tests, dependency/security/license checks and repository source-shape checks.

## Runtime defaults

The current local implementation uses:
- PostgreSQL + SQLx for structured Authority;
- OpenDAL/BLAKE3 for raw artifact CAS;
- Tantivy for lexical serving;
- USearch for dense serving;
- petgraph CSR for topology serving.

These are current implementation defaults, not permanent cognitive theory.

## VCP lineage

VCP TagMemo/RiverMemo is algorithmic research lineage for weak-cue sensing, bounded Wave-style associative expansion, actual-flow and topology-field mechanics.

VCP source code is not copied and is not a runtime dependency.
