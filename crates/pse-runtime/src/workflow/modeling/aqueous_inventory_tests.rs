// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual aqueous cell stocks and independently accumulated boundary closure.
use super::*;
use crate::{math::solves::NumericalInputs, workflow::tests as fixture};
#[cfg(feature = "solver-diffsol")]
use pse_backend_native::dynamics::Method;
use pse_backend_native::solve::{Backend, SolverSelection};
#[cfg(feature = "solver-diffsol")]
use pse_modeling::{
    SpecializedModel,
    specialize::{
        Value,
        finite::{FamilyKey, MemberKey},
    },
};
use std::collections::BTreeSet;

async fn conform(name: &str) -> ModelingConformanceReport {
    conform_precision(name, false).await
}

async fn conform_precision(name: &str, temporal_refinement: bool) -> ModelingConformanceReport {
    let (runtime, package, _spill, preparation) =
        pr_jacobian_tests::ordinary_delivery_package(false)
            .await
            .unwrap();
    let services = runtime.execution_services();
    let selected = package
        .declarations(services.preparation())
        .await
        .unwrap()
        .into_iter()
        .find(|row| row.name == name)
        .unwrap()
        .declaration_id;
    let mut solver = fixture::profile();
    solver.selection = SolverSelection::Explicit(Backend::Ipopt);
    let cancel = crate::CancelSource::new();
    if temporal_refinement {
        // Scalar defaults do not replace imported physical engineering floors.
        // Resolve actual original row owners before requesting numerical accuracy.
        let mut original = package
            .declared_execution(
                services.preparation(),
                selected,
                preparation.compiler,
                solver.clone(),
                NumericalInputs::default(),
                preparation.limits,
                &cancel,
            )
            .await
            .unwrap()
            .analysis;
        let plain = package
            .prepare_analysis(services.preparation(), &original, &cancel)
            .await
            .unwrap();
        solver.numerics.requirements = refinement_row_requirements(&plain);
        drop(plain);
        solver.numerics.supplier_action_accuracy = 1e-10;
        solver.numerics.linear_backward_error = 1e-10;
        solver.numerics.kkt.stationarity = 1e-10;
        solver.numerics.kkt.complementarity = 1e-10;
        original.solver = solver.clone();
        let checked = package
            .prepare_analysis(services.preparation(), &original, &cancel)
            .await
            .unwrap();
        assert_refinement_rows(&checked, &solver.numerics.requirements);
    }
    let requirements = solver.numerics.requirements.clone();
    let policy = ModelingConformancePolicy {
        compiler: preparation.compiler,
        solver,
        numerical: NumericalInputs::default(),
        limits: preparation.limits,
        derivatives: pse_backend_native::derivative_diagnostics::Policy {
            perturbation: 1e-6,
            relative_tolerance: 1e-4,
            maximum_cells: 20_000_000,
        },
        maximum_fixtures: 1,
        maximum_checks: 100_000,
        fixtures: ModelingFixtureSelection::Selected(BTreeSet::from([selected])),
        diagnostics: None,
    };
    let report = package.conform(&services, policy, &cancel).await.unwrap();
    if temporal_refinement {
        assert_eq!(
            report.results.len(),
            1,
            "original refinement solve required"
        );
        assert_refinement_rows(
            &report.results.values().next().unwrap().prepared,
            &requirements,
        );
        if !report.passed() {
            for result in report.results.values() {
                if let crate::math::solves::Outcome::Native(native) = &result.outcome {
                    let mut violations = native
                        .quality
                        .iter()
                        .flat_map(|quality| quality.rows.iter().chain(&quality.bounds))
                        .collect::<Vec<_>>();
                    violations.sort_by(|left, right| {
                        (right.physical / right.tolerance)
                            .total_cmp(&(left.physical / left.tolerance))
                    });
                    violations.truncate(8);
                    for violation in &violations {
                        let compiled = result.prepared.model.model.compiled();
                        let equation = compiled
                            .model
                            .equations
                            .iter()
                            .find(|equation| equation.id == violation.id);
                        let row = native.rows.iter().position(|id| *id == violation.id);
                        eprintln!(
                            "refinement violation id={} kind={} declaration={:?} instance={:?} row-bounds={:?} value={:?}",
                            violation.id,
                            if row.is_some() { "row" } else { "bound" },
                            equation.map(|equation| equation.lineage.declaration),
                            equation.map(|equation| equation.lineage.instance),
                            row.and_then(|row| native.observation.as_ref()?.bounds.get(row)),
                            row.and_then(|row| native.observation.as_ref()?.values.get(row)),
                        );
                    }
                    eprintln!(
                        "refinement native termination={:?}; qualification={:?}; original normalized violation={:?}; largest original violations={violations:?}; options={:?}; callback={:?}; validation={:?}",
                        native.termination,
                        native.qualification,
                        native
                            .quality
                            .as_ref()
                            .map(|quality| quality.normalized_max),
                        native.options,
                        native.callback_failure().map(ToString::to_string),
                        native.validation_failure().map(ToString::to_string),
                    );
                }
            }
        }
    }
    report
}

fn refinement_row_requirements(
    prepared: &ModelingSolvePreparation,
) -> Vec<pse_model::numerics::NumericalRequirement> {
    use pse_model::generated::enums::{NumericalCoordinates, NumericalTarget};
    let product = prepared.model.model.compiled();
    let scope = prepared.model.model.solved().lineage();
    let rows = prepared.model.case.compiled().plan.structure().rows();
    let resolved = prepared.solve.numerics();
    assert_eq!(
        rows.len(),
        resolved
            .targets
            .iter()
            .filter(|t| t.kind == NumericalTarget::Row)
            .count()
    );
    rows.iter().map(|row| {
        let sources = product.model.equations.iter().filter(|source| source.id == row.id)
            .collect::<Vec<_>>();
        assert_eq!(sources.len(), 1, "selected native row {} requires exact original equation lineage", row.id);
        let source = sources[0];
        assert!(product.model.instances.contains_key(&source.lineage.instance));
        let target = resolved.targets.iter().find(|target|
            target.kind == NumericalTarget::Row && target.id == row.id).unwrap();
        assert_eq!(target.quantity, row.quantity.as_id());
        // Frozen original conditioning, never the solved candidate magnitude.
        // Zero/invalid budgets cannot be silently replaced by a precision guess.
        assert!(target.nominal.is_finite() && target.nominal > 0.);
        assert!(target.budget.is_finite() && target.budget > 0.);
        let absolute = target.budget.min(1e-10 * target.nominal);
        assert!(absolute.is_finite() && absolute > 0.);
        pse_model::numerics::NumericalRequirement {
            requirement_id: pse_ids::named_id(row.id, "aqueous-temporal-refinement-row-precision"),
            model_id: scope.model_id,
            case_id: scope.case_id,
            instance_id: Some(source.lineage.instance),
            fit_id: scope.fit_id,
            target_id: row.id,
            target_kind: NumericalTarget::Row,
            nominal: None,
            scaling_factor: None,
            absolute_tolerance: Some(absolute),
            relative_tolerance: Some(0.),
            shared_engineering_allowance: None,
            engineering_rule_id: None,
            unit_id: Some(target.unit),
            coordinates: NumericalCoordinates::Physical,
            priority: 100,
            required: true,
            provenance: format!("temporal refinement numerical row precision; original source {}; instance {}; frozen nominal {} in canonical unit {}", source.lineage.declaration, source.lineage.instance, target.nominal, target.unit),
        }
    }).collect()
}

