-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse
--
-- The operational tables as the physical representation of the registry's operational
-- relations (Plan 22 O3; ADR-0112 Outcome 10). Relation `runtime.operational_<t>` is table
-- `pse_ops.<t>`; the migration-conformance test in pse-operations compares every column,
-- type, nullability, key, reference and enumeration CHECK with the registry.
--
-- Changes from the first migration:
-- - enumerations stored as text carry a CHECK on exactly the registry spellings
--   (attempts.kind, solutions.kind, solutions.backend, progress value kinds);
-- - job backoff is integral microseconds;
-- - progress values are typed columns in the runtime.solve_metrics value vocabulary, so a
--   number round-trips exactly (a jsonb payload loses the last digit of some doubles);
-- - stored seeds are typed vectors in source coordinates instead of an opaque payload;
-- - authored source documents are UTF-8 text.
--
-- The system holds no production data (ADR-0112, maintainer 2026-09-27). Rows whose old
-- representation has no typed equivalent (opaque solution payloads, jsonb progress
-- payloads) are discarded rather than converted.

-- --------------------------------------------------------------- attempts --

ALTER TABLE pse_ops.attempts DROP CONSTRAINT attempts_kind_check;
ALTER TABLE pse_ops.attempts ADD CONSTRAINT attempts_kind_check
    CHECK (kind IN ('modeling', 'simulation', 'fit'));

-- ---------------------------------------------------------------- sources --

ALTER TABLE pse_ops.source_documents
    ALTER COLUMN content TYPE text USING convert_from(content, 'UTF8');

-- ------------------------------------------------------------------- jobs --

ALTER TABLE pse_ops.jobs
    ADD COLUMN backoff_base_us bigint,
    ADD COLUMN backoff_cap_us bigint;
UPDATE pse_ops.jobs SET
    backoff_base_us = (extract(epoch FROM backoff_base) * 1000000)::bigint,
    backoff_cap_us = (extract(epoch FROM backoff_cap) * 1000000)::bigint;
-- Dropping the interval columns also drops the constraints that name them.
ALTER TABLE pse_ops.jobs
    DROP COLUMN backoff_base,
    DROP COLUMN backoff_cap,
    ALTER COLUMN backoff_base_us SET NOT NULL,
    ALTER COLUMN backoff_cap_us SET NOT NULL,
    ADD CONSTRAINT jobs_backoff_base_us_check CHECK (backoff_base_us >= 0),
    ADD CONSTRAINT jobs_backoff_cap_us_check CHECK (backoff_cap_us >= backoff_base_us);

-- ---------------------------------------------------------------- streams --

DELETE FROM pse_ops.progress_events;
ALTER TABLE pse_ops.progress_events
    DROP COLUMN payload,
    ADD COLUMN step integer NOT NULL CHECK (step >= 0),
    ADD COLUMN elapsed_seconds double precision NOT NULL
        CHECK (elapsed_seconds >= 0 AND elapsed_seconds < 'Infinity'::double precision);

