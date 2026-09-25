# Rust Crate Instructions

Crates are the Rust semantic/mechanism owners composed by Nous Kernel.

- Keep domain state and canonical mutations in their existing owners; mechanism crates do not acquire Memory or Subject semantics.
- `memory-domain` owns Memory types; `memory-service` owns Memory operations; `authority-store` owns SQL mechanics; `serving` owns rebuildable projections.
- Use the workspace dependency declarations in the root `Cargo.toml`; do not pin shared dependency versions independently in crate manifests.
- Add tests for current owner contracts and observed risks, not for speculative future behavior.
