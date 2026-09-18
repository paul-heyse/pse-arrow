-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

SELECT law.application_id, term.contribution_id, term.derivation_id
FROM compiled.law_applications AS law
JOIN inferred.scope_members AS scope ON law.scope_id = scope.scope_id
JOIN compiled.contributions AS term ON scope.entity_id = term.owner_instance_id
