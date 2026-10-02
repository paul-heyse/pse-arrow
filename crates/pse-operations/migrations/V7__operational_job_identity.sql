-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Append-only V6 transition: preserve digest bytes and retain unknown historical frame provenance.
ALTER TABLE pse_ops.attempts RENAME COLUMN request_identity TO operational_job_identity;
ALTER TABLE pse_ops.attempts RENAME CONSTRAINT attempts_request_identity_not_null TO attempts_operational_job_identity_not_null;
ALTER TABLE pse_ops.attempts ADD COLUMN operational_job_frame text;
-- PG18's immutable V5 table recreation may have suffixed six original NOT NULL
-- names while the predecessor table existed. Only these already verified names move.
DO $migration$
DECLARE column_name text; old_name text;
BEGIN
  FOREACH column_name IN ARRAY ARRAY['binding_hash','job_id','point_index','state','study_id','updated_at'] LOOP
    old_name := 'study_points_' || column_name || '_not_null1';
    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conrelid='pse_ops.study_points'::regclass AND contype='n' AND conname=old_name) THEN
      EXECUTE format('ALTER TABLE pse_ops.study_points RENAME CONSTRAINT %I TO %I', old_name, 'study_points_' || column_name || '_not_null');
    END IF;
  END LOOP;
END;
$migration$;
UPDATE pse_ops.schema_support_state SET target='ffac44aa0986c9e8b1fa2546e12ca6dfe7604c32e16ff1b7f68eef263b7994d7',source=CASE WHEN source='fresh' THEN 'fresh-v25g' ELSE source END,ready=true WHERE history='operations';
COMMENT ON SCHEMA pse_ops IS 'pse.ops.schema.v1 13d5852cb2f9124849aef19386746094eacb72cf6d5768eddc6f35e2958b0b64';
