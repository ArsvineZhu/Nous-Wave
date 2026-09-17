# NousQL Identity & Addressing Contract v0.5

> Status: **LOCKED design baseline for NousQL identity/addressing**
>
> Scope: entity names, exact references, LexicalRef, internal Authority IDs, binding and canonical representation.
>
> This contract supersedes the still-open identity conventions in NousQL v0.4.

---

# 1. Decision Summary

NousQL uses three distinct identity layers:

```text
DisplayName / Alias
    human/model-friendly surface locator
    mutable, non-unique

LexicalRef
    stable typed AI-facing exact address
    e.g. ent:amber-lotus-cello-river

Authority Identity
    machine/internal canonical identity
    UUIDv7 for Nous-owned Authority objects
    Host-owned opaque canonical ref for Host-owned objects
```

These layers MUST NOT be collapsed.

Core rule:

> **Names are for discovery and convenience. LexicalRefs are for exact model-facing reference. Internal IDs are for Authority/storage.**

---

# 2. Entity Query Syntax

## 2.1 Name form

```text
@e("Alice")
```

`"Alice"` is a surface Entity locator.

It is not itself the Entity identity.

The binder resolves it against the current Host Entity namespace.

Valid sources may include:

```text
current display name
registered exact aliases
Host-defined exact name bindings
```

Name resolution is not semantic/vector similarity.

Outcomes:

```text
0 matches   → UNKNOWN_REFERENCE
1 match     → exact bind
>1 matches  → AMBIGUOUS_REFERENCE
```

The system never guesses between multiple Alices.

## 2.2 Exact LexicalRef form

```text
@e(ent:amber-lotus-cello-river)
```

This is the preferred exact model-facing form once the reference is known.

It bypasses display-name disambiguation but still performs:

```text
type validation
visibility/Subject validation
Host binding validation
liveness/tombstone validation
```

## 2.3 Mixed multi-entity input

Allowed:

```text
@e("Alice",ent:quiet-piano-mint-cloud)
```

Every argument is an `EntityLocator`.

After binding, all participants become exact identities.

## 2.4 Joint participant semantics

```text
@e("Alice","Bob")
```

is one joint participant set.

The participant list is semantically unordered.

Therefore:

```text
@e("Alice","Bob")
```

and:

```text
@e("Bob","Alice")
```

bind to the same joint focus.

Bound canonical serialization sorts participants by stable LexicalRef.

Entity order MUST NOT encode relation direction.

---

# 3. When the LLM Should Use a Name vs LexicalRef

## 3.1 Use a name when

- the user supplied the name;
- no exact LexicalRef is currently available;
- the name is expected to bind uniquely;
- the query is an initial discovery step.

Example:

```text
@e("Alice")
```

The LLM is not required to perform an ID lookup before every ordinary query.

## 3.2 Use a LexicalRef when

- a previous result already returned it;
- an ambiguity was resolved to it;
- an exact object/entity is being followed across query steps;
- replay/debugging needs stable identity;
- two entities have similar or identical display names.

Example second-hop query:

```text
@e(ent:amber-lotus-cello-river) $memory
```

## 3.3 Never replace a known exact ref with a name merely for readability

If the model already has:

```text
ent:amber-lotus-cello-river
```

then subsequent exact reference should preserve it unless the task explicitly asks for name-oriented presentation.

---

# 4. LexicalRef Format

v0.5 locks the alpha format:

```text
<type>:<word>-<word>-<word>-<word>
```

Examples:

```text
ent:amber-lotus-cello-river
mem:quiet-piano-mint-cloud
tag:coral-frame-lake-bird
anchor:mint-river-cello-frame
art:maple-glass-lotus-field
obs:quiet-drum-coral-lake
```

## 4.1 Type prefix

The prefix is mandatory and semantic.

Initial vocabulary:

```text
ent      Entity
mem      Memory
memrev   Memory revision
tag      Tag
anchor   Anchor
art      Artifact
obs      ObservationOccurrence
src      SourceRegion
repr     DerivedRepresentation
region   DerivedRegion
session  Session
res      Resource alias where a stable AI-facing ref is useful
```

