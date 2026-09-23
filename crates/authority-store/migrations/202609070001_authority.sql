CREATE TABLE subjects (
    subject_id uuid PRIMARY KEY,
    created_at timestamptz NOT NULL,
    state_revision bigint NOT NULL DEFAULT 0,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active','suspended','purging')),
    metadata jsonb NOT NULL DEFAULT '{}'
);

CREATE TABLE artifacts (
    artifact_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    content_hash text NOT NULL CHECK (content_hash ~ '^[0-9a-f]{64}$'),
    byte_length bigint NOT NULL CHECK (byte_length >= 0),
    media_type text NOT NULL,
    storage_key text NOT NULL,
    created_at timestamptz NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}',
    UNIQUE(subject_id, content_hash)
);

CREATE TABLE observation_occurrences (
    occurrence_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    artifact_id uuid NULL REFERENCES artifacts(artifact_id) ON DELETE RESTRICT,
    source_class text NOT NULL,
    external_object_ref text NULL,
    occurred_at timestamptz NULL,
    observed_at timestamptz NOT NULL,
    conversation_ref text NULL,
    actor_entity_ref text NULL,
    context jsonb NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL
);
CREATE INDEX observation_occurrences_subject_observed_idx
    ON observation_occurrences(subject_id, observed_at DESC);

CREATE TABLE source_regions (
    source_region_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    artifact_id uuid NOT NULL REFERENCES artifacts(artifact_id) ON DELETE CASCADE,
    coordinate_kind text NOT NULL,
    coordinate jsonb NOT NULL,
    coordinate_hash text NOT NULL,
    parent_source_region_id uuid NULL REFERENCES source_regions(source_region_id),
    created_at timestamptz NOT NULL,
    UNIQUE(subject_id, artifact_id, coordinate_kind, coordinate_hash)
);

CREATE TABLE producer_signatures (
    producer_signature_id uuid PRIMARY KEY,
    signature_hash text UNIQUE NOT NULL,
    provider_class text NOT NULL,
    operation text NOT NULL,
    implementation text NOT NULL,
    model_identity text NULL,
    model_revision text NULL,
    preprocessing_identity text NOT NULL,
    preprocessing_revision text NOT NULL,
    config_digest text NOT NULL,
    created_at timestamptz NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}'
);

CREATE TABLE embedding_spaces (
    embedding_space_id uuid PRIMARY KEY,
    space_hash text UNIQUE NOT NULL,
    model_identity text NOT NULL,
    weights_revision text NOT NULL,
    task text NOT NULL,
    input_representation text NOT NULL,
    preprocessing_identity text NOT NULL,
    preprocessing_revision text NOT NULL,
    dimension integer NOT NULL CHECK (dimension > 0),
    normalization text NOT NULL,
    output_semantics text NOT NULL,
    created_at timestamptz NOT NULL
);

CREATE TABLE derived_representations (
    derived_representation_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    source_region_id uuid NOT NULL REFERENCES source_regions(source_region_id) ON DELETE CASCADE,
    representation_kind text NOT NULL,
    producer_signature_id uuid NOT NULL REFERENCES producer_signatures(producer_signature_id),
    revision integer NOT NULL CHECK (revision > 0),
    payload_text text NULL,
    payload_artifact_id uuid NULL REFERENCES artifacts(artifact_id),
    quality jsonb NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL,
    supersedes uuid NULL REFERENCES derived_representations(derived_representation_id),
    UNIQUE(subject_id, source_region_id, representation_kind, producer_signature_id, revision),
    CHECK (payload_text IS NOT NULL OR payload_artifact_id IS NOT NULL)
);

CREATE TABLE derived_regions (
    derived_region_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    derived_representation_id uuid NOT NULL REFERENCES derived_representations(derived_representation_id) ON DELETE CASCADE,
    coordinate_kind text NOT NULL,
    coordinate jsonb NOT NULL,
    coordinate_hash text NOT NULL,
    parent_derived_region_id uuid NULL REFERENCES derived_regions(derived_region_id),
    created_at timestamptz NOT NULL,
    UNIQUE(derived_representation_id, coordinate_kind, coordinate_hash)
);

CREATE TABLE coverage_needs (
    coverage_need_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    source_region_id uuid NOT NULL REFERENCES source_regions(source_region_id) ON DELETE CASCADE,
    representation_kind text NOT NULL,
    capability_operation text NOT NULL,
    requirement text NOT NULL CHECK (requirement IN ('required','preferred','opportunistic')),
    state text NOT NULL CHECK (state IN ('missing','scheduled','ready','unavailable','failed')),
    current_representation_id uuid NULL REFERENCES derived_representations(derived_representation_id),
    updated_at timestamptz NOT NULL,
    UNIQUE(subject_id, source_region_id, representation_kind, capability_operation)
);

