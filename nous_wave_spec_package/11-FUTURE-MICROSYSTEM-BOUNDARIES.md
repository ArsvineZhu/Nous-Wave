# 11 — Future Cognitive MicroSystem Boundaries

## 1. Purpose

This file prevents the Memory implementation from blocking future cognition without pretending those systems are already designed.

Everything here is **boundary-only** unless explicitly stated otherwise.

Do not create code/packages/tables for these systems in the current wave.

## 2. Shared rule

A future MicroSystem:

- owns its own semantic state;
- may consume Character Seed, Cognitive Material and Memory projections;
- may submit new self-generated Cognitive Material back through the normal ingest boundary;
- declares real dependencies/readiness;
- is invoked by a host/user/agent/scheduler, not by a hidden Nous Wave clock;
- does not gain authority over another MicroSystem's canonical state by convenience.

## 3. Persona / Self Model

### Intended responsibility

Long-lived self-related state such as values, preferences, traits, habits, capabilities, limitations, roles, boundaries, self-concepts and self-narrative where later research justifies those distinctions.

### Current frozen boundary

Consumes:

```text
Character Seed
Memory Episodes/MemoryItems/source evidence
explicit host/user self statements or corrections
future Reflection proposals
```

Provides:

```text
self/persona projection for host/model context
structured current/historical state
change proposals/revisions
```

### Explicitly not frozen

- facet taxonomy;
- trait numeric scales;
- stability/plasticity model;
- learning thresholds;
- self-narrative algorithm;
- prompt format.

### Character Seed relation

Character Seed remains foundational authored source. Persona state may evolve beyond it; neither silently overwrites the other.

## 4. Social World Model

### Intended responsibility

People/groups/roles/relationships and socially relevant history.

Consumes Memory episodes/entity refs and explicit identity/social evidence.

Owns future relationship state rather than storing it as generic Memory weight.

### Not frozen

- relationship dimensions;
- trust/familiarity/affinity measurement models;
- group-norm ontology;
- identity-resolution algorithm beyond evidence-based requirements.

## 5. Epistemic / Belief Model

### Intended responsibility

What the subject knows, believes, suspects, doubts or keeps as an open question, with evidence, confidence, contradiction and revision history.

Memory can remember propositions and evidence, but this MicroSystem owns current epistemic stance.

### Not frozen

- claim ontology;
- Bayesian/probabilistic model;
- confidence calibration;
- contradiction resolver;
- inference policy.

Do not make “LLM confidence” canonical belief confidence by default.

## 6. Goals / Commitments

### Intended distinction

```text
Goal       = desired future state
Commitment = socially/normatively meaningful obligation
```

Memory may remember them; their current fulfillment/status belongs to their future owner.

Nous Wave does not execute tasks or own a scheduler because these MicroSystems exist.

A host may use goal/commitment projections to schedule or act.

## 7. Reflection

### Intended responsibility

Explicitly invoked higher-order inspection of existing cognition that may produce proposals for other owners.

Conceptual flow:

```text
Memory + current owner projections
        ↓
Reflection operation
        ↓
reflection artifact
+ typed proposals
        ↓
owners validate/accept/reject
```

Reflection text itself is not automatic factual authority.

### Not frozen

- trigger policy;
- prompt/program;
- depth/modes;
- proposal taxonomy;
- model architecture.

There is no internal timer that starts reflection.

## 8. Diary

VCP Toolbox demonstrates a useful prototype lineage in which diary entries are file-oriented (`.txt/.md`), tagged, refer to other files and can be consolidated/archived. Nous Wave does not adopt its tightly coupled runtime design as specification.

### Intended responsibility

Diary is a self-authored narrative artifact over a period/experience, not a canonical fact database.

Expected output can enter Memory as ordinary Cognitive Material:

```text
origin_class    = SELF_GENERATED
semantic_class  = namespaced diary class
epistemic_class = NARRATIVE
media_type      = text/markdown (common, not mandatory)
source refs      = relevant Episodes/Memory/etc.
```

### Current Memory obligation

Memory must preserve the Diary artifact, provenance and epistemic classification without special-casing its file extension.

### Not frozen

- diary structure/style;
- whether there is one file/day/topic;
- when diary should be invoked;
- diary's role in Persona evolution;
- reflection depth;
- archive/consolidation rules.

A host may schedule diary generation. Nous Wave does not.

## 9. Dream / Simulation

VCP Toolbox's AgentDream prototype shows useful concepts—memory-seed recombination, association, dream narrative and interaction with diary/tool systems—but it also illustrates how dream cognition can become deeply entangled with an agent runtime and scheduler.

Nous Wave keeps the cognitive operation independent.

### Intended responsibility

Generate and explore synthetic/counterfactual/loosely constrained cognitive material using existing cognition as seeds.

Expected stored output:

```text
origin_class    = SELF_GENERATED
semantic_class  = namespaced simulation/dream class
epistemic_class = SIMULATED
```

### Hard boundary

Simulation content does not directly become observed fact, Social truth, Persona truth or Epistemic truth.

It may later support:

- hypotheses/open questions;
- weak association exploration;
- Reflection targets;
- host-visible narrative;

subject to future research.

### Not frozen

- dream seed selection;
- association dynamics;
- tool use/sandbox behavior;
- narrative style;
- memory retention class;
- whether/how simulation influences accessibility;
- scheduling.

## 10. Why no placeholders

The current Memory implementation already gives future systems the necessary shared facilities:

- subject identity;
- Character Seed source;
- general Cognitive Material;
- Artifact storage;
- provenance/epistemic classification;
- Memory recall/inspection;
- explicit invocation model.

More detailed future contracts should be added when those systems enter an implementation wave, after dedicated research.