Do not create a prefix until the referenced object is actually addressable/useful to model-facing cognition.

## 4.2 Body

The body consists of exactly four words.

Candidate vocabulary size:

```text
4096 frozen words
```

Therefore each word carries 12 bits and the body carries 48 bits of random candidate space.

Generation:

```text
CSPRNG
→ choose 4 vocabulary indices
→ build candidate
→ UNIQUE insert
→ retry on collision
```

The body is random and semantically neutral.

Never derive it from object meaning.

Bad:

```text
ent:alice-friend-school-kind
mem:tokyo-graduation-breakup-memory
```

Good:

```text
ent:amber-lotus-cello-river
mem:quiet-piano-mint-cloud
```

A reference must not become misleading when cognition later changes.

## 4.3 Separator

Canonical separator:

```text
-
```

No alternate `_`, `.`, `/`, or whitespace-separated spellings are accepted in canonical v0.5.

Reason:

- visually obvious as one lexical address;
- easy to copy;
- easy to parse without quoting;
- does not interfere with NQL whitespace rules;
- human-readable in logs.

Tokenizer measurements may influence the frozen wordlist, but do not create multiple separator syntaxes.

## 4.4 Alphabet/case

Canonical LexicalRefs are:

```text
lowercase ASCII only
```

No case-folding aliases.

Invalid:

```text
ENT:Amber-Lotus-Cello-River
ent:Amber-lotus-cello-river
```

Return a structured invalid-reference error, optionally with a canonical hint when the intended spelling is mechanically certain.

## 4.5 Wordlist constraints

The 4096-word vocabulary must be frozen/versioned and generated/curated with:

```text
lowercase ASCII
short/common words
cross-tokenizer benchmarking
no profanity/offensive terms
no near-spelling pairs where practical
no homophone-heavy pairs where practical
no morphology traps such as build/built
no NousQL reserved words
no cognition-domain keywords likely to look semantic
```

Examples of words to reserve/exclude from the random body include:

```text
memory
entity
relation
current
session
query
anchor
tag
resource
and
or
recent
```

The reference body must look opaque, not like a hidden query.

---

# 5. Why Four Words

With a 4096-word frozen vocabulary:

```text
4096 = 2^12
4 words = 48 random bits
```

v0.5 assumes **central minting with a UNIQUE constraint and retry**.

Under this model a candidate collision is not identity corruption; it causes regeneration.

The design does not rely on probabilistic uniqueness alone.

If future requirements introduce:

```text
offline minting
multi-writer disconnected generation
cross-deployment globally unique word refs
```

then the encoding must be revisited, likely toward five or more words or a namespace component.

Do not increase every current reference merely for hypothetical distributed generation.

---

# 6. LexicalRef Uniqueness Scope

LexicalRefs are unique across one Nous Wave deployment / Authority namespace, not merely within one Subject.

This prevents the same visible reference from naming different objects in different Subjects within the same installation.

The type prefix participates in the full reference identity.

A LexicalRef is never reused.

After purge/deletion:

```text
reference → tombstoned
```

not:

```text
reference → available for another object
```

This protects:

```text
logs
query traces
replay
model conversation context
external references
```

from identity rebinding.

---

# 7. Internal Authority Identity

## 7.1 Nous-owned objects

For Nous-owned Authority objects, use:

```text
UUIDv7 / 128-bit internal ID
```

stored in binary/native UUID form where practical.

Examples:

```text
MemoryId
MemoryRevisionId
TagId
AnchorId
ArtifactId
OccurrenceId
SourceRegionId
DerivedRepresentationId
DerivedRegionId
SessionId
```

The LLM normally never sees the raw UUID.

Mapping:

```text
LexicalRef
    ↕ one-to-one stable mapping
UUIDv7 AuthorityId
```

## 7.2 Why not SHA/BLAKE3 as object identity

Object identity is not content identity.

Memory, Entity, Tag and Anchor semantics may evolve while their identity remains stable.

Therefore:

```text
SHA/BLAKE3
```

must not be used as the primary semantic object identity merely because they are globally-looking strings.