fn assert_refinement_rows(
    prepared: &ModelingSolvePreparation,
    requirements: &[pse_model::numerics::NumericalRequirement],
) {
    use pse_model::generated::enums::{NumericalProvenanceField, NumericalSource, NumericalTarget};
    let resolved = prepared.solve.numerics();
    let actual = resolved
        .targets
        .iter()
        .filter(|target| target.kind == NumericalTarget::Row)
        .map(|target| target.id)
        .collect::<BTreeSet<_>>();
    let expected = requirements
        .iter()
        .map(|request| request.target_id)
        .collect::<BTreeSet<_>>();
    assert_eq!(expected.len(), requirements.len());
    assert_eq!(
        actual, expected,
        "every actual native row keeps its explicit demand"
    );
    for request in requirements {
        let target = resolved
            .targets
            .iter()
            .find(|target| target.kind == NumericalTarget::Row && target.id == request.target_id)
            .unwrap();
        assert_eq!(Some(target.unit), request.unit_id);
        assert_eq!(target.budget, request.absolute_tolerance.unwrap());
        assert_eq!(target.relative, 0.);
        assert!(target.provenance.iter().any(|source| source.selected
            && source.source == NumericalSource::Analysis
            && source.field == NumericalProvenanceField::AbsoluteTolerance
            && source.declaration == Some(request.requirement_id)
            && source.description == request.provenance));
        let original = prepared
            .model
            .model
            .compiled()
            .model
            .equations
            .iter()
            .find(|row| row.id == request.target_id)
            .unwrap();
        assert_eq!(Some(original.lineage.instance), request.instance_id);
        let scope = prepared.model.model.solved().lineage();
        assert_eq!(scope.model_id, request.model_id);
        assert_eq!(scope.case_id, request.case_id);
    }
    let tolerances = prepared.solve.tolerances();
    assert_eq!(tolerances.rows.len(), requirements.len());
    assert!(
        tolerances
            .rows
            .iter()
            .all(|budget| budget.is_finite() && *budget > 0.)
    );
    let mut actual_budgets = tolerances.rows.clone();
    let mut expected_budgets = requirements
        .iter()
        .map(|request| request.absolute_tolerance.unwrap())
        .collect::<Vec<_>>();
    actual_budgets.sort_by(f64::total_cmp);
    expected_budgets.sort_by(f64::total_cmp);
    assert_eq!(
        actual_budgets, expected_budgets,
        "native row tolerances retain the resolved demand"
    );
}

fn assert_complete(report: &ModelingConformanceReport) {
    assert!(
        report.complete && report.passed(),
        "{:?}; {:?}",
        report.checks,
        report.failures
    );
    assert_eq!(report.results.len(), 1);
    let result = report.results.values().next().unwrap();
    let closure = result
        .reports
        .iter()
        .filter(|row| row.label == "aqueous cascade global material closure")
        .collect::<Vec<_>>();
    assert_eq!(
        closure.len(),
        2,
        "both actual water and ethanol inventories require closure"
    );
    for row in closure {
        assert!(row.value.abs() <= 0.001, "{row:?}");
    }
    let inventories = result
        .reports
        .iter()
        .filter(|row| row.label == "aqueous cascade component inventory")
        .collect::<Vec<_>>();
    assert!(
        inventories.len() > 2,
        "retain physical inventory across the original timeline"
    );
    assert!(inventories.iter().all(|row| row.value >= -0.001));
    assert!(
        result
            .reports
            .iter()
            .any(|row| row.label == "aqueous cascade imposed temperature" && row.value == 300.)
    );
}

#[tokio::test]
async fn aqueous_inventory_single_cell_matches_exact_mixed_cell_response() {
    assert_complete(&conform("aqueous_single_cell_step").await);
}

#[tokio::test]
async fn aqueous_inventory_four_cells_preserve_nonzero_clock_and_global_stock_balance() {
    assert_complete(&conform("aqueous_four_cell_nonzero_clock").await);
}

#[cfg(feature = "solver-diffsol")]
fn original_temporal_reports(
    result: &ModelingResult,
    declaration: DeclarationId,
    source: DeclarationId,
    points: usize,
) -> Vec<(f64, DeclarationId, f64)> {
    let model = &result.prepared.model.model.compiled().model;
    let cascade_definition = SemanticId::parse_hex("66e03000000000000000000000000120")
        .unwrap()
        .into();
    let cascades = model
        .instances
        .values()
        .filter(|instance| instance.definition == cascade_definition)
        .collect::<Vec<_>>();
    assert_eq!(cascades.len(), 1);
    let instance = cascades[0].id;
    let tuples = finite_family(model, instance, declaration);
    assert_eq!(tuples.len(), points * 2);
    let water = SemanticId::parse_hex("01a0e1461e93764398a669ec398dd8a0")
        .unwrap()
        .into();
    let ethanol = SemanticId::parse_hex("01a0e1461e93764398a669f485d31290")
        .unwrap()
        .into();
    let rows = result
        .reports
        .iter()
        .filter(|row| row.source_id == source)
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), tuples.len());
    let mut targets = BTreeSet::new();
    let mut water_coordinates = BTreeSet::new();
    let mut ethanol_coordinates = BTreeSet::new();
    let mut observations = Vec::with_capacity(tuples.len());
    for tuple in tuples {
        let [
            coordinate @ Value::Coordinate { bits, .. },
            Value::Entity { id: component, .. },
        ] = tuple.as_slice()
        else {
            panic!("original physical time/component tuple required: {tuple:?}");
        };
        let time = f64::from_bits(*bits);
        assert!(time.is_finite() && (5. ..=10.).contains(&time));
        let coordinates = if *component == water {
            &mut water_coordinates
        } else {
            assert_eq!(*component, ethanol);
            &mut ethanol_coordinates
        };
        assert!(coordinates.insert(coordinate.clone()));
        let target = model
            .finite_facts
            .require_member(&MemberKey {
                instance,
                declaration,
                tuple: tuple.clone(),
            })
            .unwrap();
        assert!(targets.insert(target));
        let matching = rows
            .iter()
            .filter(|row| row.target_id == target)
            .collect::<Vec<_>>();
        assert_eq!(
            matching.len(),
            1,
            "one original source/target observation per member"
        );
        let row = matching[0];
        assert_eq!(row.run_id, result.run_id);
        assert!(row.value.is_finite());
        observations.push((time, *component, row.value));
    }
    assert_eq!(water_coordinates.len(), points);
    assert_eq!(water_coordinates, ethanol_coordinates);
    observations
}

