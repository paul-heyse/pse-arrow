// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public package and standalone pure conformance preserve original scientific findings.
//! No linked solver, Python, canonical store or scientific capsule is required.
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "fixture construction and independent scientific assertions fail directly"
)]

#[path = "reference_support/context.rs"]
mod support;

use futures_util::{FutureExt, future::BoxFuture};
use pse_admission::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
use pse_diagnostics::{DiagnosticCode, DiagnosticRule};
use pse_model::{
    diagnostic::{BoundaryClass, Observation},
    generated::{
        enums::{
            ModelingConformanceKind as Kind, ModelingConformanceStatus as Status,
            ModelingValidityLayer, NativeBoundaryClass,
        },
        identities::DeclarationId,
    },
};
use pse_relations::{
    columnar::RelationRow,
    generated::runtime::{modeling_conformance, modeling_findings, modeling_fixture_status},
};
use pse_runtime::{
    CancelSource, ResourceBudget,
    math::settings::SolveSettings,
    workflow::{
        ConformanceControls, ModelingConformancePolicy, ModelingConformanceReport,
        ModelingFixtureSelection, ModelingPackage, PreparationSettings, PureConformanceControls,
        WorkflowError, conform_pure_request,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroUsize,
    panic::AssertUnwindSafe,
    path::Path,
};
use support::{NativeContext, Result};

const CLAIMS: &str = r#"package pure {
  fn square(x:Scalar)->Scalar=x*x;
  fn root(x:Scalar)->Scalar valid(x>=0)=x;
  test positive fixture { dof 0; route steady; procedure check; } {
    expect square(2)==4 tolerance 1e-12 relative 1e-8;
  }
  test negative fixture {
    dof 0; route steady; procedure check;
    failure trial_rejected validity(form) form(root) variable(x);
  } { expect root(-1)==1 tolerance 0; }
}"#;

const POSITIVE: &str = r#"package pure {
  fn square(x:Scalar)->Scalar=x*x;
  test positive fixture { dof 0; route steady; procedure check; } {
    expect square(2)==4 tolerance 1e-12;
  }
}"#;

struct Inputs {
    modeling: BTreeMap<String, Vec<u8>>,
    physical: BTreeMap<String, Vec<u8>>,
}

impl Inputs {
    fn new(source: &str) -> Result<Self> {
        fn files(root: &Path, at: &Path, out: &mut BTreeMap<String, Vec<u8>>) -> Result<()> {
            for entry in std::fs::read_dir(at)? {
                let path = entry?.path();
                if path.is_dir() {
                    files(root, &path, out)?;
                } else {
                    out.insert(
                        path.strip_prefix(root)?
                            .to_str()
                            .unwrap()
                            .replace('\\', "/"),
                        std::fs::read(path)?,
                    );
                }
            }
            Ok(())
        }
        let fixtures = support::root().join("tests/fixtures/packages");
        let physical_root = fixtures.join("physical-primitives");
        let mut physical = BTreeMap::new();
        files(&physical_root, &physical_root, &mut physical)?;
        let header: toml::Value = toml::from_str(std::str::from_utf8(&physical["package.toml"])?)?;
        let version: Vec<i64> = header["package"]["version"]
            .as_str()
            .unwrap()
            .split('.')
            .map(|part| part.parse().unwrap())
            .collect();
        assert_eq!(version.len(), 3);
        let mut manifest: toml::Value = toml::from_str(&std::fs::read_to_string(
            fixtures.join("minimal_explicit/package.toml"),
        )?)?;
        manifest["package"]["id_policy"] = "named".into();
        let requirement = toml::Table::from_iter([
            ("operator".into(), "exact".into()),
            ("major".into(), version[0].into()),
            ("minor".into(), version[1].into()),
            ("patch".into(), version[2].into()),
        ]);
        manifest["package"]["dependencies"] = toml::Value::Array(vec![
            toml::Table::from_iter([
                ("package_id".into(), header["package"]["package_id"].clone()),
                ("version_req".into(), requirement.into()),
            ])
            .into(),
        ]);
        Ok(Self {
            physical,
            modeling: BTreeMap::from([
                (
                    "package.toml".into(),
                    toml::to_string(&manifest)?.into_bytes(),
                ),
                ("models/pure.pse".into(), source.as_bytes().to_vec()),
            ]),
        })
    }

