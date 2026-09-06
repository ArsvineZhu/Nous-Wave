# 08 — Public Service and Host Integration API

## 1. API role

The public API is a host integration surface for cognitive operations. It does not expose database tables, LanceDB queries, raw graph internals or model-provider mechanics.

Reference transports:

```text
Rust library/service interfaces internally
HTTP/JSON for cross-language host integration
CLI for operator/developer use
```

MCP is not a required Nous Wave server protocol in this wave. MCP/tool **results** are supported as ingestion sources through the normal Cognitive Material API.

## 2. Versioning posture

Public durable/request payloads carry an explicit schema/API version.

PRE_PRODUCTION allows direct breaking revision before a public compatibility obligation exists; do not accumulate legacy aliases or dual readers.

## 3. Service bind/security

Reference local deployment binds loopback by default.

Remote binding is explicit configuration and requires configured authentication/TLS/reverse-proxy policy appropriate to the deployment.

Do not build enterprise IAM into the local reference profile.

## 4. Error model

Return machine-readable problems with:

```text
code
title/message
operation/request id
retryability when meaningful
details/field violations where appropriate
causal references when available
```

Do not return raw database/provider exceptions as the public contract.

## 5. Subject API

### Create

```text
POST /v1/subjects
```

Input:

```text
label/metadata, optional
character_seed:
  content or Artifact input
  media_type
  author/source provenance
memory_enabled, default true in reference profile
```

Creation succeeds without pre-existing memory.

### Read/list

Expose Subject Core metadata, current seed revision and MicroSystem capability readiness.

### Character Seed revision

Explicit endpoint/operation creates a new seed revision while preserving history.

Do not treat normal Memory learning as Character Seed mutation.

## 6. Cognitive Material ingest

```text
POST /v1/subjects/{subject_id}/material
```

Input envelope includes:

```text
source descriptor
origin_class
semantic_class
epistemic_class
representation/media type
temporal context
scope
content inline OR artifact/object upload/reference
idempotency/source identity when available
host metadata, bounded
```

Support:

- short/long text;
- Markdown;
- JSON/tool results;
- file upload or artifact registration;
- derived-artifact submission by an external processor.

The response distinguishes durable acceptance from downstream processing readiness.

## 7. Tool observation convenience endpoint/client helper

Provide a convenience operation that maps tool/MCP invocations onto normal Cognitive Material:

```text
record_tool_observation(
  subject,
  tool_identity,
  operation,
  invocation_id,
  arguments_ref_or_redacted_metadata,
  result_artifact,
  outcome,
  temporal/scope metadata
)
```

This helper does not create a separate Tool Memory ontology.

## 8. Artifact API

Hosts need operations to:

- upload/register Artifact content;
- read Artifact metadata;
- obtain content when authorized;
- list derivations/source lineage;
- submit derived Artifact with processor/model provenance.

Do not turn this into a general cloud file manager.

## 9. Processing status

Expose status for accepted material and requested offline-capable operations:

```text
recorded
processing
ready
degraded
failed
```

with which capability/derivation is outstanding.

## 10. Recall API

```text
POST /v1/subjects/{subject_id}/recall
```

Input includes `RecallIntent`:

```text
cycle_id?,
target?,
objective,
temporal?,
cues,
constraints,
result_need,
effort
```

Response contains:

```text
recall_id
cycle_id when used
typed results
support/provenance
status/sufficiency indication
effort trace summary
continuation metadata
```

Do not expose HNSW/PPR/provider tuning knobs here.

## 11. Cognitive Work Cycle API

### Begin

```text
POST /v1/subjects/{subject_id}/cycles
```

Creates a bounded cycle identity and optional initial context/intention.

### Continue recall

Recall calls with the cycle ID inherit Working Cognitive State.

### Inspect

```text
POST /v1/subjects/{subject_id}/cycles/{cycle_id}/inspect
```

Returns expanded evidence/content for selected result refs and records inspection.

### Close

```text
POST /v1/subjects/{subject_id}/cycles/{cycle_id}/close
```

Caller may report:

```text
RESOLVED
INSUFFICIENT
ABANDONED
```

No generic workflow/state-machine behavior is attached to cycle close.

## 12. Exposure/use feedback

The host can explicitly record what Memory cannot infer itself:

```text
context_exposed(result refs)
followed(result/association ref)
referenced_or_used(result refs)
outcome(resolved/insufficient/abandoned + optional refs)
```

Candidate/surfaced events are generated internally. Exposure/use events require real host knowledge where applicable.

## 13. Direct read/inspect operations

Provide direct reads for:

- source/artifact provenance;
- Memory object current revision/history;
- Episode membership;
- association evidence/debug view where authorized;
- suppression state;
- current processing/readiness state.

These are different from ranked recall.

## 14. Correction

```text
POST /v1/subjects/{subject_id}/memory/{object_id}/correct
```

Input identifies:

- expected current revision;
- correction payload/interpretation;
- supporting source/evidence refs or explicit caller authority metadata;
- reason.

Creates a new revision/supersession; does not edit raw source.

## 15. Suppression

Expose suppress/unsuppress for Memory objects/source-derived recall units.

Suppression immediately affects ordinary Recall.

## 16. Purge

Purge is an operation with explicit target/scope and status.

The response reports what classes of data were removed/retained and any blocked shared-artifact condition.

## 17. Consolidation

```text
POST /v1/subjects/{subject_id}/memory/consolidate
```

Explicitly invoked. Accepts scope/target/budget hints.

No schedule field is part of the cognitive API.

## 18. Export/import

Expose subject/memory export and import operations per `09-PORTABILITY-OPERATIONS.md`.

## 19. Projection rebuild

Administrative operation:

```text
rebuild retrieval projection
```

This does not modify cognitive history.

## 20. Health/readiness

Expose:

- process health;
- PostgreSQL readiness;
- object repository readiness;
- LanceDB projection readiness;
- model-service readiness by role;
- Subject/MicroSystem readiness.

Avoid a single “healthy=true” that hides degraded cognition.

## 21. CLI

The reference CLI supports the same meaningful operations, including:

```text
subject create/show
material ingest
artifact show
recall
cycle begin/inspect/close
memory show/history/correct/suppress/purge
memory consolidate
projection rebuild
export/import
status
```

CLI does not become a separate semantic implementation.

## 22. Host integration principle

A host may build any higher-level convenience API, including chat-specific helpers, MCP adapters, game-engine bridges or agent tools.

Those adapters translate into Nous Wave's generic material/cognitive contracts. They do not modify Nous Wave core semantics.
