-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

SELECT law.application_id, law.derivation_id
FROM compiled.law_applications AS law
LEFT ANTI JOIN inferred.law_ordered_terms AS ordered ON law.application_id = ordered.application_id
