// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Canonical source-bearing continuation through the real runtime owner.
use super::*;
use crate::workflow::{CanonicalDeployment, OuterAttestation};
use pse_ids::ContentHash;
use pse_kernels::DerivativeOrder;
use pse_math::binding::CaseValues;
use pse_operations::canonical::{CanonicalOptions, CanonicalStore};
use std::{collections::BTreeSet, path::Path};

fn deployment(store: CanonicalStore) -> CanonicalDeployment {
    CanonicalDeployment::new(
        store,
        OuterAttestation {
            source: Some(ContentHash::from_bytes([168; 32])),
            build: ContentHash::from_bytes([169; 32]),
        },
        Some(crate::math::portable::fixture_producer(
            ContentHash::from_bytes([170; 32]),
        )),
    )
}
fn runtime(store: CanonicalStore) -> Runtime {
    let local = crate::workflow::tests::runtime();
    let runtime =
        Runtime::from_shared(local.shared.clone(), local.model.clone(), deployment(store)).unwrap();
    drop(local);
    runtime
}
fn bindings() -> Bindings {
    Bindings {
        demand: vec!["x".into()],
        ..Default::default()
    }
}

#[tokio::test]
async fn authored_canonical_replay_continues_with_new_second_class_and_original_domain() {
    let state = std::env::var("PSE_SURREAL_STATE").unwrap();
    let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
    options.database = format!(
        "canonical_test_authored_replay_{}",
        pse_operations::mint_id::<SemanticId>().to_hex()
    );
    let store =
        pse_operations::native_fixture::canonical_fixture_with_options(&options, true).unwrap();
    let driver = crate::CancelSource::new();
    let original = runtime(store.clone());
    let source = original.source_services();
    let declarations = pse_authoring::language::parse(
        "package p {def Root {var x:Scalar;eq cubic:x*x*x==8;eq guarded:(x*x)/x==2;}}",
        SemanticId::from_bytes([171; 16]),
        pse_authoring::language::IdentityPolicy::Named,
        Default::default(),
    )
    .unwrap();
    let root = declarations
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let instance = pse_modeling::specialize::root_instance(root);
    let package = original
        .modeling_package(declarations, crate::workflow::tests::physical())
        .await
        .unwrap();
    // The first request has no canonical mathematical candidate. Ordinary source
    // construction publishes only compiler-sealed recipes after full preparation.
    let model = package
        .prepare_authored(
            &source,
            root,
            instance,
            bindings(),
            Limits::default(),
            &driver,
        )
        .await
        .unwrap();
    let revision = package.canonical_revision().unwrap().clone();
    let x = model
        .compiled()
        .model
        .symbols
        .values()
        .find(|symbol| symbol.lineage.path == "Root.x")
        .unwrap()
        .id;
    let cubic = model
        .compiled()
        .model
        .equations
        .iter()
        .find(|row| row.lineage.path == "Root.cubic")
        .unwrap()
        .id;
    let guarded = model
        .compiled()
        .model
        .equations
        .iter()
        .find(|row| row.lineage.path == "Root.guarded")
        .unwrap()
        .id;
    let seals = model
        .compiled()
        .portable_bodies()
        .map(|body| body.semantic_identity().unwrap())
        .collect::<BTreeSet<_>>();
    assert!(!seals.is_empty());
    // Recover the exact publication key through its normal immutable description,
    // so later controlled corruption tests the production request's own candidate.
    let math = source.preparation().math();
    let driver = math.preparation_driver(&driver).unwrap();
    let deadline = math.preparation_deadline(&driver).unwrap();
    let protected = portable_preparation::ProtectedPreparation::open(
        &package,
        Some(&source),
        math,
        model.consumed_source_versions(),
        &driver,
        deadline,
    )
    .await
    .unwrap();
    let (canonical, read, namespace) = protected.as_ref().unwrap().publication().unwrap();
    let selection = crate::math::portable::selection_material(math, read).unwrap();
    let body = model.compiled().portable_bodies().next().unwrap();
    let description = crate::math::portable::describe_body(
        math,
        read,
        selection,
        namespace.producer_key().to_owned(),
        namespace.producer_class(),
        body,
        &model.compiled().compiler_context(),
    )
    .unwrap();
    let product_key = crate::math::portable::publish_description(
        math,
        canonical,
        read,
        &description,
        deadline.into(),
    )
    .await
    .unwrap();
    portable_preparation::finish(protected, Ok(()))
        .await
        .unwrap();
    drop(description);
    drop(model);
    drop(package);
    drop(source);
    original.close();
    drop(original);

    // A distinct service owns no admitted body or numerical derivative product.
    // Unlike execution-only receiving, this capability deliberately restores recipes.
    let receiving = runtime(store.clone());
    let source = receiving.source_services();
    let package = source
        .modeling_revision(
            revision,
            crate::workflow::tests::physical(),
            BTreeMap::new(),
        )
        .await
        .unwrap();
    let math = source.preparation().math();
    for seal in &seals {
        assert!(math.modeling_cache.body(*seal).is_none());
    }
    let model = package
        .prepare_authored(
            &source,
            root,
            instance,
            bindings(),
            Limits::default(),
            &driver,
        )
        .await
        .unwrap();
    assert_eq!(
        seals,
        model
            .compiled()
            .portable_bodies()
            .map(|body| body.semantic_identity().unwrap())
            .collect()
    );
    for seal in &seals {
        assert!(math.modeling_cache.body(*seal).is_some());
    }
    let mut foreign_versions = model.consumed_source_versions().clone();
    foreign_versions
        .first_entry()
        .unwrap()
        .get_mut()
        .push_str("-foreign");
    let deadline = math.preparation_deadline(&driver).unwrap();
    let error = portable_preparation::ProtectedPreparation::open(
        &package,
        Some(&source),
        math,
        &foreign_versions,
        &driver,
        deadline,
    )
    .await
    .err()
    .unwrap();
    assert!(
        error
            .to_string()
            .contains("mathematical source origin differs"),
        "{error:?}"
    );
    let before = math.preparation_effects();
    let stopped = crate::CancelSource::new();
    stopped.cancel();
    assert!(matches!(
        package
            .prepare_authored(
                &source,
                root,
                instance,
                bindings(),
                Limits::default(),
                &stopped
            )
            .await,
        Err(WorkflowError::Math(
            crate::math::MathRuntimeError::Cancelled
        ))
    ));
    let expired = crate::CancelSource::new().with_deadline(Some(std::time::Instant::now()));
    assert!(
        package
            .prepare_authored(
                &source,
                root,
                instance,
                bindings(),
                Limits::default(),
                &expired
            )
            .await
            .is_err()
    );
    assert_eq!(
        math.preparation_effects().frontiers,
        before.frontiers,
        "original stop/deadline refuses before frontier work"
    );
    let values = CaseValues {
        scalars: BTreeMap::from([(x, 2.)]),
    };
    let base = package
        .bound_case(
            source.preparation(),
            &model,
            values.clone(),
            &BTreeMap::new(),
            DerivativeOrder::Value,
            crate::workflow::tests::compiler_profile(),
            &driver,
        )
        .await
        .unwrap();
    assert!(matches!(
        base.case.compiled().presolve.class_status,
        pse_math::presolve::ClassStatus::Unassessed
    ));
    // A sampled nonzero value is not evidence that the original x!=0
    // obligation holds over a free unbounded variable. Preserve that refusal.
    let class_profile = crate::math::solves::SolverProfile {
        intent: pse_backend_native::solve::SolveIntent::FeasiblePoint,
        selection: pse_backend_native::solve::SolverSelection::Explicit(
            pse_backend_native::solve::Backend::Highs,
        ),
        ..Default::default()
    };
    let pending = math
        .discover_class(base.case.clone(), values.clone(), &class_profile, &[])
        .await
        .unwrap();
    assert!(matches!(&pending.compiled().presolve.class_status,
        pse_math::presolve::ClassStatus::Pending(dependencies) if dependencies.iter().any(|dependency|
            matches!(dependency,pse_math::presolve::ClassDependency::UnestablishedObligation {..}))));
    drop(pending);
    // The class consumer explicitly admits a positive case box, establishing the
    // original denominator condition. This new class work is charged normally;
    // the unbounded source-bearing case remains the later derivative/domain control.
    let class_case = package
        .bound_case(
            source.preparation(),
            &model,
            values.clone(),
            &BTreeMap::from([(
                x,
                pse_compiler::workspace::ModelingVariableState {
                    lower: Some(Some(1.)),
                    upper: Some(Some(3.)),
                    ..Default::default()
                },
            )]),
            DerivativeOrder::Value,
            crate::workflow::tests::compiler_profile(),
            &driver,
        )
        .await
        .unwrap();
    let class = math
        .discover_class(class_case.case, values.clone(), &class_profile, &[])
        .await
        .unwrap();
    assert!(
        matches!(
            &class.compiled().presolve.class_status,
            pse_math::presolve::ClassStatus::RuledOut(
                pse_math::presolve::ClassWitness::NonAffineRow { row }
            ) if *row==cubic
        ),
        "{:?}",
        class.compiled().presolve.class_status
    );
    assert!(class.compiled().coefficients.is_none());
    let second = math
        .prepare_order(
            base.case,
            DerivativeOrder::Second,
            pse_diagnostics::Cancellation::default(),
        )
        .await
        .unwrap();
    assert_eq!(second.compiled().plan.order(), DerivativeOrder::Second);
    let column = second
        .compiled()
        .plan
        .columns()
        .iter()
        .position(|id| *id == x)
        .unwrap();
    let rows = [cubic, guarded].map(|id| {
        second
            .compiled()
            .plan
            .structure()
            .rows()
            .iter()
            .position(|row| row.id == id)
            .unwrap()
    });
    let assembly = math.assemble(second).await.unwrap();
    math.with_worker(assembly, BTreeMap::new(), &driver, move |worker| {
        let jacobian = worker.jacobian(&values)?;
        assert_eq!(jacobian.get(rows[0], column), Some(&12.));
        assert_eq!(jacobian.get(rows[1], column), Some(&1.));
        for (row, expected) in rows.into_iter().zip([12., 0.]) {
            let mut weights = vec![0.; worker.assembly().structure().rows().len()];
            weights[row] = 1.;
            assert_eq!(
                worker
                    .hessian(&values, 0., &weights)?
                    .get(column, column)
                    .copied()
                    .unwrap_or(0.),
                expected
            );
        }
        let invalid = CaseValues {
            scalars: BTreeMap::from([(x, 0.)]),
        };
        let error = worker.constraints(&invalid).unwrap_err();
        assert!(
            matches!(cause(&error), pse_math::MathError::Domain { .. }),
            "{error:?}"
        );
        assert!(
            matches!(error, pse_math::MathError::Instance { .. }),
            "original instance attribution: {error:?}"
        );
        Ok(())
    })
    .await
    .unwrap();
    // Even a warm basis cannot turn a present corrupt canonical candidate into a
    // cache hit or an ordinary source rebuild. Receiving closes its protected read.
    crate::workflow::fixture_controls::corrupt_product_block(&store, &product_key)
        .await
        .unwrap();
    let before = math.preparation_effects();
    let refused = package
        .prepare_authored(
            &source,
            root,
            instance,
            bindings(),
            Limits::default(),
            &driver,
        )
        .await;
    assert!(refused.is_err());
    assert_eq!(math.preparation_effects().completions, before.completions);
    drop(model);
    drop(package);
    drop(source);
    receiving.close();
    drop(receiving);
    store.remove_isolated_fixture().await.unwrap();
}
fn cause(error: &pse_math::MathError) -> &pse_math::MathError {
    if let pse_math::MathError::Instance { cause: inner, .. } = error {
        cause(inner)
    } else {
        error
    }
}
