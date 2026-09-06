# Nous Wave

Standalone cognitive substrate. The governing implementation package is
[nous_wave_spec_package/README.md](nous_wave_spec_package/README.md), with execution
stages in [12-IMPLEMENTATION-PLAN.md](nous_wave_spec_package/12-IMPLEMENTATION-PLAN.md).
The package remains canonical in its original directory to preserve existing references.

## Implementation status

Implementation is in progress. The current local runtime opens PostgreSQL, runs SQLx
migrations, opens the OpenDAL filesystem repository and the LanceDB projection, and
exposes process health and dependency status. These checks do not establish that
Subject Core or Memory operations are complete; semantic readiness remains unavailable.

## Developer prerequisites

- Rust 1.98.1 (selected by `rust-toolchain.toml`).
- PostgreSQL 18.x; the reference development runtime is 18.6.
- Native Rust build tools for the platform (MSVC on Windows).
- `protoc` on PATH, or `PROTOC` set to its absolute executable path. The selected
  LanceDB dependency graph requires it.

Copy `nous_wave_spec_package/config.example.toml` to ignored `config.toml`, set local
database/object/index paths, and supply credentials through `NOUS_WAVE_POSTGRES_URL`.
Run `cargo run -p nous-wave -- serve` or `cargo run -p nous-wave -- status`.
The runtime currently accepts loopback binding only.

Model-provider integration is implemented as bounded HTTP adapters with identity,
revision, preprocessing and response-schema validation. No embedding, reranking or
structured-generation endpoint is deployed in the current environment, so provider
quality/effectiveness evaluation is `NOT_RUN`; raw ingestion, lexical/exact recall,
association, accessibility and all storage tests remain model-independent.

Verification: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
and `cargo test --workspace`. Real storage integration checks require their explicitly
documented local dependencies; unexecuted integration checks are not passing evidence.