    fn package<'a>(&'a self, context: &'a NativeContext) -> BoxFuture<'a, Result<ModelingPackage>> {
        async move {
            let pool = context.shared.pool();
            let token = pse_columnar::CancellationToken::new();
            let validation = context.runtime.validation_context();
            let load = |documents: &BTreeMap<String, Vec<u8>>| -> Result<OwnedDocumentSet> {
                Ok(OwnedDocumentSet::try_from_bundles(
                    vec![load_package_documents_owned(
                        documents,
                        context.runtime.registry(),
                        Default::default(),
                        &pool,
                        &token,
                        &validation,
                    )?],
                    &pool,
                    &token,
                )?)
            };
            let physical_documents = load(&self.physical)?;
            let physical = Box::pin(
                context
                    .runtime
                    .physical_from_documents(&physical_documents, &token),
            )
            .await?;
            let documents = load(&self.modeling)?;
            Ok(Box::pin(context.runtime.modeling_from_documents(
                &documents,
                physical,
                &CancelSource::new(),
            ))
            .await?)
        }
        .boxed()
    }
}

fn preparation(scratch_bytes: usize) -> PreparationSettings {
    let mut preparation = PreparationSettings::default();
    preparation.compiler.evaluation.scratch_bytes = scratch_bytes;
    preparation
}

fn policy(preparation: &PreparationSettings) -> ModelingConformancePolicy {
    ModelingConformancePolicy {
        compiler: preparation.compiler,
        solver: Default::default(),
        numerical: Default::default(),
        limits: preparation.limits,
        derivatives: pse_backend_native::derivative_diagnostics::Policy {
            perturbation: 1e-6,
            relative_tolerance: 1e-4,
            maximum_cells: 100,
        },
        maximum_fixtures: 10,
        maximum_checks: 32,
        fixtures: ModelingFixtureSelection::Package,
        diagnostics: None,
    }
}

fn pure_budget(spill: &Path, threads: usize) -> ResourceBudget {
    let n = |value| NonZeroUsize::new(value).unwrap();
    ResourceBudget {
        memory_limit_bytes: n(1 << 30),
        spill_dir: spill.to_owned(),
        max_temp_dir_bytes: 1 << 28,
        top_consumers: n(16),
        threads: pse_engine::ThreadBudget {
            pool_threads: n(threads),
            target_partitions: n(threads),
        },
        execution: Default::default(),
        cache: pse_runtime::CacheBudget::disabled(1024),
        math: pse_runtime::math::MathPolicy {
            worker_bytes: 8 << 20,
            workspace_bytes: 8 << 20,
            foreign_bytes: 1 << 20,
            ..Default::default()
        },
    }
}

fn assert_claims(report: &ModelingConformanceReport) {
    assert!(
        report.complete && report.passed(),
        "{:?}; {:?}",
        report.checks,
        report.failures
    );
    assert_eq!(report.fixtures().len(), 2);
    assert_eq!(report.fixture_statuses.len(), 2);
    assert!(
        report
            .fixture_statuses
            .values()
            .all(|status| *status == Status::Passed)
    );
    assert!(
        report.results.is_empty()
            && report.initializations.is_empty()
            && report.trajectories.is_empty()
    );
    // The expected-rejection fixture also records an inapplicable numeric
    // expectation. Inspect the one actually evaluated scalar comparison.
    let expectations = report
        .checks
        .iter()
        .filter(|check| check.kind == Kind::Expectation && check.deviation.is_some())
        .collect::<Vec<_>>();
    let [expectation] = expectations.as_slice() else {
        panic!(
            "one independently evaluated numeric expectation: {:?}",
            report.checks
        );
    };
    assert_eq!(expectation.status, Status::Passed);
    assert_eq!(expectation.deviation, Some(0.));
    assert!(
        expectation
            .tolerance
            .is_some_and(|tolerance| tolerance > 0.)
    );
    assert_eq!(report.failures.len(), 1);
    let failure = &report.failures[0];
    assert_eq!(failure.rule, DiagnosticRule::MathValidity);
    assert_eq!(failure.class, BoundaryClass::TrialRejected);
    let validity = failure.validity.as_ref().unwrap();
    assert_eq!(validity.layer, ModelingValidityLayer::Form);
    assert_eq!(validity.variables, [0]);
    assert_eq!(validity.form, Some(validity.source));
    assert!(validity.members.is_empty() && validity.sets.is_empty());
    assert!(
        report
            .checks
            .iter()
            .any(|check| check.failure_ordinal == Some(0) && check.status == Status::Passed)
    );
}

