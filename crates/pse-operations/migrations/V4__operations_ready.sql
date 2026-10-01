-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Immutable Plan 25d -> 25e transition (ADR-0146); append a new version for later targets.
UPDATE pse_ops.schema_support_state SET ready=true WHERE history='operations' AND target='b3e1ba34cd0f964ee1941021e52371f3d83b12373bd6b2b3656f4eebeea1c9d6';
COMMENT ON SCHEMA pse_ops IS 'pse.ops.schema.v1 60293f46ba51830bb2a376d9554cae582e4f715234fa7163c38e372994de915b';