-- Exactly the value field selected by `kind` is present (the runtime.solve_metrics rule);
-- a nonfinite real is recorded as unavailable `nonfinite`, never stored.
CREATE TABLE pse_ops.progress_values (
    attempt_id uuid NOT NULL REFERENCES pse_ops.attempts (attempt_id),
    seq bigint NOT NULL,
    name text NOT NULL CHECK (name <> ''),
    kind text NOT NULL CHECK (kind IN ('real', 'integer', 'boolean', 'text', 'unavailable')),
    "real" double precision CHECK (
        "real" > '-Infinity'::double precision AND "real" < 'Infinity'::double precision
    ),
    "integer" bigint,
    "boolean" boolean,
    "text" text,
    unavailable text CHECK (unavailable IN (
        'not_requested', 'not_computed', 'not_applicable', 'unsupported',
        'failed', 'unknown', 'nonfinite'
    )),
    PRIMARY KEY (attempt_id, seq, name),
    FOREIGN KEY (attempt_id, seq)
        REFERENCES pse_ops.progress_events (attempt_id, seq) ON DELETE CASCADE,
    CONSTRAINT progress_values_one_value CHECK (
        (kind = 'real' AND "real" IS NOT NULL AND "integer" IS NULL AND "boolean" IS NULL
            AND "text" IS NULL AND unavailable IS NULL)
        OR (kind = 'integer' AND "real" IS NULL AND "integer" IS NOT NULL AND "boolean" IS NULL
            AND "text" IS NULL AND unavailable IS NULL)
        OR (kind = 'boolean' AND "real" IS NULL AND "integer" IS NULL AND "boolean" IS NOT NULL
            AND "text" IS NULL AND unavailable IS NULL)
        OR (kind = 'text' AND "real" IS NULL AND "integer" IS NULL AND "boolean" IS NULL
            AND "text" IS NOT NULL AND unavailable IS NULL)
        OR (kind = 'unavailable' AND "real" IS NULL AND "integer" IS NULL AND "boolean" IS NULL
            AND "text" IS NULL AND unavailable IS NOT NULL)
    )
);

-- Retention removes the events of finished attempts by age.
CREATE INDEX progress_events_attempt_idx ON pse_ops.progress_events (attempt_id, step, seq);

-- -------------------------------------------------------------- solutions --

UPDATE pse_ops.incumbents SET solution_id = NULL WHERE solution_id IS NOT NULL;
DELETE FROM pse_ops.solutions;
ALTER TABLE pse_ops.solutions
    DROP COLUMN payload_format,
    DROP COLUMN payload_version,
    DROP COLUMN payload,
    DROP CONSTRAINT solutions_kind_check,
    ADD CONSTRAINT solutions_kind_check CHECK (kind IN ('root', 'nlp', 'highs')),
    ADD COLUMN backend text NOT NULL CHECK (backend IN (
        'ipopt', 'pounce', 'kinsol', 'highs', 'clarabel', 'diffsol', 'idas', 'scip'
    )),
    ADD COLUMN profile_stamp bytea NOT NULL CHECK (octet_length(profile_stamp) = 32),
    ADD COLUMN data_stamp bytea NOT NULL CHECK (octet_length(data_stamp) = 32),
    ADD COLUMN primal double precision[],
    ADD COLUMN lower_bound_duals double precision[],
    ADD COLUMN upper_bound_duals double precision[],
    ADD COLUMN column_duals double precision[],
    ADD COLUMN row_duals double precision[],
    ADD COLUMN basis_columns integer[],
    ADD COLUMN basis_rows integer[],
    -- The vectors present are fixed by the seed kind: a root seed is a primal only; an NLP
    -- seed is a primal with optional paired bound duals and row duals; a HiGHS seed holds
    -- any of a primal, paired column/row duals and a paired basis, and at least one.
    ADD CONSTRAINT solutions_vectors_check CHECK (
        (kind = 'root' AND primal IS NOT NULL AND lower_bound_duals IS NULL
            AND upper_bound_duals IS NULL AND column_duals IS NULL AND row_duals IS NULL
            AND basis_columns IS NULL AND basis_rows IS NULL)
        OR (kind = 'nlp' AND primal IS NOT NULL
            AND (lower_bound_duals IS NULL) = (upper_bound_duals IS NULL)
            AND column_duals IS NULL AND basis_columns IS NULL AND basis_rows IS NULL)
        OR (kind = 'highs' AND lower_bound_duals IS NULL AND upper_bound_duals IS NULL
            AND (column_duals IS NULL) = (row_duals IS NULL)
            AND (basis_columns IS NULL) = (basis_rows IS NULL)
            AND (primal IS NOT NULL OR column_duals IS NOT NULL OR basis_columns IS NOT NULL))
    );