Hashes remain appropriate for:

```text
content digest
deduplication
integrity
producer/config signatures
immutable payload identity/fingerprints
```

but not for mutable semantic object identity.

## 7.3 Artifact identity

Even when Artifact bytes have a BLAKE3/content digest:

```text
ArtifactId != ContentDigest
```

The digest answers:

> are these bytes identical?

The Artifact/Occurrence identity answers:

> which Authority object / encounter is this?

Do not collapse them.

---

# 8. Host-Owned Identity: Entity, Resource, External Object

Current architecture gives Host ownership to `EntityRef`, `ResourceRef`, and external `ObjectRef`.

NousQL must preserve this.

## 8.1 Entity

Conceptual mapping:

```text
ent:amber-lotus-cello-river
        ↓
Host EntityBinding
        ↓
Host-owned canonical EntityRef
```

Nous does not invent a second Kernel-owned Entity Authority merely to obtain a pretty ID.

The LexicalRef is an address alias/binding, not ownership transfer.

## 8.2 Resource

Likewise:

```text
res:quiet-piano-mint-cloud
        ↓
Host ResourceBinding
        ↓
Host-owned ResourceRef
```

For common resources the source query may simply use:

```text
@r("calendar")
```

The exact LexicalRef form is useful only when stable AI-facing addressing materially helps.

## 8.3 External object

External ObjectRefs remain Host-owned opaque references.

They may be presented through:

```text
@object("heptalogos:message:abc123")
```

A LexicalRef alias should not be created automatically for every external object unless repeated model-facing reference warrants it.

This avoids filling the lexical namespace with one-off objects.

---

# 9. Binding Data Model

A binding record conceptually contains:

```text
LexicalRef
ObjectKind
AuthorityNamespace
CanonicalIdentity
Status
CreatedAt
TombstonedAt?
```

For a Nous-owned object:

```text
CanonicalIdentity = UUIDv7 AuthorityId
```

For a Host-owned Entity:

```text
CanonicalIdentity = Host EntityRef
```

For a Host-owned Resource:

```text
CanonicalIdentity = Host ResourceRef
```

`LexicalRef` is never an authorization credential.

Possessing:

```text
ent:amber-lotus-cello-river
```

does not grant access to that Entity.

Authorization/visibility is checked independently.

---

# 10. Source Canonical Form vs Bound Canonical Form

NousQL has two relevant textual canonical levels.

## 10.1 Source Canonical Form (SCF)

Preserves user/model surface locators.

Input:

```text
@e( "Alice" )
```

SCF:

```text
@e("Alice")
```

It does not rewrite names to IDs before binding.

## 10.2 Bound Canonical Form (BCF)

After binding, every bindable identity is exact.

Example:

```text
@e(ent:amber-lotus-cello-river)
```

For joint entities:

```text
@e(ent:amber-lotus-cello-river,ent:quiet-piano-mint-cloud)
```

Participants are serialized in stable LexicalRef order because joint Entity focus is unordered.

BCF is appropriate for:

```text
query traces
replay
semantic equality
debugging
cross-turn exact follow-up
```

## 10.3 Execution identity

The Kernel receives canonical machine identities, not the LexicalRef strings as semantic Authority.

Conceptually:

```text
NQL source
→ SCF
→ bind
→ BCF
→ typed IDs / Host refs
→ Kernel QueryExpr
```

The private Host→Kernel protocol never relies on parsing LexicalRef text.

---

# 11. Query Results

Model-facing results should expose:

```text
lexical_ref
display_label/name when available
type
relevant result data
```

They should not normally expose raw UUID AuthorityIds.

Example conceptual result:

```json
{
  "entity": {
    "ref": "ent:amber-lotus-cello-river",
    "name": "Alice"
  }
}
```

For ambiguity:

```json
{
  "code": "AMBIGUOUS_REFERENCE",
  "surface": "Alice",
  "candidates": [
    {
      "ref": "ent:amber-lotus-cello-river",
      "name": "Alice",
      "hint": "work contact"
    },
    {
      "ref": "ent:quiet-piano-mint-cloud",
      "name": "Alice",
      "hint": "university friend"
    }
  ]
}
```

