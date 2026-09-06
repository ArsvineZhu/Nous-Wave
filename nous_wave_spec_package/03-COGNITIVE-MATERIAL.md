# 03 — Cognitive Material, Sources, Artifacts, and Provenance

## 1. Purpose

Memory must represent what an AI actually encounters, not only chat turns.

The canonical ingestion boundary therefore accepts **Cognitive Material** with explicit provenance and orthogonal classifications.

The model is intentionally broader than text while remaining agnostic to application-specific source systems.

## 2. Fundamental separation

```text
Source / Observation
        ↓
Artifact / Content Representation
        ↓
Interpretation / Derivation
        ↓
Memory formation
```

Do not collapse these layers.

Examples:

```text
MCP call result JSON
    = Artifact/source evidence
    != automatically a factual Memory

8,000-word IM message
    = one source event/artifact
    may form many memory objects

30-message conversation
    = many source events
    may form one coherent episode

Diary.md in the future
    = self-generated narrative Artifact
    != external factual evidence
```

## 3. Source Record

A `SourceRecord` represents where and under what circumstances material entered Nous Wave.

Core fields:

```text
source_id
subject_id
origin_class
source_kind
external/source locator fields when supplied
observed_at / occurred_at when known
recorded_at
author/actor identity when known
operation/invocation identity when known
parent/source refs
host metadata (bounded structured data)
```

Do not require a global enum for every future application. Use a small stable origin class plus namespaced `source_kind` identifiers.

## 4. Origin Class

Required coarse categories:

```text
HUMAN
TOOL
HOST
EXTERNAL_SYSTEM
SELF_GENERATED
IMPORTED
```

These answer “where did this come from?”, not “what does it mean?”

## 5. Semantic Class

Semantic class answers “what kind of material is this?”

Core classes needed by the current Memory wave:

```text
MESSAGE
CONVERSATION
DOCUMENT
TOOL_OBSERVATION
WEB_CONTENT
CODE
NOTE
REPORT
SYSTEM_OBSERVATION
GENERIC_MATERIAL
```

Reserved namespaced future classes may include `diary`, `reflection`, `simulation`, `dream`, but Memory must not hard-code special behavior for unimplemented MicroSystems.

Unknown/host-defined semantic class values are retained as namespaced strings if allowed by API policy.

## 6. Epistemic Class

Epistemic class records how the source should be interpreted, not whether a proposition is ultimately true.

Core values:

```text
OBSERVED       # Nous/host directly observed the source/event
REPORTED       # a person/tool/system reported a proposition/result
DERIVED        # deterministic/algorithmic transformation of other material
INFERRED       # model/reasoning-produced interpretation
NARRATIVE      # subjective authored narrative
SIMULATED      # counterfactual/dream/synthetic world content
```

The classification is retained through derivations unless an explicit interpretation produces a new object with a different epistemic status and provenance.

## 7. Representation / Media Type

Representation is physical/content format, normally expressed through MIME media types plus encoding metadata.

Examples:

```text
text/plain
text/markdown
text/html
application/json
application/pdf
text/x-rust
image/png
audio/...
video/...
```

MIME/type does not infer semantic class.

## 8. Artifact

An Artifact identifies an immutable content payload or content reference.

Minimum metadata:

```text
artifact_id
content_hash
media_type
size
encoding when relevant
object_locator / inline payload policy
created/received time
source_refs
classification snapshot
```

Large content belongs in the content-addressed object repository. Small bounded structured/text content may be stored inline when that materially simplifies access; the semantic contract must not depend on inline versus object storage.

## 9. Arbitrary-length text

The API and domain model MUST NOT assume one message-sized string.

Large text may be retained as one source Artifact while producing derived sections/chunks for processing and retrieval.

Chunking is a serving/derivation mechanic, not source identity.

Never rewrite the original source into chunks and then discard source boundaries.

## 10. Tool and MCP observations

Tool/MCP data is first-class cognitive material.

A tool observation should be able to retain:

```text
tool/server identity
operation name
invocation/correlation identity
arguments or arguments digest/reference
result artifact(s)
error/partial outcome when applicable
invoked/observed timestamps
host/source provenance
```

Arguments may contain secrets or high-volume data; retention policy must be explicit and may store redacted/digested forms.

The semantic meaning is:

> this tool/system returned/reported this result under this invocation

not:

> the returned proposition is unquestionable world truth.

## 11. Files and documents

Nous Wave accepts files as Artifacts without requiring Memory itself to implement every parser.

A host or derivation worker may provide:

```text
PDF -> extracted text/page artifacts
image -> caption/OCR artifacts
audio -> transcript artifacts
video -> transcript/scene artifacts
DOCX -> structural/text artifacts
```

All derived outputs retain source lineage and model/parser identity.

Memory can preserve unsupported binary Artifacts even when no searchable derivation exists yet.

## 12. Derivation

`Derivation` records a transformation from source Artifact(s) to new Artifact(s) or structured interpretations.

Minimum provenance:

```text
derivation_id
input refs
processor/model identity
processor/model revision
preprocessing identity
parameters/config digest
output refs
created_at
```

Examples:

- chunking;
- OCR;
- ASR;
- vision captioning;
- embedding;
- sparse representation;
- entity extraction;
- episode extraction;
- summary generation.

Do not silently overwrite old model-derived outputs when models change.

## 13. Cardinality rules

All are valid:

```text
one source -> one artifact
one source -> multiple artifacts
one artifact -> multiple derivations
one artifact -> multiple memory objects
multiple artifacts -> one episode
multiple sources -> one consolidated memory
```

The schema and API must not encode one-to-one assumptions.

## 14. Self-generated artifacts

Future MicroSystems can submit normal Cognitive Material using `origin_class=SELF_GENERATED` and appropriate semantic/epistemic class.

Examples:

```text
Diary:
  origin = SELF_GENERATED
  semantic = diary
  epistemic = NARRATIVE
  media = text/markdown

Dream:
  origin = SELF_GENERATED
  semantic = dream
  epistemic = SIMULATED
  media = text/markdown
```

Memory does not add diary/dream-specific code. Their classification is sufficient to preserve epistemic boundaries and provenance.

## 15. Classification preservation

Classification metadata is not a convenience filter that may be dropped during processing.

Recall results and derived Memory objects must retain enough provenance to answer:

```text
where did this come from?
was it observed, reported, inferred, narrative or simulated?
which source/artifact supports it?
which processor/model produced this interpretation?
```

## 16. Ingestion acceptance

An ingest request is valid even when:

- no embedding service is currently available;
- no parser exists for the media type;
- no immediate Memory object can yet be extracted.

If the source can be durably recorded, record it and expose processing/projection readiness truthfully.

Do not fake successful semantic processing when only raw Artifact storage succeeded.