#[cfg(feature = "solver-diffsol")]
fn independent_first_radau_stages(step: f64, initial: f64) -> [[f64; 4]; 3] {
    use faer::prelude::Solve as _;
    // Independent three-stage Radau IIA tableau for dC/dt=4*(upstream-C).
    // This does not consume the production Jacobi roots/derivative stencils.
    let q = 6_f64.sqrt();
    let a = [
        [
            (88. - 7. * q) / 360.,
            (296. - 169. * q) / 1800.,
            (-2. + 3. * q) / 225.,
        ],
        [
            (296. + 169. * q) / 1800.,
            (88. + 7. * q) / 360.,
            (-2. - 3. * q) / 225.,
        ],
        [(16. - q) / 36., (16. + q) / 36., 1. / 9.],
    ];
    let matrix = faer::Mat::<f64>::from_fn(12, 12, |row, column| {
        let (stage, component) = (row / 4, row % 4);
        let (other_stage, other_component) = (column / 4, column % 4);
        let rate = if component == other_component {
            -4.
        } else if component == other_component + 1 {
            4.
        } else {
            0.
        };
        (if row == column { 1. } else { 0. }) - step * a[stage][other_stage] * rate
    });
    let rhs = faer::Mat::<f64>::from_fn(12, 1, |row, _| {
        initial
            + if row % 4 == 0 {
                400. * step * a[row / 4].iter().sum::<f64>()
            } else {
                0.
            }
    });
    let stages = matrix.partial_piv_lu().solve(&rhs);
    for row in 0..12 {
        let residual = (0..12)
            .map(|column| matrix[(row, column)] * stages[(column, 0)])
            .sum::<f64>()
            - rhs[(row, 0)];
        assert!(residual.abs() < 1e-10);
    }
    std::array::from_fn(|stage| std::array::from_fn(|component| stages[(4 * stage + component, 0)]))
}

#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn aqueous_inventory_temporal_refinement_preserves_complete_original_populations() {
    for step in [0.5, 0.25] {
        assert!(
            independent_first_radau_stages(step, 10.)
                .iter()
                .flatten()
                .all(|c| *c > 0.)
        );
    }
    let declaration = |text| SemanticId::parse_hex(text).unwrap().into();
    let ethanol = declaration("01a0e1461e93764398a669f485d31290");
    let mut errors = Vec::new();
    for (name, points) in [
        ("aqueous_four_cell_coarse_retained_step", 31),
        ("aqueous_four_cell_fine_retained_step", 61),
    ] {
        let report = conform_precision(name, true).await;
        assert_complete(&report);
        let result = report.results.values().next().unwrap();
        let outlet = original_temporal_reports(
            result,
            declaration("66e0300000000000000000000000012e"),
            declaration("66e03000000000000000000000000134"),
            points,
        );
        let inventory = original_temporal_reports(
            result,
            declaration("66e0300000000000000000000000012f"),
            declaration("66e03000000000000000000000000135"),
            points,
        );
        assert!(
            inventory
                .iter()
                .all(|(_, component, amount)| if *component == ethanol {
                    *amount >= -0.001
                } else {
                    (*amount - 55.).abs() <= 0.001
                })
        );
        let concentration = |time: f64| {
            let selected = outlet
                .iter()
                .filter(|(at, component, _)| *at == time && *component == ethanol)
                .collect::<Vec<_>>();
            assert_eq!(selected.len(), 1, "original common-time member at {time}");
            selected[0].2
        };
        assert!((concentration(5.) - 10.).abs() <= 1.);
        let mut maximum_error = 0_f64;
        for time in [6., 7., 10.] {
            // Exact four-cell cascade response (Q/V=1/s), not continuum flow.
            // Original clock starts at5s; each actual cell holds V/4*C.
            let z: f64 = 4. * (time - 5.);
            let expected = 10. + 90. * (1. - (-z).exp() * (1. + z + z * z / 2. + z * z * z / 6.));
            let error = (concentration(time) - expected).abs();
            assert!(error <= 1., "{name} at {time}: {error}mol/m3");
            maximum_error = maximum_error.max(error);
        }
        if points == 61 {
            assert!((concentration(5.25) - 11.708934118853846).abs() <= 1.);
        }
        eprintln!(
            "{name}: {points} original points; common-time maximum concentration error={maximum_error}mol/m3"
        );
        errors.push(maximum_error);
    }
    // Three-stage right Radau's linear step map predicts approximately30x
    // improvement here. An8x discriminator allows solver roundoff without
    // claiming a general convergence order or widening physical tolerances.
    assert!(errors[1] * 8. < errors[0], "coarse/fine errors: {errors:?}");
}

#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn aqueous_inventory_tight_zero_initial_collocation_preserves_original_refusal() {
    let coarse = independent_first_radau_stages(0.5, 0.);
    let fine = independent_first_radau_stages(0.25, 0.);
    assert!(coarse[0][3] < -0.22);
    assert!(fine[0][2] < -0.019 && fine[0][3] < -0.013);
    let report = conform_precision("aqueous_four_cell_coarse_nonzero_clock", true).await;
    assert!(report.complete && !report.passed());
    assert_eq!(report.results.len(), 1);
    let result = report.results.values().next().unwrap();
    let crate::math::solves::Outcome::Native(native) = &result.outcome else {
        panic!("original native diagnostic candidate required");
    };
    assert!(native.candidate.is_some());
    assert_eq!(
        native.termination.category,
        pse_backend_native::solve::Termination::Infeasible
    );
    assert_eq!(
        native.qualification,
        pse_backend_native::solve::Qualification::Unqualified
    );
    let quality = native.quality.as_ref().unwrap();
    assert!(!quality.feasible());
    let conservation = SemanticId::parse_hex("66e0300000000000000000000000011b")
        .unwrap()
        .into();
    let model = &result.prepared.model.model.compiled().model;
    assert!(quality.rows.iter().any(|violation| {
        violation.physical > violation.tolerance
            && model.equations.iter().any(|equation| {
                equation.id == violation.id && equation.lineage.declaration == conservation
            })
    }));
    assert!(
        report
            .failures
            .iter()
            .any(|failure| failure.class == pse_model::diagnostic::BoundaryClass::TrialRejected)
    );
}

#[tokio::test]
async fn aqueous_inventory_zero_flow_retains_original_component_stock() {
    assert_complete(&conform("aqueous_four_cell_zero_flow").await);
}

