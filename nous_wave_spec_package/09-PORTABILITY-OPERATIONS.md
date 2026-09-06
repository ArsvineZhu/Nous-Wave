# 09 — Portability, Purge, Deployment, and Operations

## 1. Deployment posture

Primary target:

```text
personal computer / workstation
one process
one local PostgreSQL instance/cluster accessible to Nous Wave
embedded LanceDB
local content-addressed artifact store
external/local model service(s)
```

Server deployment is supported architecturally but does not force enterprise machinery into the local profile.

## 2. No autonomous service scheduling semantics

Operating-system service managers may keep the Nous Wave process alive. That is deployment lifecycle, not cognitive scheduling.

Nous Wave itself does not create cron jobs/timers that invoke cognitive operations.

## 3. Configuration roots

Reference local configuration identifies:

- PostgreSQL connection/profile;
- LanceDB directory;
- Artifact CAS root/OpenDAL backend;
- model-service endpoints/identities;
- enabled MicroSystems;
- typed Memory/retrieval budgets;
- network bind/security settings.

Secrets should be supplied through OS/environment/secret facility appropriate to deployment rather than committed plaintext config.

## 4. Subject portability unit

Define a **Subject Bundle** that can preserve the subject independently from a particular host application.

Current bundle covers:

- Subject identity metadata and Character Seed history;
- Cognitive Material/source metadata;
- portable owned Artifacts according to export policy;
- Memory objects/revisions/history;
- Episodes;
- association/accessibility state worth preserving;
- cognitive-use history required by retained learning semantics;
- derivation provenance and selected durable derivations;
- manifest/version/model provenance.

Serving index layouts and process caches are not required portable Authority.

## 5. Bundle format

Do not freeze an elaborate archive standard prematurely.

Current implementation may use a deterministic directory or ZIP64 archive with:

```text
manifest.json
structured JSON/JSONL exports
objects/<content-hash>...
```

The semantic manifest is the stable contract; physical archive packaging can evolve before compatibility obligations exist.

## 6. Export semantics

Export pins a coherent Subject/Memory revision boundary.

Long export work does not hold one database transaction for its duration. Pin/snapshot identifiers first, then stream referenced content.

Include content hashes so import can verify Artifacts.

## 7. Import semantics

Import validates:

- manifest/schema versions;
- content hashes;
- subject identity policy (preserve vs explicit clone/new identity);
- required artifacts;
- supported model/derivation metadata.

Imported serving projections are rebuilt locally by default rather than trusting foreign index layout.

## 8. Character Seed portability

The original and subsequent Character Seed revisions are part of Subject portability even if Persona is not installed.

## 9. Model-derived portability

Retain valuable durable derivations when their recomputation is costly or the original model may be unavailable.

Serving projection rows/index structures need not be exported.

## 10. Suppression portability

Suppression state is cognitive/governance state and should travel with the subject unless export policy explicitly excludes it.

## 11. Purge workflow

Purge computes a dependency set from the authoritative target:

```text
target source/memory/artifact
        ↓
owned revisions/derivations
        ↓
association evidence/state
        ↓
retrieval projections
        ↓
exclusive object-store bytes when no retained owner remains
```

Shared content-addressed bytes are physically deleted only when no retained authorized reference remains.

### Mixed-source derivations — execution decision, 2026-09-06

When a purged source supports an Episode, summary or MemoryItem together with retained
sources, preserve the logical object identity and regenerate its representation from
the remaining sources. Remove old derived content and revisions containing the purged
source. Do not serve the old representation while regeneration is outstanding.
The user's explicit execution decision authorizes this behavior; it is not an
executor-selected interpretation of shared-byte retention.

## 12. Backup

Installation backup is distinct from Subject Bundle export.

A simple backup strategy may snapshot PostgreSQL + object directory + LanceDB projection for operational recovery, but semantic portability remains the Subject Bundle contract.

Do not build a general backup orchestrator unless actual deployment requires it.

## 13. Projection rebuild

A valid installation must be able to delete/recreate LanceDB and other serving projections from canonical/durable state.

Rebuild status is observable and recall readiness may be degraded during rebuild.

## 14. Model changes

Changing embedding/rerank/generation models does not create a new Subject.

New derivations/projections are versioned by model identity/revision.

Old derivations are retained or retired according to configured retention rather than silently relabeled as new-model output.

## 15. Schema evolution

Current project is PRE_PRODUCTION.

Rewrite current schema/API forms directly when architecture evolves. Do not add legacy readers, aliases or migration compatibility solely because an earlier development version existed.

Real exported/public compatibility obligations, once declared, must be handled explicitly rather than inferred from history.

## 16. Observability

Use structured tracing/logging for:

- request/operation identity;
- ingest stages;
- model invocations;
- projection updates;
- Recall effort/channel use;
- durable-operation failures;
- purge/export/import.

Avoid logging entire sensitive artifacts/prompts/tool results by default.

## 17. Metrics

Useful Memory metrics include:

- ingest/material backlog;
- derivation latency;
- projection lag;
- Recall P50/P95;
- cycle latency;
- recalls per resolved cycle;
- repeated-candidate ratio;
- candidate/context waste;
- association expansion work;
- model call cost/count;
- object/DB/index sizes.

Metrics do not become semantic state.

## 18. Resource pressure

Local systems must fail/degrade before uncontrolled OOM/disk exhaustion where practical.

Use bounded candidate/association/context/artifact processing budgets and expose disk/model/index failures truthfully.

Do not create a distributed resource governor in this wave.

## 19. Local service security

Default loopback bind.

File/directory permissions protect local database/artifact/config data.

Remote exposure is opt-in and requires authentication plus transport policy. Do not claim local loopback defaults are suitable for hostile multi-user remote deployment.

## 20. Recovery

First-order recovery priorities:

```text
canonical PostgreSQL truth
artifact integrity
projection obligation replay/rebuild
restart
explicit operator action when uncertain
```

Do not build heroic multi-layer rollback systems for pre-production local operation.
