# Project Data-Source Conversation Index

These four objects were present as distinct project data sources. Two pairs share the same display filename, so migration filenames include timestamp + original file ID to prevent overwrite.

| Title | Created | File ID | Migration copy | Recovery status | Notes |
|---|---|---|---|---|---|
| `审查执行完成Spec.txt` | 2026-09-09T13:28:00Z | `file_000000008c088206a2a3e81ad30c8deb` | `2026-09-09T132800Z__审查执行完成Spec__file_000000008c088206a2a3e81ad30c8deb.txt` | `byte_exact_runtime_copy` | Source object referencing Session-Handoff-2026-09-09-20-47.md. |
| `审查执行完成Spec.txt` | 2026-09-09T13:28:07Z | `file_00000000bf24820689da87f8b84a9d94` | `2026-09-09T132807Z__审查执行完成Spec__file_00000000bf24820689da87f8b84a9d94.txt` | `recovered_from_file_library_read` | Source object summarizing the Cognitive Runtime / Projection / Context / Language design record. |
| `收敛新架构实现方式.txt` | 2026-09-10T06:52:27Z | `file_00000000ed948209b7a8641922982238` | `2026-09-10T065227Z__收敛新架构实现方式__file_00000000ed948209b7a8641922982238.txt` | `byte_exact_runtime_copy` | NousQL v0.2 capability-complete design discussion. |
| `收敛新架构实现方式.txt` | 2026-09-10T08:33:07Z | `file_000000007800820698ac9c4e8e10c931` | `2026-09-10T083307Z__收敛新架构实现方式__file_000000007800820698ac9c4e8e10c931.txt` | `recovered_from_file_library_read` | LexicalRef / Identity & Addressing convergence discussion. |

`byte_exact_runtime_copy` means the active runtime contained that source file and it was copied byte-for-byte. `recovered_from_file_library_read` means the distinct File Library object was readable but its original file bytes were not mountable; the migration copy was reconstructed from the content returned by File Library and is explicitly marked as such.
