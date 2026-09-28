-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse
--
-- Cross-process cancellation (ADR-0114 Outcome 15): the durable flag is the authority;
-- NOTIFY only shortens latency.

--! request_cancel
UPDATE pse_ops.attempts
SET cancel_requested = true,
    cancel_requested_at = coalesce(cancel_requested_at, now()),
    updated_at = now()
WHERE attempt_id = :attempt_id::pse_ops.attempt_id;

--! cancel_requested
SELECT cancel_requested FROM pse_ops.attempts
WHERE attempt_id = :attempt_id::pse_ops.attempt_id;