#[tokio::test]
async fn aqueous_geometry_requires_readmission_instead_of_an_independent_numeric_override() {
    let (runtime, package, _spill, preparation) =
        pr_jacobian_tests::ordinary_delivery_package(false)
            .await
            .unwrap();
    let services = runtime.execution_services();
    let cancel = crate::CancelSource::new();
    let selected = package
        .declarations(services.preparation())
        .await
        .unwrap()
        .into_iter()
        .find(|row| row.name == "aqueous_single_cell_integrated")
        .unwrap()
        .declaration_id;
    let mut solver = fixture::profile();
    solver.selection = SolverSelection::Explicit(Backend::Ipopt);
    let declared = package
        .declared_execution(
            services.preparation(),
            selected,
            preparation.compiler,
            solver,
            NumericalInputs::default(),
            preparation.limits,
            &cancel,
        )
        .await
        .unwrap();
    // Constructor volume belongs to original admission; the derived width is
    // neither an independently mutable coordinate nor a numeric permission.
    for (path, expected) in [
        ("root.volume", "unknown instantiated member volume"),
        (
            "root.cell_fraction",
            "case value root.cell_fraction requires a finite independent coordinate",
        ),
    ] {
        let mut changed = declared.analysis.clone();
        changed.case.values.insert(path.into(), 0.002);
        let error = package
            .prepare_analysis(services.preparation(), &changed, &cancel)
            .await
            .unwrap_err();
        assert!(error.to_string().contains(expected), "{path}: {error}");
    }
}

#[cfg(feature = "solver-diffsol")]
#[derive(Clone)]
struct ComponentRead {
    component: DeclarationId,
    original_stock: f64,
    stocks: Vec<(SemanticId, usize)>,
    incoming: usize,
    outgoing: usize,
}

#[cfg(feature = "solver-diffsol")]
fn finite_family(
    model: &SpecializedModel,
    instance: InstanceId,
    declaration: DeclarationId,
) -> &[Vec<Value>] {
    model
        .finite_facts
        .family(&FamilyKey {
            instance,
            declaration,
            role: "binding.indices.domain".into(),
            parents: Vec::new(),
        })
        .unwrap()
}

#[cfg(feature = "solver-diffsol")]
fn component_reads(trajectory: &ModelingTrajectory, initial_tracer: f64) -> Vec<ComponentRead> {
    let declaration = |text| SemanticId::parse_hex(text).unwrap().into();
    let model = &trajectory.prepared().model().compiled().model;
    let contract = trajectory.prepared().contract();
    let cascade = model
        .instances
        .values()
        .find(|instance| instance.definition == declaration("66e03000000000000000000000000120"))
        .unwrap()
        .id;
    let supplied = declaration("66e03000000000000000000000000130");
    let discharged = declaration("66e03000000000000000000000000131");
    let phase_amount = declaration("64e02000000000000000000000000207");
    let cell = declaration("66e03000000000000000000000000110");
    let components = finite_family(model, cascade, supplied);
    assert_eq!(components.len(), 2);
    // Existing species catalogue identities, not quantity or row-order guesses.
    [
        (declaration("01a0e1461e93764398a669ec398dd8a0"), 55.),
        (
            declaration("01a0e1461e93764398a669f485d31290"),
            initial_tracer,
        ),
    ]
    .into_iter()
    .map(|(component, original_stock)| {
        let tuple = components
            .iter()
            .find(
                |tuple| matches!(tuple.as_slice(), [Value::Entity { id, .. }] if *id == component),
            )
            .unwrap();
        let quadrature = |declaration| {
            let member = model
                .finite_facts
                .require_member(&MemberKey {
                    instance: cascade,
                    declaration,
                    tuple: tuple.clone(),
                })
                .unwrap();
            let integral = model.integral_of(member).unwrap();
            contract
                .quadratures
                .iter()
                .position(|id| *id == integral)
                .unwrap()
        };
        let stocks = model
            .instances
            .values()
            .filter(|instance| instance.definition == cell)
            .map(|instance| {
                let candidates = finite_family(model, instance.id, phase_amount)
                    .iter()
                    .filter(|phase_tuple| phase_tuple.last() == tuple.first())
                    .collect::<Vec<_>>();
                assert_eq!(
                    candidates.len(),
                    1,
                    "one actual liquid-phase stock per component/cell"
                );
                let member = model
                    .finite_facts
                    .require_member(&MemberKey {
                        instance: instance.id,
                        declaration: phase_amount,
                        tuple: candidates[0].clone(),
                    })
                    .unwrap();
                let index = contract.states.iter().position(|id| *id == member).unwrap();
                assert!(contract.differential[index]);
                (member, index)
            })
            .collect();
        ComponentRead {
            component,
            original_stock,
            stocks,
            incoming: quadrature(supplied),
            outgoing: quadrature(discharged),
        }
    })
    .collect()
}

#[cfg(feature = "solver-diffsol")]
fn prefix_balances(
    trajectory: &ModelingTrajectory,
    samples: &[&pse_backend_native::dynamics::Sample],
    times: &[f64],
    reads: &[ComponentRead],
) -> Result<(), String> {
    if samples.iter().map(|sample| sample.time).collect::<Vec<_>>() != times {
        return Err("original requested physical point is missing or reordered".into());
    }
    let contract = trajectory.prepared().contract();
    for sample in samples {
        if sample.integrals.len() != contract.quadratures.len() {
            return Err("native cumulative quadrature extent differs".into());
        }
        // This owner only projects produced native coordinates to canonical
        // physical values; it does not reevaluate a model or a definite integral.
        let (_, physical, _) = trajectory
            .sample_context(sample)
            .map_err(|error| error.to_string())?;
        for read in reads {
            let amount = read.stocks.iter().try_fold(0., |total, (member, index)| {
                if contract.states.get(*index) != Some(member) || sample.state.get(*index).is_none()
                {
                    return Err(
                        "actual phase stock/native coordinate correspondence differs".to_owned(),
                    );
                }
                physical
                    .scalars
                    .get(member)
                    .map(|amount| total + amount)
                    .ok_or_else(|| "actual phase stock is unavailable".to_owned())
            })?;
            let error = (amount - read.original_stock)
                - (sample.integrals[read.incoming] - sample.integrals[read.outgoing]);
            if !error.is_finite() || error.abs() > 0.001 {
                return Err(format!(
                    "component {:?} at {}s: independent global stock/flux error {error}mol",
                    read.component, sample.time
                ));
            }
        }
    }
    Ok(())
}

