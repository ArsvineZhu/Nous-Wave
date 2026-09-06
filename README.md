# Nous Wave

Nous Wave is a standalone cognitive substrate with Subject Core and an optional
Memory MicroSystem. The active implementation authority is the
[Memory Product Closure spec](docs/Nous_Wave_Memory_Product_Closure_Spec_2026-09-06.md).

Closure is in progress. Gate A (Memory Product) and Gate B (source-less Windows
release) have not passed. No standalone release is currently qualified.

The implementation includes PostgreSQL canonical state, immutable CAS artifacts,
LanceDB retrieval projections, source-grounded memory formation, correction,
suppression, purge, cognitive work cycles and schema V2 export/import. Material
input supports text/JSON, existing artifacts and bounded streaming file upload;
artifact bytes can be streamed back through HTTP or CLI.

Memory availability is separate from Subject enablement. Disabling the global
Memory capability leaves Subject Core and material storage usable. Enabled Memory
with unavailable embedding or dense projection reports degradation.

Local FastEmbed and external HTTP embedding paths are implemented. The fixed
multilingual model bake-off and retrieval ablations are still pending; the current
local model choice is provisional. External generation/rerank services have not
been deployed, so their live-provider effectiveness remains `NOT_RUN`.

## Development

Use Rust 1.98.1, native build tools (MSVC on Windows), and `protoc` on PATH or in
`PROTOC`. Real integration tests start disposable PostgreSQL 18.6 instances using
PostgreSQL Embedded 0.21.0 and exercise real CAS/LanceDB mechanics. Set
`POSTGRESQL_VERSION=18.6.0` for the pinned bundled database build.

Copy [config.example.toml](config.example.toml) to ignored `config.toml`, configure
an external PostgreSQL 18 database, and run `cargo run -p nous-wave -- status` or
`cargo run -p nous-wave -- serve`. `NOUS_WAVE_POSTGRES_URL` overrides the configured
connection string. Current service binding is loopback only.

CLI operations are discoverable with `--help`. File I/O examples:

```text
nous-wave material ingest SUBJECT --input material.json
nous-wave material upload SUBJECT --file document.bin --metadata envelope.json
nous-wave source show SUBJECT SOURCE
nous-wave artifact show SUBJECT ARTIFACT
nous-wave artifact get SUBJECT ARTIFACT --output document.bin
nous-wave artifact lineage SUBJECT ARTIFACT
```

Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`
and `cargo test --workspace` at integration boundaries. Passing these commands does
not establish the unrun algorithm bake-off or portable-release claims.

## Research attribution

VCP TagMemo/RiverMemo informs the independently implemented residual cue recovery,
competitive activation, request-flow observation and direct-anchor design described
in the active spec. VCP code, constants, protocol and file structure are not imported;
VCP is not a dependency. This attribution does not establish algorithm effectiveness,
which requires the spec's fixed evaluation fixtures.
