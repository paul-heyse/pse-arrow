// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Admission rejects malformed actual declarations independently of their identities.
use pse_ids::SemanticId;
use pse_quantity::basis::Basis;
use pse_quantity::conversion::ConversionRule;
use pse_quantity::kind::QuantityKind;
use pse_quantity::operation::QuantityOperation;
use pse_quantity::quantity_type::{QuantityType, QuantityTypeKey};
use pse_quantity::reference_state::ReferenceState;
use pse_quantity::registry::QuantityRegistryBuilder;
use pse_quantity::unit::{Unit, convert_spec, convert_value};
use pse_quantity::unit_set::UnitSet;
use pse_quantity::*;

fn raw(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn unit(n: u8, dimension: DimensionVector) -> Unit {
    Unit {
        id: UnitId::from_id(raw(n)),
        symbol: format!("u{n}"),
        dimension,
        scale_to_canonical: 1.0,
        offset_to_canonical: 0.0,
        is_affine: false,
        reference_state: None,
    }
}
fn kind(n: u8) -> QuantityKind {
    QuantityKind {
        id: QuantityKindId::from_id(raw(n)),
        dimension: DimensionVector::DIMENSIONLESS,
        extensive: false,
        addition_kind: QuantityAdditionKind::Additive,
    }
}
fn ty(n: u8) -> QuantityType {
    QuantityType {
        id: QuantityTypeId::from_id(raw(n)),
        key: QuantityTypeKey {
            kind: QuantityKindId::from_id(raw(1)),
            basis: None,
            reference_state: None,
            scale_kind: ScaleKind::Point,
            shape: vec![],
            subject_kind: None,
        },
        canonical_unit: UnitId::from_id(raw(1)),
        nominal_magnitude: None,
    }
}
fn builder() -> QuantityRegistryBuilder {
    let mut b = QuantityRegistryBuilder::new();
    b.unit(unit(1, DimensionVector::DIMENSIONLESS))
        .kind(kind(1))
        .quantity_type(ty(1))
        .neutral_dimensionless(QuantityTypeId::from_id(raw(1)));
    b
}
fn operation() -> QuantityOperation {
    QuantityOperation {
        id: OperationId::from_id(raw(1)),
        opcode: Opcode::Mul,
        input_kinds: vec![QuantityKindId::from_id(raw(1)); 2],
        result_kind: QuantityKindId::from_id(raw(1)),
        basis_rule: BasisRule::Preserve,
        reference_rule: ReferenceRule::Preserve,
        scale_rule: QuantityScaleRule::Preserve,
        shape_rule: QuantityShapeRule::Preserve,
        basis_source: Some(1),
        reference_source: Some(1),
        scale_source: Some(1),
        shape_source: Some(1),
        subject_rule: SubjectRule::Preserve,
        subject_source: Some(1),
        result_basis: None,
        result_reference_state: None,
        result_subject_kind: None,
        input_conversions: vec![],
        precondition_invariants: vec![],
    }
}
#[test]
fn complete_key_lookup_and_neutral_designation_use_actual_components() {
    let reg = builder().build().expect("valid registry");
    assert_eq!(reg.resolve_key(&ty(1).key).expect("exact key"), ty(1).id);
    assert!(
        reg.quantity_type(ty(1).id)
            .expect("type")
            .is_neutral_scalar(&reg)
    );
    let mut changed = ty(1).key;
    changed.subject_kind = Some(SubjectKind::Species);
    assert!(reg.resolve_key(&changed).is_err());
    assert_eq!(
        reg.unit_by_symbol("u1").expect("symbol").id,
        UnitId::from_id(raw(1))
    );
    assert!(reg.unit(UnitId::from_id(raw(99))).is_err());
    let mut no_binding = QuantityRegistryBuilder::new();
    no_binding
        .unit(unit(1, DimensionVector::DIMENSIONLESS))
        .kind(kind(1))
        .quantity_type(ty(1));
    assert_eq!(
        no_binding
            .build()
            .expect("valid without designation")
            .neutral_dimensionless(),
        None
    );
}
#[test]
fn duplicate_identity_symbol_and_semantic_key_are_rejected() {
    let mut b = builder();
    b.unit(unit(1, DimensionVector::DIMENSIONLESS));
    assert!(b.build().is_err());
    let mut b = builder();
    let mut other = unit(2, DimensionVector::DIMENSIONLESS);
    other.symbol = "u1".into();
    b.unit(other);
    assert!(b.build().is_err());
    let mut b = builder();
    b.quantity_type(ty(2));
    assert!(b.build().is_err());
}
#[test]
fn invalid_unit_numbers_and_flags_fail_before_conversion() {
    for scale in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let mut value = unit(2, DimensionVector::DIMENSIONLESS);
        value.scale_to_canonical = scale;
        assert!(
            convert_spec(
                &value,
                &unit(1, DimensionVector::DIMENSIONLESS),
                ScaleKind::Point
            )
            .is_err()
        );
        let mut b = builder();
        b.unit(value);
        assert!(b.build().is_err());
    }
    for offset in [1.0, f64::NAN, f64::INFINITY] {
        let mut value = unit(2, DimensionVector::DIMENSIONLESS);
        value.offset_to_canonical = offset;
        let mut b = builder();
        b.unit(value);
        assert!(b.build().is_err());
    }
}
#[test]
fn quantity_unit_nominal_and_reference_contracts_are_checked() {
    for change in 0..6 {
        let mut b = QuantityRegistryBuilder::new();
        let mut value = ty(1);
        b.unit(unit(1, DimensionVector::DIMENSIONLESS))
            .kind(kind(1));
        match change {
            0 => value.canonical_unit = UnitId::from_id(raw(99)),
            1 => value.key.basis = Some(BasisId::from_id(raw(99))),
            2 => value.key.reference_state = Some(ReferenceStateId::from_id(raw(99))),
            3 => value.nominal_magnitude = Some(-1.0),
            4 => value.nominal_magnitude = Some(f64::NAN),
            _ => value.key.scale_kind = ScaleKind::Difference,
        }
        b.quantity_type(value);
        assert!(b.build().is_err(), "case {change}");
    }
    let mut b = QuantityRegistryBuilder::new();
    b.unit(unit(1, DimensionVector::base(BaseDimension::Mass)))
        .kind(kind(1))
        .quantity_type(ty(1));
    assert!(b.build().is_err());
}
#[test]
fn neutral_binding_rejects_composition_obligations() {
    let mut b = QuantityRegistryBuilder::new();
    let mut value = ty(1);
    value.key.subject_kind = Some(SubjectKind::Species);
    b.unit(unit(1, DimensionVector::DIMENSIONLESS))
        .kind(kind(1))
        .quantity_type(value)
        .neutral_dimensionless(ty(1).id);
    assert!(b.build().is_err());
}
#[test]
fn reference_states_and_standard_volume_bases_are_admitted_together() {
    let reference = ReferenceState {
        id: ReferenceStateId::from_id(raw(1)),
        kind: ReferenceStateKind::Custom,
        temperature: Some(298.15),
        pressure: Some(101_325.0),
        include_enthalpy_of_formation: true,
        phase: None,
    };
    let basis = Basis {
        id: BasisId::from_id(raw(1)),
        kind: BasisKind::StandardVolume,
        composition_basis: None,
        rate_basis: None,
        reference_conditions: Some(reference.id),
    };
    let mut b = builder();
    b.reference_state(reference.clone()).basis(basis.clone());
    assert!(b.build().is_ok());
    let mut b = builder();
    b.basis(basis.clone());
    assert!(b.build().is_err());
    let mut b = builder();
    b.basis(Basis {
        reference_conditions: None,
        ..basis
    });
    assert!(b.build().is_err());
    let mut b = builder();
    b.reference_state(ReferenceState {
        pressure: Some(f64::NAN),
        ..reference
    });
    assert!(b.build().is_err());
}
#[test]
fn operation_sources_and_explicit_preconditions_are_checked() {
    let mut b = builder();
    b.operation(operation());
    let reg = b.build().expect("explicit sources");
    assert_eq!(reg.operations_for(Opcode::Mul).count(), 1);
    for source in [None, Some(2)] {
        let mut b = builder();
        b.operation(QuantityOperation {
            basis_source: source,
            ..operation()
        });
        assert!(b.build().is_err());
    }
    let mut declared = operation();
    declared.basis_rule = BasisRule::DeclaredResult;
    declared.basis_source = None;
    let mut b = builder();
    b.operation(declared.clone());
    assert!(b.build().is_err());
    declared
        .precondition_invariants
        .push(InvariantId::from_id(raw(1)));
    let mut b = builder();
    b.operation(declared);
    assert!(b.build().is_ok());
}
#[test]
fn conversion_declarations_require_coefficients_or_kernel_without_hidden_dependencies() {
    let rule = ConversionRule {
        id: ConversionId::from_id(raw(1)),
        from: ty(1).id,
        to: ty(1).id,
        kind: ConversionKind::Scale,
        kernel: None,
        required_parameters: vec![],
        scale: Some(1.0),
        offset: Some(0.0),
    };
    let mut b = builder();
    b.conversion(rule.clone());
    let reg = b.build().expect("scale declaration");
    assert_eq!(reg.conversions_between(ty(1).id, ty(1).id).count(), 1);
    for broken in [
        ConversionRule {
            offset: Some(1.0),
            ..rule.clone()
        },
        ConversionRule {
            scale: None,
            ..rule.clone()
        },
        ConversionRule {
            kernel: Some(raw(7)),
            ..rule.clone()
        },
    ] {
        let mut b = builder();
        b.conversion(broken);
        assert!(b.build().is_err());
    }
    let mut kernel = ConversionRule {
        kind: ConversionKind::Kernel,
        kernel: Some(raw(2)),
        required_parameters: vec!["mw".into()],
        scale: None,
        offset: None,
        ..rule
    };
    let mut b = builder();
    b.conversion(kernel.clone());
    assert!(b.build().is_ok());
    kernel.required_parameters.push("mw".into());
    let mut b = builder();
    b.conversion(kernel);
    assert!(b.build().is_err());
}
#[test]
fn affine_unit_conversions_work_in_both_directions_and_differences_ignore_offsets() {
    let kelvin = unit(1, DimensionVector::base(BaseDimension::Temperature));
    let celsius = Unit {
        id: UnitId::from_id(raw(2)),
        symbol: "degC".into(),
        offset_to_canonical: 273.15,
        is_affine: true,
        reference_state: None,
        ..kelvin.clone()
    };
    let c_to_k = convert_spec(&celsius, &kelvin, ScaleKind::Point).expect("C to K");
    let k_to_c = convert_spec(&kelvin, &celsius, ScaleKind::Point).expect("K to C");
    assert_eq!(convert_value(&c_to_k, 0.0).to_bits(), 273.15_f64.to_bits());
    assert_eq!(convert_value(&k_to_c, 273.15).to_bits(), 0.0_f64.to_bits());
    let delta = convert_spec(&celsius, &kelvin, ScaleKind::Difference).expect("difference");
    assert_eq!(convert_value(&delta, 10.0).to_bits(), 10.0_f64.to_bits());
    assert!(!c_to_k.same_contract(&delta));
    let f = Unit {
        id: UnitId::from_id(raw(3)),
        symbol: "degF".into(),
        scale_to_canonical: 5.0 / 9.0,
        offset_to_canonical: 273.15 - 32.0 * (5.0 / 9.0),
        is_affine: true,
        reference_state: None,
        ..kelvin
    };
    assert!(
        (convert_value(
            &convert_spec(&f, &celsius, ScaleKind::Point).expect("F to C"),
            212.0
        ) - 100.0)
            .abs()
            < 1e-12
    );
    let spec = UnitConvertSpec {
        from: celsius.id,
        to: f.id,
        scale: 1.0 + f64::EPSILON,
        offset: -1.0,
    };
    let value = 1.0 - f64::EPSILON;
    assert_ne!(
        convert_value(&spec, value).to_bits(),
        value.mul_add(spec.scale, spec.offset).to_bits()
    );
}
#[test]
fn unit_sets_validate_base_selection_and_derive_pressure() {
    let mut b = builder();
    let mut base = [None; 8];
    for d in BaseDimension::ALL.iter().take(5) {
        let u = unit(10 + d.ordinal(), DimensionVector::base(*d));
        base[usize::from(d.ordinal())] = Some(u.id);
        b.unit(u);
    }
    let reg = b.build().expect("base units");
    let set = UnitSet::new(UnitSetId::from_id(raw(1)), base, &reg).expect("SI");
    let pressure = DimensionVector::base(BaseDimension::Mass)
        .div(&DimensionVector::base(BaseDimension::Length))
        .expect("mass/length")
        .div(
            &DimensionVector::base(BaseDimension::Time)
                .pow(Ratio::new(2, 1).expect("two"))
                .expect("time squared"),
        )
        .expect("pressure");
    assert_eq!(
        set.derived_unit(pressure, &reg)
            .expect("pressure")
            .scale_to_canonical
            .to_bits(),
        1.0_f64.to_bits()
    );
    assert!(
        set.derived_unit(DimensionVector::base(BaseDimension::Currency), &reg)
            .is_err()
    );
    let mut wrong = base;
    wrong[0] = base[1];
    assert!(UnitSet::new(set.id, wrong, &reg).is_err());
    let mut missing = base;
    missing[0] = None;
    assert!(UnitSet::new(set.id, missing, &reg).is_err());
}

#[test]
fn neutral_singleton_and_scale_rule_coefficients_are_substantively_checked() {
    let mut b = builder();
    b.neutral_dimensionless(ty(1).id);
    assert!(b.build().is_err());
    let mut b = builder();
    b.conversion(ConversionRule {
        id: ConversionId::from_id(raw(1)),
        from: ty(1).id,
        to: ty(1).id,
        kind: ConversionKind::Scale,
        kernel: None,
        required_parameters: vec![],
        scale: Some(2.0),
        offset: None,
    });
    assert!(b.build().is_err());
}
