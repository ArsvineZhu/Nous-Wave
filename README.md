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
multilingual bake-off selected `BAAI/bge-m3` from FastEmbed 6.0.2: its private
bilingual fixture scored Recall@10/MRR/nDCG@10 of 1.00/1.00/1.00. The base model
passed correctness but scored MRR 0.943, outside the required three-point MRR band;
the small model missed one cross-language target (Recall@10 0.929). The selected
asset revision is `5617a9f61b028005a4858fdac845db406aefb181` with
`identity-l2-v1` preprocessing. External generation/rerank services have not been
deployed, so their live-provider effectiveness remains `NOT_RUN`.

## Development

Use Rust 1.98.1, native build tools (MSVC on Windows), and `protoc` on PATH or in
`PROTOC`. Real integration tests start disposable PostgreSQL 18.6 instances using
PostgreSQL Embedded 0.21.0 and exercise real CAS/LanceDB mechanics. Set
`POSTGRESQL_VERSION=18.6.0` for the pinned bundled database build.

Copy [config.example.toml](config.example.toml) to ignored `config.toml` and run
`cargo run -p nous-wave -- status` or `cargo run -p nous-wave -- serve`. The example
uses the managed PostgreSQL 18.6 profile and keeps its database beneath the executable
directory. Developer deployments may set `postgres.mode = "external"` and provide a
PostgreSQL URL; `NOUS_WAVE_POSTGRES_URL` selects that external mode. Current service
binding is loopback only.

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
and `cargo test --workspace` at integration boundaries. The fixed algorithm
ablation is `cargo run -p nous-memory-retrieval --example ablation`; its current
evidence is written to `.local/evaluation/ablation-v1.json`. A source-less package
is assembled with `scripts/package-portable.ps1` after the product checks; clean
machine and relocation qualification remain `NOT_RUN` until the extracted package
has been exercised separately.

## Research attribution

VCP TagMemo/RiverMemo informs the independently implemented residual cue recovery,
competitive activation, request-flow observation and direct-anchor design described
in the active spec. VCP code, constants, protocol and file structure are not imported;
VCP is not a dependency. This attribution does not establish algorithm effectiveness,
which requires the spec's fixed evaluation fixtures.
