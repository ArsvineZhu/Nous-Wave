# 02 — Subject Core

## 1. Responsibility

Subject Core is the smallest required semantic owner in Nous Wave.

It answers:

- which persistent subject is being addressed;
- how the subject was initially authored;
- which cognitive MicroSystems are available;
- what current subject-level revision/config identity should fence operations.

It does **not** implement Persona, Memory content, Social state, beliefs, scheduling or agent behavior.

## 2. Subject identity

Each subject has a stable generated identifier independent of:

- LLM/model/provider;
- host process;
- chat conversation;
- messaging account;
- UI session;
- Memory backend/index generation.

Use UUIDv7 or an equivalent time-sortable opaque ID for generated logical identity.

A subject may additionally have human-friendly labels, but labels are not identity.

## 3. Character Seed

### Definition

`CharacterSeed` is the authored initial character/role source used to establish a new subject without requiring pre-existing memories.

Minimum semantics:

```text
CharacterSeed
├─ seed_id
├─ subject_id
├─ content artifact/reference
├─ authored_by / source descriptor
├─ created_at
├─ revision
└─ provenance
```

The seed content is free-form. Plain text and Markdown MUST be supported.

The public API SHOULD also permit referencing another supported Artifact representation instead of forcing the seed into a database text field.

## 4. Seed history

The original seed is retained.

If an authorized caller later changes the role card, create a new Character Seed revision or explicit superseding seed entry. Do not mutate history in place.

This supports future questions such as:

```text
what was the original design?
how did authored intent change?
how did learned Persona later diverge from authored seed?
```

## 5. Character Seed is not Persona

Character Seed may later be consumed by a Persona/Self MicroSystem as foundational source material.

It remains source material rather than becoming a giant mutable Persona record.

When Persona is unavailable, a host may directly project the Character Seed into its own model prompt/context if desired. That is a host integration behavior, not Persona implementation inside Subject Core.

## 6. Subject creation

Subject creation requires:

```text
character_seed
```

plus optional metadata/configuration.

It MUST NOT require:

- imported message history;
- pre-generated episodes;
- embeddings;
- Persona facets;
- relationship data;
- beliefs;
- diary/dream artifacts.

Creating a subject with zero experiences is normal.

## 7. MicroSystem capability state

Expose a compact status projection:

```text
CapabilityStatus
├─ capability_id
├─ status: READY | DEGRADED | UNAVAILABLE | FAILED
├─ reason/problem, optional
└─ generation/config identity when useful
```

Subject Core does not own every subsystem lifecycle state machine; it aggregates their truthful readiness for host inspection.

## 8. Subject revision

Maintain a small subject-level revision/fence that advances on changes which invalidate a caller's subject-level assumptions, such as explicit Character Seed replacement or subject deletion state.

Do not advance it for every recalled memory or serving-index update.

Memory owns its own domain revisions.

## 9. Deletion

Deleting a subject is an explicit operation that coordinates owned MicroSystems and artifact references.

For the current wave, subject purge must cover Subject Core + Memory + exclusively owned artifacts/derivations according to purge policy.

Do not implement cross-future-MicroSystem deletion machinery before those owners exist; instead define a typed owner cleanup contribution point usable when new owners are added.

## 10. Public projection

The normal subject read model includes:

```text
subject_id
label/metadata
current Character Seed reference/revision
created_at
subject revision
MicroSystem capability statuses
```

It does not dump all Memory or future cognition into one object.
