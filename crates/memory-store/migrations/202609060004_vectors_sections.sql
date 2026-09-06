CREATE TABLE document_sections (
    artifact_id uuid PRIMARY KEY REFERENCES artifacts,
    subject_id uuid NOT NULL REFERENCES subjects,
    source_artifact uuid NOT NULL REFERENCES artifacts,
    ordinal integer NOT NULL,
    byte_start bigint NOT NULL,
    byte_end bigint NOT NULL,
    section_text text NOT NULL,
    search_text tsvector GENERATED ALWAYS AS (to_tsvector('simple', section_text)) STORED,
    UNIQUE(source_artifact,ordinal)
);
CREATE INDEX document_sections_fts ON document_sections USING gin(search_text);
CREATE INDEX document_sections_trigram ON document_sections USING gin(section_text gin_trgm_ops);
CREATE TABLE retained_embeddings (
    embedding_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL,
    revision_id uuid NOT NULL,
    model_identity text NOT NULL,
    model_revision text NOT NULL,
    preprocessing_identity text NOT NULL,
    config_digest text NOT NULL,
    projection_table text NOT NULL,
    vector real[] NOT NULL,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    UNIQUE(revision_id,model_identity,model_revision,preprocessing_identity,config_digest),
    FOREIGN KEY(subject_id,revision_id) REFERENCES memory_revisions(subject_id,revision_id),
    CHECK(cardinality(vector)>0)
);