Candidate hints are presentation metadata, not part of identity.

---

# 12. Relation Parameter Simplification

v0.5 applies the same identity rule to typed constructor parameters.

Because `$rel.from` and `$rel.to` are already typed as `EntityLocator`, do not redundantly nest `@e(...)`.

Preferred source form:

```text
$rel(from="Alice",to="Bob",kind="trust")
```

Exact form:

```text
$rel(
  from=ent:amber-lotus-cello-river,
  to=ent:quiet-piano-mint-cloud,
  kind="trust"
)
```

Canonical single line:

```text
$rel(from=ent:amber-lotus-cello-river,to=ent:quiet-piano-mint-cloud,kind="trust")
```

Mixed form is legal:

```text
$rel(from="Alice",to=ent:quiet-piano-mint-cloud,kind="trust")
```

This follows the razor:

> If a parameter signature already declares the identity type, do not repeat the selector solely to restate that type.

The same principle may later simplify other typed constructors.

---

# 13. Parser Types

Conceptual TypeScript:

```ts
type EntityLocator =
  | { kind: "name"; value: string }
  | { kind: "lexical_ref"; value: EntityLexicalRef };

type BoundEntityLocator = {
  lexicalRef: EntityLexicalRef;
  canonicalEntityRef: EntityRef;
};
```

Selector:

```ts
type EntitySelectorSyntax = {
  kind: "entity_selector";
  participants: EntityLocator[];
  span: Span;
};
```

Bound:

```ts
type EntitySetAtom = {
  kind: "entity_set";
  participants: BoundEntityLocator[];
};
```

Names do not survive as semantic identity after binding, though they may remain in trace/presentation metadata.

---

# 14. Errors

Identity-specific errors:

```text
INVALID_LEXICAL_REF
UNKNOWN_REFERENCE
AMBIGUOUS_REFERENCE
REFERENCE_TYPE_MISMATCH
REFERENCE_NOT_VISIBLE
REFERENCE_TOMBSTONED
DUPLICATE_ENTITY_PARTICIPANT
```

Examples:

```text
@e(mem:amber-lotus-cello-river)
→ REFERENCE_TYPE_MISMATCH
```

```text
@ref(ENT:Amber-Lotus-Cello-River)
→ INVALID_LEXICAL_REF
```

```text
@e("Alice")
→ AMBIGUOUS_REFERENCE
```

when multiple exact Entity bindings exist.

No fuzzy auto-correction.

---

# 15. Locked v0.5 Conventions

The following are now LOCKED for the alpha implementation design:

```text
@e("Alice")
    name/alias locator, resolved at bind time

@e(ent:amber-lotus-cello-river)
    exact AI-facing Entity reference

DisplayName != LexicalRef != Authority identity

LexicalRef format:
    type:word-word-word-word

word count:
    4

word vocabulary:
    frozen 4096 words

body entropy:
    48 random bits

generation:
    CSPRNG + central UNIQUE insert + retry

separator:
    hyphen

case:
    lowercase ASCII

reuse:
    never; tombstone after deletion/purge

Nous-owned internal identity:
    UUIDv7 128-bit

Host-owned Entity/Resource/Object identity:
    remains Host canonical opaque ref

SHA/BLAKE3:
    content/integrity/signature roles only, not semantic object primary identity

source query:
    may use names

bound query:
    uses exact LexicalRefs

Kernel protocol:
    uses typed canonical IDs/refs, not raw NQL or lexical-ref parsing

model result:
    expose LexicalRef + display label; hide raw UUID normally
```

---

# 16. Deferred, Not Open-Ended

The following can still be empirically qualified without reopening the identity architecture:

1. Exact 4096-word vocabulary contents.
2. Cross-tokenizer word selection.
3. Whether a small checksum/error-detection feature becomes necessary after copy-error measurement.
4. Whether some high-volume one-off object types should not receive LexicalRefs by default.
5. Whether offline/distributed minting ever becomes a real requirement.

These are encoding/operational refinements.

They do not change the three-layer identity architecture.
