// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public native adoption of fit covariance, physical custody and downstream propagation.
//! The scientific expectations are independent weighted normal equations and J=(1,-1).
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr,
    reason = "public scientific fixtures and independent analytic assertions fail directly"
)]

#[path = "reference_support/mod.rs"]
mod reference_support;

use futures_util::{FutureExt, future::BoxFuture};
use pse_backend_native::{
    presolve::PolicyKind,
    solve::{Backend, SolveIntent},
};
use pse_ids::SemanticId;
use pse_model::{
    generated::{
        enums::{CovarianceInputOrigin, DerivedQuantity, WithheldReason},
        identities::DeclarationId,
    },
    scalars::FiniteBound,
};
use pse_relations::{
    columnar::RelationRow,
    generated::runtime::{local_validity, parameter_covariances, propagated_covariances},
};
use pse_runtime::{
    CancelSource,
    math::settings::{
        CovarianceOrigin, ParameterCovariance, Propagation, SensitivityRequest, SolveSettings,
    },
    workflow::{
        FitDeclarations, FitPreparationDocument, ModelingInspectionRequest, ModelingPackage,
        OperationSource, PreparationSettings, PreparedOperation, RunResult,
        RuntimeOperationRequest,
    },
};
use reference_support::{NativeContext, Result};
use std::{collections::BTreeMap, panic::AssertUnwindSafe};

fn id(byte: u8) -> SemanticId {
    SemanticId::from_bytes([byte; 16])
}

fn source(objective: bool) -> String {
    let objective = if objective {
        "let cost:Scalar=0.5*z*z; annotation objective cost(minimize);"
    } else {
        ""
    };
    let mut text = format!(
        "package regression {{ def Root {{
        param a:Scalar=1; param b:Scalar=1; var z:Scalar;
        eq contrast:z==a-b; annotation start z(0); {objective}
        let y1:Scalar=a+b; let y2:Scalar=a+2*b;
        let y3:Scalar=a+3*b; let y4:Scalar=a+4*b; }}
        entity kind origin provenance {{attribute title:Text;}}
        entity origin experiment {{title=\"weighted regression\"}}
        enum role {{measured facets(measured)}}
        entity kind reading {{attribute value:Scalar?;attribute sigma:Scalar?;}} "
    );
    for (index, (value, sigma)) in [(3.1, 0.5), (4.9, 1.), (7.2, 0.5), (8.8, 2.)]
        .into_iter()
        .enumerate()
    {
        text.push_str(&format!("@id(\"{}\") entity reading point{index} provenance(experiment,role.measured) {{value={value},sigma={sigma}}}", id(170 + u8::try_from(index).unwrap()).to_hex()));
    }
    text.push('}');
    text
}

fn documents(objective: bool) -> Result<BTreeMap<String, Vec<u8>>> {
    // Match reference_support::NativeContext::package's complete original inventory.
    let manifest = std::fs::read_to_string(reference_support::root().join("tests/fixtures/packages/minimal_explicit/package.toml"))?
        .replace("id_policy = \"explicit\"", "id_policy = \"named\"")
        .replace("dependencies = []", "dependencies = [{ package_id = \"5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a\", version_req = { operator = \"exact\", major = 1, minor = 0, patch = 0 } }]");
    Ok(BTreeMap::from([
        ("package.toml".into(), manifest.into_bytes()),
        ("models/model.pse".into(), source(objective).into_bytes()),
    ]))
}

async fn members(
    context: &NativeContext,
    package: &ModelingPackage,
    case: DeclarationId,
) -> Result<[SemanticId; 3]> {
    let inspection = Box::pin(package.inspect_case_request(
        &context.runtime.preparation_services(),
        &ModelingInspectionRequest {
            case,
            preparation: Default::default(),
            settings: Default::default(),
            offset: 0,
            limit: 128,
        },
        &CancelSource::new(),
    ))
    .await?;
    Ok(["a", "b", "z"].map(|name| {
        let found: Vec<_> = inspection
            .members
            .iter()
            .filter(|member| {
                member
                    .lineage
                    .path
                    .rsplit('.')
                    .next()
                    .unwrap()
                    .split('[')
                    .next()
                    == Some(name)
            })
            .collect();
        assert_eq!(found.len(), 1, "one explicitly named {name}");
        found[0].id
    }))
}

