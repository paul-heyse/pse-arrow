-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse
--
-- Content-addressed authored source bundles for job execution (ADR-0114 Outcome 14).
-- A bundle is immutable: storing it again is a no-op. Its documents are inserted by
-- binary COPY (src/generated/copy.rs) with the bundle.

--! insert_bundle
INSERT INTO pse_ops.source_bundles (bundle_hash, manifest)
VALUES (:bundle_hash, :manifest)
ON CONFLICT (bundle_hash) DO NOTHING;

--! bundle
SELECT b FROM pse_ops.source_bundles AS b
WHERE b.bundle_hash = :bundle_hash::pse_ops.source_bundle_id;

--! documents
SELECT d FROM pse_ops.source_documents AS d
WHERE d.bundle_hash = :bundle_hash::pse_ops.source_bundle_id
ORDER BY d.path;
