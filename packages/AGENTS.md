# TypeScript Package Instructions

This scope contains consumer-facing TypeScript packages.

- Keep Client convenience separate from Kernel Authority; the Client cannot bypass the Core API or create canonical state directly.
- Treat generated protocol output as derived. Change `proto/` and the Buf generation inputs, then run the declared generation/check commands.
- Preserve the public TypeScript surface and cancellation/error mapping when changing generated or Client contracts.