fn regression<'a>(
    context: &'a NativeContext,
    importance: f64,
) -> BoxFuture<'a, Result<(ModelingPackage, DeclarationId, [SemanticId; 3])>> {
    Box::pin(async move {
        let package = context.package(&source(false)).await?;
        let case = package
            .declarations(&context.runtime.preparation_services())
            .await?
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let ids = members(context, &package, case).await?;
        let parameters: Vec<_> = ids[..2].iter().zip([1., 2.]).map(|(id, scale)| serde_json::json!({"symbol_id": id, "fixed": false, "value": 1., "lower": -100., "upper": 100., "scale": scale})).collect();
        let bindings: Vec<_> = ids[..2]
            .iter()
            .zip(["a", "b"])
            .map(|(id, path)| serde_json::json!({"parameter_id": id, "path": path}))
            .collect();
        let observations: Vec<_> = (0..4_u8).map(|index| serde_json::json!({
            "observation_id": id(170 + index), "value_attribute": "value", "standard_deviation_attribute": "sigma",
            "experiment_id": id(181), "output_path": format!("y{}", index + 1),
            "time": null, "time_basis": null, "time_unit_id": null, "included": true,
            "importance": if index == 2 { importance } else { 1. },
        })).collect();
        let mut fits = FitDeclarations::default();
        fits.fits.push(serde_json::from_value(serde_json::json!({"fit_id": id(180), "parameters": parameters,
            "experiments": [{"experiment_id": id(181), "case_id": case, "route": "steady", "bindings": bindings}],
            "observations": observations}))?);
        let package =
            Box::pin(package.with_fit_declarations(&context.runtime.source_services(), fits))
                .await?;
        Ok((package, case, ids))
    })
}

fn fit<'a>(
    context: &'a NativeContext,
    package: &'a ModelingPackage,
) -> BoxFuture<'a, Result<std::sync::Arc<RunResult>>> {
    Box::pin(async move {
        let document: FitPreparationDocument = serde_json::from_value(
            serde_json::json!({"optimizer": "native_nlp", "solver": SolveSettings {
                intent: SolveIntent::Optimize, backend: Some(Backend::Ipopt), ..Default::default()
            }}),
        )?;
        let services = context.runtime.execution_services();
        let prepared = Box::pin(package.prepare_fit(
            services.preparation(),
            id(180).into(),
            document.profile()?,
            Default::default(),
            Default::default(),
            &CancelSource::new(),
        ))
        .await?;
        let handle = prepared.start(&services)?;
        Ok(Box::pin(handle.wait()).await?)
    })
}

fn revision<'a>(
    context: &'a NativeContext,
    package: &'a ModelingPackage,
    case: DeclarationId,
    ids: [SemanticId; 3],
) -> BoxFuture<'a, Result<ModelingPackage>> {
    Box::pin(async move {
        let target = Box::pin(package.reload_from_parent(
            &context.runtime.source_services(),
            &[documents(true)?],
            &CancelSource::new(),
        ))
        .await?;
        assert_ne!(target.revision_id(), package.revision_id());
        assert_eq!(members(context, &target, case).await?, ids);
        Ok(target)
    })
}

