-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Immutable ready barrier after the catalog-owned inventory transition (ADR-0146).
UPDATE pse_ops.schema_support_state SET source=CASE WHEN source='fresh' THEN 'fresh-v25f' ELSE source END,ready=true WHERE history='operations';
UPDATE pse_ops.schema_support_state SET ready=true WHERE history='catalog';
COMMENT ON SCHEMA pse_ops IS 'pse.ops.schema.v1 7a1649ecfeec229fd3fa87f96e8eb7239003146c66783edb6a0f9b66ea6f3d4e';