CREATE TABLE derivations (
    derivation_id uuid PRIMARY KEY,
    derivation_key text UNIQUE NOT NULL,
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    source_region_id uuid NOT NULL REFERENCES source_regions(source_region_id) ON DELETE CASCADE,
    representation_kind text NOT NULL,
    producer_signature_id uuid NOT NULL REFERENCES producer_signatures(producer_signature_id),
    state text NOT NULL CHECK (state IN ('pending','running','succeeded','failed','unavailable')),
    successful_representation_id uuid NULL REFERENCES derived_representations(derived_representation_id),
    created_at timestamptz NOT NULL,
    updated_at timestamptz NOT NULL
);

CREATE TABLE derivation_attempts (
    attempt_id uuid PRIMARY KEY,
    derivation_id uuid NOT NULL REFERENCES derivations(derivation_id) ON DELETE CASCADE,
    attempt_no integer NOT NULL CHECK (attempt_no > 0),
    state text NOT NULL CHECK (state IN ('running','succeeded','transient_failed','permanent_failed')),
    lease_owner text NULL,
    lease_until timestamptz NULL,
    started_at timestamptz NOT NULL,
    finished_at timestamptz NULL,
    problem_code text NULL,
    problem_detail jsonb NOT NULL DEFAULT '{}',
    UNIQUE(derivation_id, attempt_no)
);

CREATE TABLE memory_objects (
    memory_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    memory_class text NOT NULL CHECK (memory_class IN ('specific','integrative','procedural')),
    current_revision_id uuid NULL,
    created_at timestamptz NOT NULL,
    status text NOT NULL CHECK (status IN ('active','suppressed','purging'))
);

CREATE TABLE memory_revisions (
    memory_revision_id uuid PRIMARY KEY,
    memory_id uuid NOT NULL REFERENCES memory_objects(memory_id) ON DELETE CASCADE,
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    revision_no integer NOT NULL CHECK (revision_no > 0),
    parent_revision_id uuid NULL REFERENCES memory_revisions(memory_revision_id),
    semantic_role text NOT NULL,
    title text NULL,
    representation_text text NOT NULL,
    attributes jsonb NOT NULL DEFAULT '{}',
    epistemic_class text NOT NULL,
    valid_from timestamptz NULL,
    valid_to timestamptz NULL,
    created_at timestamptz NOT NULL,
    revision_lifecycle text NOT NULL CHECK (revision_lifecycle IN ('current','superseded','revoked')),
    UNIQUE(memory_id, revision_no),
    CHECK (valid_to IS NULL OR valid_from IS NULL OR valid_to >= valid_from)
);
ALTER TABLE memory_objects
    ALTER COLUMN current_revision_id SET NOT NULL,
    ADD CONSTRAINT memory_objects_current_revision_fk
        FOREIGN KEY (current_revision_id) REFERENCES memory_revisions(memory_revision_id)
        DEFERRABLE INITIALLY DEFERRED;

CREATE TABLE memory_revision_evidence (
    memory_revision_id uuid NOT NULL REFERENCES memory_revisions(memory_revision_id) ON DELETE CASCADE,
    evidence_no integer NOT NULL CHECK (evidence_no >= 0),
    occurrence_id uuid NULL REFERENCES observation_occurrences(occurrence_id) ON DELETE CASCADE,
    source_region_id uuid NULL REFERENCES source_regions(source_region_id) ON DELETE CASCADE,
    derived_representation_id uuid NULL REFERENCES derived_representations(derived_representation_id) ON DELETE CASCADE,
    derived_region_id uuid NULL REFERENCES derived_regions(derived_region_id) ON DELETE CASCADE,
    support_role text NOT NULL CHECK (support_role IN ('direct','corroborating','interpretation','contradiction','contextual')),
    PRIMARY KEY(memory_revision_id, evidence_no),
    CHECK (num_nonnulls(occurrence_id, source_region_id, derived_representation_id, derived_region_id) = 1)
);

CREATE TABLE memory_revision_relations (
    from_revision_id uuid NOT NULL REFERENCES memory_revisions(memory_revision_id) ON DELETE CASCADE,
    to_revision_id uuid NOT NULL REFERENCES memory_revisions(memory_revision_id) ON DELETE CASCADE,
    relation text NOT NULL CHECK (relation IN ('supersedes','contradicts','integrates','derived_from','proceduralizes')),
    created_at timestamptz NOT NULL,
    PRIMARY KEY(from_revision_id, to_revision_id, relation)
);

