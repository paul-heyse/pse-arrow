// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A test double for the reference YAML units package (blueprint §8.2–§8.3).
//!
//! This fixture is never platform authority. Its currency years 2000/2001 use synthetic
//! CE indices 500/1000 solely to exercise scale ratios; they are not historical prices.
use crate::infer::{InvariantChecker, OpRequest, Operand};
use crate::{
    BaseDimension, Basis, BasisId, BasisKind, BasisRule, ConversionKind, ConversionRule,
    DimensionVector, InvariantId, Opcode, QuantityAdditionKind, QuantityError, QuantityKind,
    QuantityOperation, QuantityRegistry, QuantityRegistryBuilder, QuantityScaleRule,
    QuantityShapeRule, QuantityType, QuantityTypeKey, Ratio, ReferenceRule, ReferenceState,
    ReferenceStateId, ReferenceStateKind, ScaleKind, SubjectKind, SubjectRule, Unit, UnitSet,
};

/// Named-policy fixture identities; the qualified name is the declaration's stable key.
pub mod ids {
    use crate::{
        BasisId, ConversionId, InvariantId, OperationId, QuantityKindId, QuantityTypeId,
        ReferenceStateId, UnitId, UnitSetId,
    };
    use pse_ids::{SemanticId, named_id};
    fn named(family: &str, name: &str) -> SemanticId {
        let package = named_id(SemanticId::NIL, "pse.test.standard");
        named_id(package, &format!("{family}.{name}"))
    }
    macro_rules! identity {
        ($function:ident,$ty:ty) => {
            #[doc=concat!("Named identity of a fixture `",stringify!($function),"` declaration.")]
            pub fn $function(name: &str) -> $ty {
                <$ty>::from_id(named(stringify!($function), name))
            }
        };
    }
    identity!(unit, UnitId);
    identity!(kind, QuantityKindId);
    identity!(quantity, QuantityTypeId);
    identity!(basis, BasisId);
    identity!(reference, ReferenceStateId);
    identity!(operation, OperationId);
    identity!(invariant, InvariantId);
    identity!(conversion, ConversionId);
    identity!(unit_set, UnitSetId);
}
/// Assemble the fixture using the production admission path.
///
/// # Errors
/// Reports any invalid fixture declaration; no panic or unchecked registry is used.
pub fn standard_registry() -> Result<QuantityRegistry, QuantityError> {
    let mut b = QuantityRegistryBuilder::new();
    let dimensions = Dimensions::new()?;
    declare_units(&mut b, &dimensions);
    declare_types(&mut b, &dimensions);
    declare_operations(&mut b);
    b.neutral_dimensionless(ids::quantity("neutral"));
    b.unit_set(UnitSet {
        id: ids::unit_set("si"),
        base: [
            Some(ids::unit("m")),
            Some(ids::unit("kg")),
            Some(ids::unit("s")),
            Some(ids::unit("K")),
            Some(ids::unit("mol")),
            Some(ids::unit("A")),
            Some(ids::unit("cd")),
            Some(ids::unit("USD_2000")),
        ],
    });
    b.build()
}
struct Dimensions {
    energy: DimensionVector,
    molar_energy: DimensionVector,
    gas_constant: DimensionVector,
    pressure: DimensionVector,
    flow: DimensionVector,
    energy_flow: DimensionVector,
    molar_density: DimensionVector,
}
impl Dimensions {
    fn new() -> Result<Self, QuantityError> {
        let length = DimensionVector::base(BaseDimension::Length);
        let mass = DimensionVector::base(BaseDimension::Mass);
        let time = DimensionVector::base(BaseDimension::Time);
        let amount = DimensionVector::base(BaseDimension::Amount);
        let temperature = DimensionVector::base(BaseDimension::Temperature);
        let energy = mass
            .mul(&length.pow(Ratio::new(2, 1)?)?)?
            .div(&time.pow(Ratio::new(2, 1)?)?)?;
        let molar_energy = energy.div(&amount)?;
        Ok(Self {
            energy,
            molar_energy,
            gas_constant: molar_energy.div(&temperature)?,
            pressure: energy.div(&length.pow(Ratio::new(3, 1)?)?)?,
            flow: amount.div(&time)?,
            energy_flow: energy.div(&time)?,
            molar_density: amount.div(&length.pow(Ratio::new(3, 1)?)?)?,
        })
    }
}
fn add_unit(
    b: &mut QuantityRegistryBuilder,
    name: &str,
    dimension: DimensionVector,
    scale: f64,
    offset: f64,
    affine: bool,
) {
    b.unit(Unit {
        id: ids::unit(name),
        symbol: name.to_owned(),
        dimension,
        scale_to_canonical: scale,
        offset_to_canonical: offset,
        is_affine: affine,
        reference_state: None,
    });
}
fn declare_units(b: &mut QuantityRegistryBuilder, d: &Dimensions) {
    for (dimension, name) in [
        (BaseDimension::Length, "m"),
        (BaseDimension::Mass, "kg"),
        (BaseDimension::Time, "s"),
        (BaseDimension::Temperature, "K"),
        (BaseDimension::Amount, "mol"),
        (BaseDimension::Current, "A"),
        (BaseDimension::LuminousIntensity, "cd"),
        (BaseDimension::Currency, "USD_2000"),
    ] {
        add_unit(b, name, DimensionVector::base(dimension), 1.0, 0.0, false);
    }
    for (name, dimension) in [
        ("1", DimensionVector::DIMENSIONLESS),
        ("J", d.energy),
        ("J/mol", d.molar_energy),
        ("J/(mol*K)", d.gas_constant),
        ("Pa", d.pressure),
        ("mol/s", d.flow),
        ("W", d.energy_flow),
        ("mol/m3", d.molar_density),
    ] {
        add_unit(b, name, dimension, 1.0, 0.0, false);
    }
    let temperature = DimensionVector::base(BaseDimension::Temperature);
    add_unit(b, "degC", temperature, 1.0, 273.15, true);
    add_unit(
        b,
        "degF",
        temperature,
        5.0 / 9.0,
        273.15 - 32.0 * (5.0 / 9.0),
        true,
    );
    b.unit(Unit {
        id: ids::unit("psig"),
        symbol: "psig".to_owned(),
        dimension: d.pressure,
        scale_to_canonical: 6_894.757_293_168,
        offset_to_canonical: 0.0,
        is_affine: false,
        reference_state: Some(ids::reference("standard")),
    });
    add_unit(
        b,
        "USD_2001",
        DimensionVector::base(BaseDimension::Currency),
        2.0,
        0.0,
        false,
    );
}
#[expect(
    clippy::too_many_arguments,
    reason = "fixture helper declares the complete quantity tuple without defaults"
)]
fn add_type(
    b: &mut QuantityRegistryBuilder,
    name: &str,
    kind: &str,
    unit: &str,
    scale: ScaleKind,
    basis: Option<BasisId>,
    reference: Option<ReferenceStateId>,
    subject: Option<SubjectKind>,
) {
    b.quantity_type(QuantityType {
        id: ids::quantity(name),
        key: QuantityTypeKey {
            kind: ids::kind(kind),
            basis,
            reference_state: reference,
            scale_kind: scale,
            shape: vec![],
            subject_kind: subject,
        },
        canonical_unit: ids::unit(unit),
        nominal_magnitude: None,
    });
}
#[expect(
    clippy::too_many_lines,
    reason = "the fixture keeps coupled kind, basis, reference and type declarations together"
)]
fn declare_types(b: &mut QuantityRegistryBuilder, d: &Dimensions) {
    for (name, dimension, addition) in [
        (
            "neutral",
            DimensionVector::DIMENSIONLESS,
            QuantityAdditionKind::Additive,
        ),
        (
            "temperature",
            DimensionVector::base(BaseDimension::Temperature),
            QuantityAdditionKind::OriginSensitive,
        ),
        (
            "temperature_scale",
            DimensionVector::base(BaseDimension::Temperature),
            QuantityAdditionKind::Additive,
        ),
        (
            "pressure",
            d.pressure,
            QuantityAdditionKind::OriginSensitive,
        ),
        ("molar_flow", d.flow, QuantityAdditionKind::Additive),
        (
            "molar_enthalpy",
            d.molar_energy,
            QuantityAdditionKind::OriginSensitive,
        ),
        ("energy_flow", d.energy_flow, QuantityAdditionKind::Additive),
        (
            "mole_fraction",
            DimensionVector::DIMENSIONLESS,
            QuantityAdditionKind::Additive,
        ),
        ("component_flow", d.flow, QuantityAdditionKind::Additive),
        (
            "gas_constant",
            d.gas_constant,
            QuantityAdditionKind::Additive,
        ),
        (
            "molar_energy",
            d.molar_energy,
            QuantityAdditionKind::Additive,
        ),
        (
            "activation_energy",
            d.molar_energy,
            QuantityAdditionKind::Additive,
        ),
        (
            "molar_density",
            d.molar_density,
            QuantityAdditionKind::Additive,
        ),
        (
            "log_mole_fraction",
            DimensionVector::DIMENSIONLESS,
            QuantityAdditionKind::Additive,
        ),
    ] {
        b.kind(QuantityKind {
            id: ids::kind(name),
            dimension,
            extensive: matches!(name, "molar_flow" | "energy_flow" | "component_flow"),
            addition_kind: addition,
        });
    }
    b.reference_state(ReferenceState {
        id: ids::reference("standard"),
        kind: ReferenceStateKind::Custom,
        temperature: Some(298.15),
        pressure: Some(101_325.0),
        include_enthalpy_of_formation: false,
        phase: None,
    });
    b.basis(Basis {
        id: ids::basis("molar"),
        kind: BasisKind::Molar,
        composition_basis: None,
        rate_basis: None,
        reference_conditions: None,
    });
    let molar = Some(ids::basis("molar"));
    let datum = Some(ids::reference("standard"));
    let species = Some(SubjectKind::Species);
    for (name, kind, unit, scale, basis, reference, subject) in [
        (
            "neutral",
            "neutral",
            "1",
            ScaleKind::Point,
            None,
            None,
            None,
        ),
        (
            "temperature.point",
            "temperature",
            "K",
            ScaleKind::Point,
            None,
            None,
            None,
        ),
        (
            "temperature.difference",
            "temperature",
            "K",
            ScaleKind::Difference,
            None,
            None,
            None,
        ),
        (
            "temperature_scale",
            "temperature_scale",
            "K",
            ScaleKind::Point,
            None,
            None,
            None,
        ),
        (
            "pressure.absolute",
            "pressure",
            "Pa",
            ScaleKind::Point,
            None,
            None,
            None,
        ),
        (
            "pressure.gauge",
            "pressure",
            "Pa",
            ScaleKind::Point,
            None,
            datum,
            None,
        ),
        (
            "molar_flow",
            "molar_flow",
            "mol/s",
            ScaleKind::Point,
            molar,
            None,
            None,
        ),
        (
            "molar_enthalpy.point",
            "molar_enthalpy",
            "J/mol",
            ScaleKind::Point,
            molar,
            datum,
            None,
        ),
        (
            "molar_enthalpy.difference",
            "molar_enthalpy",
            "J/mol",
            ScaleKind::Difference,
            molar,
            datum,
            None,
        ),
        (
            "energy_flow",
            "energy_flow",
            "W",
            ScaleKind::Point,
            None,
            datum,
            None,
        ),
        (
            "mole_fraction",
            "mole_fraction",
            "1",
            ScaleKind::Point,
            molar,
            None,
            species,
        ),
        (
            "component_flow",
            "component_flow",
            "mol/s",
            ScaleKind::Point,
            molar,
            None,
            species,
        ),
        (
            "gas_constant",
            "gas_constant",
            "J/(mol*K)",
            ScaleKind::Point,
            molar,
            None,
            None,
        ),
        (
            "molar_energy",
            "molar_energy",
            "J/mol",
            ScaleKind::Point,
            molar,
            None,
            None,
        ),
        (
            "activation_energy",
            "activation_energy",
            "J/mol",
            ScaleKind::Point,
            molar,
            None,
            None,
        ),
        (
            "molar_density",
            "molar_density",
            "mol/m3",
            ScaleKind::Point,
            molar,
            None,
            None,
        ),
        (
            "log_mole_fraction",
            "log_mole_fraction",
            "1",
            ScaleKind::Point,
            molar,
            None,
            species,
        ),
    ] {
        add_type(b, name, kind, unit, scale, basis, reference, subject);
    }
    b.conversion(ConversionRule {
        id: ids::conversion("gauge_to_absolute"),
        from: ids::quantity("pressure.gauge"),
        to: ids::quantity("pressure.absolute"),
        kind: ConversionKind::Affine,
        kernel: None,
        required_parameters: vec![],
        scale: Some(1.0),
        offset: Some(101_325.0),
    });
}
fn operation(name: &str, opcode: Opcode, inputs: &[&str], result: &str) -> QuantityOperation {
    QuantityOperation {
        id: ids::operation(name),
        opcode,
        input_kinds: inputs.iter().map(|name| ids::kind(name)).collect(),
        result_kind: ids::kind(result),
        basis_rule: BasisRule::RequireEqual,
        reference_rule: ReferenceRule::RequireEqual,
        scale_rule: QuantityScaleRule::Point,
        shape_rule: QuantityShapeRule::SameIndices,
        basis_source: None,
        reference_source: None,
        scale_source: None,
        shape_source: None,
        subject_rule: SubjectRule::RequireEqual,
        subject_source: None,
        result_subject_kind: None,
        result_basis: None,
        result_reference_state: None,
        input_conversions: vec![],
        precondition_invariants: vec![],
    }
}
fn declare_operations(b: &mut QuantityRegistryBuilder) {
    let mut enthalpy = operation(
        "flow_enthalpy",
        Opcode::Mul,
        &["molar_flow", "molar_enthalpy"],
        "energy_flow",
    );
    enthalpy.basis_rule = BasisRule::Cancel;
    enthalpy.reference_rule = ReferenceRule::Preserve;
    enthalpy.reference_source = Some(1);
    enthalpy
        .precondition_invariants
        .push(ids::invariant("matching_molar_basis"));
    b.operation(enthalpy);
    let mut component = operation(
        "flow_composition",
        Opcode::Mul,
        &["molar_flow", "mole_fraction"],
        "component_flow",
    );
    component.basis_rule = BasisRule::Preserve;
    component.basis_source = Some(0);
    component.subject_rule = SubjectRule::Preserve;
    component.subject_source = Some(1);
    b.operation(component);
    let mut rt = operation(
        "gas_constant_temperature",
        Opcode::Mul,
        &["gas_constant", "temperature"],
        "molar_energy",
    );
    rt.basis_rule = BasisRule::Preserve;
    rt.basis_source = Some(0);
    b.operation(rt);
    let mut density = operation(
        "pressure_molar_energy",
        Opcode::Div,
        &["pressure", "molar_energy"],
        "molar_density",
    );
    density.basis_rule = BasisRule::Preserve;
    density.basis_source = Some(1);
    b.operation(density);
    b.operation(operation(
        "temperature_scale",
        Opcode::Div,
        &["temperature", "temperature_scale"],
        "neutral",
    ));
    let mut activation = operation(
        "activation_ratio",
        Opcode::Div,
        &["activation_energy", "molar_energy"],
        "neutral",
    );
    activation.basis_rule = BasisRule::Cancel;
    activation
        .precondition_invariants
        .push(ids::invariant("matching_molar_basis"));
    b.operation(activation);
    b.operation(operation(
        "log_mole_fraction",
        Opcode::Log,
        &["mole_fraction"],
        "log_mole_fraction",
    ));
    for opcode in [
        Opcode::Exp,
        Opcode::Log,
        Opcode::Log10,
        Opcode::Sin,
        Opcode::Cos,
        Opcode::Tan,
        Opcode::Asin,
        Opcode::Acos,
        Opcode::Atan,
        Opcode::Sinh,
        Opcode::Cosh,
        Opcode::Tanh,
        Opcode::Erf,
        Opcode::Sqrt,
        Opcode::SafeSqrt,
        Opcode::SafeLog,
    ] {
        b.operation(operation(
            &format!("{}.neutral", opcode.as_str()),
            opcode,
            &["neutral"],
            "neutral",
        ));
    }
}
/// Fixture checker that establishes the one composition invariant from actual contracts.
/// It cannot certify weighted means because no actual weight values were supplied.
#[derive(Debug)]
pub struct StandardInvariantChecker;
impl InvariantChecker for StandardInvariantChecker {
    fn check(
        &self,
        id: InvariantId,
        request: &OpRequest<'_>,
        operation: Option<&QuantityOperation>,
        operands: &[Operand<'_>],
        registry: &QuantityRegistry,
    ) -> Result<(), QuantityError> {
        let supported = operation.is_some_and(|rule| {
            [
                ids::operation("flow_enthalpy"),
                ids::operation("activation_ratio"),
            ]
            .contains(&rule.id)
        });
        if id != ids::invariant("matching_molar_basis")
            || !supported
            || !matches!(request, OpRequest::Mul | OpRequest::Div)
            || operands.len() != 2
        {
            return Err(QuantityError::InferencePrecondition {
                rule: "fixture.invariant_scope",
                detail: "request is outside the invariant's declared composition scope".to_owned(),
            });
        }
        for operand in operands {
            if registry.quantity_type(operand.quantity_type)?.key.basis != Some(ids::basis("molar"))
            {
                return Err(QuantityError::InferencePrecondition {
                    rule: "fixture.matching_molar_basis",
                    detail: "actual operand basis is not the declared molar basis".to_owned(),
                });
            }
        }
        Ok(())
    }
}
