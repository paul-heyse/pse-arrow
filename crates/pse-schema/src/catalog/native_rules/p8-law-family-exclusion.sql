-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

SELECT law.application_id, term.contribution_id,
  named_struct('kind', 'excluded', 'included', NULL,
    'excluded', named_struct('reason', 'family_mismatch')) AS decision,
  term.derivation_id
FROM inferred.law_candidates AS candidate
JOIN compiled.law_applications AS law ON candidate.application_id = law.application_id
JOIN compiled.contributions AS term ON candidate.contribution_id = term.contribution_id

WHERE law.source_family <> term.law_family
