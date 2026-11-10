// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Production numerical acquisition across distinct producer/receiver processes.
use super::*;
use crate::workflow::{CanonicalDeployment, OuterAttestation};
use datafusion::execution::cache::Cache;
use pse_ids::SemanticId;
use pse_kernels::DerivativeOrder;
use pse_modeling::{Bindings, Limits};
use std::{collections::BTreeMap, path::Path};

const CHILD: &str = "math::portable::numerical::tests::canonical_numerical_child";
const MODE: &str = "PSE_NUMERICAL_RECEIVING_MODE";
const FILE: &str = "PSE_NUMERICAL_RECEIVING_FILE";
#[derive(Serialize, Deserialize)]
struct Context {
    database: String,
    revision: Option<String>,
    root: Option<pse_modeling::DeclarationId>,
    x: Option<SemanticId>,
    y: Option<SemanticId>,
    rows: Vec<SemanticId>,
}
fn producer() -> ReplayAdmission {
    // Named fixture qualification exercises the real consumer mechanics. It is
    // not a claim about the production worker's deployed-input qualification.
    fixture_producer(ContentHash::from_bytes([154; 32]))
}
fn bindings() -> Bindings {
    Bindings {
        demand: vec!["x".into(), "y".into()],
        ..Default::default()
    }
}
fn check(result: &pse_math::guarded::Evaluation, order: DerivativeOrder) {
    assert_eq!(result.values.len(), 2);
    assert!((result.values[0] - 4_f64.ln()).abs() < 2e-15);
    assert_eq!(result.values[1], 5.);
    if order >= DerivativeOrder::First {
        assert_eq!(result.jacobian.len(), 4);
        for (actual, expected) in result.jacobian.iter().zip([0., 0.25, 0.8, 0.6]) {
            assert!((actual - expected).abs() < 2e-15);
        }
    } else {
        assert!(result.jacobian.is_empty());
    }
    if order >= DerivativeOrder::Second {
        assert_eq!(result.hessians.len(), 8);
        for (actual, expected) in result
            .hessians
            .iter()
            .zip([0., 0., 0., -0.0625, 0.072, -0.096, -0.096, 0.128])
        {
            assert!((actual - expected).abs() < 2e-15);
        }
    } else {
        assert!(result.hessians.is_empty());
    }
}

#[test]
fn numerical_wire_nesting_refuses_before_unreadable_publication() {
    for depth in [MAX_NUMERICAL_JSON_DEPTH, MAX_NUMERICAL_JSON_DEPTH + 1] {
        let mut value = serde_json::json!("escaped \"{[\\ value");
        for _ in 0..depth {
            value = serde_json::Value::Array(vec![value]);
        }
        let mut bytes = Vec::new();
        let result = value.serialize(&mut serde_json::Serializer::with_formatter(
            &mut bytes,
            BoundedNesting::default(),
        ));
        if depth == MAX_NUMERICAL_JSON_DEPTH {
            result.unwrap();
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&bytes).unwrap(),
                value
            );
        } else {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("numerical wire nesting extent")
            );
        }
    }
}

