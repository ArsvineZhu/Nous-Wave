CREATE TABLE subjects (
    subject_id uuid PRIMARY KEY,
    label text,
    metadata jsonb NOT NULL DEFAULT '{}',
    revision bigint NOT NULL DEFAULT 1 CHECK (revision > 0),
    memory_enabled boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp()
);

CREATE TABLE source_records (
    source_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects,
    source_kind text NOT NULL,
    origin_class text NOT NULL CHECK (origin_class IN ('HUMAN','TOOL','HOST','EXTERNAL_SYSTEM','SELF_GENERATED','IMPORTED')),
    semantic_class text NOT NULL,
    epistemic_class text NOT NULL CHECK (epistemic_class IN ('OBSERVED','REPORTED','DERIVED','INFERRED','NARRATIVE','SIMULATED')),
    scope text NOT NULL DEFAULT 'subject-private',
    occurred tstzrange,
    approximate_time boolean NOT NULL DEFAULT false,
    observed_at timestamptz,
    recorded_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    actor text,
    invocation_id text,
    idempotency_key text,
    metadata jsonb NOT NULL DEFAULT '{}',
    UNIQUE(subject_id, source_id),
    UNIQUE(subject_id, idempotency_key)
);

CREATE TABLE source_parents (
    subject_id uuid NOT NULL,
    source_id uuid NOT NULL,
    parent_source_id uuid NOT NULL,
    PRIMARY KEY(source_id, parent_source_id),
    FOREIGN KEY(subject_id, source_id) REFERENCES source_records(subject_id, source_id),
    FOREIGN KEY(subject_id, parent_source_id) REFERENCES source_records(subject_id, source_id),
    CHECK(source_id <> parent_source_id)
);

CREATE TABLE artifacts (
    artifact_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects,
    content_hash text NOT NULL CHECK(content_hash ~ '^[0-9a-f]{64}$'),
    media_type text NOT NULL,
    byte_size bigint NOT NULL CHECK(byte_size >= 0),
    origin_class text NOT NULL,
    semantic_class text NOT NULL,
    epistemic_class text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    UNIQUE(subject_id, artifact_id)
);
CREATE INDEX artifacts_content_hash ON artifacts(content_hash);

CREATE TABLE artifact_sources (
    subject_id uuid NOT NULL,
    artifact_id uuid NOT NULL,
    source_id uuid NOT NULL,
    PRIMARY KEY(artifact_id, source_id),
    FOREIGN KEY(subject_id, artifact_id) REFERENCES artifacts(subject_id, artifact_id),
    FOREIGN KEY(subject_id, source_id) REFERENCES source_records(subject_id, source_id)
);

CREATE TABLE character_seed_revisions (
    revision_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects,
    artifact_id uuid NOT NULL,
    authored_by text NOT NULL,
    parent_revision uuid,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    UNIQUE(subject_id, revision_id),
    FOREIGN KEY(subject_id, artifact_id) REFERENCES artifacts(subject_id, artifact_id),
    FOREIGN KEY(subject_id, parent_revision) REFERENCES character_seed_revisions(subject_id, revision_id)
);

CREATE TABLE character_seed_heads (
    subject_id uuid PRIMARY KEY REFERENCES subjects,
    revision_id uuid NOT NULL,
    FOREIGN KEY(subject_id, revision_id) REFERENCES character_seed_revisions(subject_id, revision_id)
);
