CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE memory_objects (
    object_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects,
    object_kind text NOT NULL CHECK(object_kind IN ('EPISODE','EPISODIC','SEMANTIC','PROCEDURAL','CONCEPTUAL','REFERENCE')),
    scope text NOT NULL,
    formation_key text,
    formation_class text NOT NULL DEFAULT 'SOURCE_REFERENCE' CHECK(formation_class IN ('SOURCE_REFERENCE','HOST_EPISODE','MODEL_EXTRACT','DUPLICATE_CONSOLIDATION','EPISODE_ABSTRACTION')),
    formation_metadata jsonb NOT NULL DEFAULT '{}',
    availability text NOT NULL DEFAULT 'ready' CHECK(availability IN ('ready','regenerating','failed')),
    superseded_by uuid,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    UNIQUE(subject_id, object_id),
    UNIQUE(subject_id, formation_key),
    FOREIGN KEY(subject_id, superseded_by) REFERENCES memory_objects(subject_id, object_id)
);
CREATE TABLE memory_revisions (
    revision_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL,
    object_id uuid NOT NULL,
    title text NOT NULL,
    representation_text text NOT NULL,
    origin_class text NOT NULL,
    semantic_class text NOT NULL,
    epistemic_class text NOT NULL,
    occurred tstzrange,
    approximate_time boolean NOT NULL DEFAULT false,
    observed_at timestamptz,
    recorded_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    content_digest text NOT NULL,
    reason text NOT NULL,
    authority_metadata jsonb,
    search_text tsvector GENERATED ALWAYS AS (to_tsvector('simple', title || ' ' || representation_text)) STORED,
    UNIQUE(subject_id, revision_id),
    UNIQUE(object_id, revision_id),
    FOREIGN KEY(subject_id, object_id) REFERENCES memory_objects(subject_id, object_id)
);
CREATE INDEX memory_revisions_fts ON memory_revisions USING gin(search_text);
CREATE INDEX memory_revisions_trigram ON memory_revisions USING gin(representation_text gin_trgm_ops);
CREATE INDEX memory_revisions_time ON memory_revisions USING gist(occurred);
CREATE INDEX memory_revisions_history ON memory_revisions(object_id, recorded_at DESC);
CREATE TABLE memory_current_heads (
    object_id uuid PRIMARY KEY REFERENCES memory_objects,
    revision_id uuid NOT NULL,
    FOREIGN KEY(object_id, revision_id) REFERENCES memory_revisions(object_id, revision_id)
);
CREATE TABLE memory_revision_sources (
    subject_id uuid NOT NULL,
    revision_id uuid NOT NULL,
    source_id uuid NOT NULL,
    PRIMARY KEY(revision_id, source_id),
    FOREIGN KEY(subject_id, revision_id) REFERENCES memory_revisions(subject_id, revision_id),
    FOREIGN KEY(subject_id, source_id) REFERENCES source_records(subject_id, source_id)
);
CREATE INDEX memory_revision_source_lookup ON memory_revision_sources(source_id, revision_id);
CREATE TABLE memory_revision_artifacts (
    subject_id uuid NOT NULL,
    revision_id uuid NOT NULL,
    artifact_id uuid NOT NULL,
    PRIMARY KEY(revision_id, artifact_id),
    FOREIGN KEY(subject_id, revision_id) REFERENCES memory_revisions(subject_id, revision_id),
    FOREIGN KEY(subject_id, artifact_id) REFERENCES artifacts(subject_id, artifact_id)
);
CREATE TABLE memory_revision_derivations (
    subject_id uuid NOT NULL,
    revision_id uuid NOT NULL,
    derivation_id uuid NOT NULL,
    PRIMARY KEY(revision_id, derivation_id),
    FOREIGN KEY(subject_id, revision_id) REFERENCES memory_revisions(subject_id, revision_id),
    FOREIGN KEY(subject_id, derivation_id) REFERENCES derivations(subject_id, derivation_id)
);
CREATE TABLE memory_revision_parents (
    subject_id uuid NOT NULL,
    revision_id uuid NOT NULL,
    parent_revision_id uuid NOT NULL,
    relation text NOT NULL CHECK(relation IN ('CORRECTION','WORLD_EVOLUTION','CONSOLIDATION')),
    PRIMARY KEY(revision_id, parent_revision_id),
    FOREIGN KEY(subject_id, revision_id) REFERENCES memory_revisions(subject_id, revision_id),
    FOREIGN KEY(subject_id, parent_revision_id) REFERENCES memory_revisions(subject_id, revision_id),
    CHECK(revision_id <> parent_revision_id)
);
CREATE TABLE episode_members (
    subject_id uuid NOT NULL,
    episode_id uuid NOT NULL,
    member_id uuid NOT NULL,
    PRIMARY KEY(episode_id,member_id),
    FOREIGN KEY(subject_id,episode_id) REFERENCES memory_objects(subject_id,object_id),
    FOREIGN KEY(subject_id,member_id) REFERENCES memory_objects(subject_id,object_id),
    CHECK(episode_id <> member_id)
);
CREATE TABLE memory_entities (
    entity_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects,
    entity_kind text NOT NULL,
    label text NOT NULL,
    identity_key text,
    UNIQUE(subject_id,entity_id),
    UNIQUE(subject_id,identity_key)
);
CREATE TABLE memory_entity_mentions (
    subject_id uuid NOT NULL,
    entity_id uuid NOT NULL,
    revision_id uuid NOT NULL,
    PRIMARY KEY(entity_id,revision_id),
    FOREIGN KEY(subject_id,entity_id) REFERENCES memory_entities(subject_id,entity_id),
    FOREIGN KEY(subject_id,revision_id) REFERENCES memory_revisions(subject_id,revision_id)
);
CREATE TABLE suppression_state (
    object_id uuid PRIMARY KEY REFERENCES memory_objects,
    suppressed boolean NOT NULL,
    reason text NOT NULL,
    changed_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
CREATE TABLE accessibility_state (
    object_id uuid PRIMARY KEY REFERENCES memory_objects,
    meaningful_uses bigint NOT NULL DEFAULT 0 CHECK(meaningful_uses >= 0),
    last_meaningful_use timestamptz,
    retention_hint double precision NOT NULL DEFAULT 0 CHECK(retention_hint BETWEEN 0 AND 1)
);
