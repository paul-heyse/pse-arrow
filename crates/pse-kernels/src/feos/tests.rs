// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    clippy::many_single_char_names,
    clippy::too_many_lines,
    reason = "compact numerical finite-difference fixtures"
)]
use super::*;
use pse_quantity::*;
use std::sync::atomic::AtomicBool;

fn fixture() -> (QuantityRegistry, FeosPorts) {
    let id = |n| SemanticId::from_bytes([n; 16]);
    let reference = ReferenceStateId::from_id(id(80));
    let entropy_reference = ReferenceStateId::from_id(id(83));
    let basis = BasisId::from_id(id(81));
    let composition = BasisId::from_id(id(82));
    let mut b = QuantityRegistryBuilder::new();
    b.reference_state(ReferenceState {
        id: reference,
        kind: ReferenceStateKind::Custom,
        temperature: Some(298.15),
        pressure: None,
        include_enthalpy_of_formation: false,
        phase: None,
    });
    b.reference_state(ReferenceState {
        id: entropy_reference,
        kind: ReferenceStateKind::Custom,
        temperature: Some(298.15),
        pressure: Some(100_000.0),
        include_enthalpy_of_formation: false,
        phase: None,
    });
    b.basis(Basis {
        id: basis,
        kind: BasisKind::Molar,
        composition_basis: None,
        rate_basis: None,
        reference_conditions: None,
    });
    b.basis(Basis {
        id: composition,
        kind: BasisKind::Molar,
        composition_basis: Some(CompositionBasis::MoleFraction),
        rate_basis: None,
        reference_conditions: None,
    });
    let dims = [
        [0, 0, 0, 1, 0, 0, 0, 0],
        [-3, 0, 0, 0, 1, 0, 0, 0],
        [0; 8],
        [0; 8],
        [-1, 1, -2, 0, 0, 0, 0, 0],
        [2, 1, -2, 0, -1, 0, 0, 0],
        [2, 1, -2, -1, -1, 0, 0, 0],
        [0; 8],
        [0; 8],
        [0; 8],
    ];
    let ports: Vec<_> = dims
        .into_iter()
        .enumerate()
        .map(|(i, d)| {
            let n = u8::try_from(i + 1).unwrap();
            let role = match i {
                3 => 2,
                8 | 9 => 7,
                _ => i,
            };
            let r = u8::try_from(role + 1).unwrap();
            let u = UnitId::from_id(id(r));
            let q = QuantityTypeId::from_id(id(r + 20));
            let k = QuantityKindId::from_id(id(u8::try_from(role + 41).unwrap()));
            let dimension = DimensionVector::new(d.map(|e| Ratio::new(e, 1).unwrap()));
            if i == role {
                b.unit(Unit {
                    id: u,
                    symbol: format!("si-{n}"),
                    dimension,
                    scale_to_canonical: 1.0,
                    offset_to_canonical: 0.0,
                    is_affine: false,
                    reference_state: None,
                });
                b.kind(QuantityKind {
                    id: k,
                    dimension,
                    extensive: false,
                    addition_kind: QuantityAdditionKind::OriginSensitive,
                });
                b.quantity_type(QuantityType {
                    id: q,
                    key: QuantityTypeKey {
                        kind: k,
                        basis: if i == 2 || i == 3 {
                            Some(composition)
                        } else {
                            ((i == 5) || (i == 6)).then_some(basis)
                        },
                        reference_state: match i {
                            5 => Some(reference),
                            6 => Some(entropy_reference),
                            _ => None,
                        },
                        scale_kind: ScaleKind::Point,
                        shape: vec![],
                        subject_kind: None,
                    },
                    canonical_unit: u,
                    nominal_magnitude: None,
                });
            }
            Port {
                id: id(n + 60),
                quantity: q,
                unit: u,
            }
        })
        .collect();
    let range = |l, u| crate::envelope::Interval::new(l, u).unwrap();
    let bindings = FeosPorts {
        envelope: crate::envelope::StateEnvelope {
            temperature: range(250.0, 500.0),
            density: range(0.0, 25000.0),
            pressure: range(0.0, 1e8),
            composition: vec![range(0.0, 1.0); 3],
            provenance: "declared unit-test window; empirical validity unestablished".into(),
        },
        inputs: ports[..4].to_vec(),
        outputs: ports[4..].to_vec(),
        enthalpy_reference: reference,
        entropy_reference,
        components: ["74-82-8", "74-84-0", "74-98-6"]
            .into_iter()
            .enumerate()
            .map(|(i, cas)| ComponentBinding {
                species: id(u8::try_from(90 + i).unwrap()),
                pcsaft_cas: cas.into(),
                ideal_gas_cas: cas.into(),
            })
            .collect(),
        dependent_species: id(92),
        kinds: FeosKinds {
            temperature: QuantityKindId::from_id(id(41)),
            density: QuantityKindId::from_id(id(42)),
            fraction: QuantityKindId::from_id(id(43)),
            pressure: QuantityKindId::from_id(id(45)),
            enthalpy: QuantityKindId::from_id(id(46)),
            entropy: QuantityKindId::from_id(id(47)),
            ln_fugacity: QuantityKindId::from_id(id(48)),
        },
        data: FeosData::light_hydrocarbons(),
        formulation: ThermodynamicFormulation::HomogeneousDensity,
        stability: StabilityPolicy::Unchecked,
    };
    (b.build().unwrap(), bindings)
}