fn downstream<'a>(
    context: &'a NativeContext,
    target: &'a ModelingPackage,
    case: DeclarationId,
    ids: [SemanticId; 3],
    covariance: ParameterCovariance,
) -> BoxFuture<'a, Result<PreparedOperation>> {
    Box::pin(async move {
        Ok(Box::pin(target.prepare_operation(
            &context.runtime.preparation_services(),
            &OperationSource::of(target),
            &PreparationSettings::default(),
            RuntimeOperationRequest::DeclaredCase {
                case,
                settings: Box::new(SolveSettings {
                    intent: SolveIntent::Optimize,
                    backend: Some(Backend::Ipopt),
                    presolve: PolicyKind::Off,
                    sensitivity: Some(SensitivityRequest {
                        parameters: ids[..2].to_vec(),
                        reduced_hessian: false,
                        propagation: Some(Propagation {
                            covariance,
                            outputs: vec![ids[2]],
                        }),
                    }),
                    ..Default::default()
                }),
            },
            None,
            &CancelSource::new(),
        ))
        .await?)
    })
}

fn decoded(covariance: &ParameterCovariance) -> Result<ParameterCovariance> {
    Ok(serde_json::from_slice(&serde_json::to_vec(covariance)?)?)
}

fn analytic_covariance() -> [f64; 4] {
    // Independent inverse of X'WX for x=(1,2,3,4), sigma=(.5,1,.5,2).
    let (mut s00, mut s01, mut s11) = (0., 0., 0.);
    for (x, weight) in [(1., 4.), (2., 1.), (3., 4.), (4., 0.25)] {
        s00 += weight;
        s01 += weight * x;
        s11 += weight * x * x;
    }
    let determinant = s00 * s11 - s01 * s01;
    [
        s11 / determinant,
        -s01 / determinant,
        -s01 / determinant,
        s00 / determinant,
    ]
}

fn close_to(actual: f64, expected: f64) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= 1e-7,
        "actual={actual}, analytic={expected}"
    );
}

fn assert_covariance(covariance: &ParameterCovariance, result: &RunResult, ids: [SemanticId; 3]) {
    assert_eq!(covariance.parameters, ids[..2]);
    assert_eq!(
        covariance
            .scales
            .iter()
            .map(|value| value.into_inner())
            .collect::<Vec<_>>(),
        [1., 2.]
    );
    for (actual, expected) in covariance.values.iter().zip(analytic_covariance()) {
        close_to(actual.into_inner(), expected);
    }
    assert_eq!(covariance.values.len(), 4);
    assert!(
        matches!(covariance.origin, CovarianceOrigin::QualifiedFit { run_id, .. } if run_id == result.run_id)
    );
}

fn assert_contrast(
    result: &RunResult,
    covariance: [f64; 4],
) -> Result<propagated_covariances::Row> {
    assert!(result.usable(), "{:?}", result.report());
    let mut rows =
        propagated_covariances::Row::rows(&result.table("runtime.propagated_covariances")?)?;
    assert_eq!(rows.len(), 1);
    let row = rows.remove(0);
    assert_eq!(row.values.len(), 1);
    close_to(
        row.values[0],
        covariance[0] - covariance[1] - covariance[2] + covariance[3],
    );
    assert!(
        local_validity::Row::rows(&result.table("runtime.local_validity")?)?
            .iter()
            .any(|row| row.quantity == DerivedQuantity::PropagatedCovariance
                && row.validity.certified)
    );
    assert!(result.parameter_covariance().is_none());
    Ok(row)
}

fn mutations(actual: &ParameterCovariance) -> Vec<ParameterCovariance> {
    let mut changes = vec![actual.clone(); 5];
    changes[0].values[0] = FiniteBound::try_new(actual.values[0].into_inner() * 2.).unwrap();
    changes[1].parameters.reverse();
    changes[2].parameter_quantities[0] = id(250);
    changes[3].parameter_units[0] = id(250);
    changes[4].scales[0] = FiniteBound::try_new(2.).unwrap();
    changes
}

fn finish<T>(outcome: std::thread::Result<Result<T>>, closed: Result<()>) -> Result<T> {
    match (outcome, closed) {
        (Err(panic), closed) => {
            if let Err(error) = closed {
                eprintln!(
                    "native adoption cleanup failed while preserving original panic: {error}"
                );
            }
            std::panic::resume_unwind(panic)
        }
        (Ok(Ok(value)), Ok(())) => Ok(value),
        (Ok(Err(error)), Ok(())) | (Ok(Ok(_)), Err(error)) => Err(error),
        (Ok(Err(error)), Err(cleanup)) => Err(std::io::Error::other(format!(
            "native adoption failed: {error}; independent cleanup failed: {cleanup}"
        ))
        .into()),
    }
}

