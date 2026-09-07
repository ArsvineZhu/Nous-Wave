# Nous Wave Production Implementation Package

**Date:** 2026-09-07  
**Target repository:** `ArsvineZhu/Nous-Wave`  
**Observed baseline:** `d1e1d2611d786cc74e357abf32900babcbacbe83`  
**Status:** READY_FOR_EXECUTION  
**Posture:** PRE_PRODUCTION direct rewrite; no development-history compatibility obligation.

This package contains two primary documents:

1. `Nous_Wave_Production_Implementation_Spec_2026-09-07.md`
   - decision-complete Coding-Agent implementation specification;
   - freezes the first production architecture and execution path;
   - includes persistence, domain model, runtime, multimodal derivation, provider/capability model, cognitive query protocol, retrieval, clean-room Wave/Topology implementation, API, migration/reset and verification requirements.

2. `Nous_Wave_System_Description_2026-09-07.md`
   - detailed human-readable description of the resulting cognition system;
   - explains how evidence, memory, runtime, resources, entities, Tags/Anchors/Associations, multimodal material and Wave retrieval cooperate;
   - records which ideas from mature memory systems are actually transferred into Nous Wave and which are deliberately rejected.

`SOURCES.md` records the research lineage and license boundary.

## Authority

For this implementation wave, the Production Implementation Spec supersedes executable direction from the 2026-09-06 Memory Product Closure documents, the abandoned 2026-09-07 Cognitive Rebase package, the earlier IR package's benchmark-as-selection-gate requirement, and the stale repository `AGENTS.md` statement `CurrentWave = MEMORY_PRODUCT_CLOSURE`.

Semantic architecture from `Nous_Wave_Cognitive_Architecture_Baseline_2026-09-07.md` remains authoritative unless this package explicitly refines a previously-open implementation question.

## Critical correction

There is no pre-execution benchmark window. Physical and algorithmic choices are therefore made in the Spec before Coding-Agent execution. Later tests verify the chosen architecture; they do not reopen selection by default.

Memoria Next is treated primarily as an engineering-mechanics source. The primary Wave/retrieval algorithm research lineage is current VCP Toolbox TagMemo V9.1/V9.2 + RiverMemo Topology V3.1. Because VCP Toolbox is CC BY-NC-SA 4.0, this package requires a clean-room Rust implementation of learned concepts and forbids copying VCP source code into Nous Wave.
