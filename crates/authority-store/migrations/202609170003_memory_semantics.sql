ALTER TABLE memory_objects ADD COLUMN accessibility_mode text NOT NULL DEFAULT 'auto'
    CHECK (accessibility_mode IN ('auto','normal','deep','explicit'));
ALTER TABLE memory_revisions ADD COLUMN revision_intent text NULL
    CHECK (revision_intent IN ('correct','rephrase','reinterpret','revoke'));

CREATE TABLE memory_revision_entities (
    memory_revision_id uuid NOT NULL REFERENCES memory_revisions(memory_revision_id) ON DELETE CASCADE,
    entity_ref text NOT NULL,
    role text NOT NULL,
    provenance jsonb NOT NULL,
    PRIMARY KEY(memory_revision_id,entity_ref,role)
);
CREATE INDEX memory_revision_entities_identity ON memory_revision_entities(entity_ref,memory_revision_id);

-- Queryable lineage, derived from actual encounter records. No invented Memory observation time.
CREATE VIEW memory_evidence_occurrences AS
SELECT DISTINCT e.memory_revision_id,o.occurrence_id,o.occurred_at,o.observed_at,o.source_class
FROM memory_revision_evidence e
JOIN memory_revisions m ON m.memory_revision_id=e.memory_revision_id
LEFT JOIN source_regions direct_region ON direct_region.source_region_id=e.source_region_id
LEFT JOIN derived_representations representation ON representation.derived_representation_id=e.derived_representation_id
LEFT JOIN source_regions representation_region ON representation_region.source_region_id=representation.source_region_id
LEFT JOIN derived_regions region ON region.derived_region_id=e.derived_region_id
LEFT JOIN derived_representations region_representation ON region_representation.derived_representation_id=region.derived_representation_id
LEFT JOIN source_regions region_source ON region_source.source_region_id=region_representation.source_region_id
JOIN observation_occurrences o ON o.subject_id=m.subject_id AND
    (o.occurrence_id=e.occurrence_id OR o.artifact_id=COALESCE(direct_region.artifact_id,representation_region.artifact_id,region_source.artifact_id));

CREATE VIEW memory_temporal_evidence AS
SELECT memory_revision_id,min(occurred_at) AS occurred_min,max(occurred_at) AS occurred_max,
    min(observed_at) AS observed_min,max(observed_at) AS observed_max
FROM memory_evidence_occurrences GROUP BY memory_revision_id;