#[tokio::test]
async fn native_fit_covariance_retains_physical_custody_and_refuses_payload_tampering() -> Result<()>
{
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let (package, case, ids) = regression(&context, 1.).await?;
        let fitted = fit(&context, &package).await?;
        assert!(fitted.usable());
        let actual = fitted
            .parameter_covariance()
            .unwrap()
            .expect("qualified fit covariance");
        assert_covariance(&actual, &fitted, ids);
        assert!(matches!(
            actual.origin,
            CovarianceOrigin::QualifiedFit { record: None, .. }
        ));
        let mut input = decoded(&actual)?;
        assert_eq!(input, actual);
        let target = revision(&context, &package, case, ids).await?;
        assert!(
            downstream(&context, &target, case, ids, input.clone())
                .await
                .is_err(),
            "serialized run identity cannot issue fit authority"
        );
        let error = context
            .runtime
            .reconcile_covariance(&mut input, &CancelSource::new().token())
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains("decoded in-process covariance"),
            "{error}"
        );
        for mut changed in mutations(&input) {
            let error = fitted
                .reconcile_parameter_covariance(&mut changed)
                .unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("differs from the actual retained qualified fit"),
                "{error}"
            );
        }
        fitted.reconcile_parameter_covariance(&mut input)?;
        let prepared = downstream(&context, &target, case, ids, input).await?;
        let run_id = fitted.run_id;
        drop(actual);
        drop(fitted);
        drop(package);
        drop(target);
        let result =
            Box::pin(prepared.start(&context.runtime.execution_services(), &CancelSource::new()))
                .await?
                .wait()
                .await?;
        let row = assert_contrast(&result, analytic_covariance())?;
        assert_eq!(row.covariance_run_id, Some(run_id));
        assert_eq!(row.parameters, ids[..2]);
        Ok(())
    })
    .catch_unwind()
    .await;
    finish(outcome, context.close().await)
}

#[tokio::test]
async fn native_external_covariance_accepts_singular_psd_and_refuses_indefinite_product()
-> Result<()> {
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let (package, case, ids) = regression(&context, 1.).await?;
        let fitted = fit(&context, &package).await?;
        let actual = fitted
            .parameter_covariance()
            .unwrap()
            .expect("qualified fit covariance");
        let external = ParameterCovariance::external(
            actual.parameters,
            actual.parameter_quantities,
            actual.parameter_units,
            vec![FiniteBound::try_new(1.)?; 4],
            actual.scales,
            "declared singular".into(),
            vec!["common error".into()],
        );
        let target = revision(&context, &package, case, ids).await?;
        let prepared = downstream(&context, &target, case, ids, external.clone()).await?;
        let result =
            Box::pin(prepared.start(&context.runtime.execution_services(), &CancelSource::new()))
                .await?
                .wait()
                .await?;
        let row = assert_contrast(&result, [1.; 4])?;
        assert_eq!(row.covariance_run_id, None);
        assert_eq!(row.covariance_origin, CovarianceInputOrigin::External);
        assert_eq!(row.covariance_label, "declared singular");
        assert_eq!(row.covariance_assumptions, ["common error"]);
        let mut indefinite = external;
        indefinite.values = [1., 2., 2., 1.]
            .into_iter()
            .map(FiniteBound::try_new)
            .collect::<std::result::Result<_, _>>()?;
        let prepared = downstream(&context, &target, case, ids, indefinite).await?;
        let result =
            Box::pin(prepared.start(&context.runtime.execution_services(), &CancelSource::new()))
                .await?
                .wait()
                .await?;
        // The explicitly requested product remains lazy; the actual relation boundary refuses.
        let error = result.table("runtime.propagated_covariances").unwrap_err();
        assert!(error.to_string().contains("negative eigenvalue"), "{error}");
        Ok(())
    })
    .catch_unwind()
    .await;
    finish(outcome, context.close().await)
}

