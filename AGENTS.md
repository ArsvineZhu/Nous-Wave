# AGENTS.md

Nous Wave repository execution contract.

## Project posture

DevelopmentMode = RAPID_EVOLUTION  
CompatibilityEpoch = PRE_PRODUCTION

Development history creates no compatibility obligation. Rewrite or delete obsolete internal APIs, schemas, tests and documents when the approved current design requires it. Do not add legacy aliases, dual readers or compatibility layers without a declared obligation.

## Knowledge authority

Long-term product and cognition design lives in `Heptalogos-Devs/Architecture-Vault`.

For design work, read the relevant Vault target-design chapters and accepted decisions first. Design rationale and research explain decisions but do not override target design.

For implementation work, use this order:

1. this file;
2. the designated active Plan under `project/plans/active/`;
3. current Specs owned by this repository;
4. `docs/Nous_Wave/CURRENT_STATE.md` and affected package documentation;
5. current code and focused verification.

If implementation requires a semantic decision absent from Vault and the active Plan, stop that branch as `PLAN_GAP`. Do not let existing code decide architecture by inertia.

## Current repository role

This repository implements Nous Wave. It owns current code, executable Specs, Plans, qualification evidence and implementation facts. It does not duplicate the long-term target design stored in Architecture-Vault.

The current code predates parts of the target design. Existing names such as `MemoryClass`, `Anchor`, current accessibility rules and current retrieval algorithms are implementation facts, not permanent design commitments.

## Core invariants

- Subject identity is independent of model, provider, process and session.
- Cognitive Authority, cognitive runtime and rebuildable serving projections remain separate.
- Artifact, observation occurrence, evidence and cognition have distinct identities.
- Retrieval or presentation does not create long-term reinforcement.
- Suppression, accessibility and destructive purge are distinct operations.
- Models and cross-domain reasoning produce proposals unless an owning deterministic contract authorizes direct commit.
- Domain owners own semantics; mature libraries and platform facilities should own generic mechanics when suitable.
- Do not invent numeric cognitive scales without an operational definition.

## Engineering

Use the narrowest useful check while iterating. At meaningful acceptance boundaries run `just verify` when the active Plan requires full repository verification.

Do not begin implementation from superseded design material recovered from Git history. Historical files are evidence of past work only.

## Completion

When the active Plan's acceptance conditions are met and required verification is green, stop. New architecture work requires a new accepted design decision or Plan.
