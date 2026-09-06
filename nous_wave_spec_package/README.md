# Nous Wave — Architecture & Memory MicroSystem Implementation Spec Package

**Status:** implementation specification for the current rolling wave  
**Date:** 2026-09-06  
**Project:** Nous Wave  
**Repository posture:** standalone, general-purpose, independently deployable and publishable

## 1. Purpose

Nous Wave is a general cognitive substrate for long-lived AI subjects. It is not tied to a chat product, an IM platform, an agent framework, a specific LLM provider, or a specific host application.

A host may use Nous Wave for an assistant, game character, autonomous agent, companion, research subject, or another persistent AI identity. The host supplies observations and invokes cognitive operations; Nous Wave owns cognitive state and memory semantics.

The current implementation wave establishes:

1. the minimal **Subject Core** required for any subject to exist;
2. the **MicroSystem architecture** used by independently optional cognitive subsystems;
3. one complete production-quality **Memory MicroSystem**;
4. explicit boundaries for future Persona/Self, Social, Epistemic, Goals/Commitments, Reflection, Diary, Dream/Simulation, and related MicroSystems without pretending their internals are already designed.

## 2. Current implementation scope

This wave MUST fully implement the Memory MicroSystem rather than a `remember(text) / search(query)` demo.

Memory includes:

- heterogeneous cognitive-material ingestion;
- arbitrary-length text and structured content;
- tool/MCP observations;
- file and media artifacts through stable artifact references;
- provenance and source classification;
- epistemic classification;
- episodic memory formation;
- durable derivations;
- revision/history semantics;
- lexical, exact, dense, temporal, entity, episodic, relation and associative candidate retrieval;
- structured recall intent;
- explicit recall effort;
- Cognitive Work Cycle and incremental recollection;
- association and accessibility dynamics;
- automatic cognitive-use feedback;
- forgetting distinct from suppression and purge;
- correction and revision;
- consolidation hooks and memory-native consolidation that does not require future higher cognition;
- export/import and projection rebuild;
- measurable cognitive and retrieval evaluation.

## 3. Current non-scope

The following are recognized Nous Wave MicroSystems but their internal ontology and algorithms are **not** frozen by this package:

- Persona / Self Model;
- Social World Model;
- Epistemic / Belief Model;
- Goals / Commitments;
- Reflection;
- Diary;
- Dream / Simulation;
- higher-order appraisal or emotion models;
- autonomous scheduling or agent-runtime behavior.

Do not create placeholder implementations for them.

Their only current contract is the minimum interoperability surface required so that the Memory implementation does not prevent them from being added later.

## 4. Central architectural rules

```text
Subject != Model
Character Seed != Persona state
Artifact != Memory
Message != Memory
Tool output != Truth
File format != semantic class
Stored != Accessible != Likely Recalled
Retrieved != Reinforced
Forgetting != Suppression != Purge
Logical MicroSystem != network microservice
Offline-capable operation != autonomous scheduler
```

Nous Wave never decides that it is “time to dream”, “time to write a diary”, or “time to consolidate” merely because wall-clock time passed. The host invokes operations. A host scheduler may exist, but it is external to Nous Wave.

## 5. Technology baseline

Current locked/default direction inherited from the architecture work preceding this package:

- Rust owns the core service and memory hot path;
- PostgreSQL 18.x owns canonical structured cognitive state and revision history;
- LanceDB OSS embedded is the default PC/local retrieval projection unless implementation evidence exposes a hard blocker;
- Apache OpenDAL owns object-storage mechanics; local filesystem is the first backend;
- external model workers/providers own embedding/reranking/generation model execution;
- PC/workstation deployment is primary; server deployment remains possible without forcing distributed infrastructure into the local design.

Exact model family, embedding dimension, quantization, sparse representation and server-scale retrieval engine remain research seams.

## 6. Document map

| File | Purpose |
| --- | --- |
| `00-DECISIONS.md` | decision status, locked boundaries, open research seams |
| `01-MICROSYSTEM-ARCHITECTURE.md` | standalone system and MicroSystem composition model |
| `02-SUBJECT-CORE.md` | Subject identity, Character Seed, capability/readiness boundary |
| `03-COGNITIVE-MATERIAL.md` | source/artifact/provenance/classification model |
| `04-MEMORY-DOMAIN.md` | complete Memory semantic model and invariants |
| `05-RECALL-ASSOCIATION.md` | Structured Recall Intent, effort, Work Cycle, association algorithms |
| `06-MEMORY-LEARNING.md` | ingestion processing, episode formation, feedback, forgetting, consolidation |
| `07-PERSISTENCE-PROJECTIONS.md` | PostgreSQL Authority, OpenDAL CAS, LanceDB projections, rebuildability |
| `08-API.md` | service/API contracts for host integration |
| `09-PORTABILITY-OPERATIONS.md` | export/import, purge, diagnostics, local/server deployment |
| `10-EVALUATION.md` | semantic, cognitive, retrieval and performance acceptance |
| `11-FUTURE-MICROSYSTEM-BOUNDARIES.md` | boundary-only contracts for Persona, Social, Diary, Dream, etc. |
| `12-IMPLEMENTATION-PLAN.md` | decision-complete execution plan for Coding Agent |
| `RESEARCH.md` | algorithm lineage and external-system references |
| `AGENTS.md` | executor constraints for the new repository |
| `config.example.toml` | configuration shape; no autonomous schedule |

## 7. How to use this package

The Coding Agent is an executor, not the research owner.

It MUST NOT:

- shrink the Memory scope into an MVP;
- implement future MicroSystems because their names appear in the architecture;
- invent a generic plugin framework;
- add an internal scheduler;
- turn every cognitive concept into a scalar;
- reduce ingestion to chat messages;
- treat model/tool output as canonical truth;
- preserve incompatible development shapes for “legacy compatibility”;
- reopen locked dependency roles without a real blocker.

Material semantic gaps are escalated instead of hidden behind generic abstractions.
