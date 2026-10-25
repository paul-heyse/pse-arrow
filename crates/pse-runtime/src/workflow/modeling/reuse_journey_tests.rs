// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One locally owned authored edit journey, with actual preparation counters and original residuals.
use super::*;
use crate::workflow::{BindingAssignment, BindingQuantity, BindingTarget, PointOverlay};
use pse_backend_native::solve::{Backend, SolverSelection};
use pse_model::scalars::FiniteBound;
use std::io::Read;

fn marker(phase: &str, boundary: &str) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    eprintln!(
        "preparation-journey phase={phase} boundary={boundary} unix_seconds={}.{:09} receiving_pid={}",
        timestamp.as_secs(),
        timestamp.subsec_nanos(),
        std::process::id()
    );
}

async fn observed_phase<T>(
    service: &crate::math::MathService,
    phase: &str,
    operation: impl Future<Output = T>,
) -> (
    T,
    crate::math::PreparationCounts,
    crate::math::PreparationEffectCounts,
) {
    marker(phase, "begin");
    let before = service.preparation_effects();
    let (result, counts) = crate::math::counted(operation).await;
    let effects = service.preparation_effects().since(before);
    marker(phase, "end");
    eprintln!("preparation-journey phase={phase} counts={counts:?} effects={effects:?}");
    (result, counts, effects)
}

fn receiving_maps() -> (usize, BTreeSet<String>) {
    let mut bytes = Vec::new();
    std::fs::File::open("/proc/self/maps")
        .unwrap()
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .unwrap();
    assert!(
        bytes.len() <= 1024 * 1024,
        "bounded receiving-image map observation"
    );
    let text = std::str::from_utf8(&bytes).unwrap();
    let paths = text
        .lines()
        .filter_map(|line| line.split_whitespace().nth(5))
        .filter(|path| path.starts_with('/'))
        .map(str::to_owned)
        .collect();
    (text.lines().count(), paths)
}

fn sources(parameter: u8, nonlinear: bool) -> BTreeMap<String, Vec<u8>> {
    let expression = if nonlinear { "x*x==p" } else { "x==p" };
    BTreeMap::from([
        (
            "package.toml".into(),
            include_bytes!("../../../../../tests/fixtures/packages/minimal_named/package.toml")
                .to_vec(),
        ),
        (
            "models/bank.pse".into(),
            format!(
                "package minimal_named {{test Root fixture {{dof 0;}} {{param p:Scalar={parameter}; var x:Scalar; var y:Scalar; eq e:{expression}; eq sibling:y==7; annotation start x(1); annotation start y(7);}}}}"
            )
            .into_bytes(),
        ),
    ])
}

async fn point(
    package: &ModelingPackage,
    root: DeclarationId,
    overlay: &PointOverlay,
    phase: &str,
) -> (
    ModelingSolvePreparation,
    crate::math::PreparationCounts,
    crate::math::PreparationEffectCounts,
) {
    let service = package.runtime.shared.math();
    let mut profile = crate::workflow::tests::profile();
    profile.selection = SolverSelection::Explicit(Backend::Kinsol);
    let (prepared, counts, effects) = observed_phase(
        service,
        phase,
        package.prepare_bound_solve(
            root,
            overlay,
            crate::workflow::tests::compiler_profile(),
            profile,
            Default::default(),
            Limits::default(),
            &crate::CancelSource::new(),
        ),
    )
    .await;
    (prepared.unwrap(), counts, effects)
}

fn row(model: &ModelingPreparation, name: &str) -> SemanticId {
    // Inspection uses actual admitted lineage without adding path demand to the
    // exact preparation request whose reuse counters this journey measures.
    let model = &model.compiled().model;
    let mut members = model
        .symbols
        .values()
        .map(|symbol| (symbol.id, &symbol.lineage))
        .chain(
            model
                .equations
                .iter()
                .map(|equation| (equation.id, &equation.lineage)),
        )
        .filter(|(_, lineage)| lineage.path.rsplit('.').next() == Some(name));
    let (id, _) = members.next().unwrap();
    assert!(
        members.next().is_none(),
        "fixture member must be unique: {name}"
    );
    id
}

fn body(
    model: &ModelingPreparation,
    row: SemanticId,
) -> &Arc<pse_compiler::typed_math::AdmittedBody> {
    let mut owners = model
        .compiled()
        .admitted
        .case()
        .instances()
        .iter()
        .filter(|instance| {
            instance
                .contributions
                .iter()
                .any(|contribution| contribution.target == pse_math::binding::Target::Row(row))
        });
    let owner = owners.next().unwrap();
    assert!(owners.next().is_none());
    &model.compiled().admitted.bodies[&owner.body]
}

