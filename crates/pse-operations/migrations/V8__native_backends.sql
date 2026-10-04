-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Preserve every row and previous enum ordinal while admitting the declared native profiles.
ALTER TYPE pse_ops.native_backend ADD VALUE 'uno' AFTER 'pounce_convex';
ALTER TYPE pse_ops.native_backend ADD VALUE 'petsc' AFTER 'uno';
UPDATE pse_ops.schema_support_state SET target='a5f63f9cff284c206f2423dedca4af5fc7db2804007d3e7a50f61fb19b16c307',source=CASE WHEN source='fresh' THEN 'fresh-v25h' ELSE source END,ready=true WHERE history='operations';
COMMENT ON SCHEMA pse_ops IS 'pse.ops.schema.v1 b3bf5e2861764d9065d8de18b0f0904f171b8fe4c90b073941498464f0ed1c5d';
