// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Seed data owns the vessel equations, observations and original physical checks.
use super::fixtures::*;
use pse_backend_native::solve::Termination;
use pse_runtime::{CancelSource, workflow::RunReport};

#[tokio::test]
async fn authored_vessel_fitting_preserves_shared_parameters_checks_and_identifiability() {
    let owner = WorkflowRuntime::new().unwrap();
    let package = seed_package(&owner).await;
    for kind in ["steady", "transient", "mixed", "unidentifiable"] {
        let (id, profile) = heat_fit(&package, kind).await;
        let result = package
            .prepare_fit(id, profile, compiler(), seed_limits(), &CancelSource::new())
            .await
            .unwrap()
            .start()
            .unwrap()
            .wait()
            .await
            .unwrap();
        let RunReport::Fit(report) = result.report().unwrap() else {
            panic!("missing fit")
        };
        assert!(
            matches!(
                report.solve.as_ref().unwrap().termination.category,
                Termination::Success | Termination::Acceptable
            ),
            "{kind}: {report:?}"
        );
        assert!(
            report.quality.as_ref().unwrap().feasible(),
            "{kind}: {report:?}"
        );
        near(report.candidate.as_ref().unwrap()[0], 10., 2e-3);
        assert!(report.objective.unwrap() < 1e-6, "{kind}: {report:?}");
        assert_eq!(
            report.rank,
            Some(1),
            "zero inlet flow leaves inlet enthalpy unidentifiable: {report:?}"
        );
        assert_eq!(
            report.candidate.as_ref().unwrap().len(),
            if kind == "unidentifiable" { 2 } else { 1 }
        );
        for sensitivity in report.responses.as_ref().unwrap().col(0).iter() {
            near(*sensitivity, 1., 2e-3);
        }
        assert!(
            report.checks_complete && report.checks.iter().all(|c| c.satisfied),
            "{kind}: {report:?}"
        );
        assert!(!report.checks.is_empty());
        assert_eq!(report.estimate_qualified(), kind != "unidentifiable");
        if report.estimate_qualified() {
            use pse_relations::{
                columnar::RelationRow, generated::runtime::fitted_parameter_cells,
            };
            let exported =
                fitted_parameter_cells::Row::rows(&result.export_fit_parameters().unwrap())
                    .unwrap();
            assert_eq!(exported.len(), 1);
            let cell = &exported[0];
            assert_eq!(cell.run_id, result.run_id);
            assert_eq!(cell.fit_id, id);
            near(cell.value, 10., 2e-3);
            if kind == "steady" {
                // Publication is an explicit new admission with its fit/run/source receipt;
                // the fitted bank never mutates the measurements or the solved revision.
                let bank = format!(
                    "package fitted_bank {{identifier scheme run;identifier scheme fit;identifier scheme revision;entity kind receipt provenance {{attribute run_id:Id<run>;attribute fit_id:Id<fit>;attribute source_revision:Id<revision>;attribute fit_source:Id<revision>;}} enum role {{fitted facets(requires_fit)}} entity receipt qualified {{run_id=Id<run>(\"{}\"),fit_id=Id<fit>(\"{}\"),source_revision=Id<revision>(\"{}\"),fit_source=Id<revision>(\"{}\")}} entity kind heat_parameter {{attribute value:EnergyTransferRate;}} entity heat_parameter estimate provenance(qualified,role.fitted,lineage(fit qualified)) {{value={}{{W}}}} }}",
                    cell.run_id, cell.fit_id, cell.source_revision, cell.fit_source, cell.value
                );
                let declarations = |text: &str| {
                    pse_authoring::language::parse(
                        text,
                        pse_ids::SemanticId::NIL,
                        pse_authoring::language::IdentityPolicy::Named,
                        Default::default(),
                    )
                    .unwrap()
                };
                let physical = physical(&owner).await;
                let admitted = runtime(&owner)
                    .modeling_package(declarations(&bank), physical.clone())
                    .await
                    .unwrap();
                let knowledge = admitted
                    .knowledge(None, 128, 1 << 20, &owner.cancel)
                    .await
                    .unwrap();
                let cells = pse_relations::generated::runtime::modeling_knowledge::Row::rows(
                    knowledge.table(),
                )
                .unwrap();
                let fitted = cells.iter().find(|row| row.slot == "value").unwrap();
                assert_eq!(
                    fitted.value.last().unwrap().quantity_type_id,
                    Some(cell.quantity_type_id)
                );
                assert_eq!(fitted.value.last().unwrap().magnitude, Some(cell.value));
                assert_eq!(fitted.lineage.len(), 1);
                assert_eq!(
                    fitted.lineage[0].kind,
                    pse_model::generated::enums::ModelingLineageKind::Fit
                );
                let refused = bank.replace(",lineage(fit qualified)", "");
                assert!(
                    runtime(&owner)
                        .modeling_package(declarations(&refused), physical)
                        .await
                        .is_err()
                );
            }
        } else {
            assert!(result.export_fit_parameters().is_err());
        }
        assert!(
            result.usable(),
            "a checked prediction remains usable when a parameter is unidentifiable"
        );
        assert!(
            result
                .table("authored.modeling_declarations")
                .unwrap()
                .batch()
                .num_rows()
                > 0
        );
        assert_eq!(
            result
                .table("authored.fit_cases")
                .unwrap()
                .batch()
                .num_rows(),
            4
        );
        assert!(result.table("authored.computation_models").is_err());
    }
}

