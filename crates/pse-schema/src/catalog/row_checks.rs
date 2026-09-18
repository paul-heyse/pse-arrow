// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native SQL row contracts. These declarations bind identically for candidate
//! diagnostics, provider requirements and persisted Delta CHECK constraints.

use crate::model::Namespace;
use std::collections::BTreeMap;

pub(super) fn for_relation(namespace: Namespace, name: &str) -> BTreeMap<String, String> {
    use Namespace::{Authored, Compiled, Inferred, Normalized, Reference};
    let (check, sql) = match (namespace, name) {
        (Normalized, "expression_index_bindings") => ("domain_name", "domain.kind = 'actual' OR domain.template.name <> ''".into()),
        (Normalized, "predicate_nodes") => ("domain_name", r#"value.kind <> 'in' OR value."in".domain.kind = 'actual' OR value."in".domain.template.name <> ''"#.into()),
        (Reference, "units") => ("positive_scale", "scale_to_canonical > 0".into()),
        (Reference, "quantity_types") => (
            "positive_nominal",
            "nominal_magnitude IS NULL OR nominal_magnitude > 0".into(),
        ),
        (Authored, "continuous_domains") => ("ordered_bounds", r#""lower" < "upper""#.into()),
        (Inferred, "tear_candidates") => ("nonnegative_cost", "cost >= 0".into()),
        (Authored, "template_contribution_contracts") | (Compiled, "contributions" | "law_applications") => (
            "distinct_subject_axes",
            "subject.kind <> 'phase_species' OR subject.phase_species.member.kind <> 'axis' OR subject.phase_species.phase.kind <> 'axis' OR subject.phase_species.member.axis.position <> subject.phase_species.phase.axis.position".into(),
        ),
        _ => return BTreeMap::new(),
    };
    let mut checks = BTreeMap::from([(check.into(), sql)]);
    if namespace == Compiled && name == "law_applications" {
        checks.insert("species_phase_slice".into(),
            "subject.kind NOT IN ('species', 'element') OR coalesce(subject.species.phase.kind, subject.element.phase.kind, 'fixed') = 'fixed'".into());
    }
    checks
}
