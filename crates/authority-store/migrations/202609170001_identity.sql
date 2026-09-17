CREATE TABLE lexical_bindings (
    lexical_ref text PRIMARY KEY,
    object_kind text NOT NULL,
    canonical_ref text NOT NULL,
    wordlist_version integer NOT NULL CHECK (wordlist_version=1),
    tombstoned_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (object_kind,canonical_ref)
);
CREATE TABLE lexical_visibility (
    lexical_ref text NOT NULL REFERENCES lexical_bindings(lexical_ref),
    subject_id uuid NOT NULL REFERENCES subjects(subject_id),
    display_name text NOT NULL,
    aliases text[] NOT NULL,
    PRIMARY KEY(subject_id,lexical_ref)
);
CREATE INDEX lexical_aliases ON lexical_visibility USING gin(aliases);

CREATE FUNCTION tombstone_lexical_identity() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  UPDATE lexical_bindings SET tombstoned_at=now()
    WHERE object_kind=TG_ARGV[0] AND canonical_ref=(to_jsonb(OLD)->>TG_ARGV[1]);
  RETURN OLD;
END;
$$;
CREATE TRIGGER memory_lexical_tombstone AFTER DELETE ON memory_objects FOR EACH ROW EXECUTE FUNCTION tombstone_lexical_identity('memory','memory_id');
CREATE TRIGGER revision_lexical_tombstone AFTER DELETE ON memory_revisions FOR EACH ROW EXECUTE FUNCTION tombstone_lexical_identity('memory_revision','memory_revision_id');
CREATE TRIGGER artifact_lexical_tombstone AFTER DELETE ON artifacts FOR EACH ROW EXECUTE FUNCTION tombstone_lexical_identity('artifact','artifact_id');
CREATE TRIGGER occurrence_lexical_tombstone AFTER DELETE ON observation_occurrences FOR EACH ROW EXECUTE FUNCTION tombstone_lexical_identity('occurrence','occurrence_id');
CREATE TRIGGER source_lexical_tombstone AFTER DELETE ON source_regions FOR EACH ROW EXECUTE FUNCTION tombstone_lexical_identity('source_region','source_region_id');
CREATE TRIGGER representation_lexical_tombstone AFTER DELETE ON derived_representations FOR EACH ROW EXECUTE FUNCTION tombstone_lexical_identity('derived_representation','derived_representation_id');
CREATE TRIGGER region_lexical_tombstone AFTER DELETE ON derived_regions FOR EACH ROW EXECUTE FUNCTION tombstone_lexical_identity('derived_region','derived_region_id');
CREATE TRIGGER tag_lexical_tombstone AFTER DELETE ON tags FOR EACH ROW EXECUTE FUNCTION tombstone_lexical_identity('tag','tag_id');
CREATE TRIGGER anchor_lexical_tombstone AFTER DELETE ON anchors FOR EACH ROW EXECUTE FUNCTION tombstone_lexical_identity('anchor','anchor_id');
CREATE TRIGGER session_lexical_tombstone AFTER DELETE ON cognitive_sessions FOR EACH ROW EXECUTE FUNCTION tombstone_lexical_identity('session','session_id');

ALTER TABLE anchor_revisions ADD COLUMN confirmed boolean NOT NULL DEFAULT false;
