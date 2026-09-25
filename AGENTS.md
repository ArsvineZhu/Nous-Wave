# Execution Contract

## Authority

- Architecture-Vault owns long-term target semantics, accepted design decisions, rationale, and long-term research.
- This repository owns current implementation behavior, code-level contracts, implementation plans, and verification evidence.
- Current source, Protobuf definitions, manifests, and tests establish what this checkout implements. Do not infer implementation from target design or earlier documentation.
- `docs/plans/active/` contains current implementation authorization. Do not start code work from a completed, superseded, or research-only document.

## Architecture boundaries

- Keep Subject Core, Cognitive Runtime, Memory, material/evidence, Authority Store, retrieval, and Serving behind their current owners.
- Memory is optional to the runtime composition. Do not move Subject Core or generic Cognitive Runtime ownership into Memory.
- Long-term target semantics for Memory classes, CognitiveSchema, Self, Social Cognition, Motivation, and cross-system behavior are owned by Architecture-Vault. This codebase does not maintain a second target ontology.
- Current code implements TypeScript Core + Rust Kernel. Do not claim Self/Social/Motivation, Desired Condition, Pursuit, or Heptalogos live cognition integration is implemented without current code, protocol, and test evidence.
- Current code contains EPA/Residual and bounded Wave implementations. They are not permanent design authority or production-default claims. VCP is research lineage; its source code is not copied.

## Implementation and verification

- Use the dependency and tool routes selected in workspace manifests and lockfiles. Prefer suitable mature libraries for generic mechanics, behind Nous-owned interfaces.
- Compatibility is required only for an explicit current obligation. TDD is optional; tests protect current contracts, observed risks, or meaningful uncertainty.
- During iteration, use the narrowest useful check. At an authorized acceptance boundary use `corepack pnpm check` and `just verify` as required by the active Plan.
- Do not add speculative fallback paths, schedulers, validators, recovery layers, or verification processes for hypothetical future work.
- Report evidence as `PASS`, `FAIL`, `NOT_RUN`, or `BLOCKED` and keep each claim within what actually ran.
