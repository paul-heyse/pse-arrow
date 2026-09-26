// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native SQL row contracts. These declarations bind identically for candidate
//! diagnostics, provider requirements and persisted Delta CHECK constraints.

use crate::model::Namespace;
use std::collections::BTreeMap;

pub(super) fn for_relation(namespace: Namespace, name: &str) -> BTreeMap<String, String> {
    use Namespace::{Authored, Reference, Runtime};
    let (check, sql) = match (namespace, name) {
        (Runtime, "solve_metrics") => ("one_evidence_value", [
            ("real", "real"), ("integer", "integer"), ("boolean", "boolean"), ("text", "text"), ("unavailable", "unavailable"),
        ].iter().map(|(kind, selected)| {
            let fields = ["real", "integer", "boolean", "text", "unavailable"].iter().map(|field| format!("\"{field}\" IS {}NULL", if field == selected { "NOT " } else { "" })).collect::<Vec<_>>().join(" AND ");
            format!("(kind = '{kind}' AND {fields})")
        }).collect::<Vec<_>>().join(" OR ")),
        (Reference, "units") => ("positive_scale", "scale_to_canonical > 0".into()),
        (Reference, "quantity_types") => (
            "positive_nominal",
            "nominal_magnitude IS NULL OR nominal_magnitude > 0".into(),
        ),
        (Authored, "continuous_domains") => ("ordered_bounds", r#""lower" < "upper""#.into()),
        (Authored, "template_contribution_contracts") => (
            "distinct_subject_axes",
            "subject.kind <> 'phase_species' OR subject.phase_species.member.kind <> 'axis' OR subject.phase_species.phase.kind <> 'axis' OR subject.phase_species.member.axis.position <> subject.phase_species.phase.axis.position".into(),
        ),
        _ => return BTreeMap::new(),
    };
    BTreeMap::from([(check.into(), sql)])
}
