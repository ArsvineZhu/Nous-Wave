# Nous Wave Documentation

This directory contains the current architecture and system documentation.

## Authority order

1. [`ARCHITECTURE.md`](ARCHITECTURE.md) — semantic ownership and system invariants.
2. [`DECISIONS.md`](DECISIONS.md) — decision status: LOCKED / DERIVED / DEFAULT / OPEN / FUTURE.
3. Active implementation handoff — current execution-only decisions.
4. [`DESIGN_TRANSFER.md`](DESIGN_TRANSFER.md) — mature-system research transfer and rejection boundaries.
5. Current implementation.

[`SYSTEM.md`](SYSTEM.md) explains the implementation and operating flows. It is explanatory and cannot override architecture or decision status.

[`SOURCES.md`](SOURCES.md) records research lineage and dependency/evidence sources.

## 新研究

2026 年 9 月的新架构、认知运行时、NousQL 与身份寻址材料见[研究入口](research/README.md)。整理稿使用中文，包含主题归纳、来源原文和与当前实现的差距；研究中的 `LOCKED` / `CONVERGED` 不代表代码已落地或新的执行授权。完整文档目录见 [INDEX.md](INDEX.md)。

## Key distinction

A physical/algorithmic `DEFAULT` can be:

```text
ExecutionStatus = FROZEN_FOR_CURRENT_WAVE
```

without becoming permanent cognitive architecture.

The repository must remain understandable from these current-state documents without requiring chat history or temporary execution material.
