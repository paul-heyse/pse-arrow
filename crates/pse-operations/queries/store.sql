-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse
--
-- The store handle and its schema lifecycle (ADR-0114 Outcomes 21 and 23). Cornucopia
-- compiles these statements into crates/pse-operations-queries (`just codegen`).

--! server_version
SELECT current_setting('server_version_num')::integer AS version_num,
       current_setting('server_version') AS version;

--! schema_comment : (comment?)
SELECT obj_description(oid, 'pg_namespace') AS comment
FROM pg_namespace
WHERE nspname = 'pse_ops';

--! advisory_lock
SELECT true AS locked FROM pg_advisory_xact_lock(:key::bigint);

-- Delivered to listeners when the transaction commits; the payload is an identity.
--! notify
SELECT true AS notified FROM pg_notify(:channel, :payload);
