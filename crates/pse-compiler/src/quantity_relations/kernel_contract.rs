// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete declared kernel metadata is admitted independently of execution availability.
use super::invalid;
use crate::CompilerError;
use pse_ids::SemanticId;
use pse_quantity::{QuantityRegistry, QuantityTypeId, UnitId};
use pse_relations::generated::reference;
use std::collections::BTreeSet;

#[expect(
    clippy::too_many_lines,
    reason = "check_descriptor keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
pub(crate) fn check_descriptor(
    descriptor: &reference::kernel_specs::Row,
    physical: &QuantityRegistry,
) -> Result<(), CompilerError> {
    if descriptor.name.is_empty()
        || descriptor.version.is_empty()
        || descriptor.provider.is_empty()
        || descriptor.test_suite.is_empty()
    {
        return Err(invalid(
            "kernel descriptor lacks identity/version/provider/conformance contract",
        ));
    }
    let inputs = descriptor.inputs.iter().map(|port| {
        (
            &port.name,
            port.quantity_type_id,
            port.natural_unit_id,
            &port.shape,
        )
    });
    let outputs = descriptor.outputs.iter().map(|port| {
        (
            &port.name,
            port.quantity_type_id,
            port.natural_unit_id,
            &port.shape,
        )
    });
    let parameters = descriptor.parameters.iter().map(|port| {
        (
            &port.name,
            port.quantity_type_id,
            port.natural_unit_id,
            &port.indexed_by,
        )
    });
    for ports in [
        inputs.collect::<Vec<_>>(),
        outputs.collect(),
        parameters.collect(),
    ] {
        let mut names = BTreeSet::new();
        for (name, ty, unit, shape) in ports {
            if name.is_empty() || !names.insert(name) {
                return Err(invalid("kernel descriptor port name is empty/repeated"));
            }
            let quantity = physical.quantity_type(QuantityTypeId::from_id(ty))?;
            if quantity
                .key
                .shape
                .iter()
                .map(|kind| kind.as_str())
                .ne(shape.iter().map(|kind| kind.as_str()))
            {
                return Err(invalid(
                    "kernel descriptor shape differs from complete port quantity",
                ));
            }
            pse_quantity::convert_spec_for_type(
                physical.unit(UnitId::from_id(unit))?,
                physical.unit(quantity.canonical_unit)?,
                &quantity.key,
            )?;
        }
    }
    let mut derivatives = BTreeSet::new();
    for derivative in &descriptor.derivative_bindings {
        if derivative.implementation_id == SemanticId::NIL || !derivatives.insert(derivative.kind) {
            return Err(invalid(
                "kernel derivative binding identity absent or kind repeated",
            ));
        }
        let mut invariants = BTreeSet::new();
        if derivative
            .validity_invariant_ids
            .iter()
            .any(|id| *id == SemanticId::NIL || !invariants.insert(*id))
        {
            return Err(invalid(
                "kernel derivative validity inventory is empty-valued/repeated",
            ));
        }
    }
    let mut forms = BTreeSet::new();
    for form in &descriptor.execution_forms {
        if form.implementation_id == SemanticId::NIL || !forms.insert(form.form) {
            return Err(invalid("kernel execution form identity absent or repeated"));
        }
    }
    let mut backends = BTreeSet::new();
    for binding in &descriptor.bindings {
        if binding.implementation_id == SemanticId::NIL
            || binding.conformance_suite.is_empty()
            || !backends.insert(binding.backend)
        {
            return Err(invalid("kernel backend declaration incomplete or repeated"));
        }
    }
    for bound in &descriptor.validity {
        if !descriptor
            .inputs
            .iter()
            .any(|input| input.name == bound.input)
        {
            return Err(invalid("kernel validity names an undeclared input"));
        }
        if let (Some(lower), Some(upper)) = (bound.lower.value, bound.upper.value)
            && lower > upper
        {
            return Err(invalid("kernel validity lower bound exceeds upper bound"));
        }
    }
    for monotonicity in &descriptor.monotonicity {
        if !descriptor
            .inputs
            .iter()
            .any(|input| input.name == monotonicity.input)
        {
            return Err(invalid("kernel monotonicity names an undeclared input"));
        }
    }
    if descriptor
        .bindings
        .iter()
        .any(|binding| binding.backend.as_str() == "nl_external_function")
        && descriptor
            .nl_function_name
            .as_ref()
            .is_none_or(String::is_empty)
    {
        return Err(invalid(
            "NL kernel binding has no actual exported function name",
        ));
    }
    Ok(())
}

pub(crate) fn scalar_parameter(
    parameter: &reference::method_parameters::Row,
    port: &reference::kernel_specs::ReferenceKernelSpecsFieldParametersItem,
    value: &pse_relations::generated::normalized::parameter_values::Row,
    quantities: &QuantityRegistry,
) -> Result<f64, CompilerError> {
    if parameter.name != port.name
        || !parameter.indexed_by.is_empty()
        || !port.indexed_by.is_empty()
        || !value.index.is_empty()
    {
        return Err(invalid(
            "kernel scalar parameter differs from the exact name/axis contract",
        ));
    }
    pse_quantity::admission::require_same_contract(
        QuantityTypeId::from_id(parameter.quantity_type_id),
        QuantityTypeId::from_id(port.quantity_type_id),
        quantities,
    )?;
    let quantity = quantities.quantity_type(QuantityTypeId::from_id(port.quantity_type_id))?;
    if !quantity.key.shape.is_empty() {
        return Err(invalid(
            "scalar kernel parameter has an indexed quantity contract",
        ));
    }
    let conversion = pse_quantity::convert_spec_for_type(
        quantities.unit(UnitId::from_id(value.unit_id))?,
        quantities.unit(UnitId::from_id(port.natural_unit_id))?,
        &quantity.key,
    )?;
    let number = pse_quantity::convert_value(&conversion, value.value);
    if !value.value.is_finite() || !number.is_finite() {
        return Err(invalid("kernel parameter source/conversion is nonfinite"));
    }
    Ok(number)
}