CREATE TABLE entity_mentions (
    mention_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    occurrence_id uuid NULL REFERENCES observation_occurrences(occurrence_id) ON DELETE CASCADE,
    source_region_id uuid NULL REFERENCES source_regions(source_region_id) ON DELETE CASCADE,
    derived_region_id uuid NULL REFERENCES derived_regions(derived_region_id) ON DELETE CASCADE,
    surface text NOT NULL,
    semantic_role text NULL,
    created_at timestamptz NOT NULL,
    CHECK (num_nonnulls(occurrence_id, source_region_id, derived_region_id) >= 1)
);

CREATE TABLE entity_binding_revisions (
    binding_revision_id uuid PRIMARY KEY,
    mention_id uuid NOT NULL REFERENCES entity_mentions(mention_id) ON DELETE CASCADE,
    revision_no integer NOT NULL CHECK (revision_no > 0),
    entity_ref text NULL,
    binding_state text NOT NULL CHECK (binding_state IN ('bound','unbound','disputed')),
    host_resolution_ref text NULL,
    reason text NULL,
    created_at timestamptz NOT NULL,
    UNIQUE(mention_id, revision_no)
);

CREATE TABLE tags (
    tag_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    current_revision_id uuid NOT NULL,
    created_at timestamptz NOT NULL,
    status text NOT NULL CHECK (status IN ('active','superseded','revoked'))
);
CREATE TABLE tag_revisions (
    tag_revision_id uuid PRIMARY KEY,
    tag_id uuid NOT NULL REFERENCES tags(tag_id) ON DELETE CASCADE,
    revision_no integer NOT NULL CHECK (revision_no > 0),
    label text NOT NULL,
    description text NULL,
    kind_hint text NULL,
    origin text NOT NULL,
    producer_signature_id uuid NULL REFERENCES producer_signatures(producer_signature_id),
    created_at timestamptz NOT NULL,
    UNIQUE(tag_id, revision_no)
);
ALTER TABLE tags ADD CONSTRAINT tags_current_revision_fk
    FOREIGN KEY (current_revision_id) REFERENCES tag_revisions(tag_revision_id)
    DEFERRABLE INITIALLY DEFERRED;
CREATE TABLE memory_revision_tags (
    memory_revision_id uuid NOT NULL REFERENCES memory_revisions(memory_revision_id) ON DELETE CASCADE,
    tag_id uuid NOT NULL REFERENCES tags(tag_id) ON DELETE CASCADE,
    role text NOT NULL,
    ordinal integer NULL,
    order_provenance jsonb NULL,
    provenance jsonb NOT NULL DEFAULT '{}',
    PRIMARY KEY(memory_revision_id, tag_id, role)
);

CREATE TABLE anchors (
    anchor_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    current_revision_id uuid NOT NULL,
    created_at timestamptz NOT NULL,
    status text NOT NULL CHECK (status IN ('active','superseded','revoked'))
);
CREATE TABLE anchor_revisions (
    anchor_revision_id uuid PRIMARY KEY,
    anchor_id uuid NOT NULL REFERENCES anchors(anchor_id) ON DELETE CASCADE,
    revision_no integer NOT NULL CHECK (revision_no > 0),
    label text NULL,
    description text NOT NULL,
    origin text NOT NULL,
    producer_signature_id uuid NULL REFERENCES producer_signatures(producer_signature_id),
    created_at timestamptz NOT NULL,
    UNIQUE(anchor_id, revision_no)
);
ALTER TABLE anchors ADD CONSTRAINT anchors_current_revision_fk
    FOREIGN KEY (current_revision_id) REFERENCES anchor_revisions(anchor_revision_id)
    DEFERRABLE INITIALLY DEFERRED;
CREATE TABLE anchor_support (
    anchor_revision_id uuid NOT NULL REFERENCES anchor_revisions(anchor_revision_id) ON DELETE CASCADE,
    support_ref_kind text NOT NULL,
    support_ref text NOT NULL,
    support_role text NOT NULL,
    provenance jsonb NOT NULL DEFAULT '{}',
    PRIMARY KEY(anchor_revision_id, support_ref_kind, support_ref, support_role)
);

