# Nous Wave

Nous Wave is a pre-production Subject cognition system. This repository owns the current TypeScript Core, Rust Kernel, protocol, Memory implementation, and implementation-level references.

## Target design

- [Nous Wave Target Design](https://github.com/Heptalogos-Devs/Architecture-Vault/blob/main/docs/Nous-Wave/TARGET_DESIGN.md)
- [Architecture-Vault](https://github.com/Heptalogos-Devs/Architecture-Vault)

Architecture-Vault owns long-term product semantics, accepted design decisions, rationale, and research. This repository maintains code facts, current implementation architecture and gaps, active implementation plans, and code-specific API references.

## Current implementation documentation

- [Repository INDEX](INDEX.md)
- [Documentation overview](docs/README.md)
- [Documentation INDEX](docs/INDEX.md)
- [Current implementation architecture](docs/architecture/current-implementation.md)
- [Current state and gaps](docs/current-state/CURRENT_STATE.md)
- [Active plans](docs/plans/README.md)
- [NousQL implementation reference](docs/reference/NOUSQL.md)

## Development

The pinned Rust toolchain is specified by [`rust-toolchain.toml`](rust-toolchain.toml). Rust workspace verification is provided by [`justfile`](justfile): `just fmt`, `just check`, `just lint`, `just test`, and `just verify`. The TypeScript/Protobuf check is `corepack pnpm check`.