#[cfg(feature = "solver-diffsol")]
async fn assert_integrated(
    name: &str,
    cells: usize,
    times: &[f64],
    response: &str,
    stock: &str,
    initial_tracer: f64,
    counterexamples: bool,
) -> ModelingConformanceReport {
    let report = conform(name).await;
    assert!(
        report.complete && report.passed(),
        "{:?}; {:?}",
        report.checks,
        report.failures
    );
    assert!(
        report.results.is_empty(),
        "an integrated trajectory is its own native result"
    );
    assert_eq!(report.trajectories.len(), 1);
    let trajectory = report.trajectories.values().next().unwrap();
    assert!(trajectory.accepted() && trajectory.checks_complete());
    assert_eq!(trajectory.prepared().profile().method, Method::Diffsol);
    assert_eq!(
        trajectory
            .report()
            .samples
            .iter()
            .map(|sample| sample.time)
            .collect::<Vec<_>>(),
        times
    );
    let cell: DeclarationId = SemanticId::parse_hex("66e03000000000000000000000000110")
        .unwrap()
        .into();
    assert_eq!(
        trajectory
            .prepared()
            .model()
            .compiled()
            .model
            .instances
            .values()
            .filter(|instance| instance.definition == cell)
            .count(),
        cells,
        "requested output times must not create additional physical cells"
    );
    assert_eq!(
        trajectory
            .prepared()
            .contract()
            .differential
            .iter()
            .filter(|state| **state)
            .count(),
        cells * 2,
        "water and ethanol actual amount are the differential states per cell"
    );
    for source in [response, stock] {
        let source: DeclarationId = SemanticId::parse_hex(source).unwrap().into();
        for &time in times {
            let checks = trajectory
                .checks()
                .iter()
                .filter(|check| check.source_id == source && check.time == Some(time))
                .collect::<Vec<_>>();
            assert_eq!(
                checks
                    .iter()
                    .map(|check| check.target_id)
                    .collect::<BTreeSet<_>>()
                    .len(),
                2,
                "original source {source:?} must check both component targets at physical time {time}"
            );
            assert!(checks.iter().all(|check| check.satisfied));
        }
    }
    let reads = component_reads(trajectory, initial_tracer);
    assert!(reads.iter().all(|read| read.stocks.len() == cells));
    let samples = trajectory.report().samples.iter().collect::<Vec<_>>();
    prefix_balances(trajectory, &samples, times, &reads).unwrap();
    if counterexamples {
        let mut wrong_component = reads.clone();
        wrong_component[1].incoming = reads[0].incoming;
        wrong_component[1].outgoing = reads[0].outgoing;
        assert!(
            prefix_balances(trajectory, &samples, times, &wrong_component).is_err(),
            "water boundary quadratures cannot certify ethanol inventory"
        );
        let missing = samples
            .iter()
            .copied()
            .filter(|sample| sample.time != 5.25)
            .collect::<Vec<_>>();
        assert!(
            prefix_balances(trajectory, &missing, times, &reads).is_err(),
            "missing the original5.25s point cannot be treated as full coverage"
        );
    }
    let closure = trajectory
        .reports()
        .iter()
        .filter(|row| row.label == "aqueous cascade global material closure")
        .collect::<Vec<_>>();
    // Sample checks above retain physical time. Reports are terminal whole-domain
    // observations, not a second time-indexed trajectory table.
    assert_eq!(
        closure.len(),
        2,
        "both component closures retain their terminal original report"
    );
    assert!(closure.iter().all(|row| row.value.abs() <= 0.001));
    let terminal_source: DeclarationId = SemanticId::parse_hex("66e03000000000000000000000000157")
        .unwrap()
        .into();
    let terminal = trajectory
        .checks()
        .iter()
        .filter(|check| check.source_id == terminal_source)
        .collect::<Vec<_>>();
    assert_eq!(
        terminal
            .iter()
            .map(|check| check.target_id)
            .collect::<BTreeSet<_>>()
            .len(),
        2
    );
    assert!(
        terminal
            .iter()
            .all(|check| check.time == times.last().copied() && check.satisfied)
    );
    report
}

#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn aqueous_integrated_single_cell_preserves_original_response_and_stock() {
    assert_integrated(
        "aqueous_single_cell_integrated",
        1,
        &[0., 1., 5.],
        "66e03000000000000000000000000242",
        "66e03000000000000000000000000243",
        0.,
        false,
    )
    .await;
}

#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn aqueous_integrated_four_cells_use_one_nonzero_physical_clock() {
    assert_integrated(
        "aqueous_four_cell_integrated_nonzero_clock",
        4,
        &[5., 5.25, 6., 7., 10.],
        "66e03000000000000000000000000252",
        "66e03000000000000000000000000253",
        0.,
        true,
    )
    .await;
}

#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn aqueous_integrated_four_cells_preserve_site_initial_profile() {
    let report = assert_integrated(
        "aqueous_four_cell_integrated_initial_profile",
        4,
        &[5., 5.25, 6., 7., 10.],
        "66e030000000000000000000000002a2",
        "66e030000000000000000000000002a3",
        0.0125,
        false,
    )
    .await;
    let trajectory = report.trajectories.values().next().unwrap();
    let reads = component_reads(trajectory, 0.0125);
    let initial = [5., 10., 15., 20.];
    for sample in &trajectory.report().samples {
        let (_, physical, _) = trajectory.sample_context(sample).unwrap();
        let mut actual = reads[1]
            .stocks
            .iter()
            .map(|(member, _)| physical.scalars[member])
            .collect::<Vec<_>>();
        // Matrix exponential of the equal-volume upwind cascade, shifted by the inlet
        // concentration. The subdiagonal series terminates at the current cell.
        let z = 4. * (sample.time - 5.);
        let mut expected = (0..initial.len())
            .map(|cell| {
                let mut power_over_factorial = 1.;
                let mut deviation = initial[cell] - 100.;
                for (order, concentration) in (1_u32..).zip(initial[..cell].iter().rev()) {
                    power_over_factorial *= z / f64::from(order);
                    deviation += power_over_factorial * (concentration - 100.);
                }
                0.00025 * (100. + (-z).exp() * deviation)
            })
            .collect::<Vec<_>>();
        // Native state identity order is not spatial order. The fixture separately
        // observes the last cell at its authored outlet and distinguishes reversal.
        actual.sort_by(f64::total_cmp);
        expected.sort_by(f64::total_cmp);
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!(
                (actual - expected).abs() < 1e-7,
                "at {}s: expected cell stock {expected}mol, observed {actual}mol",
                sample.time
            );
        }
    }
}

