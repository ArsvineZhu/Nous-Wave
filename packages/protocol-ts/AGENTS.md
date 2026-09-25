# Protocol Binding Instructions

This package contains derived TypeScript bindings.

- Do not hand-edit generated output. Update the canonical `.proto` source and Buf configuration, then regenerate.
- Preserve API versioning and optional-field semantics in `proto/`.
- Verify generation and the TypeScript consumer surface after protocol changes.