fn assert_scratch_refusal(report: &ModelingConformanceReport) {
    assert!(!report.passed());
    assert_eq!(report.failures.len(), 1);
    let failure = &report.failures[0];
    assert_eq!(failure.code, DiagnosticCode::RuntimeResourceLimit);
    assert_eq!(failure.rule, DiagnosticRule::MathLimit);
    assert_eq!(failure.class, BoundaryClass::ResourceLimit);
    assert!(
        matches!(failure.observations.get("detail"), Some(Observation::Text(detail))
        if detail == "math limit exceeded: worker scratch bytes")
    );
}

#[tokio::test]
async fn standalone_pure_request_preserves_validity_failure_tables_and_partial_inventory()
-> Result<()> {
    // This caller constructs neither NativeContext nor SharedRuntime. Each pure
    // request owns its independent budget and finishes before returning the report.
    let inputs = Inputs::new(CLAIMS)?;
    let spill = tempfile::tempdir()?;
    for threads in [1, 2] {
        let report = Box::pin(conform_pure_request(
            vec![inputs.modeling.clone()],
            inputs.physical.clone(),
            pure_budget(spill.path(), threads),
            PureConformanceControls {
                maximum_checks: 32,
                ..Default::default()
            },
            preparation(1_000_000),
            &CancelSource::new(),
        ))
        .await?;
        assert_claims(&report);
        let expected_checks = report.checks.clone();
        let table = report.table()?;
        let inventory = report.fixture_statuses_table()?;
        let findings = report.findings_table()?;
        drop(report);
        assert_eq!(modeling_conformance::Row::rows(&table)?, expected_checks);
        let statuses = modeling_fixture_status::Row::rows(&inventory)?;
        assert_eq!(statuses.len(), 2);
        assert!(statuses.iter().all(|row| row.status == Status::Passed));
        let findings = modeling_findings::Row::rows(&findings)?;
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, DiagnosticRule::MathValidity);
        assert_eq!(findings[0].class, NativeBoundaryClass::TrialRejected);
        let lineage = findings[0].validity.as_ref().unwrap();
        assert_eq!(lineage.layer, ModelingValidityLayer::Form);
        assert_eq!(lineage.variables, [0]);
    }
    let limited = Box::pin(conform_pure_request(
        vec![inputs.modeling.clone()],
        inputs.physical.clone(),
        pure_budget(spill.path(), 2),
        PureConformanceControls {
            maximum_checks: 1,
            ..Default::default()
        },
        preparation(1_000_000),
        &CancelSource::new(),
    ))
    .await?;
    assert!(!limited.complete && !limited.passed());
    assert_eq!(limited.checks.len(), 1);
    assert_eq!(limited.fixtures().len(), 2);
    assert_eq!(
        limited
            .fixture_statuses
            .values()
            .filter(|status| **status == Status::Inconclusive)
            .count(),
        1
    );
    assert_eq!(
        limited
            .fixture_statuses
            .values()
            .filter(|status| **status == Status::Unattempted)
            .count(),
        1
    );

    // An invalid selection must fail before a fixture reaches the deliberately
    // insufficient evaluator allowance; it cannot be converted into a check failure.
    let unknown = DeclarationId::from_bytes([7; 16]);
    let error = Box::pin(conform_pure_request(
        vec![inputs.modeling.clone()],
        inputs.physical.clone(),
        pure_budget(spill.path(), 2),
        PureConformanceControls {
            maximum_checks: 32,
            fixtures: Some(vec![unknown]),
            ..Default::default()
        },
        preparation(1),
        &CancelSource::new(),
    ))
    .await
    .unwrap_err();
    assert!(matches!(&error, WorkflowError::Input(_)), "{error}");
    assert!(error.to_string().contains(&unknown.to_string()), "{error}");
    Ok(())
}

