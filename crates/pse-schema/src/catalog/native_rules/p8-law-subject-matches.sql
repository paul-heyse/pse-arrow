-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

SELECT law.application_id, term.contribution_id, term.derivation_id
FROM inferred.law_candidates AS candidate
JOIN compiled.law_applications AS law ON candidate.application_id = law.application_id
JOIN compiled.contributions AS term ON candidate.contribution_id = term.contribution_id
WHERE (law.subject.kind = 'total'
    OR (law.subject.kind = 'species' AND term.subject.kind IN ('species', 'phase_species'))
    OR (law.subject.kind = 'element' AND law.subject_projection = 'species_to_element'
        AND term.subject.kind IN ('species', 'phase_species'))
    OR law.subject.kind = term.subject.kind)
  AND (coalesce(law.subject.species.member.fixed.entity_id, law.subject.element.member.fixed.entity_id, law.subject.phase_species.member.fixed.entity_id) IS NULL OR coalesce(term.subject.species.member.fixed.entity_id, term.subject.element.member.fixed.entity_id, term.subject.phase_species.member.fixed.entity_id) IS NULL OR coalesce(law.subject.species.member.fixed.entity_id, law.subject.element.member.fixed.entity_id, law.subject.phase_species.member.fixed.entity_id) = coalesce(term.subject.species.member.fixed.entity_id, term.subject.element.member.fixed.entity_id, term.subject.phase_species.member.fixed.entity_id) OR law.subject_projection = 'species_to_element')
  AND (coalesce(law.subject.total.phase.fixed.entity_id, law.subject.energy.phase.fixed.entity_id, law.subject.momentum.phase.fixed.entity_id, law.subject.species.phase.fixed.entity_id, law.subject.element.phase.fixed.entity_id, law.subject.phase_species.phase.fixed.entity_id) IS NULL OR coalesce(term.subject.total.phase.fixed.entity_id, term.subject.energy.phase.fixed.entity_id, term.subject.momentum.phase.fixed.entity_id, term.subject.phase_species.phase.fixed.entity_id) IS NULL OR coalesce(law.subject.total.phase.fixed.entity_id, law.subject.energy.phase.fixed.entity_id, law.subject.momentum.phase.fixed.entity_id, law.subject.species.phase.fixed.entity_id, law.subject.element.phase.fixed.entity_id, law.subject.phase_species.phase.fixed.entity_id) = coalesce(term.subject.total.phase.fixed.entity_id, term.subject.energy.phase.fixed.entity_id, term.subject.momentum.phase.fixed.entity_id, term.subject.phase_species.phase.fixed.entity_id))
