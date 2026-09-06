CREATE TABLE derivations (
    derivation_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects,
    processor_identity text NOT NULL,
    processor_revision text NOT NULL,
    preprocessing_identity text NOT NULL,
    config_digest text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    UNIQUE(subject_id, derivation_id)
);
CREATE TABLE derivation_inputs (
    subject_id uuid NOT NULL,
    derivation_id uuid NOT NULL,
    artifact_id uuid NOT NULL,
    PRIMARY KEY(derivation_id, artifact_id),
    FOREIGN KEY(subject_id, derivation_id) REFERENCES derivations(subject_id, derivation_id),
    FOREIGN KEY(subject_id, artifact_id) REFERENCES artifacts(subject_id, artifact_id)
);
CREATE TABLE derivation_outputs (
    subject_id uuid NOT NULL,
    derivation_id uuid NOT NULL,
    artifact_id uuid NOT NULL,
    PRIMARY KEY(derivation_id, artifact_id),
    FOREIGN KEY(subject_id, derivation_id) REFERENCES derivations(subject_id, derivation_id),
    FOREIGN KEY(subject_id, artifact_id) REFERENCES artifacts(subject_id, artifact_id)
);

CREATE TABLE processing_obligations (
    obligation_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects,
    kind text NOT NULL,
    payload_version integer NOT NULL CHECK(payload_version = 1),
    source_id uuid,
    payload jsonb NOT NULL,
    state text NOT NULL DEFAULT 'pending' CHECK(state IN ('pending','running','succeeded','failed')),
    attempt_id uuid,
    lease_until timestamptz,
    attempts integer NOT NULL DEFAULT 0,
    problem_code text,
    result jsonb,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    UNIQUE(subject_id, kind, source_id),
    FOREIGN KEY(subject_id, source_id) REFERENCES source_records(subject_id, source_id),
    CHECK((state='running') = (attempt_id IS NOT NULL AND lease_until IS NOT NULL))
);
CREATE INDEX processing_pending ON processing_obligations(state, lease_until, created_at)
    WHERE state IN ('pending','running');
