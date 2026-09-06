CREATE TABLE memory_operation_results (
    operation_id uuid PRIMARY KEY,
    subject_id uuid NOT NULL,
    kind text NOT NULL,
    request jsonb NOT NULL,
    state text NOT NULL,
    result jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT clock_timestamp()
);
