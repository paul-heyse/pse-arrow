// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared source closure for bounded actual-reference compiler/runtime controls.
use crate::scientific_inputs;
use pse_authoring::language::{Declaration, IdentityPolicy};
use std::collections::BTreeMap;

/// Read current source documents, including the process consumers and both caloric bindings.
/// External data transport occurrences are outside this selected declaration control; numerical
/// cases supply small synthetic keyed parameter rows for the selected declarations. Full package/data transport is
/// qualified in Plan 25k. No source function or process equation is copied into this test.
pub(crate) fn reference_family_sources(family: &str) -> String {
    // The retained seed selection below has fixture data lineage to the synthetic
    // pressure rows. Admit that data-only dependency without changing its science.
    scientific_inputs::family_sources(family).replacen(
        "package bt_ideal {",
        "package bt_ideal { use transfer_parameters @\"1.0.0\";",
        1,
    )
}

pub(crate) fn rows(text: &str) -> Vec<Declaration> {
    let mut parsed = scientific_inputs::parse_sources(text, IdentityPolicy::Named);
    // Data transport is not this compiler control's scope. Keep scientific declarations
    // byte-derived from the current documents, while omitting external dataset bindings
    // and their derived dataset occurrences. Numerical cases supply selected synthetic data.
    let packages = parsed
        .iter()
        .filter(|row| row.parent_id.is_none())
        .map(|row| (row.declaration_id, row.name.clone()))
        .collect::<BTreeMap<_, _>>();
    // Retain the actual seed's pressure choice and its completeness/coherence
    // contract. Only its test data lineage changes: the selected synthetic rows
    // replace the external parameter transport for this bounded control.
    for row in &mut parsed {
        if row.name == "pressure_bindings"
            && row
                .parent_id
                .and_then(|id| packages.get(&id))
                .is_some_and(|p| p == "bt_ideal")
            && let Some(dataset) = &mut row.value.dataset
        {
            for lineage in &mut dataset.provenance.lineage {
                if lineage.path == ["idaes_thermo", "rpp4_pressure_oracles"] {
                    lineage.path = vec!["transfer_parameters".into(), "pressure".into()];
                }
            }
        }
    }
    let mut excluded = parsed
        .iter()
        .filter(|row| {
            row.value
                .dataset
                .as_ref()
                .is_some_and(|dataset| dataset.document.is_some())
        })
        .map(|row| row.declaration_id)
        .collect::<std::collections::BTreeSet<_>>();
    loop {
        let names = parsed
            .iter()
            .filter(|row| excluded.contains(&row.declaration_id))
            .filter_map(|row| Some(format!("{}.{}", packages.get(&row.parent_id?)?, row.name)))
            .collect::<std::collections::BTreeSet<_>>();
        let before = excluded.len();
        for row in &parsed {
            if row.value.dataset.as_ref().is_some_and(|dataset| {
                dataset
                    .provenance
                    .lineage
                    .iter()
                    .any(|lineage| names.contains(&lineage.path.join(".")))
            }) {
                excluded.insert(row.declaration_id);
            }
        }
        if excluded.len() == before {
            break;
        }
    }
    parsed
        .into_iter()
        .filter(|row| !excluded.contains(&row.declaration_id))
        .collect()
}