#[tokio::test]
async fn native_nonunit_importance_withholds_fit_covariance() -> Result<()> {
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let (package, _, _) = regression(&context, 2.).await?;
        let fitted = fit(&context, &package).await?;
        assert!(
            parameter_covariances::Row::rows(&fitted.table("runtime.parameter_covariances")?)?
                .is_empty()
        );
        let unavailable = fitted.parameter_covariance().unwrap().unwrap_err();
        assert_eq!(unavailable.reason, WithheldReason::NonunitImportance);
        Ok(())
    })
    .catch_unwind()
    .await;
    finish(outcome, context.close().await)
}

#[cfg(feature = "native-study-tests")]
mod canonical {
    use super::*;
    use pse_ids::{Frame, FramedHasher};
    use pse_operations::{
        canonical::CanonicalStore,
        native_service::{NativeCanonicalService, NativeModuleSource, NativeServiceSpec},
    };
    use pse_runtime::{
        math::settings::FitCovarianceRecord,
        workflow::{
            CanonicalDeployment, DurabilitySelection, LeasePolicy, OuterAttestation, Runtime,
        },
    };
    use std::{
        path::PathBuf,
        sync::Arc,
        time::{Duration, Instant},
    };

    fn service() -> Result<NativeCanonicalService> {
        let environment = std::env::vars_os().collect();
        let helper = PathBuf::from(
            std::env::var_os("PSE_NATIVE_FIXTURE_SERVICE_HELPER")
                .expect("explicit already-built native-service helper"),
        );
        assert!(helper.is_absolute() && helper.is_file());
        let receipt = PathBuf::from(
            std::env::var_os("PSE_NATIVE_FIXTURE_MODULE_RECEIPT")
                .expect("explicit prepared private module receipt"),
        );
        let module =
            pse_host::study_module::select(&receipt, pse_host::study_module::Variant::Production)?;
        Ok(NativeCanonicalService::new(NativeServiceSpec {
            server: pse_host::server::select(&environment)?,
            helper,
            module_source: NativeModuleSource::Prepared(module),
            state_parent: std::env::temp_dir(),
        })?)
    }

    fn attach(context: &mut NativeContext, store: &CanonicalStore, worker: &str) -> Result<()> {
        // Replace the deliberately local initial adapter with this private service's
        // explicit canonical capability; never consult configured application state.
        context.runtime.close();
        context.runtime.clear_program_cache();
        let factory = Arc::new(
            context
                .shared
                .session_factory(pse_engine::session::native_engine_profile())?,
        );
        let mut executable = std::fs::File::open(std::env::current_exe()?)?;
        let mut hasher = FramedHasher::new(Frame::CanonicalPayloadV1);
        hasher.part_reader(executable.metadata()?.len(), &mut executable, &|| Ok(()))?;
        context.runtime = Runtime::from_shared(
            context.shared.clone(),
            factory.bind_model(pse_schema::shared_registry()?)?,
            CanonicalDeployment::new(
                store.clone(),
                OuterAttestation {
                    source: None,
                    build: hasher.finish_hash(),
                },
                None,
            ),
        )?
        .with_durability(DurabilitySelection::Durable {
            worker: worker.into(),
            policy: LeasePolicy::new(
                Duration::from_secs(30),
                Duration::from_millis(100),
                64,
                Duration::from_millis(20),
            )?,
        })?;
        Ok(())
    }

