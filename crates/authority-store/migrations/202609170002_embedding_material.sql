CREATE TABLE embedding_materials (
    subject_id uuid NOT NULL REFERENCES subjects(subject_id),
    content_digest text NOT NULL,
    space_hash text NOT NULL,
    producer_hash text NOT NULL,
    vector real[] NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (subject_id,content_digest,space_hash,producer_hash)
);

-- Residency and its revision become visible in the same commit, including purge/eviction.
CREATE FUNCTION advance_resident_session() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  UPDATE cognitive_sessions SET state_revision=state_revision+1,last_activity_at=now()
    WHERE session_id=COALESCE(NEW.session_id,OLD.session_id);
  RETURN COALESCE(NEW,OLD);
END;
$$;
CREATE TRIGGER resident_session_revision AFTER INSERT OR UPDATE OR DELETE ON resident_refs
FOR EACH ROW EXECUTE FUNCTION advance_resident_session();
