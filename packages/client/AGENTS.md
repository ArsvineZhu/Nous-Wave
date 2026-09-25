# Client Package Instructions

This package owns the official TypeScript Client wrapper.

- Keep transport convenience and typed errors aligned with the Protobuf/Core contract.
- Do not add direct database, Kernel-private, or provider-specific bypasses.
- Update the public reference when Client method behavior, cancellation, or error mapping changes.