#[test]
fn mixture_outputs_partials_and_one_state_per_demand() {
    let (registry, ports) = fixture();
    let package = FeosPackage::new(ports, &registry).unwrap();
    let mut worker = package.worker();
    let cancel = AtomicBool::new(false);
    let context = EvaluationContext {
        cancelled: &cancel,
        max_result_bytes: 4096,
    };
    let x = [350.0, 34.565_566_349_336_066, 0.2, 0.3];
    let request = ProviderRequest::all(worker.spec(), DerivativeOrder::Second);
    let v = worker.evaluate(&x, &request, &context).unwrap();
    assert!((v.values[0] - 1e5).abs() < 1e-6);
    assert!((v.values[1] - 3_240.889_128_547_157_3).abs() < 1e-8);
    assert_eq!(worker.state_evaluations(), 1);
    assert_eq!(worker.evaluate(&x, &request, &context).unwrap(), v);
    assert_eq!(worker.state_evaluations(), 1);
    let selected = ProviderRequest {
        outputs: vec![5, 0],
        order: DerivativeOrder::First,
    };
    let projection = worker.evaluate(&x, &selected, &context).unwrap();
    assert_eq!(projection.values, vec![v.values[5], v.values[0]]);
    assert_eq!(&projection.jacobian[..4], &v.jacobian[20..24]);
    assert!(projection.hessians.is_empty());
    assert_eq!(worker.state_evaluations(), 1);
    for output in 0..6 {
        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (v.hessians[output * 16 + i * 4 + j] - v.hessians[output * 16 + j * 4 + i])
                        .abs()
                        < 1e-8
                );
            }
        }
    }
    for coordinate in 0..4 {
        for step in [1e-4, 3e-5] {
            let h = step * x[coordinate].abs().max(1.0);
            let mut left = x;
            let mut right = x;
            left[coordinate] -= h;
            right[coordinate] += h;
            let first = ProviderRequest::all(worker.spec(), DerivativeOrder::First);
            let a = worker.evaluate(&left, &first, &context).unwrap();
            let b = worker.evaluate(&right, &first, &context).unwrap();
            for output in 0..6 {
                let derivative = (b.values[output] - a.values[output]) / (2.0 * h);
                let expected = v.jacobian[output * 4 + coordinate];
                assert!((derivative - expected).abs() < 2e-5 * expected.abs().max(1.0));
                for j in 0..4 {
                    let derivative =
                        (b.jacobian[output * 4 + j] - a.jacobian[output * 4 + j]) / (2.0 * h);
                    let expected = v.hessians[output * 16 + j * 4 + coordinate];
                    assert!((derivative - expected).abs() < 3e-4 * expected.abs().max(1.0));
                }
            }
        }
    }
}