async fn residuals(package: &ModelingPackage, prepared: &ModelingSolvePreparation) -> (f64, f64) {
    let model = &prepared.model.model;
    let equation = row(model, "e");
    let sibling = row(model, "sibling");
    let mut values = prepared.model.values.clone();
    values.scalars.insert(row(model, "x"), 2.);
    values.scalars.insert(row(model, "y"), 7.);
    let observed = package
        .observe(
            model.clone(),
            BTreeSet::from([equation, sibling]),
            values,
            crate::workflow::tests::compiler_profile(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    (observed[&equation], observed[&sibling])
}

#[tokio::test]
async fn local_authored_value_binding_and_structural_edit_journey_preserves_exact_reuse() {
    marker("startup", "begin");
    let runtime = crate::workflow::tests::runtime();
    let physical = crate::workflow::tests::physical();
    let cancel = crate::CancelSource::new();
    let service = runtime.shared.math();
    let startup = service.preparation_effects();
    marker("startup", "end");
    eprintln!("preparation-journey phase=startup effects={startup:?}");
    let (original, _, _) = observed_phase(
        service,
        "source-ingress",
        runtime.package_from_sources(&[sources(1, false)], physical.clone(), &cancel),
    )
    .await;
    let original = original.unwrap();
    let root = original
        .declarations()
        .await
        .unwrap()
        .iter()
        .find(|declaration| declaration.name == "Root")
        .unwrap()
        .declaration_id;
    let empty = PointOverlay::default();
    let (cold, cold_counts, cold_effects) = point(&original, root, &empty, "cold").await;
    assert!(cold_counts.views > 0);
    assert_eq!(
        cold_effects.workspaces,
        1 + cold_counts.views + cold_counts.observations,
        "this cold fixture creates one semantic lane plus its numerical views"
    );
    let (warm, warm_counts, warm_effects) = point(&original, root, &empty, "warm").await;
    assert_eq!(warm_counts.views, 0);
    assert_eq!(warm_effects.workspaces, 0);
    assert_eq!(warm_effects.frontiers, 0);
    assert_eq!(warm_effects.completions, 0);
    assert!(Arc::ptr_eq(
        &cold.model.case.compiled().plan,
        &warm.model.case.compiled().plan
    ));
    assert_eq!(residuals(&original, &cold).await, (1., 0.)); // x-p = 2-1; y-7 = 0.

    // These maps describe this test's receiving process only. The separately admitted
    // solver worker retains its own producer/configuration evidence and is not observed here.
    let (before_count, before_maps) = receiving_maps();
    eprintln!(
        "preparation-journey phase=first-native-before receiving_pid={} map_count={before_count} mapped_paths={}",
        std::process::id(),
        before_maps.len()
    );
    let (solved, _, _) = observed_phase(
        service,
        "first-native",
        original.solve_case(
            cold.clone(),
            crate::workflow::tests::compiler_profile(),
            &cancel,
        ),
    )
    .await;
    let solved = solved.unwrap();
    assert!(
        solved.accepted,
        "first native solve qualifies its original equations"
    );
    assert!((solved.values.scalars[&row(&cold.model.model, "x")] - 1.).abs() < 1e-8);
    assert!((solved.values.scalars[&row(&cold.model.model, "y")] - 7.).abs() < 1e-8);
    let (after_count, after_maps) = receiving_maps();
    eprintln!(
        "preparation-journey phase=first-native-after receiving_pid={} map_count={after_count} mapped_paths={} added_paths={:?} removed_paths={:?}",
        std::process::id(),
        after_maps.len(),
        after_maps.difference(&before_maps).collect::<Vec<_>>(),
        before_maps.difference(&after_maps).collect::<Vec<_>>()
    );

    let (edited, _, _) = observed_phase(
        service,
        "authored-value-reload",
        original.reload_from_parent(&[sources(2, false)], &cancel),
    )
    .await;
    let edited = edited.unwrap();
    assert!(original.canonical_revision().is_none());
    assert!(edited.canonical_revision().is_none());
    assert_ne!(original.revision_id(), edited.revision_id());
    let (value, _, value_effects) = point(&edited, root, &empty, "authored-value").await;
    assert_eq!(
        value_effects.workspaces, 0,
        "the live source lineage retains its compiler"
    );
    let e = row(&cold.model.model, "e");
    let sibling = row(&cold.model.model, "sibling");
    assert!(Arc::ptr_eq(
        body(&cold.model.model, e).math(),
        body(&value.model.model, e).math()
    ));
    assert_eq!(residuals(&edited, &value).await, (0., 0.)); // 2-2, independently specified.
    let resolved = edited
        .solve_case(
            value.clone(),
            crate::workflow::tests::compiler_profile(),
            &cancel,
        )
        .await
        .unwrap();
    assert!(resolved.accepted);
    assert!((resolved.values.scalars[&row(&value.model.model, "x")] - 2.).abs() < 1e-8);

    let quantity = edited
        .quantities
        .quantity_types()
        .find(|quantity| quantity.name.as_deref() == Some("Scalar"))
        .unwrap();
    let overlay = PointOverlay {
        assignments: vec![BindingAssignment {
            target: BindingTarget::Member(row(&value.model.model, "p")),
            value: BindingQuantity {
                magnitude: FiniteBound::try_new(3.).unwrap(),
                quantity: quantity.id,
                unit: quantity.canonical_unit,
            },
        }],
    };
    let revision = edited.revision.identity();
    let (bound, bound_counts, bound_effects) =
        point(&edited, root, &overlay, "typed-binding").await;
    assert_eq!(bound_counts.views, 0);
    assert_eq!(bound_effects.workspaces, 0);
    assert_eq!(bound_effects.frontiers, 0);
    assert_eq!(bound_effects.completions, 0);
    assert_eq!(edited.revision.identity(), revision);
    assert_eq!(bound.bound_source.as_ref().unwrap().revision, revision);
    assert!(Arc::ptr_eq(
        &value.model.case.compiled().plan,
        &bound.model.case.compiled().plan
    ));
    assert_eq!(residuals(&edited, &bound).await, (-1., 0.)); // 2-3, without a source mutation.
    assert_eq!(residuals(&edited, &value).await, (0., 0.)); // Retained authored point stays at p=2.

    let (structural, _, _) = observed_phase(
        service,
        "structural-reload",
        edited.reload_from_parent(&[sources(2, true)], &cancel),
    )
    .await;
    let structural = structural.unwrap();
    assert!(structural.canonical_revision().is_none());
    assert_ne!(edited.revision_id(), structural.revision_id());
    let (changed, changed_counts, changed_effects) =
        point(&structural, root, &empty, "structural").await;
    assert!(changed_counts.views > 0);
    assert!(changed_effects.frontiers > 0);
    assert!(changed_effects.completions > 0);
    assert_eq!(
        changed_effects.workspaces,
        changed_counts.views + changed_counts.observations,
        "changed numerical products allocate workspaces without reconstructing the semantic lane"
    );
    let before_lane_selection = service.preparation_effects();
    drop(structural.pure_workspace().unwrap());
    assert_eq!(
        service
            .preparation_effects()
            .since(before_lane_selection)
            .workspaces,
        0,
        "the structural revision still selects the retained semantic lane"
    );
    assert_ne!(
        body(&value.model.model, e).semantic_identity(),
        body(&changed.model.model, e).semantic_identity()
    );
    assert!(!Arc::ptr_eq(
        body(&value.model.model, e).math(),
        body(&changed.model.model, e).math()
    ));
    assert!(Arc::ptr_eq(
        body(&value.model.model, sibling).math(),
        body(&changed.model.model, sibling).math()
    ));
    assert_eq!(residuals(&structural, &changed).await, (2., 0.)); // x*x-p = 4-2.
    assert_ne!(
        value.model.model.consumed_source_versions(),
        changed.model.model.consumed_source_versions()
    );

    marker("fresh-startup", "begin");
    let fresh_runtime = crate::workflow::tests::runtime();
    marker("fresh-startup", "end");
    let (fresh, _, _) = observed_phase(
        fresh_runtime.shared.math(),
        "fresh-source-ingress",
        fresh_runtime.package_from_sources(&[sources(2, true)], physical, &cancel),
    )
    .await;
    let fresh = fresh.unwrap();
    let (fresh_point, _, _) = point(&fresh, root, &empty, "fresh-oracle").await;
    assert_eq!(residuals(&fresh, &fresh_point).await, (2., 0.));
    for (package, prepared) in [(&structural, &changed), (&fresh, &fresh_point)] {
        let result = package
            .solve_case(
                prepared.clone(),
                crate::workflow::tests::compiler_profile(),
                &cancel,
            )
            .await
            .unwrap();
        assert!(result.accepted);
        let x = result.values.scalars[&row(&prepared.model.model, "x")];
        let budget = crate::workflow::tests::engineering_target(
            prepared.solve.numerics(),
            pse_model::generated::enums::NumericalTarget::Row,
            row(&prepared.model.model, "e"),
        )
        .budget;
        // Independent original mathematics at the frozen requested row accuracy.
        // The authored unbounded square equation admits both signs; no private
        // solver tolerance or unrequested root-sheet selection is invented here.
        assert!(
            (x * x - 2.).abs() <= budget,
            "original equation: {x}, budget {budget}"
        );
        assert!((x.abs() - 2_f64.sqrt()).abs() <= budget / 2_f64.sqrt());
        assert!((result.values.scalars[&row(&prepared.model.model, "y")] - 7.).abs() < 1e-8);
    }
    assert_eq!(
        body(&changed.model.model, e).semantic_identity(),
        body(&fresh_point.model.model, e).semantic_identity()
    );
    assert_eq!(
        residuals(&original, &warm).await,
        (1., 0.),
        "historical source remains immutable"
    );
    service.clear_program_cache();
    fresh_runtime.shared.math().clear_program_cache();
}
