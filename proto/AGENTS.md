# Protobuf Source Instructions

This scope owns the canonical wire schema.

- Update `.proto` sources rather than generated Rust/TypeScript output.
- Keep public Core/Client and private Kernel contracts in their existing versioned namespaces.
- Preserve explicit optional/presence semantics and typed identifiers; do not use untyped payloads as a generic ontology.
- Run Buf lint/generation and affected Rust/TypeScript checks after a contract change.
