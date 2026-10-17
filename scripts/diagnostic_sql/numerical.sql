-- Whole event rows only. Fields absent at this checkpoint stay NULL.
SELECT s.track_id, s.ts, s.name AS phase,
       EXTRACT_ARG(s.arg_set_id, 'debug.family') AS family,
       (SELECT display_value FROM args WHERE arg_set_id=s.arg_set_id AND key='debug.source_sequence') AS source_sequence,
       EXTRACT_ARG(s.arg_set_id, 'debug.generation') AS generation,
       EXTRACT_ARG(s.arg_set_id, 'debug.history_partial') AS history_partial,
       EXTRACT_ARG(s.arg_set_id, 'debug.iteration') AS iteration,
       EXTRACT_ARG(s.arg_set_id, 'debug.primal.native') AS primal_native,
       EXTRACT_ARG(s.arg_set_id, 'debug.inf_pr') AS pounce_primal,
       EXTRACT_ARG(s.arg_set_id, 'debug.step.norm') AS step_norm,
       EXTRACT_ARG(s.arg_set_id, 'debug.d_norm') AS pounce_step,
       EXTRACT_ARG(s.arg_set_id, 'debug.step_available') AS step_available,
       EXTRACT_ARG(s.arg_set_id, 'debug.step_norm_kind') AS step_norm_kind,
       EXTRACT_ARG(s.arg_set_id, 'debug.step_coordinates') AS step_coordinates
FROM slice s WHERE EXTRACT_ARG(s.arg_set_id, 'debug.family') IS NOT NULL
ORDER BY s.track_id, s.ts, s.id;