#[test]
fn failures_clear_cache_and_workers_are_independent() {
    let (registry, ports) = fixture();
    let package = FeosPackage::new(ports, &registry).unwrap();
    let mut a = package.worker();
    let mut b = package.worker();
    let cancel = AtomicBool::new(false);
    let context = EvaluationContext {
        cancelled: &cancel,
        max_result_bytes: 4096,
    };
    let request = ProviderRequest {
        outputs: vec![1, 0],
        order: DerivativeOrder::Value,
    };
    let x = [350.0, 34.5, 0.2, 0.3];
    a.evaluate(&x, &request, &context).unwrap();
    for bad in [
        [0.0, 34.5, 0.2, 0.3],
        [350.0, -1.0, 0.2, 0.3],
        [350.0, 34.5, 0.8, 0.3],
        [350.0, 34.5, 0.0, 0.3],
    ] {
        assert!(matches!(
            a.evaluate(&bad, &request, &context),
            Err(ProviderError::Trial(_))
        ));
    }
    a.evaluate(&x, &request, &context).unwrap();
    assert_eq!(a.state_evaluations(), 2);
    assert_eq!(b.state_evaluations(), 0);
    assert_eq!(
        a.evaluate(&x, &request, &context).unwrap(),
        b.evaluate(&x, &request, &context).unwrap()
    );
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(matches!(
        a.evaluate(&x, &request, &context),
        Err(ProviderError::Cancelled)
    ));
    cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    let tiny = EvaluationContext {
        max_result_bytes: 1,
        ..context
    };
    assert!(matches!(
        a.evaluate(&x, &request, &tiny),
        Err(ProviderError::Limit(_))
    ));
    let bad = ProviderRequest {
        outputs: vec![0, 0],
        order: DerivativeOrder::Value,
    };
    assert!(a.evaluate(&x, &bad, &context).is_err());
}

#[test]
fn data_identity_contract_and_npt_initialization_are_explicit() {
    let (registry, ports) = fixture();
    let package = FeosPackage::new(ports.clone(), &registry).unwrap();
    let registration = crate::Registration::new(Arc::new(package.clone()), &registry).unwrap();
    assert_eq!(registration.spec().key(), package.spec.key());
    let mut changed = ports.clone();
    changed.outputs.swap(0, 1);
    assert!(FeosPackage::new(changed, &registry).is_err());
    let mut changed = ports;
    changed.components[1] = changed.components[0].clone();
    assert!(FeosPackage::new(changed, &registry).is_err());
    let cancel = AtomicBool::new(false);
    let context = EvaluationContext {
        cancelled: &cancel,
        max_result_bytes: 4096,
    };
    let state = package
        .initialize_npt(
            350.0,
            1e5,
            &[0.2, 0.3],
            InitialPhase::Vapor,
            false,
            &context,
        )
        .unwrap();
    assert!((state.molar_density - 34.565_566_349_336_066).abs() < 1e-10);
    assert_eq!(state.global_stability.status, StabilityStatus::NotRequested);
    assert!(
        package
            .initialize_npt(
                350.0,
                -1.0,
                &[0.2, 0.3],
                InitialPhase::Vapor,
                false,
                &context
            )
            .is_err()
    );
}

/// Frozen values come from independent teqp PC-SAFT plus Decimal DIPPR integrals.
/// Execution belongs to the M22 component campaign, not the mechanism unit slice.
#[test]
fn independent_pcsaft_reference_values_and_caloric_increments() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/plan14/thermo-reference.json"
    ))
    .unwrap();
    assert_eq!(reference["generator"]["library"], "teqp");
    assert_eq!(reference["generator"]["version"], "0.23.1");
    let (registry, ports) = fixture();
    let package = FeosPackage::new(ports, &registry).unwrap();
    let mut worker = package.worker();
    let cancel = AtomicBool::new(false);
    let context = EvaluationContext {
        cancelled: &cancel,
        max_result_bytes: 4096,
    };
    let request = ProviderRequest::all(worker.spec(), DerivativeOrder::Second);
    let tolerance = &reference["tolerances"];
    let near = |actual: f64, expected: f64, absolute: &str| {
        let limit = tolerance[absolute].as_f64().unwrap()
            + tolerance["relative"].as_f64().unwrap() * expected.abs();
        assert!(
            actual.is_finite() && (actual - expected).abs() <= limit,
            "actual {actual}, independent {expected}, tolerance {limit}"
        );
    };
    for case in reference["cases"].as_array().unwrap() {
        let input: Vec<_> = case["input"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let result = worker.evaluate(&input, &request, &context).unwrap();
        near(
            result.values[0],
            case["pressure"].as_f64().unwrap(),
            "pressure_absolute",
        );
        near(
            result.values[1],
            case["enthalpy"].as_f64().unwrap(),
            "enthalpy_absolute",
        );
        for i in 0..3 {
            near(
                result.values[3 + i],
                case["ln_phi"][i].as_f64().unwrap(),
                "ln_phi_absolute",
            );
        }
        assert!(
            result
                .jacobian
                .iter()
                .chain(&result.hessians)
                .all(|x| x.is_finite())
        );
    }
    assert_eq!(worker.state_evaluations(), 5);
    // Stability is an explicit library calculation, separate from density guessing.
    let initialized = package
        .initialize_npt(350.0, 1e5, &[0.2, 0.3], InitialPhase::Vapor, true, &context)
        .unwrap();
    assert_eq!(initialized.global_stability.status, StabilityStatus::Stable);
    assert!((initialized.molar_density - 34.565_566_349_336_066).abs() < 1e-10);
}

