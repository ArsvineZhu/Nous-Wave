CREATE TABLE runtime_checkpoints (
    subject_id uuid NOT NULL REFERENCES subjects(subject_id),
    session_id uuid NOT NULL REFERENCES cognitive_sessions(session_id),
    owner_kind text NOT NULL CHECK (owner_kind IN ('focus', 'context', 'steward')),
    owner_key text NOT NULL CHECK (length(owner_key) BETWEEN 1 AND 256),
    schema_version integer NOT NULL CHECK (schema_version > 0),
    revision bigint NOT NULL CHECK (revision > 0),
    payload bytea NOT NULL CHECK (octet_length(payload) <= 1048576),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (session_id, owner_kind, owner_key)
);
ALTER TABLE cognitive_sessions ADD COLUMN active_focus_key text NULL;

CREATE TABLE observation_request_bindings (
    request_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects(subject_id),
    request_digest text NOT NULL,
    occurrence_id uuid NOT NULL REFERENCES observation_occurrences(occurrence_id),
    accepted jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE memory_objects ADD COLUMN head_revision bigint NOT NULL DEFAULT 0;
CREATE FUNCTION advance_memory_head() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  NEW.head_revision := OLD.head_revision + 1;
  RETURN NEW;
END;
$$;
CREATE TRIGGER memory_head_revision BEFORE UPDATE ON memory_objects
FOR EACH ROW EXECUTE FUNCTION advance_memory_head();
