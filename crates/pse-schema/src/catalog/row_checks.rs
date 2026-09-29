// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native SQL row contracts. These declarations bind identically for candidate
//! diagnostics, provider requirements and persisted Delta CHECK constraints.

use crate::model::Namespace;
use std::collections::BTreeMap;

pub(super) fn for_relation(namespace: Namespace, name: &str) -> BTreeMap<String, String> {
    use Namespace::{Reference, Runtime};
    let (check, sql) = match (namespace, name) {
        (Runtime, "solve_metrics" | "operational_progress_values") => (
            "one_evidence_value",
            [
                ("real", "real"),
                ("integer", "integer"),
                ("boolean", "boolean"),
                ("text", "text"),
                ("unavailable", "unavailable"),
            ]
            .iter()
            .map(|(kind, selected)| {
                let fields = ["real", "integer", "boolean", "text", "unavailable"]
                    .iter()
                    .map(|field| {
                        format!(
                            "\"{field}\" IS {}NULL",
                            if field == selected { "NOT " } else { "" }
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" AND ");
                format!("(kind = '{kind}' AND {fields})")
            })
            .collect::<Vec<_>>()
            .join(" OR "),
        ),
        (Reference, "units") => {
            // Version two: an atomic unit authors its measure, a defined unit only its
            // composition (ADR-0124).
            return BTreeMap::from([
                (
                    "positive_scale".into(),
                    "scale_to_canonical IS NULL OR scale_to_canonical > 0".into(),
                ),
                (
                    "atomic_or_defined".into(),
                    "(definition IS NULL AND dimension IS NOT NULL \
                     AND scale_to_canonical IS NOT NULL AND offset_to_canonical IS NOT NULL \
                     AND is_affine IS NOT NULL) \
                     OR (definition IS NOT NULL AND dimension IS NULL \
                     AND scale_to_canonical IS NULL AND offset_to_canonical IS NULL \
                     AND is_affine IS NULL AND reference_state_id IS NULL)"
                        .into(),
                ),
            ]);
        }
        (Reference, "quantity_types") => (
            "positive_nominal",
            "nominal_magnitude IS NULL OR nominal_magnitude > 0".into(),
        ),
        _ => return BTreeMap::new(),
    };
    BTreeMap::from([(check.into(), sql)])
}