#[test]
fn declared_envelopes_bind_identity_and_reject_unrequested_pressure() {
    let (q, mut ports) = fixture();
    ports.envelope.composition = vec![crate::envelope::Interval::new(1e-6, 1. - 1e-6).unwrap(); 3];
    let first = FeosPackage::new(ports.clone(), &q).unwrap();
    ports.envelope.provenance.push_str(" revised");
    let second = FeosPackage::new(ports.clone(), &q).unwrap();
    assert_ne!(first.spec().identity(), second.spec().identity());
    let cancel = AtomicBool::new(false);
    let context = EvaluationContext {
        cancelled: &cancel,
        max_result_bytes: 4096,
    };
    let request = ProviderRequest {
        outputs: vec![1],
        order: DerivativeOrder::Value,
    };
    let good = [350., 34.565_566_349_336_066, 0.2, 0.3];
    let mut worker = first.worker();
    worker.evaluate(&good, &request, &context).unwrap();
    let mut bad = good;
    bad[0] = 501.;
    assert!(
        matches!(worker.evaluate(&bad,&request,&context),Err(ProviderError::OutsideEnvelope{axis,..}) if axis=="temperature")
    );
    worker.evaluate(&good, &request, &context).unwrap();
    for (axis, mut trial) in [
        ("density", good),
        ("composition[0]", good),
        ("composition[1]", good),
        ("composition[2]", good),
    ] {
        match axis {
            "density" => trial[1] = 25001.,
            "composition[0]" => trial[2] = 1e-8,
            "composition[1]" => trial[3] = 1e-8,
            _ => {
                trial[2] = 0.5;
                trial[3] = 0.5 - 1e-8;
            }
        }
        assert!(
            matches!(worker.evaluate(&trial,&request,&context),Err(ProviderError::OutsideEnvelope{axis: found,..}) if found == axis)
        );
        worker.evaluate(&good, &request, &context).unwrap();
    }
    assert!(
        matches!(first.initialize_npt(350., 1e8+1., &[0.2,0.3], InitialPhase::Vapor, false, &context),Err(ProviderError::OutsideEnvelope{axis,..}) if axis == "pressure")
    );
    let mut narrow = ports.clone();
    narrow.envelope.density = crate::envelope::Interval::new(1., 2.).unwrap();
    assert!(
        matches!(FeosPackage::new(narrow,&q).unwrap().initialize_npt(350.,1e5,&[0.2,0.3],InitialPhase::Vapor,false,&context),Err(ProviderError::OutsideEnvelope{axis,..}) if axis == "density")
    );
    ports.envelope.pressure = crate::envelope::Interval::new(0., 100.).unwrap();
    let tight = FeosPackage::new(ports, &q).unwrap();
    assert!(
        matches!(tight.worker().evaluate(&good,&request,&context),Err(ProviderError::OutsideEnvelope{axis,..}) if axis=="pressure")
    );
}

