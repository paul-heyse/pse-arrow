// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::workspace::modeling::tests::setup;
use pse_math::construction::ConstructionAdmission;
use pse_modeling::specialize::root_instance;
use std::sync::atomic::AtomicUsize;

const SOURCE: &str =
    "package p {def Root {var x:Scalar;var y:Scalar;let f:Scalar=x*y;let g:Scalar=x*x+y;}}";
#[derive(Debug, Default)]
struct RetentionProbe(AtomicUsize);
impl ModelingBodyRetention for RetentionProbe {
    fn generation(&self) -> u64 {
        0
    }
    fn get(
        &self,
        _: SemanticBodyHash,
    ) -> std::result::Result<Option<Arc<AdmittedBody>>, MathError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(None)
    }
    fn retain(
        &self,
        _: u64,
        _: SemanticBodyHash,
        body: Arc<AdmittedBody>,
    ) -> std::result::Result<Arc<AdmittedBody>, MathError> {
        Ok(body)
    }
}
#[derive(Debug)]
struct Construction;
impl ConstructionAdmission for Construction {
    fn try_grow(&self, _: usize) -> std::result::Result<(), MathError> {
        Ok(())
    }
}
fn fixture(
    order: DerivativeOrder,
) -> (
    CompilerWorkspace,
    Arc<NumericalObservationRequest>,
    NumericalObservationBuildRecipe,
    CaseValues,
    Arc<RetentionProbe>,
) {
    let (_, rows, physical_scope, root) = setup(SOURCE);
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let probe = Arc::new(RetentionProbe::default());
    workspace.attach_body_retention(probe.clone()).unwrap();
    workspace.publish_modeling(rows, physical_scope).unwrap();
    let frontier = workspace
        .plan_modeling_cancellable(
            root,
            root_instance(root),
            Bindings {
                demand: vec!["f".into(), "g".into(), "x".into(), "y".into()],
                ..Default::default()
            },
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let model = frontier.model();
    let rows = ["g", "f"]
        .map(|path| ModelingOutput::Member(model.paths[path]).row_id())
        .to_vec();
    let coordinates = [model.paths["y"], model.paths["x"]].to_vec();
    let values = frontier
        .case_values(&ModelingCaseBindings {
            values: [("x".into(), 2.), ("y".into(), 3.)].into(),
            ..Default::default()
        })
        .unwrap();
    let (request, recipe) = workspace
        .numerical_observation_request(
            &frontier,
            rows,
            coordinates,
            order,
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
        .unwrap();
    (workspace, request, recipe, values, probe)
}

#[test]
fn numerical_request_seals_original_custody_before_any_math_materialization() {
    let (mut workspace, request, recipe, values, probe) = fixture(DerivativeOrder::First);
    assert_eq!(
        probe.0.load(Ordering::SeqCst),
        0,
        "neither frontier planning nor passive request issuance asks for a semantic body"
    );
    assert_eq!(request.binding().rows().len(), 2);
    assert_eq!(request.instances().len(), 2);
    assert!(
        request
            .instances()
            .iter()
            .all(|instance| instance.formal_count() > 0)
    );
    assert!(!request.request_bytes().is_empty());
    let warm = workspace
        .compile_numerical_observation(&request, &recipe, Arc::new(AtomicBool::new(false)))
        .unwrap();
    assert!(
        probe.0.load(Ordering::SeqCst) > 0,
        "source mathematics is admitted only by the explicit warm miss recipe"
    );
    assert!(
        warm.iter()
            .zip(request.instances())
            .all(
                |(warm, original)| warm.semantic_identity() == original.semantic_identity()
                    && warm.instance() == original.attribution().instance()
            )
    );
    let mut wrong = request.as_ref().clone();
    wrong.key = ContentHash::from_bytes([91; 32]);
    assert!(
        workspace
            .compile_numerical_observation(&wrong, &recipe, Arc::new(AtomicBool::new(false)))
            .is_err()
    );
    let cancelled = Arc::new(AtomicBool::new(true));
    assert!(
        workspace
            .compile_numerical_observation(&request, &recipe, cancelled)
            .is_err()
    );
    let mut missing = values;
    missing.scalars.pop_first();
    assert!(
        request
            .binding()
            .inputs()
            .validate_values(&missing)
            .is_err()
    );
}

#[test]
fn numerical_only_observations_preserve_ordered_value_first_second_after_source_retirement() {
    for order in [
        DerivativeOrder::Value,
        DerivativeOrder::First,
        DerivativeOrder::Second,
    ] {
        let (mut workspace, request, recipe, mut values, _) = fixture(order);
        let original_scope =
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None);
        let warm = workspace
            .compile_numerical_observation(
                &request,
                &recipe,
                Arc::clone(original_scope.cancellation()),
            )
            .unwrap();
        let templates = warm
            .iter()
            .map(|artifact| {
                let scalars = artifact.request().build_scalars(&original_scope).unwrap();
                let compiled = artifact
                    .request()
                    .build_reusing(&original_scope, &scalars)
                    .unwrap();
                let captured = pse_math::transfer::capture(&compiled, &original_scope, |_| {
                    Ok::<Arc<dyn pse_math::AllocationOwner>, &str>(Arc::new(()))
                })
                .unwrap();
                assert_eq!(
                    captured.snapshot().first_layout.is_some(),
                    order >= DerivativeOrder::First
                );
                assert_eq!(
                    captured.snapshot().second.is_some(),
                    order >= DerivativeOrder::Second
                );
                let restored = pse_math::transfer::restore(
                    captured,
                    original_scope.clone(),
                    Arc::new(Construction),
                    |_| Ok::<Arc<dyn pse_math::AllocationOwner>, &str>(Arc::new(())),
                )
                .unwrap();
                Arc::new(restored.into_template())
            })
            .collect();
        drop(warm);
        drop(recipe);
        drop(workspace);
        original_scope.cancellation_control().cancel();
        let assembly = request.binding().assemble(templates).unwrap();
        let control = pse_diagnostics::Cancellation::default();
        let scope = pse_kernels::ExecutionScope::new(control.clone(), None);
        assert!(
            assembly
                .worker_scoped(scope.clone(), |_| Err::<
                    Arc<dyn pse_math::AllocationOwner>,
                    _,
                >("refused"))
                .is_err()
        );
        let mut worker = assembly
            .worker_scoped(scope, |bytes| {
                assert!(bytes > 0);
                Ok::<Arc<dyn pse_math::AllocationOwner>, &str>(Arc::new(()))
            })
            .unwrap();
        let first = worker.evaluate(&values).unwrap();
        assert_eq!(first.values, [7., 6.]);
        assert_eq!(
            first.jacobian,
            if order >= DerivativeOrder::First {
                vec![1., 4., 2., 3.]
            } else {
                vec![]
            }
        );
        assert_eq!(
            first.hessians,
            if order >= DerivativeOrder::Second {
                vec![0., 0., 0., 2., 0., 1., 1., 0.]
            } else {
                vec![]
            }
        );
        let y = request.binding().coordinates()[0];
        let x = request.binding().coordinates()[1];
        values.scalars.insert(x, 4.);
        values.scalars.insert(y, 5.);
        assert_eq!(worker.evaluate(&values).unwrap().values, [21., 20.]);
        values.scalars.insert(x, f64::INFINITY);
        assert!(worker.evaluate(&values).is_err());
        values.scalars.insert(x, 4.);
        control.cancel();
        assert!(worker.evaluate(&values).is_err());
        let population = (0_u32..17)
            .map(|index| {
                let mut point = values.clone();
                point.scalars.insert(x, 2. + f64::from(index));
                point.scalars.insert(y, 3. + f64::from(index));
                point
            })
            .collect::<Vec<_>>();
        let mut batch = assembly
            .batch_worker_scoped(
                pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
                17,
                |_| Ok::<Arc<dyn pse_math::AllocationOwner>, &str>(Arc::new(())),
            )
            .unwrap();
        for count in [0, 1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17] {
            let results = batch.evaluate(&population[..count]).unwrap();
            assert_eq!(results.len(), count);
            for (index, result) in results.iter().enumerate() {
                let result = result.as_ref().unwrap();
                let index = f64::from(u32::try_from(index).unwrap());
                let x = 2. + index;
                let y = 3. + index;
                assert_eq!(result.values, [x * x + y, x * y]);
                assert_eq!(
                    result.jacobian,
                    if order >= DerivativeOrder::First {
                        vec![1., 2. * x, x, y]
                    } else {
                        vec![]
                    }
                );
                assert_eq!(
                    result.hessians,
                    if order >= DerivativeOrder::Second {
                        vec![0., 0., 0., 2., 0., 1., 1., 0.]
                    } else {
                        vec![]
                    }
                );
            }
        }
        let mut invalid = population;
        invalid[5].scalars.insert(x, f64::INFINITY);
        let results = batch.evaluate(&invalid).unwrap();
        assert!(results[5].is_err());
        assert!(
            results
                .iter()
                .enumerate()
                .all(|(i, result)| i == 5 || result.is_ok())
        );
    }
}

#[test]
fn numerical_batch_guard_failures_keep_the_first_original_instance_and_other_points() {
    let source = "package p {fn positive(a:Scalar)->Scalar valid(a>0)=sqrt(a);fn logarithm(b:Scalar)->Scalar valid(b>0)=log(b);def Root {var x:Scalar;var y:Scalar;let f:Scalar=positive(x);let g:Scalar=logarithm(y);}}";
    let (mut workspace, _, _, root) = setup(source);
    let frontier = workspace
        .plan_modeling_cancellable(
            root,
            root_instance(root),
            Bindings {
                demand: ["f", "g", "x", "y"].map(str::to_owned).to_vec(),
                ..Default::default()
            },
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let rows = ["f", "g"]
        .map(|path| ModelingOutput::Member(frontier.model().paths[path]).row_id())
        .to_vec();
    let (request, recipe) = workspace
        .numerical_observation_request(
            &frontier,
            rows,
            vec![],
            DerivativeOrder::Value,
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
        .unwrap();
    let point = |x, y| {
        frontier
            .case_values(&ModelingCaseBindings {
                values: [("x".into(), x), ("y".into(), y)].into(),
                ..Default::default()
            })
            .unwrap()
    };
    let points = [point(4., 1.), point(-1., -1.), point(9., 1.)];
    let control = pse_diagnostics::Cancellation::default();
    let scope = pse_kernels::ExecutionScope::new(control, None);
    let warm = workspace
        .compile_numerical_observation(&request, &recipe, Arc::clone(scope.cancellation()))
        .unwrap();
    let templates = warm
        .iter()
        .map(|artifact| {
            let scalars = artifact.request().build_scalars(&scope).unwrap();
            let compiled = artifact.request().build_reusing(&scope, &scalars).unwrap();
            let captured = pse_math::transfer::capture(&compiled, &scope, |_| {
                Ok::<Arc<dyn pse_math::AllocationOwner>, &str>(Arc::new(()))
            })
            .unwrap();
            Arc::new(
                pse_math::transfer::restore(
                    captured,
                    scope.clone(),
                    Arc::new(Construction),
                    |_| Ok::<Arc<dyn pse_math::AllocationOwner>, &str>(Arc::new(())),
                )
                .unwrap()
                .into_template(),
            )
        })
        .collect();
    drop(warm);
    drop(recipe);
    drop(workspace);
    drop(frontier);
    let assembly = request.binding().assemble(templates).unwrap();
    let mut scalar = assembly
        .worker_scoped(scope.clone(), |_| {
            Ok::<Arc<dyn pse_math::AllocationOwner>, &str>(Arc::new(()))
        })
        .unwrap();
    let expected = scalar.evaluate(&points[1]).unwrap_err().to_string();
    let mut batch = assembly
        .batch_worker_scoped(scope, 3, |_| {
            Ok::<Arc<dyn pse_math::AllocationOwner>, &str>(Arc::new(()))
        })
        .unwrap();
    let results = batch.evaluate(&points).unwrap();
    assert_eq!(results[0].as_ref().unwrap().values, [2., 0.]);
    assert_eq!(results[2].as_ref().unwrap().values, [3., 0.]);
    assert_eq!(results[1].as_ref().unwrap_err().to_string(), expected);
    assert!(
        matches!(results[1].as_ref().unwrap_err(),MathError::Instance{instance,..} if *instance==request.instances()[0].attribution().instance())
    );
}

#[test]
fn numerical_request_separates_portable_semantics_from_local_native_realization() {
    let (mut workspace, _, _, root) = setup(SOURCE);
    let frontier = workspace
        .plan_modeling_cancellable(
            root,
            root_instance(root),
            Bindings {
                demand: vec!["f".into(), "x".into(), "y".into()],
                ..Default::default()
            },
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let row = ModelingOutput::Member(frontier.model().paths["f"]).row_id();
    let x = frontier.model().paths["x"];
    let y = frontier.model().paths["y"];
    let mut make = |rows: Vec<_>, axes: Vec<_>, order, profile| {
        workspace
            .numerical_observation_request(
                &frontier,
                rows,
                axes,
                order,
                profile,
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap()
            .unwrap()
            .0
    };
    let interpreted = make(
        vec![row],
        vec![x, y],
        DerivativeOrder::First,
        Profile::default(),
    );
    let mut native = Profile::default();
    native.optimization.evaluator = pse_math::library::Evaluator::CompiledScalar {
        module_bytes: 64 << 20,
    };
    let native = make(vec![row], vec![x, y], DerivativeOrder::First, native);
    assert_eq!(native.key(), interpreted.key());
    assert_eq!(native.request_bytes(), interpreted.request_bytes());
    assert_ne!(
        make(
            vec![row],
            vec![y, x],
            DerivativeOrder::First,
            Profile::default()
        )
        .key(),
        interpreted.key()
    );
    assert_ne!(
        make(
            vec![row],
            vec![x, y],
            DerivativeOrder::Value,
            Profile::default()
        )
        .key(),
        interpreted.key()
    );
    let mut changed = Profile::default();
    changed.optimization.cpe_iterations += 1;
    assert_ne!(
        make(vec![row], vec![x, y], DerivativeOrder::First, changed).key(),
        interpreted.key()
    );
    assert!(
        workspace
            .numerical_observation_request(
                &frontier,
                vec![row, row],
                vec![x, y],
                DerivativeOrder::First,
                Profile::default(),
                Arc::new(AtomicBool::new(false))
            )
            .is_err()
    );
}

#[test]
fn numerical_request_declines_anonymous_source_before_math_and_preserves_source_capability() {
    let rows = pse_authoring::language::parse(
        SOURCE,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let probe = Arc::new(RetentionProbe::default());
    workspace.attach_body_retention(probe.clone()).unwrap();
    workspace
        .publish_modeling(rows, PhysicalScope::default())
        .unwrap();
    let frontier = workspace
        .plan_modeling_cancellable(
            root,
            root_instance(root),
            Bindings {
                demand: vec!["f".into(), "g".into(), "x".into(), "y".into()],
                ..Default::default()
            },
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let row = ModelingOutput::Member(frontier.model().paths["f"]).row_id();
    let coordinate = frontier.model().paths["x"];
    let passive = workspace
        .numerical_observation_request(
            &frontier,
            vec![row],
            vec![coordinate],
            DerivativeOrder::Value,
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert!(
        passive.is_none(),
        "anonymous source locations cannot qualify durable execution custody"
    );
    assert_eq!(
        probe.0.load(Ordering::SeqCst),
        0,
        "ineligibility precedes semantic-body lookup"
    );
    let prepared = workspace
        .complete_modeling_cancellable(frontier, Arc::new(AtomicBool::new(false)))
        .unwrap();
    assert!(!prepared.model.paths.is_empty());
    assert!(probe.0.load(Ordering::SeqCst) > 0);
}