#[tokio::test]
async fn public_package_conformance_and_pure_request_honor_preparation_and_selection() -> Result<()>
{
    let context = NativeContext::new()?;
    let body = AssertUnwindSafe(async {
        let inputs = Inputs::new(CLAIMS)?;
        let package = inputs.package(&context).await?;
        let services = context.runtime.execution_services();
        let preparation = preparation(1_000_000);
        let report =
            Box::pin(package.conform(&services, policy(&preparation), &CancelSource::new()))
                .await?;
        assert_claims(&report);
        let declarations = package.declarations(services.preparation()).await?;
        let positive = declarations
            .iter()
            .find(|row| row.name == "positive")
            .unwrap()
            .declaration_id;
        let form = declarations
            .iter()
            .find(|row| row.name == "root")
            .unwrap()
            .declaration_id;
        assert_eq!(
            report.failures[0].validity.as_ref().unwrap().form,
            Some(form.as_id())
        );
        let retained = report.table()?;
        let expected = report.checks.clone();
        drop(report);

        let selected = BTreeSet::from([positive]);
        let mut selected_policy = policy(&preparation);
        selected_policy.fixtures = ModelingFixtureSelection::Selected(selected.clone());
        let report =
            Box::pin(package.conform(&services, selected_policy, &CancelSource::new())).await?;
        assert!(report.complete && report.passed());
        assert_eq!(report.fixtures(), selected);
        assert!(report.failures.is_empty());
        let coverage = report
            .checks
            .iter()
            .filter(|check| check.kind == Kind::Coverage)
            .collect::<Vec<_>>();
        assert_eq!(coverage.len(), 1);
        assert_eq!(coverage[0].status, Status::NotApplicable);
        drop(report);

        let unknown = DeclarationId::from_bytes([7; 16]);
        let mut invalid = policy(&preparation);
        invalid.compiler.evaluation.scratch_bytes = 1;
        invalid.fixtures = ModelingFixtureSelection::Selected(BTreeSet::from([unknown]));
        let error = Box::pin(package.conform(&services, invalid, &CancelSource::new()))
            .await
            .unwrap_err();
        assert!(matches!(&error, WorkflowError::Input(_)), "{error}");
        assert!(error.to_string().contains(&unknown.to_string()), "{error}");
        drop(package);

        let inputs = Inputs::new(POSITIVE)?;
        let positive = inputs.package(&context).await?;
        let controls = ConformanceControls {
            maximum_checks: 32,
            ..Default::default()
        };
        let successful = Box::pin(positive.conform_request(
            &services,
            &preparation,
            SolveSettings::default(),
            controls.clone(),
            &CancelSource::new(),
        ))
        .await?;
        assert!(successful.complete && successful.passed());
        let mut insufficient = preparation.clone();
        insufficient.compiler.evaluation.scratch_bytes = 1;
        let refused = Box::pin(positive.conform_request(
            &services,
            &insufficient,
            SolveSettings::default(),
            controls,
            &CancelSource::new(),
        ))
        .await?;
        assert_scratch_refusal(&refused);
        let spill = tempfile::tempdir()?;
        for (preparation, scratch_refusal) in [(preparation, false), (insufficient, true)] {
            let report = Box::pin(conform_pure_request(
                vec![inputs.modeling.clone()],
                inputs.physical.clone(),
                pure_budget(spill.path(), 2),
                PureConformanceControls {
                    maximum_checks: 32,
                    ..Default::default()
                },
                preparation,
                &CancelSource::new(),
            ))
            .await?;
            if scratch_refusal {
                assert_scratch_refusal(&report);
            } else {
                assert!(report.complete && report.passed());
            }
        }
        drop(positive);
        context.close().await?;
        assert_eq!(modeling_conformance::Row::rows(&retained)?, expected);
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    })
    .catch_unwind()
    .await;
    let cleanup = context.close().await;
    match body {
        Ok(result) => {
            result?;
            cleanup
        }
        Err(panic) => {
            let _ = cleanup;
            std::panic::resume_unwind(panic)
        }
    }
}
