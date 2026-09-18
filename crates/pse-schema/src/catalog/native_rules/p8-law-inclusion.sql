-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

SELECT law.application_id, term.contribution_id,
  named_struct('kind', 'included',
    'included', named_struct('sign',
      CASE WHEN term.orientation IN ('into_scope', 'generation') THEN 'positive'
           WHEN term.orientation IN ('out_of_scope', 'accumulation') THEN 'negative' END,
      'conversion_id', NULL),
    'excluded', NULL) AS decision,
  term.derivation_id
FROM inferred.law_candidates AS candidate
JOIN compiled.law_applications AS law ON candidate.application_id = law.application_id
JOIN compiled.contributions AS term ON candidate.contribution_id = term.contribution_id
JOIN inferred.law_subject_matches AS subject
  ON law.application_id = subject.application_id AND term.contribution_id = subject.contribution_id
LEFT ANTI JOIN inferred.law_internal_transfers AS internal
  ON law.application_id = internal.application_id AND term.contribution_id = internal.contribution_id
WHERE law.source_family = term.law_family
  AND term.orientation IN ('into_scope', 'generation', 'out_of_scope', 'accumulation')
