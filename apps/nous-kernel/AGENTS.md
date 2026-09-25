# Nous Kernel Instructions

This scope owns the private Rust Kernel process and composition of Rust owners.

- Keep canonical mutations inside the owning domain service and Authority Store boundary.
- Keep public HTTP/Connect and presentation logic in Nous Core; Kernel RPC remains private and authenticated.
- Preserve the separation between Subject Core, Cognitive Runtime, material, Memory, Serving, and persistence.
- Treat generated protocol types as transport contracts; do not move product semantics into generated files.
- Run the narrow affected crate checks during iteration and `just verify` at the authorized acceptance boundary.
