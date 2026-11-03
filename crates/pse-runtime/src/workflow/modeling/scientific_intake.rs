// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded interpretation of selected published scientific inputs into ordinary source deltas.
//!
//! This adapter changes a data document, not scientific source-role eligibility or publication
//! authority. The receiver must still admit the delta and publish against its original parent.
use crate::workflow::{SourceDelta, WorkflowError, contract};
use sha2::{Digest, Sha256};

#[derive(serde::Deserialize)]
struct FeosIdentifier {
    cas: String,
}
#[derive(serde::Deserialize)]
struct FeosCaloric {
    identifier: FeosIdentifier,
    #[serde(default, rename = "DIPPR100")]
    coefficients: Vec<f64>,
}

/// Reconcile benzene/toluene ideal-gas Cp from the selected FeOS Poling JSON artifact.
///
/// Original coefficients retain J/(kmol K^(n+1)); ordinary physical admission converts
/// them to the authored mol coordinates without a second numerical unit interpreter.
/// The existing dataset declaration retains its source, parameterization and consumed
/// envelope. The caller must retain the exact artifact/revision in its source records;
/// source acquisition and empirical qualification are not inferred from JSON decoding.
///
/// # Errors
/// Invalid extent, missing/duplicate/nonfinite selected records, or a different target
/// dataset refuses. Contextual delta admission separately checks the exact parent.
pub fn feos_poling2000_bt_caloric_delta(
    parent: &str,
    target: &pse_authoring::language::Declaration,
    artifact: &[u8],
) -> Result<SourceDelta, WorkflowError> {
    if parent.is_empty() || artifact.len() > 256 << 10 || target.name != "bt_vapor_caloric" {
        return Err(contract(
            "FeOS BT caloric intake requires the selected dataset and exact parent",
        ));
    }
    let original = target
        .value
        .dataset
        .as_ref()
        .filter(|dataset| {
            matches!(
                dataset.target.as_str(),
                "correlations.dippr100" | "feos_poling_cp"
            ) && dataset.document.is_none()
        })
        .ok_or_else(|| {
            contract("FeOS BT caloric target requires the existing inline DIPPR100 dataset")
        })?;
    let records: Vec<FeosCaloric> = serde_json::from_slice(artifact)
        .map_err(|error| contract(format!("FeOS Poling caloric artifact: {error}")))?;
    let mut rows = String::new();
    for (cas, subject) in [("71-43-2", "chem.benzene"), ("108-88-3", "chem.toluene")] {
        let mut selected = records.iter().filter(|record| record.identifier.cas == cas);
        let record = selected
            .next()
            .ok_or_else(|| contract(format!("FeOS Poling caloric record absent: {cas}")))?;
        if selected.next().is_some()
            || record.coefficients.len() != 5
            || record.coefficients.iter().any(|value| !value.is_finite())
        {
            return Err(contract(format!(
                "FeOS Poling caloric record ambiguous/nonfinite: {cas}"
            )));
        }
        let values = record
            .coefficients
            .iter()
            .enumerate()
            .map(|(index, coefficient)| {
                let power = index + 1;
                let unit = if power == 1 {
                    "J/(kmol*K)".to_owned()
                } else {
                    format!("J/(kmol*K^{power})")
                };
                format!("{coefficient}{{{unit}}}")
            })
            .collect::<Vec<_>>()
            .join(",");
        rows.push_str(&format!("[{subject}]=[{values}];"));
    }
    const ARTIFACT_SHA256: [u8; 32] = [
        0x8c, 0xd2, 0x76, 0x12, 0x01, 0xf0, 0xc1, 0x94, 0x26, 0x79, 0xa5, 0x8d, 0x92, 0x66, 0xdf,
        0x39, 0x82, 0x9e, 0x84, 0xe7, 0x97, 0x44, 0xe0, 0x8b, 0xd4, 0x67, 0x00, 0xcc, 0x67, 0xed,
        0x08, 0xf8,
    ];
    if Sha256::digest(artifact)[..] != ARTIFACT_SHA256 {
        return Err(contract(
            "FeOS Poling bytes differ from the pinned source artifact; select a new source revision before changing scientific values",
        ));
    }
    // Parse the source-specific typed cells through the ordinary authoring vocabulary;
    // there is no parallel expression or quantity interpreter in the adapter.
    let source = format!(
        "package intake {{dataset bank:correlations.dippr100 provenance(source,Role.published) {{{rows}}}}}"
    );
    let parsed = pse_authoring::language::parse(
        &source,
        pse_ids::SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .map_err(|error| contract(format!("FeOS Poling typed cells: {error}")))?;
    let rows = parsed
        .iter()
        .find_map(|declaration| declaration.value.dataset.as_ref())
        .ok_or_else(|| contract("FeOS Poling typed dataset absent"))?
        .rows
        .clone();
    let mut replacement = target.clone();
    let mut dataset = original.clone();
    dataset.rows = rows;
    replacement.value.dataset = Some(dataset);
    Ok(SourceDelta {
        parent: parent.into(),
        replacements: vec![replacement],
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn feos_bt_intake_refuses_changed_publication_and_missing_or_duplicate_records() {
        let declarations = pse_authoring::language::parse(
            "package p {dataset bt_vapor_caloric:correlations.dippr100 provenance(source,Role.published) { [chem.benzene]=[1]; }}",
            pse_ids::SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default()).unwrap();
        let target = declarations
            .iter()
            .find(|row| row.name == "bt_vapor_caloric")
            .unwrap();
        let input = br#"[{"identifier":{"cas":"71-43-2"},"DIPPR100":[1000,2000,3000,4000,5000]},{"identifier":{"cas":"108-88-3"},"DIPPR100":[6000,7000,8000,9000,10000]}]"#;
        let error = feos_poling2000_bt_caloric_delta("original-parent", target, input).unwrap_err();
        assert!(error.to_string().contains("pinned source artifact"));
        assert_eq!(
            target.value.dataset.as_ref().unwrap().rows.len(),
            1,
            "parent remains unchanged"
        );
        assert!(feos_poling2000_bt_caloric_delta("parent", target, b"[]").is_err());
        let duplicate = br#"[{"identifier":{"cas":"71-43-2"},"DIPPR100":[1,2,3,4,5]},{"identifier":{"cas":"71-43-2"},"DIPPR100":[1,2,3,4,5]}]"#;
        assert!(feos_poling2000_bt_caloric_delta("parent", target, duplicate).is_err());
    }

    #[test]
    fn feos_bt_intake_reimports_frozen_artifact_through_authored_cells() {
        let source = include_str!(
            "../../../../../packages/reference/data/ordinary-bt/models/parameters.pse"
        );
        let rows = pse_authoring::language::parse(
            source,
            pse_ids::SemanticId::parse_hex("c90a89ffdd9e409a9a0b9420405598a5").unwrap(),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let target = rows
            .iter()
            .find(|row| row.name == "bt_vapor_caloric")
            .unwrap();
        let artifact = include_bytes!(
            "../../../../../packages/reference/data/ordinary-bt/source-artifacts/feos-poling2000.json"
        );
        let delta = feos_poling2000_bt_caloric_delta("selected-parent", target, artifact).unwrap();
        let imported = delta.replacements[0].value.dataset.as_ref().unwrap();
        let original = target.value.dataset.as_ref().unwrap();
        assert_eq!(imported.bindings, original.bindings);
        assert_eq!(imported.provenance, original.provenance);
        assert_eq!(imported.rows.len(), 2);
        assert_eq!(
            imported.rows, original.rows,
            "unchanged source reimport preserves original scientific cells including units"
        );
        assert_eq!(
            imported.rows[0].values[0]
                .value
                .quantity
                .as_ref()
                .unwrap()
                .magnitude,
            29524.656757062152
        );
        assert_eq!(delta.replacements[0].declaration_id, target.declaration_id);
    }

    #[cfg(all(feature = "solver-kinsol", feature = "solver-root-isolation"))]
    #[tokio::test]
    async fn ordinary_bt_sources_admit_and_reimport_through_exact_parent_delta() {
        let (runtime, package, _spill, _) =
            super::super::pr_jacobian_tests::ordinary_source_package()
                .await
                .unwrap();
        let declarations = package
            .declaration_path(
                &runtime.preparation_services(),
                &["ordinary_bt", "bt_vapor_caloric"],
                &crate::CancelSource::new(),
            )
            .await
            .unwrap()
            .unwrap();
        let target = &declarations[0];
        let artifact = include_bytes!(
            "../../../../../packages/reference/data/ordinary-bt/source-artifacts/feos-poling2000.json"
        );
        let delta =
            feos_poling2000_bt_caloric_delta(package.revision_id(), target, artifact).unwrap();
        let changed = package
            .with_delta(
                &runtime.source_services(),
                delta.clone(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(
            changed.revision_id(),
            package.revision_id(),
            "unchanged ordinary import is a true source no-op"
        );
        let actual = changed
            .declaration_path(
                &runtime.preparation_services(),
                &["ordinary_bt", "bt_vapor_caloric"],
                &crate::CancelSource::new(),
            )
            .await
            .unwrap()
            .unwrap();
        let bank = &actual[0];
        assert_eq!(bank.declaration_id, target.declaration_id);
        assert_eq!(
            bank.value.dataset.as_ref().unwrap().provenance,
            target.value.dataset.as_ref().unwrap().provenance
        );
        let mut wrong = delta;
        wrong.parent = "another-parent".into();
        assert!(
            package
                .with_delta(
                    &runtime.source_services(),
                    wrong,
                    &crate::CancelSource::new()
                )
                .await
                .is_err()
        );
        assert!(
            changed
                .declaration_path(
                    &runtime.preparation_services(),
                    &["ordinary_bt", "feos_unknown"],
                    &crate::CancelSource::new(),
                )
                .await
                .unwrap()
                .is_some(),
            "unknown empirical coverage remains a typed scientific claim"
        );
    }
}