CREATE TABLE association_evidence (
    bridge_hint boolean NOT NULL DEFAULT false,
    association_evidence_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    from_ref_kind text NOT NULL,
    from_ref text NOT NULL,
    to_ref_kind text NOT NULL,
    to_ref text NOT NULL,
    association_kind text NOT NULL,
    polarity text NOT NULL CHECK (polarity IN ('positive','negative')),
    support_class text NOT NULL CHECK (support_class IN ('host_explicit','memory_evidence','consolidation','meaningful_use','derived_structure')),
    support_value double precision NOT NULL CHECK (support_value >= 0 AND support_value <= 1),
    occurrence_id uuid NULL REFERENCES observation_occurrences(occurrence_id) ON DELETE CASCADE,
    memory_revision_id uuid NULL REFERENCES memory_revisions(memory_revision_id) ON DELETE CASCADE,
    producer_signature_id uuid NULL REFERENCES producer_signatures(producer_signature_id),
    valid_from timestamptz NULL,
    valid_to timestamptz NULL,
    created_at timestamptz NOT NULL,
    revoked_at timestamptz NULL
);

CREATE TABLE resources (
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    resource_ref text NOT NULL,
    display_label text NULL,
    authority_class text NOT NULL,
    coverage jsonb NOT NULL DEFAULT '{}',
    query_dimensions jsonb NOT NULL DEFAULT '{}',
    modalities jsonb NOT NULL DEFAULT '[]',
    freshness_policy jsonb NOT NULL DEFAULT '{}',
    access_cost_class text NOT NULL,
    readiness text NOT NULL CHECK (readiness IN ('ready','degraded','unavailable')),
    updated_at timestamptz NOT NULL,
    PRIMARY KEY(subject_id, resource_ref)
);

CREATE TABLE cognitive_sessions (
    session_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    opened_at timestamptz NOT NULL,
    last_activity_at timestamptz NOT NULL,
    last_meaningful_use_at timestamptz NULL,
    closed_at timestamptz NULL,
    state_revision bigint NOT NULL DEFAULT 0,
    metadata jsonb NOT NULL DEFAULT '{}'
);
CREATE TABLE resident_refs (
    session_id uuid NOT NULL REFERENCES cognitive_sessions(session_id) ON DELETE CASCADE,
    ref_kind text NOT NULL,
    ref_value text NOT NULL,
    entered_at timestamptz NOT NULL,
    entry_reason text NOT NULL,
    last_meaningful_use_at timestamptz NULL,
    hold_until timestamptz NULL,
    state text NOT NULL CHECK (state IN ('resident','provisional','evicted')),
    metadata jsonb NOT NULL DEFAULT '{}',
    PRIMARY KEY(session_id, ref_kind, ref_value)
);
CREATE TABLE cognitive_use_events (
    use_event_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    session_id uuid NULL REFERENCES cognitive_sessions(session_id) ON DELETE SET NULL,
    ref_kind text NOT NULL,
    ref_value text NOT NULL,
    use_kind text NOT NULL,
    consumer_ref text NULL,
    occurred_at timestamptz NOT NULL,
    context jsonb NOT NULL DEFAULT '{}'
);

CREATE TABLE serving_generations (
    generation_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    kind text NOT NULL,
    space_signature text NULL,
    input_generation bigint NOT NULL,
    artifact_location text NOT NULL,
    artifact_hash text NOT NULL,
    state text NOT NULL CHECK (state IN ('building','ready','retired','failed')),
    built_at timestamptz NOT NULL,
    published_at timestamptz NULL,
    metadata jsonb NOT NULL DEFAULT '{}'
);
CREATE TABLE serving_current (
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    kind text NOT NULL,
    space_signature text NOT NULL DEFAULT '',
    generation_id uuid NOT NULL REFERENCES serving_generations(generation_id) ON DELETE CASCADE,
    PRIMARY KEY(subject_id, kind, space_signature)
);
CREATE INDEX memory_revisions_subject_time_idx
    ON memory_revisions(subject_id, created_at DESC);
CREATE INDEX memory_revision_evidence_occurrence_idx
    ON memory_revision_evidence(occurrence_id);
CREATE INDEX entity_binding_current_lookup_idx
    ON entity_binding_revisions(mention_id, revision_no DESC);
CREATE INDEX resident_refs_session_state_idx
    ON resident_refs(session_id, state, last_meaningful_use_at DESC);
-- Subject initialization source lineage is independent of Memory.
CREATE TABLE character_seeds (
    seed_revision_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects(subject_id),
    revision_no integer NOT NULL CHECK (revision_no > 0),
    artifact_id uuid NOT NULL REFERENCES artifacts(artifact_id),
    media_type text NOT NULL,
    provenance jsonb NOT NULL,
    created_at timestamptz NOT NULL,
    UNIQUE (subject_id, revision_no)
);
CREATE TABLE projection_watermarks (
    subject_id uuid NOT NULL REFERENCES subjects(subject_id) ON DELETE CASCADE,
    family text NOT NULL,
    space_signature text NOT NULL DEFAULT '',
    desired_revision bigint NOT NULL,
    PRIMARY KEY(subject_id, family, space_signature)
);