/// ADR-0118 item 8: zero inlet flow leaves the inlet enthalpy unidentifiable, so the fit's
/// covariance is withheld as rank deficient, and the response directions report the
/// identifiable subspace: the heat alone.
#[tokio::test]
async fn unidentifiable_fit_withholds_covariance() {
    use pse_relations::{
        columnar::RelationRow,
        generated::{
            enums::{CovarianceApproximation, DerivedQuantity, WithheldReason},
            runtime::{local_validity, response_directions},
        },
    };
    let owner = WorkflowRuntime::new().unwrap();
    let package = seed_package(&owner).await;
    let (id, profile) = heat_fit(&package, "unidentifiable").await;
    let result = package
        .prepare_fit(id, profile, compiler(), seed_limits(), &CancelSource::new())
        .await
        .unwrap()
        .start()
        .unwrap()
        .wait()
        .await
        .unwrap();
    let RunReport::Fit(report) = result.report().unwrap() else {
        panic!("missing fit")
    };
    let covariance = report.covariance.as_ref().unwrap();
    // A quasi-Newton fit's covariance is Gauss–Newton.
    assert_eq!(
        covariance.approximation,
        CovarianceApproximation::GaussNewton
    );
    assert!(
        matches!(
            covariance.values,
            Err(pse_runtime::workflow::FitWithheld::RankDeficient {
                rank: 1,
                parameters: 2
            })
        ),
        "{covariance:?}"
    );
    // The identifiable direction is the heat, in declared-scale coordinates.
    let directions = report.directions.as_ref().unwrap();
    near(directions[(0, 0)].abs(), 1., 1e-6);
    near(directions[(1, 0)], 0., 1e-6);
    let validity =
        local_validity::Row::rows(&result.table("runtime.local_validity").unwrap()).unwrap();
    assert_eq!(validity.len(), 1);
    assert_eq!(validity[0].quantity, DerivedQuantity::ParameterCovariance);
    assert!(!validity[0].validity.certified);
    assert_eq!(
        validity[0].validity.reason,
        Some(WithheldReason::RankDeficient)
    );
    assert_eq!(
        result
            .table("runtime.parameter_covariances")
            .unwrap()
            .batch()
            .num_rows(),
        0
    );
    let published =
        response_directions::Row::rows(&result.table("runtime.response_directions").unwrap())
            .unwrap();
    assert_eq!(published.len(), 4);
    assert_eq!(published.iter().filter(|r| r.identifiable).count(), 2);
    assert!(
        published
            .iter()
            .filter(|r| !r.identifiable)
            .all(|r| r.direction == 1)
    );
}