#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn aqueous_integrated_feed_wash_feed_carries_stock_through_authored_holds() {
    let report = assert_integrated(
        "aqueous_single_cell_integrated_feed_wash_feed",
        1,
        &[5., 5.5, 6., 6.5, 8., 9., 10.],
        "66e030000000000000000000000002b2",
        "66e030000000000000000000000002b3",
        0.01,
        false,
    )
    .await;
    let trajectory = report.trajectories.values().next().unwrap();
    let prepared = trajectory.prepared();
    let model = &prepared.model().compiled().model;
    let reads = component_reads(trajectory, 0.01);
    let [schedule] = prepared.profile().schedule.as_slice() else {
        panic!("expected one authored inlet schedule")
    };
    assert_eq!(schedule.times, [6., 8.]);
    let scheduled = prepared.contract().parameters[schedule.parameter];
    let inlet = SemanticId::parse_hex("66e03000000000000000000000000159")
        .unwrap()
        .into();
    let cascade = model.symbols[&scheduled].lineage.instance;
    let tuple = finite_family(model, cascade, inlet)
        .iter()
        .find(|tuple| {
            matches!(tuple.as_slice(), [Value::Entity { id, .. }] if *id == reads[1].component)
        })
        .unwrap();
    assert_eq!(
        model
            .finite_facts
            .require_member(&MemberKey {
                instance: cascade,
                declaration: inlet,
                tuple: tuple.clone(),
            })
            .unwrap(),
        scheduled
    );

    let native = trajectory.report();
    assert_eq!(native.events.len(), 2);
    for (transition, time) in native.events.iter().zip([6., 8.]) {
        assert_eq!(transition.time, time);
        assert!(transition.event.is_none(), "a hold change is not a root event");
        let after = transition.after.as_ref().expect("hold must settle");
        for read in &reads {
            let [(_, index)] = read.stocks.as_slice() else {
                panic!("one physical stock per species in a single cell")
            };
            assert!(
                (transition.before[*index] - after[*index]).abs() < 1e-7,
                "stock jumped across the hold change at {time}s"
            );
        }
    }

    // Q/V = 1/s. Propagate concentration and independently integrate each
    // exact hold before adding its contribution to the cumulative prefix.
    let holds = [(5., 6., 100.), (6., 8., 0.), (8., 10., 100.)];
    for sample in &native.samples {
        let (_, physical, _) = trajectory.sample_context(sample).unwrap();
        let mut concentration = 10.;
        let mut supplied = 0.;
        let mut discharged = 0.;
        let mut active_inlet = 100.;
        for (start, end, inlet) in holds {
            if sample.time < start {
                break;
            }
            active_inlet = inlet;
            let elapsed = sample.time.min(end) - start;
            let decay = (-elapsed).exp();
            supplied += 0.001 * inlet * elapsed;
            discharged += 0.001 * (inlet * elapsed + (concentration - inlet) * (1. - decay));
            concentration = inlet + (concentration - inlet) * decay;
        }
        let read = &reads[1];
        assert!((physical.scalars[&scheduled] - active_inlet).abs() < 1e-9);
        for (label, actual, expected) in [
            ("stock", physical.scalars[&read.stocks[0].0], 0.001 * concentration),
            ("supplied", sample.integrals[read.incoming], supplied),
            ("discharged", sample.integrals[read.outgoing], discharged),
        ] {
            assert!(
                (actual - expected).abs() < 1e-7,
                "at {}s: {label} expected {expected}mol, observed {actual}mol",
                sample.time
            );
        }
    }
}

#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn aqueous_integrated_zero_flow_preserves_original_nonzero_inventory() {
    assert_integrated(
        "aqueous_four_cell_integrated_zero_flow",
        4,
        &[0., 1., 5.],
        "66e03000000000000000000000000262",
        "66e03000000000000000000000000263",
        0.01,
        false,
    )
    .await;
}