    fn journey(store: &CanonicalStore) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            let mut producer = NativeContext::new()?;
            let outcome = AssertUnwindSafe(async {
                attach(&mut producer, store, "covariance-producer")?;
                let (package, _, ids) = regression(&producer, 1.).await?;
                let fitted = fit(&producer, &package).await?;
                assert!(fitted.usable());
                let actual = fitted.parameter_covariance().unwrap().expect("qualified canonical fit covariance");
                assert_covariance(&actual, &fitted, ids);
                let record = FitCovarianceRecord {
                    run: fitted.canonical_run_key().expect("acknowledged exact run").into(),
                    attempt: fitted.canonical_attempt_key().expect("acknowledged exact attempt").into(),
                };
                assert!(matches!(&actual.origin, CovarianceOrigin::QualifiedFit { record: Some(selected), .. } if selected == &record));
                // Keep only the document payload and exact keys after the producer
                // runtime, cache, source package and result have been torn down.
                Ok((decoded(&actual)?, record, ids))
            }).catch_unwind().await;
            let (original, record, original_ids) = finish(outcome, producer.close().await)?;
            drop(producer);

            let mut reopened = NativeContext::new()?;
            let outcome = AssertUnwindSafe(async {
                attach(&mut reopened, store, "covariance-consumer")?;
                let cancel = CancelSource::new();
                let actual = reopened
                    .runtime
                    .fit_covariance(record.clone(), &cancel.token())
                    .await?;
                assert_eq!(
                    actual, original,
                    "reopen exact content, physical axes, order, scales and producer attribution"
                );
                for (value, expected) in actual.values.iter().zip(analytic_covariance()) {
                    close_to(value.into_inner(), expected);
                }
                let (package, case, ids) = regression(&reopened, 1.).await?;
                assert_eq!(ids, original_ids);
                let target = revision(&reopened, &package, case, ids).await?;
                let mut input = decoded(&actual)?;
                // The protected canonical reference restores evidence without a
                // retained in-process fit handle or an inferred latest attempt.
                reopened
                    .runtime
                    .reconcile_covariance(&mut input, &cancel.token())
                    .await?;
                let prepared = downstream(&reopened, &target, case, ids, input).await?;
                let result =
                    Box::pin(prepared.start(&reopened.runtime.execution_services(), &cancel))
                        .await?
                        .wait()
                        .await?;
                let row = assert_contrast(&result, analytic_covariance())?;
                let CovarianceOrigin::QualifiedFit { run_id, .. } = actual.origin else {
                    panic!("exact original qualified fit")
                };
                assert_eq!(row.covariance_run_id, Some(run_id));
                let missing = FitCovarianceRecord {
                    run: record.run.clone(),
                    attempt: "missing-fit-attempt".into(),
                };
                assert!(
                    reopened
                        .runtime
                        .fit_covariance(missing, &cancel.token())
                        .await
                        .is_err()
                );
                for mut changed in mutations(&original) {
                    let error = reopened
                        .runtime
                        .reconcile_covariance(&mut changed, &cancel.token())
                        .await
                        .unwrap_err();
                    assert!(
                        error
                            .to_string()
                            .contains("differs from the actual qualified fit content"),
                        "{error}"
                    );
                }
                Ok(())
            })
            .catch_unwind()
            .await;
            finish(outcome, reopened.close().await)
        })
    }

    #[tokio::test]
    async fn native_canonical_covariance_reopens_exact_fit_and_refuses_tampered_or_missing_reference()
    -> Result<()> {
        let mut service = service()?;
        let outcome = AssertUnwindSafe(service.run(
            Instant::now() + Duration::from_secs(300),
            |service| {
                Box::pin(async move {
                    service
                        .launch(Instant::now() + Duration::from_secs(90))
                        .await?;
                    let store = service.ready()?.store().clone();
                    // Keep scientific failures intact while run() independently
                    // settles every physical service/connection owner on all exits.
                    Ok(journey(&store).await)
                })
            },
        ))
        .catch_unwind()
        .await;
        let disposal = service.dispose().map_err(Into::into);
        let outcome = outcome.map(|service_result| {
            service_result
                .map_err(Into::into)
                .and_then(|scientific| scientific)
        });
        finish(outcome, disposal)
    }
}
