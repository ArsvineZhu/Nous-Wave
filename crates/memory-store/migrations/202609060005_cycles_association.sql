CREATE TABLE cognitive_cycles (
    cycle_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects,
    process_id uuid NOT NULL,
    status text NOT NULL CHECK(status IN ('active','resolved','insufficient','abandoned')),
    context jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    closed_at timestamptz,
    UNIQUE(subject_id,cycle_id)
);
CREATE TABLE cognitive_use_events (
    event_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL,
    cycle_id uuid,
    kind text NOT NULL,
    object_refs uuid[] NOT NULL,
    occurred_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    causation_id uuid,
    FOREIGN KEY(subject_id,cycle_id) REFERENCES cognitive_cycles(subject_id,cycle_id)
);
CREATE INDEX cognitive_use_object_refs ON cognitive_use_events USING gin(object_refs);
CREATE TABLE association_evidence (
    evidence_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL REFERENCES subjects,
    from_kind text NOT NULL,
    from_id uuid NOT NULL,
    to_kind text NOT NULL,
    to_id uuid NOT NULL,
    evidence_class text NOT NULL CHECK(evidence_class IN ('explicit_source','episode','temporal_adjacency','meaningful_co_recall','followed','host_explicit')),
    support double precision NOT NULL CHECK(support>=0 AND support<'Infinity'),
    source_id uuid,
    use_event_id uuid REFERENCES cognitive_use_events,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    FOREIGN KEY(subject_id,source_id) REFERENCES source_records(subject_id,source_id)
);
CREATE INDEX association_outgoing ON association_evidence(subject_id,from_id,from_kind);
CREATE INDEX association_incoming ON association_evidence(subject_id,to_id,to_kind);
