# Nous Core Instructions

This scope owns the TypeScript host and public Core-facing composition.

- Keep canonical Authority operations behind the Rust Kernel client; do not connect Core directly to the Kernel database.
- Keep Focus/Projection/Managed Context orchestration and NousQL parsing in this host boundary.
- Keep model/provider objects behind `ModelRuntime`; do not let generated protocol objects or provider internals become domain Authority.
- Mark external systems and cognitive domains as unimplemented unless current protocol, code, and tests prove otherwise.
- Run `corepack pnpm typecheck` and `corepack pnpm test` after an authorized Core change; use `corepack pnpm check` at the accepted integration boundary.