#[test]
fn declared_entropy_datum_agrees_with_independent_residuals() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/thermo-entropy-reference.json"
    ))
    .unwrap();
    let (registry, ports) = fixture();
    let cancel = AtomicBool::new(false);
    let context = EvaluationContext {
        cancelled: &cancel,
        max_result_bytes: 4096,
    };
    let offset: f64 = reference["feos_raw_minus_reference_j_per_mol_k"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let request = ProviderRequest {
        outputs: vec![2],
        order: DerivativeOrder::Value,
    };
    for case in reference["cases"].as_array().unwrap().iter() {
        let x: Vec<_> = case["input"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let cas: Vec<_> = case["cas"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let mut worker = FeosPackage::new(select_components(ports.clone(), &cas), &registry)
            .unwrap()
            .worker();
        let raw = worker.evaluate(&x, &request, &context).unwrap().values[0];
        let independent = case["entropy"].as_f64().unwrap();
        assert!(
            (raw - independent).abs() < 2e-7,
            "raw {raw}, reference {independent}, derived offset {offset}"
        );
    }
}

fn select_components(mut ports: FeosPorts, cas: &[&str]) -> FeosPorts {
    let old = ports.components.clone();
    ports.components = cas
        .iter()
        .map(|cas| old.iter().find(|c| c.pcsaft_cas == *cas).unwrap().clone())
        .collect();
    ports.dependent_species = ports.components.last().unwrap().species;
    ports.inputs.truncate(2);
    for i in 1..cas.len() {
        ports.inputs.push(Port {
            id: named_id(SemanticId::NIL, &format!("coordinate{i}")),
            quantity: QuantityTypeId::from_id(SemanticId::from_bytes([23; 16])),
            unit: UnitId::from_id(SemanticId::from_bytes([3; 16])),
        });
    }
    ports.outputs.truncate(3);
    for i in 0..cas.len() {
        ports.outputs.push(Port {
            id: named_id(SemanticId::NIL, &format!("phi{i}")),
            quantity: QuantityTypeId::from_id(SemanticId::from_bytes([28; 16])),
            unit: UnitId::from_id(SemanticId::from_bytes([8; 16])),
        });
    }
    ports.envelope.composition = vec![crate::envelope::Interval::new(0.0, 1.0).unwrap(); cas.len()];
    ports
}
#[test]
fn explicit_records_permutation_and_wide_derivative_blocks() {
    let (q, ports) = fixture();
    let cancel = AtomicBool::new(false);
    let context = EvaluationContext {
        cancelled: &cancel,
        max_result_bytes: 16384,
    };
    let request = ProviderRequest {
        outputs: vec![0, 1, 2, 3, 4, 5],
        order: DerivativeOrder::Second,
    };
    let mut original = FeosPackage::new(ports.clone(), &q).unwrap().worker();
    let a = original
        .evaluate(&[350., 30., 0.2, 0.3], &request, &context)
        .unwrap();
    let p = select_components(ports.clone(), &["74-98-6", "74-82-8", "74-84-0"]);
    let mut permuted = FeosPackage::new(p, &q).unwrap().worker();
    let b = permuted
        .evaluate(&[350., 30., 0.5, 0.2], &request, &context)
        .unwrap();
    for i in 0..3 {
        assert!((a.values[i] - b.values[i]).abs() < 1e-9);
    }
    for (i, j) in [(3, 4), (4, 5), (5, 3)] {
        assert!((a.values[i] - b.values[j]).abs() < 1e-12);
    }
    let mut bad = ports.clone();
    bad.components[0].pcsaft_cas = "unknown".into();
    assert!(FeosPackage::new(bad, &q).is_err());
    let mut bad = ports.clone();
    bad.data.missing_interactions = MissingInteractionPolicy::RequireExplicit;
    assert!(FeosPackage::new(bad, &q).is_err());
    let mut bad = ports.clone();
    bad.kinds.enthalpy = bad.kinds.entropy;
    assert!(FeosPackage::new(bad, &q).is_err());
    let mut bad = ports.clone();
    bad.formulation = ThermodynamicFormulation::PhaseEquilibrium;
    assert!(FeosPackage::new(bad, &q).is_err());
    // A fourth synthetic identity sharing methane parameters exercises width, not empirical validity.
    let mut wide = ports;
    for text in [&mut wide.data.pcsaft, &mut wide.data.ideal_gas] {
        let mut records: Vec<serde_json::Value> = serde_json::from_str(text).unwrap();
        let mut extra = records[0].clone();
        extra["identifier"]["cas"] = "synthetic-width-control".into();
        records.push(extra);
        *text = serde_json::to_string(&records).unwrap();
    }
    wide.components.push(ComponentBinding {
        species: SemanticId::from_bytes([93; 16]),
        pcsaft_cas: "synthetic-width-control".into(),
        ideal_gas_cas: "synthetic-width-control".into(),
    });
    wide = select_components(
        wide,
        &["74-82-8", "74-84-0", "74-98-6", "synthetic-width-control"],
    );
    let mut worker = FeosPackage::new(wide, &q).unwrap().worker();
    let request = ProviderRequest {
        outputs: vec![0, 1, 2, 6],
        order: DerivativeOrder::Second,
    };
    let x = [350., 30., 0.2, 0.3, 0.1];
    let v = worker.evaluate(&x, &request, &context).unwrap();
    assert_eq!(worker.state_evaluations(), 3);
    for axis in 0..5 {
        let h = x[axis].abs().max(1.) * 1e-5;
        let mut left = x;
        let mut right = x;
        left[axis] -= h;
        right[axis] += h;
        let first = ProviderRequest {
            order: DerivativeOrder::First,
            ..request.clone()
        };
        let a = worker.evaluate(&left, &first, &context).unwrap();
        let b = worker.evaluate(&right, &first, &context).unwrap();
        for output in 0..4 {
            for j in 0..5 {
                let expected = v.hessians[output * 25 + j * 5 + axis];
                let actual = (b.jacobian[output * 5 + j] - a.jacobian[output * 5 + j]) / (2. * h);
                assert!(
                    (actual - expected).abs() < 4e-4 * expected.abs().max(1.),
                    "axis {axis}, {j}: {actual} vs {expected}"
                );
            }
        }
    }
}
#[test]
fn declared_stability_policy_distinguishes_mechanical_global_and_failure() {
    let stable = StabilityObservation {
        status: StabilityStatus::Stable,
        detail: None,
    };
    let unstable = StabilityObservation {
        status: StabilityStatus::Unstable,
        detail: None,
    };
    let failed = StabilityObservation {
        status: StabilityStatus::Failed,
        detail: Some("iteration did not converge".into()),
    };
    assert!(enforce_stability(StabilityPolicy::Unchecked, -1., &unstable).is_ok());
    assert!(enforce_stability(StabilityPolicy::Mechanical, 1., &unstable).is_ok());
    assert!(enforce_stability(StabilityPolicy::Mechanical, -1., &stable).is_err());
    assert!(enforce_stability(StabilityPolicy::Global, 1., &unstable).is_err());
    assert!(
        enforce_stability(StabilityPolicy::Global, 1., &failed)
            .unwrap_err()
            .to_string()
            .contains("failed")
    );
    assert!(enforce_stability(StabilityPolicy::Global, 1., &stable).is_ok());
}

#[test]
fn same_dimension_internal_energy_cannot_bind_enthalpy_role() {
    let (q, mut ports) = fixture();
    let mut builder = q.to_builder();
    let mut kind = q.kind(ports.kinds.enthalpy).unwrap().clone();
    kind.id = SemanticId::from_bytes([140; 16]).into();
    builder.kind(kind.clone());
    let mut quantity = q.quantity_type(ports.outputs[1].quantity).unwrap().clone();
    quantity.id = SemanticId::from_bytes([141; 16]).into();
    quantity.key.kind = kind.id;
    builder.quantity_type(quantity.clone());
    ports.outputs[1].quantity = quantity.id;
    assert!(matches!(
        FeosPackage::new(ports, &builder.build().unwrap()),
        Err(ProviderError::Contract(_))
    ));
}

#[test]
fn mechanical_requirement_changes_admissibility_only_when_selected() {
    let (q, ports) = fixture();
    let unrestricted = FeosPackage::new(ports.clone(), &q).unwrap();
    let mut strict = ports;
    strict.stability = StabilityPolicy::Mechanical;
    let strict = FeosPackage::new(strict, &q).unwrap();
    assert_ne!(unrestricted.spec().identity(), strict.spec().identity());
    let cancel = AtomicBool::new(false);
    let context = EvaluationContext {
        cancelled: &cancel,
        max_result_bytes: 4096,
    };
    let request = ProviderRequest {
        outputs: vec![0],
        order: DerivativeOrder::First,
    };
    let mut a = unrestricted.worker();
    let mut b = strict.worker();
    let mut witness = None;
    for temperature in [250., 275., 300.] {
        for density in [500., 1000., 2000., 3000., 4000., 5000., 6000., 8000.] {
            let x = [temperature, density, 0.2, 0.3];
            if let Ok(value) = a.evaluate(&x, &request, &context)
                && value.jacobian[1] < 0.0
            {
                witness = Some(x);
                break;
            }
        }
    }
    let x = witness.expect("positive-pressure metastable-region control");
    assert!(matches!(
        b.evaluate(&x, &request, &context),
        Err(ProviderError::Trial(_))
    ));
    assert!(a.evaluate(&x, &request, &context).is_ok());
}
