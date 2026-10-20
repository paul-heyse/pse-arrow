-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse
-- Whole event rows only. Fields absent at this checkpoint stay NULL.
-- Unsigned identities/counts use explicit canonical text annotations, never wrapped SQL integers.
-- Earlier traces without these companions leave the corresponding columns unavailable.
SELECT s.track_id, s.ts, s.name AS phase,
       EXTRACT_ARG(s.arg_set_id, 'debug.operation_id') AS operation_id,
       EXTRACT_ARG(s.arg_set_id, 'debug.producer_sequence.u64_text') AS producer_sequence,
       EXTRACT_ARG(s.arg_set_id, 'debug.family') AS family,
       EXTRACT_ARG(s.arg_set_id, 'debug.source_sequence.u64_text') AS source_sequence,
       EXTRACT_ARG(s.arg_set_id, 'debug.family_sequence.u64_text') AS family_sequence,
       EXTRACT_ARG(s.arg_set_id, 'debug.generation.u64_text') AS generation,
       EXTRACT_ARG(s.arg_set_id, 'debug.history_partial') AS history_partial,
       EXTRACT_ARG(s.arg_set_id, 'debug.iteration.u64_text') AS iteration,
       EXTRACT_ARG(s.arg_set_id, 'debug.primal.native') AS primal_native,
       EXTRACT_ARG(s.arg_set_id, 'debug.inf_pr') AS pounce_primal,
       EXTRACT_ARG(s.arg_set_id, 'debug.step.norm') AS step_norm,
       EXTRACT_ARG(s.arg_set_id, 'debug.d_norm') AS pounce_step,
       EXTRACT_ARG(s.arg_set_id, 'debug.step_available') AS step_available,
       EXTRACT_ARG(s.arg_set_id, 'debug.step_norm_kind') AS step_norm_kind,
       EXTRACT_ARG(s.arg_set_id, 'debug.step_coordinates') AS step_coordinates
FROM slice s WHERE s.ts >= __START__ AND s.ts < __END__ AND EXTRACT_ARG(s.arg_set_id, 'debug.family') IS NOT NULL
ORDER BY s.track_id, s.ts, s.id;