#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn aqueous_integrated_original_native_outcome_survives_protected_export_and_portable_read() {
    use base64::Engine as _;
    use buffa_descriptor::{
        DescriptorPool, DynamicMessage, ReflectMessage as _, Value as WireValue,
        ValueRef as WireValueRef,
    };
    use datafusion::arrow::{
        array::{FixedSizeBinaryArray, Float64Array, Int64Array},
        ipc::reader::StreamReader,
    };
    use pse_compiler::workspace::ModelingOutput;

    let (runtime, package, _spill, preparation) =
        pr_jacobian_tests::ordinary_delivery_package(true)
            .await
            .unwrap();
    let physical = package.physical_context().clone();
    let runtime = super::super::durable_tests::durable_runtime_on(runtime);
    let store = runtime.canonical_store().unwrap();
    store
        .create_project(
            "aqueous-lifecycle-create",
            "aqueous-lifecycle",
            "Original aqueous inventory lifecycle",
        )
        .await
        .unwrap();
    let project = store.project("aqueous-lifecycle").await.unwrap().unwrap();
    let source = Box::pin(runtime.project_fork(
        &project,
        "aqueous-lifecycle-main",
        "aqueous-lifecycle-problem",
        "aqueous-lifecycle-fork",
        &package,
    ))
    .await
    .unwrap();
    let original_revision = source.key.clone();
    let original = Box::pin(runtime.modeling_revision(source, physical, BTreeMap::new()))
        .await
        .unwrap();
    let services = runtime.execution_services();
    let case = original
        .declarations(services.preparation())
        .await
        .unwrap()
        .into_iter()
        .find(|row| row.name == "aqueous_four_cell_integrated_nonzero_clock")
        .unwrap()
        .declaration_id;
    let cancel = crate::CancelSource::new();
    let wrong_route = Box::pin(original.prepare_operation(
        services.preparation(),
        &crate::workflow::OperationSource::of(&original),
        &preparation,
        crate::workflow::RuntimeOperationRequest::DeclaredCase {
            case,
            settings: Box::new(crate::math::settings::SolveSettings::default()),
        },
        None,
        &cancel,
    ))
    .await
    .unwrap_err();
    let WorkflowError::Boundary(wrong_route) = wrong_route else {
        panic!("the Case operation must attribute the unsupported authored procedure: {wrong_route:?}");
    };
    assert_eq!(wrong_route.sources, vec![case.as_id()]);
    assert!(matches!(
        wrong_route.observations.get("reason"),
        Some(pse_model::diagnostic::Observation::Text(reason))
            if reason.contains("requires a Simulation operation")
    ));
    let operation = Box::pin(original.prepare_operation(
        services.preparation(),
        &crate::workflow::OperationSource::of(&original),
        &preparation,
        crate::workflow::RuntimeOperationRequest::Study {
            operation: Box::new(crate::workflow::OperationRequest::Simulation(
                crate::workflow::SimulationOperation {
                    case,
                    settings: None,
                },
            )),
            output: crate::workflow::StudyOutputDemand::Full {},
            admitted_horizon: None,
        },
        None,
        &cancel,
    ))
    .await
    .unwrap();
    let handle = operation.start(&services, &cancel).await.unwrap();
    let run_key = handle.canonical_run_key().unwrap().to_owned();
    let attempt_key = handle.canonical_attempt_key().unwrap().to_owned();
    let run = handle.wait().await.unwrap();
    assert!(run.usable());
    let crate::workflow::RunReport::Simulation(trajectory) = run.report().unwrap() else {
        panic!("the authored integrate procedure must produce its original native simulation");
    };
    let native_permission = trajectory.governing_permission(&run).unwrap();
    assert!(native_permission.permits_result() && native_permission.has_native_proof());
    assert_eq!(native_permission.run_id(), run.run_id);
    assert!(native_permission.attempt_id().is_some());
    assert!(std::ptr::eq(
        trajectory.qualified_report(&run).unwrap(),
        trajectory.report()
    ));
    assert!(trajectory.accepted() && trajectory.checks_complete());
    assert_eq!(trajectory.prepared().profile().method, Method::Diffsol);
    let times = [5., 5.25, 6., 7., 10.];
    let samples = trajectory.report().samples.iter().collect::<Vec<_>>();
    prefix_balances(
        trajectory,
        &samples,
        &times,
        &component_reads(trajectory, 0.),
    )
    .unwrap();
    // Expectations come directly from the producing native vectors and their
    // admitted member/physical contracts, not the relation projection also used
    // by the canonical writer.
    let prepared = trajectory.prepared();
    let product = prepared.model().compiled();
    let mut expected = BTreeMap::new();
    for (sample, point) in trajectory.report().samples.iter().enumerate() {
        assert_eq!(point.time.to_bits(), times[sample].to_bits());
        assert_eq!(point.outputs.len(), prepared.contract().outputs.len());
        for (column, target) in prepared.contract().outputs.iter().enumerate() {
            let ModelingOutput::Member(member) = product
                .admitted
                .outputs
                .iter()
                .find(|output| output.row_id() == *target)
                .unwrap()
            else {
                panic!("native trajectory output is not an original member");
            };
            let quantity = product
                .admitted
                .case()
                .rows()
                .iter()
                .find(|row| row.id == *target)
                .unwrap()
                .quantity;
            let unit = original
                .physical_context()
                .quantities()
                .quantity_type(quantity)
                .unwrap()
                .canonical_unit;
            assert!(
                expected
                    .insert(
                        (*member, sample as i64),
                        (
                            trajectory.run_id(),
                            quantity.as_id(),
                            unit.as_id(),
                            point.time.to_bits(),
                            point.outputs[column].to_bits()
                        )
                    )
                    .is_none()
            );
        }
    }
    assert_eq!(
        expected.len(),
        times.len() * prepared.contract().outputs.len()
    );
    assert!(!expected.is_empty());
    // Challenge the receiving comparator with actual independently captured
    // native coordinates. None of these readable rows supplies live permission.
    let output_coordinates = expected
        .iter()
        .map(|((symbol, _), (_, quantity, unit, _, _))| {
            (
                pse_modeling::derived::Relationship::MemberOutput { symbol: *symbol }.identity(),
                (*quantity, *unit),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let ((symbol, sample), (run_id, quantity, unit, time, value)) =
        expected.first_key_value().unwrap();
    let original_row = pse_relations::generated::runtime::simulation_samples::Row {
        run_id: *run_id,
        sample: *sample,
        time: f64::from_bits(*time),
        symbol_id: *symbol,
        quantity_id: *quantity,
        unit_id: *unit,
        value: f64::from_bits(*value),
    };
    for ((symbol, sample), (run_id, quantity, unit, time, value)) in &expected {
        let selected = trajectory
            .qualified_output_at(&run, f64::from_bits(*time), *symbol)
            .unwrap()
            .unwrap();
        assert_eq!(selected.run_id, *run_id);
        assert_eq!(selected.symbol_id, *symbol);
        assert_eq!(selected.sample, *sample);
        assert_eq!(selected.quantity_id, *quantity);
        assert_eq!(selected.unit_id, *unit);
        assert_eq!(selected.time.to_bits(), *time);
        assert_eq!(selected.value.to_bits(), *value);
    }
    assert!(
        trajectory
            .qualified_output_at(&run, 5.125, *symbol)
            .unwrap()
            .is_none()
    );
    assert!(
        trajectory
            .qualified_output_at(&run, f64::NAN, *symbol)
            .unwrap()
            .is_none()
    );
    assert!(
        trajectory
            .qualified_output_at(
                &run,
                f64::from_bits(*time),
                pse_ids::named_id(*symbol, "foreign-output")
            )
            .unwrap()
            .is_none()
    );
    let matches = |row: &pse_relations::generated::runtime::simulation_samples::Row, observed| {
        crate::workflow::connected_results::trajectory_permission::sample_matches(
            row,
            *run_id,
            &times,
            observed,
            &output_coordinates,
        )
    };
    assert!(matches(&original_row, times.len()));
    for change in 0..7 {
        let mut foreign = original_row.clone();
        match change {
            0 => foreign.run_id = pse_operations::mint_id(),
            1 => foreign.symbol_id = pse_ids::named_id(*symbol, "foreign-original-output"),
            2 => foreign.quantity_id = pse_ids::named_id(*quantity, "foreign-original-quantity"),
            3 => foreign.unit_id = pse_ids::named_id(*unit, "foreign-original-unit"),
            4 => foreign.time += 0.125,
            5 => foreign.sample += 1,
            _ => foreign.value = f64::NAN,
        }
        assert!(
            !matches(&foreign, times.len()),
            "foreign coordinate {change}"
        );
    }
    assert!(!matches(&original_row, 0));
    let completion = run.completion().unwrap();
    let header = completion.computation.as_ref().unwrap();
    let context = prepared
        .scientific_context(run.run_id, header.profile_identity)
        .unwrap();
    let context_matches = |row| {
        use pse_relations::generated::runtime::scientific_contexts;
        let mut builder = scientific_contexts::Builder::with_registry(
            runtime.registry(),
            1,
            &runtime.validation_context(),
        )
        .unwrap();
        builder.push(row).unwrap();
        let checked = builder.finish().unwrap();
        let view = scientific_contexts::View::from_checked(&checked).unwrap();
        crate::workflow::connected_results::trajectory_permission::context_matches(
            &view,
            header,
            &completion.lineage[0],
        )
        .unwrap()
    };
    assert!(context_matches(context.clone()));
    for change in 0..11 {
        let mut foreign = context.clone();
        match change {
            0 => foreign.run_id = pse_operations::mint_id(),
            1 => foreign.step = 1,
            2 => foreign.instance_id = Some(pse_operations::mint_id()),
            3 => foreign.source_identity = pse_ids::ContentHash::from_bytes([11; 32]),
            4 => foreign.physical_identity = pse_ids::ContentHash::from_bytes([12; 32]),
            5 => foreign.preparation_identity = pse_ids::ContentHash::from_bytes([13; 32]),
            6 => foreign.profile_identity = pse_ids::ContentHash::from_bytes([14; 32]),
            7 => foreign.model_id = Some(pse_operations::mint_id()),
            8 => foreign.fit_id = Some(pse_operations::mint_id()),
            9 => foreign.case_id = Some(pse_operations::mint_id()),
            _ => foreign.time = None,
        }
        assert!(!context_matches(foreign), "foreign context {change}");
    }
    let row_count = expected.len() as u64;
    drop((run, handle, operation, original, package));
    runtime.clear_program_cache();

    // This reader acquires the original sealed run/attempt after producer handles drop.
    // Neither the selection observation nor its portable copy grants execution.
    let token = pse_columnar::CancellationToken::new();
    let mut reader = runtime
        .results(
            &run_key,
            &attempt_key,
            "runtime.simulation_samples",
            0,
            row_count,
            token.clone(),
        )
        .await
        .unwrap();
    let selection = reader.pack_selection().await.unwrap();
    assert_eq!((selection.row_start, selection.row_end), (0, row_count));
    // This is the original producer's whole-member decision, resolved against
    // the same protected attempt/context/coordinates after all live handles drop.
    assert_eq!(
        selection.usability,
        crate::workflow::PackScientificState::Usable
    );
    let stored = crate::workflow::durable::result_scientific_completion(
        runtime.canonical_store().unwrap(),
        &runtime.shared.pool(),
        reader.selection(),
    )
    .await
    .unwrap()
    .unwrap();
    let [original_permission] = stored
        .completion
        .as_ref()
        .unwrap()
        .member_permissions
        .as_slice()
    else {
        panic!("one original root trajectory member required");
    };
    assert_eq!(original_permission.attempt_id(), Some(stored.attempt_id));
    assert!(original_permission.recorded_permission().permits_result());
    assert!(!original_permission.has_native_proof() && !original_permission.permits_result());
    drop(stored);
    let partial = runtime
        .results(
            &run_key,
            &attempt_key,
            "runtime.simulation_samples",
            0,
            1,
            pse_columnar::CancellationToken::new(),
        )
        .await
        .unwrap();
    assert_eq!(
        partial.pack_selection().await.unwrap().usability,
        crate::workflow::PackScientificState::Partial
    );
    let projection = partial
        .inspection_projection(&[partial.schema().index_of("value").unwrap()])
        .unwrap();
    assert_eq!(
        partial
            .inspection_pack_selection(&projection)
            .await
            .unwrap()
            .usability,
        crate::workflow::PackScientificState::Partial
    );
    drop(partial);
    let context = selection.context.as_ref().unwrap();
    assert_eq!(context.problem, "aqueous-lifecycle-problem");
    assert_eq!(context.revision, original_revision);
    assert_eq!(context.manifest, reader.selection().manifest().digest);
    let directory = tempfile::tempdir().unwrap();
    let destination = directory.path().join("aqueous-original.arrow");
    reader
        .export_ipc_bounded(&destination, 8 << 20, &token)
        .await
        .unwrap();
    drop(reader);
    assert!(
        !directory
            .path()
            .join("aqueous-original.arrow.incomplete")
            .exists()
    );
    let artifact = std::fs::File::open(&destination).unwrap();
    let readable_bytes = artifact.metadata().unwrap().len();
    crate::workflow::portable_payload::validate_scientific_ipc(
        artifact,
        readable_bytes,
        8 << 20,
        &selection,
    )
    .unwrap();

    // Arrow and dynamic Protobuf read the actual protected export independently of
    // the generated native result adapter and generated relation row decoder.
    let mut portable =
        StreamReader::try_new(std::fs::File::open(&destination).unwrap(), None).unwrap();
    let schema = portable.schema();
    assert!(!schema.metadata().contains_key("pse.canonical.attempt"));
    let encoded = &schema.metadata()[crate::workflow::result_selection::SELECTION_METADATA_KEY];
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .unwrap();
    let world =
        Arc::new(DescriptorPool::decode(crate::recorded_result_projection::DESCRIPTORS).unwrap());
    let message = world
        .message_index("pse.engineering.v1.RecordedResultSelection")
        .unwrap();
    let observed = DynamicMessage::decode(world, message, &bytes).unwrap();
    for (tag, value) in [
        (1, run_key.as_str()),
        (2, attempt_key.as_str()),
        (3, "runtime.simulation_samples"),
    ] {
        assert_eq!(
            observed.field_by_number(tag),
            Some(&WireValue::String(value.into()))
        );
    }
    // Distinguish canonical wire absence from its descriptor-issued proto3 scalar default.
    let row_start = observed.message_descriptor().field(5).unwrap();
    assert!(observed.field_by_number(5).is_none());
    assert!(!observed.has(row_start));
    assert!(matches!(observed.get(row_start), WireValueRef::U64(0)));
    assert_eq!(
        observed.field_by_number(6),
        Some(&WireValue::U64(row_count))
    );
    let Some(WireValue::Message(usability)) = observed.field_by_number(10) else {
        panic!("original passive usability absent");
    };
    assert!(
        matches!(usability.field_by_number(2), Some(WireValue::Message(_))),
        "the original recorded Usable oneof alternative must survive copying"
    );
    for alternate in [1, 3, 4, 5] {
        assert!(
            usability.field_by_number(alternate).is_none(),
            "portable delivery must preserve exactly the original decision"
        );
    }
    let Some(WireValue::Message(observed_context)) = observed.field_by_number(11) else {
        panic!("original native context absent");
    };
    for (tag, value) in [
        (1, context.problem.as_str()),
        (2, context.revision.as_str()),
        (3, context.manifest.as_str()),
        (4, context.interpretation.as_str()),
    ] {
        assert_eq!(
            observed_context.field_by_number(tag),
            Some(&WireValue::String(value.into()))
        );
    }
    crate::workflow::portable_payload::validate_scientific_lineage(schema.as_ref(), &selection)
        .unwrap();
    let mut foreign = selection.clone();
    foreign.context.as_mut().unwrap().revision = "foreign-original-revision".into();
    assert!(
        crate::workflow::portable_payload::validate_scientific_lineage(schema.as_ref(), &foreign)
            .is_err()
    );

    let mut previous = None;
    for batch in &mut portable {
        let batch = batch.unwrap();
        assert!(
            batch
                .columns()
                .iter()
                .all(|column| column.null_count() == 0)
        );
        let ids = |name| {
            batch
                .column_by_name(name)
                .unwrap()
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .unwrap()
        };
        let sample = batch
            .column_by_name("sample")
            .unwrap()
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap();
        let time = batch
            .column_by_name("time")
            .unwrap()
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        let value = batch
            .column_by_name("value")
            .unwrap()
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        for index in 0..batch.num_rows() {
            let symbol = SemanticId::from_bytes(ids("symbol_id").value(index).try_into().unwrap());
            let key = (symbol, sample.value(index));
            assert!(
                previous.is_none_or(|before| before < key),
                "canonical output/sample order is original"
            );
            previous = Some(key);
            let original = expected
                .remove(&key)
                .expect("portable member/sample must have an actual native producer");
            assert_eq!(ids("run_id").value(index), original.0.as_bytes());
            assert_eq!(ids("quantity_id").value(index), original.1.as_bytes());
            assert_eq!(ids("unit_id").value(index), original.2.as_bytes());
            assert_eq!(time.value(index).to_bits(), original.3);
            assert_eq!(value.value(index).to_bits(), original.4);
        }
    }
    assert!(
        expected.is_empty(),
        "every original native member/sample must survive portable delivery"
    );
    assert!(
        portable.is_finished(),
        "independent portable read must consume the complete export"
    );
    drop(services);
    runtime.close();
    runtime.shared.math().shutdown_native_lanes().await.unwrap();
    let fixture = store.clone();
    drop(runtime);
    fixture.remove_isolated_fixture().await.unwrap();
}
