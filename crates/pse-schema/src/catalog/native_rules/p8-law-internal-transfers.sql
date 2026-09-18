-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

SELECT law.application_id, term.contribution_id, term.derivation_id
FROM inferred.law_candidates AS candidate
JOIN compiled.law_applications AS law ON candidate.application_id = law.application_id
JOIN compiled.contributions AS term ON candidate.contribution_id = term.contribution_id
JOIN compiled.contributions AS paired ON term.transfer_connection_id = paired.transfer_connection_id
JOIN inferred.boundary_crossings AS cut ON law.scope_id = cut.scope_id AND term.transfer_connection_id = cut.connection_id
JOIN inferred.scope_members AS paired_scope ON law.scope_id = paired_scope.scope_id AND paired.owner_instance_id = paired_scope.entity_id
WHERE cut.classification = 'internal'
  AND term.law_family = paired.law_family
  AND term.contribution_id <> paired.contribution_id
  AND term.owner_instance_id <> paired.owner_instance_id
  AND ((term.orientation = 'into_scope' AND paired.orientation = 'out_of_scope')
    OR (term.orientation = 'out_of_scope' AND paired.orientation = 'into_scope'))
  AND term.quantity_type_id = paired.quantity_type_id
  AND term.product_id = paired.product_id
  AND (term.subject IS NOT DISTINCT FROM paired.subject)
