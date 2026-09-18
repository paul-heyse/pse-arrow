-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

SELECT terms.application_id,
  array_agg(pse_require_nonnull(terms.contribution_id) ORDER BY terms.contribution_id ASC) AS contribution_ids,
  min(pse_require_nonnull(terms.derivation_id)) AS derivation_id
FROM inferred.law_participation_decisions AS terms
WHERE terms.decision.kind = 'included'
GROUP BY terms.application_id
