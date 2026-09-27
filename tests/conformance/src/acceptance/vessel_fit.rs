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