#[tokio::test]
async fn canonical_numerical_child() {
    let Ok(mode) = std::env::var(MODE) else {
        return;
    };
    assert!(
        pse_math::context().is_err(),
        "receiver starts without the producer's symbol pool"
    );
    let file = std::path::PathBuf::from(std::env::var_os(FILE).unwrap());
    let mut context: Context = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    let state = std::env::var("PSE_SURREAL_STATE").unwrap();
    let mut options =
        pse_operations::canonical::CanonicalOptions::from_state(Path::new(&state)).unwrap();
    options.database = context.database.clone();
    let store =
        pse_operations::native_fixture::canonical_fixture_with_options(&options, false).unwrap();
    store.open().await.unwrap();
    let local = crate::workflow::tests::runtime();
    let runtime = crate::workflow::Runtime::from_shared(
        local.shared.clone(),
        local.model.clone(),
        CanonicalDeployment::new(
            store.clone(),
            OuterAttestation {
                source: Some(ContentHash::from_bytes([155; 32])),
                build: ContentHash::from_bytes([156; 32]),
            },
            Some(producer()),
        ),
    )
    .unwrap();
    drop(local);
    let source = runtime.source_services();
    let driver = crate::CancelSource::new();
    let package = if mode == "produce" {
        let declarations=pse_authoring::language::parse(
            "package p {def Root {var x:Scalar;var y:Scalar;eq a:log1p(x)==0;eq b:magnitude(x,y)==0;}}",
            SemanticId::from_bytes([157;16]),pse_authoring::language::IdentityPolicy::Named,Default::default(),
        ).unwrap();
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, crate::workflow::tests::physical())
            .await
            .unwrap();
        let model = package
            .prepare_authored(
                &source,
                root,
                pse_modeling::specialize::root_instance(root),
                bindings(),
                Limits::default(),
                &driver,
            )
            .await
            .unwrap();
        context.root = Some(root);
        context.x = Some(
            model
                .compiled()
                .model
                .symbols
                .values()
                .find(|symbol| symbol.lineage.path == "Root.x")
                .unwrap()
                .id,
        );
        context.y = Some(
            model
                .compiled()
                .model
                .symbols
                .values()
                .find(|symbol| symbol.lineage.path == "Root.y")
                .unwrap()
                .id,
        );
        context.rows = ["Root.a", "Root.b"]
            .into_iter()
            .map(|path| {
                model
                    .compiled()
                    .model
                    .equations
                    .iter()
                    .find(|row| row.lineage.path == path)
                    .unwrap()
                    .id
            })
            .collect();
        context.revision = Some(package.canonical_revision().unwrap().key.clone());
        drop(model);
        package
    } else {
        source
            .modeling_revision(
                store
                    .revision(context.revision.as_ref().unwrap())
                    .await
                    .unwrap()
                    .unwrap(),
                crate::workflow::tests::physical(),
                BTreeMap::new(),
            )
            .await
            .unwrap()
    };
    let root = context.root.unwrap();
    let x = context.x.unwrap();
    let y = context.y.unwrap();
    let math = source.preparation().math();
    #[cfg(feature = "compiled-evaluator")]
    let mut profiles = vec![pse_compiler::workspace::Profile::default()];
    #[cfg(not(feature = "compiled-evaluator"))]
    let profiles = vec![pse_compiler::workspace::Profile::default()];
    #[cfg(feature = "compiled-evaluator")]
    if mode == "receive" {
        for evaluator in [
            pse_math::library::Evaluator::CompiledScalar {
                module_bytes: 16 << 20,
            },
            pse_math::library::Evaluator::CompiledX4 {
                module_bytes: 16 << 20,
            },
        ] {
            let mut profile = pse_compiler::workspace::Profile::default();
            profile.optimization.evaluator = evaluator;
            profiles.push(profile);
        }
    }
    for profile in profiles {
        for order in [
            DerivativeOrder::Value,
            DerivativeOrder::First,
            DerivativeOrder::Second,
        ] {
            let coordinates = if order == DerivativeOrder::Value {
                Vec::new()
            } else {
                vec![y, x]
            };
            let result = package
                .observe_authored(
                    &source,
                    root,
                    pse_modeling::specialize::root_instance(root),
                    bindings(),
                    Limits::default(),
                    context.rows.clone(),
                    coordinates,
                    order,
                    pse_math::binding::CaseValues {
                        scalars: BTreeMap::from([(x, 3.), (y, 4.)]),
                    },
                    profile,
                    &driver,
                )
                .await
                .unwrap();
            check(&result, order);
        }
        if mode == "receive" {
            for (vx, vy) in [(1e-30_f64, 1e-30_f64), (1e308_f64, 1e308_f64)] {
                let result = package
                    .observe_authored(
                        &source,
                        root,
                        pse_modeling::specialize::root_instance(root),
                        bindings(),
                        Limits::default(),
                        context.rows.clone(),
                        vec![y, x],
                        DerivativeOrder::Second,
                        pse_math::binding::CaseValues {
                            scalars: BTreeMap::from([(x, vx), (y, vy)]),
                        },
                        profile,
                        &driver,
                    )
                    .await
                    .unwrap();
                let magnitude = vx.hypot(vy);
                let rx = vx / magnitude;
                let ry = vy / magnitude;
                let first = 1. / (1. + vx);
                let expected = [
                    vx.ln_1p(),
                    magnitude,
                    0.,
                    first,
                    ry,
                    rx,
                    0.,
                    0.,
                    0.,
                    -first * first,
                    rx * rx / magnitude,
                    -rx * ry / magnitude,
                    -rx * ry / magnitude,
                    ry * ry / magnitude,
                ];
                for (actual, expected) in result
                    .values
                    .iter()
                    .chain(&result.jacobian)
                    .chain(&result.hessians)
                    .zip(expected)
                {
                    assert!(
                        (actual - expected).abs() <= expected.abs() * 5e-14 + f64::from_bits(16),
                        "{vx},{vy}: {actual} vs {expected}"
                    );
                }
            }
            let zero = pse_math::binding::CaseValues {
                scalars: BTreeMap::from([(x, 0.), (y, 0.)]),
            };
            let result = package
                .observe_authored(
                    &source,
                    root,
                    pse_modeling::specialize::root_instance(root),
                    bindings(),
                    Limits::default(),
                    context.rows.clone(),
                    Vec::new(),
                    DerivativeOrder::Value,
                    zero.clone(),
                    profile,
                    &driver,
                )
                .await
                .unwrap();
            assert_eq!(result.values, vec![0., 0.]);
            assert!(
                package
                    .observe_authored(
                        &source,
                        root,
                        pse_modeling::specialize::root_instance(root),
                        bindings(),
                        Limits::default(),
                        context.rows.clone(),
                        vec![y, x],
                        DerivativeOrder::Second,
                        zero,
                        profile,
                        &driver
                    )
                    .await
                    .is_err(),
                "joint zero keeps original derivative refusal"
            );
        }
    }
    if mode == "receive" {
        let entries = math.entries.list_entries();
        assert!(!entries.is_empty());
        assert!(
            entries
                .iter()
                .all(|(key, _)| matches!(key, crate::math::artifacts::Key::Numerical(..))),
            "cold receiving must create no source-bearing artifact or scalar family"
        );
        for (key, _) in entries {
            let Some(crate::math::artifacts::Value::Numerical(artifact)) = math.entries.get(&key)
            else {
                panic!("numerical entry");
            };
            for instance in artifact.request().instances() {
                assert!(
                    math.modeling_cache
                        .body(instance.semantic_identity())
                        .is_none(),
                    "numerical hit reconstructs no admitted body"
                );
            }
        }
        assert_eq!(math.preparation_effects().completions, 0);
        assert_eq!(math.preparation_effects().function_assemblies, 0);
        let before = math.preparation_effects();
        let stopped = crate::CancelSource::new();
        stopped.cancel();
        assert!(
            package
                .observe_authored(
                    &source,
                    root,
                    pse_modeling::specialize::root_instance(root),
                    bindings(),
                    Limits::default(),
                    context.rows.clone(),
                    Vec::new(),
                    DerivativeOrder::Value,
                    pse_math::binding::CaseValues {
                        scalars: BTreeMap::from([(x, 3.), (y, 4.)])
                    },
                    Default::default(),
                    &stopped
                )
                .await
                .is_err()
        );
        let expired = crate::CancelSource::new().with_deadline(Some(std::time::Instant::now()));
        assert!(
            package
                .observe_authored(
                    &source,
                    root,
                    pse_modeling::specialize::root_instance(root),
                    bindings(),
                    Limits::default(),
                    context.rows.clone(),
                    Vec::new(),
                    DerivativeOrder::Value,
                    pse_math::binding::CaseValues {
                        scalars: BTreeMap::from([(x, 3.), (y, 4.)])
                    },
                    Default::default(),
                    &expired
                )
                .await
                .is_err()
        );
        assert_eq!(math.preparation_effects().frontiers, before.frontiers);

        let namespace = qualify_replay(
            math,
            Some(producer()),
            math.preparation_deadline(&driver).unwrap(),
            &driver,
        )
        .await
        .unwrap()
        .unwrap();
        let request = math
            .entries
            .list_entries()
            .into_iter()
            .find_map(|(key, _)| {
                let Some(crate::math::artifacts::Value::Numerical(artifact)) =
                    math.entries.get(&key)
                else {
                    return None;
                };
                (artifact.request().order() == DerivativeOrder::Value)
                    .then(|| artifact.request().clone())
            })
            .unwrap();
        let mut read = SelectedRead::new(
            store
                .protect(
                    package.canonical_revision().unwrap().clone(),
                    std::time::Duration::from_secs(600),
                )
                .await
                .unwrap(),
        );
        let candidate = store
            .product_candidate(
                read.selection(),
                request.request_bytes(),
                namespace.producer_key(),
                namespace.producer_class(),
                "",
            )
            .await
            .unwrap()
            .unwrap();
        let key = candidate.key().to_owned();
        let expected = math.retained_numerical_artifact(&request).unwrap();
        let received = reuse_execution_current(
            math,
            &store,
            &mut read,
            &namespace,
            request,
            &Arc::new(AtomicBool::new(false)),
        )
        .await
        .unwrap()
        .unwrap();
        let ExecutionAcquisition::Retained(retained) = received else {
            panic!("identical qualified warm bytes must reuse the admitted realization");
        };
        assert!(Arc::ptr_eq(&retained, &expected));
        read.finish().await.unwrap();
        crate::workflow::fixture_controls::corrupt_product_block(&store, &key)
            .await
            .unwrap();
        assert!(
            package
                .observe_authored(
                    &source,
                    root,
                    pse_modeling::specialize::root_instance(root),
                    bindings(),
                    Limits::default(),
                    context.rows.clone(),
                    Vec::new(),
                    DerivativeOrder::Value,
                    pse_math::binding::CaseValues {
                        scalars: BTreeMap::from([(x, 3.), (y, 4.)])
                    },
                    Default::default(),
                    &driver
                )
                .await
                .is_err(),
            "warm numerical cache cannot hide a corrupt canonical candidate"
        );
        assert_eq!(math.preparation_effects().completions, before.completions);
    } else {
        std::fs::write(&file, serde_json::to_vec(&context).unwrap()).unwrap();
    }
    driver.cancel();
    drop(package);
    drop(source);
    runtime.close();
    drop(runtime);
    store
        .drain_owned_connection_until(pse_operations::transport::original_deadline(
            pse_operations::canonical::REQUEST_TIMEOUT,
        ))
        .await
        .unwrap();
}

#[tokio::test]
async fn canonical_numerical_observation_survives_producer_exit_without_source_reconstruction() {
    let state = std::env::var("PSE_SURREAL_STATE").unwrap();
    let mut options =
        pse_operations::canonical::CanonicalOptions::from_state(Path::new(&state)).unwrap();
    options.database = format!(
        "canonical_test_numerical_receiver_{}",
        pse_operations::mint_id::<SemanticId>().to_hex()
    );
    let fixture =
        pse_operations::native_fixture::canonical_fixture_with_options(&options, true).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("receiving.json");
    std::fs::write(
        &file,
        serde_json::to_vec(&Context {
            database: options.database.clone(),
            revision: None,
            root: None,
            x: None,
            y: None,
            rows: Vec::new(),
        })
        .unwrap(),
    )
    .unwrap();
    for mode in ["produce", "receive"] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD, "--nocapture"])
            .env(MODE, mode)
            .env(FILE, &file)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{mode}: stdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fixture.remove_isolated_fixture().await.unwrap();
}
