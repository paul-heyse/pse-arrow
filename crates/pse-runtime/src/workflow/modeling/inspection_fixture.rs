// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive bounded observation of immutable authored fixture inputs.
use super::*;
use crate::workflow::{WorkflowError, contract};
use pse_modeling::specialize::{Fixture, SpecializedModel};
use pse_quantity::{QuantityRegistry, QuantityTypeId, UnitId};

pse_contract::projection_struct! {
/// Original authored scalar specification, resolved by the native owner.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InspectionSpecification {
    projection: { name: "InspectionSpecification", rust_path: crate::workflow::InspectionSpecification, version: 1, direction: pse_contract::boundary_projection::Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Original authored model path.
    pub path: String => { tag: 1, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original resolved scalar identity.
    pub target: SemanticId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Complete original physical type, including transfer and reference roles.
    pub ty: pse_modeling::Type => { tag: 3, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Native resolved physical quantity.
    pub quantity_id: QuantityTypeId => { tag: 4, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Canonical unit of every numeric specification endpoint.
    pub unit_id: UnitId => { tag: 5, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original value; absence keeps the authored model value.
    pub value: Option<f64> => { tag: 6, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original fix/free selection; absence keeps the authored model role.
    pub fixed: Option<bool> => { tag: 7, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original lower override; absence keeps the authored bound.
    pub lower: Option<f64> => { tag: 8, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original upper override; absence keeps the authored bound.
    pub upper: Option<f64> => { tag: 9, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
}
}
pse_contract::projection_struct! {
/// Original authored quadrature tolerance, in the integral's canonical unit.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InspectionQuadrature {
    projection: { name: "InspectionQuadrature", rust_path: crate::workflow::InspectionQuadrature, version: 1, direction: pse_contract::boundary_projection::Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Original integral identity.
    pub target: SemanticId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Integral's physical quantity.
    pub quantity_id: QuantityTypeId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Canonical integral unit.
    pub unit_id: UnitId => { tag: 3, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original canonical absolute tolerance.
    pub absolute_tolerance: f64 => { tag: 4, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
}
}
pse_contract::projection_struct! {
/// Original bounds of a schedule declared free by the authored shooting fixture.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InspectionScheduleControl {
    projection: { name: "InspectionScheduleControl", rust_path: crate::workflow::InspectionScheduleControl, version: 1, direction: pse_contract::boundary_projection::Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Original lower bound; absent means unbounded.
    pub lower: Option<f64> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original upper bound; absent means unbounded.
    pub upper: Option<f64> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
}
}
pse_contract::projection_struct! {
/// One original interval, preserving schedule declaration and interval order.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InspectionScheduleInterval {
    projection: { name: "InspectionScheduleInterval", rust_path: crate::workflow::InspectionScheduleInterval, version: 1, direction: pse_contract::boundary_projection::Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Original zero-based schedule position.
    pub schedule: usize => { tag: 1, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original scheduled scalar.
    pub target: SemanticId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Target's physical quantity.
    pub quantity_id: QuantityTypeId => { tag: 3, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Canonical value and control-bound unit.
    pub unit_id: UnitId => { tag: 4, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original zero-based interval position.
    pub interval: usize => { tag: 5, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original change time in the axis unit; absent for the first interval at axis lower.
    pub change_time: Option<f64> => { tag: 6, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original authored canonical value, or shooting starting guess for a free control.
    pub value: f64 => { tag: 7, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original free-control selection; absence is a prescribed input.
    pub control: Option<InspectionScheduleControl> => { tag: 8, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
}
}
pse_contract::projection_struct! {
/// Original integration request; these observations are not applied solver settings.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InspectionIntegration {
    projection: { name: "InspectionIntegration", rust_path: crate::workflow::InspectionIntegration, version: 1, direction: pse_contract::boundary_projection::Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Native original integrated axis.
    pub axis: SemanticId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Physical time quantity of that axis.
    pub quantity_id: QuantityTypeId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Axis canonical unit for lower, upper, samples, step and schedule change times.
    pub unit_id: UnitId => { tag: 3, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original integrated domain lower endpoint; not inferred from sample zero.
    pub lower: f64 => { tag: 4, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original integrated domain upper endpoint.
    pub upper: f64 => { tag: 5, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original requested initial step.
    pub initial_step: f64 => { tag: 6, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original relative tolerance.
    pub relative_tolerance: f64 => { tag: 7, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original absolute tolerance in normalized state coordinates.
    pub normalized_absolute_tolerance: f64 => { tag: 8, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Original quadrature relative tolerance; absence preserves native default selection.
    pub quadrature_relative_tolerance: Option<f64> => { tag: 9, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Selected page of original requested samples, in their original order.
    #[schemars(transform = inspection_members_schema)]
    pub samples: Vec<f64> => { tag: 10, direction: pse_contract::boundary_projection::Direction::Output, rules: [INSPECTION_MEMBERS] };
    /// Selected page of original explicit quadrature tolerances.
    #[schemars(transform = inspection_members_schema)]
    pub quadratures: Vec<InspectionQuadrature> => { tag: 11, direction: pse_contract::boundary_projection::Direction::Output, rules: [INSPECTION_MEMBERS] };
    /// Selected page of original schedules, flattened by schedule then interval.
    #[schemars(transform = inspection_members_schema)]
    pub schedule_intervals: Vec<InspectionScheduleInterval> => { tag: 12, direction: pse_contract::boundary_projection::Direction::Output, rules: [INSPECTION_MEMBERS] };
}
}
pse_contract::projection_struct! {
/// Immutable authored/admitted-original fixture, independent of public execution overlays.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InspectionFixture {
    projection: { name: "InspectionFixture", rust_path: crate::workflow::InspectionFixture, version: 1, direction: pse_contract::boundary_projection::Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Original declaration supplying the fixture and its specifications.
    pub declaration: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Selected page of original scalar specifications, in native path-map order.
    #[schemars(transform = inspection_members_schema)]
    pub specifications: Vec<InspectionSpecification> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Output, rules: [INSPECTION_MEMBERS] };
    /// Original integration request; absence is distinct from a page with no samples.
    pub integration: Option<InspectionIntegration> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Selected page of original physical initial-equation identities; these are not starts.
    #[schemars(transform = inspection_members_schema)]
    pub initial_equations: Vec<SemanticId> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Output, rules: [INSPECTION_MEMBERS] };
}
}

fn physical(
    model: &SpecializedModel,
    target: SemanticId,
    quantities: &QuantityRegistry,
) -> Result<(QuantityTypeId, UnitId), WorkflowError> {
    let symbol = model
        .symbols
        .get(&target)
        .ok_or_else(|| contract("original inspection specification target absent"))?;
    let quantity = symbol
        .ty
        .quantity_scheme()
        .ok_or_else(|| contract("original inspection specification is not a physical scalar"))?
        .resolve(quantities, &BTreeMap::new())
        .map_err(|error| contract(error.to_string()))?;
    let unit = quantities
        .quantity_type(quantity)
        .map_err(|error| contract(error.to_string()))?
        .canonical_unit;
    Ok((quantity, unit))
}

pub(super) fn selected_extent(
    model: &SpecializedModel,
    fixture: Option<&Fixture>,
    offset: usize,
    limit: usize,
) -> Result<usize, WorkflowError> {
    let Some(fixture) = fixture else { return Ok(0) };
    let mut extent = 512usize;
    for (path, value) in fixture.specifications.iter().skip(offset).take(limit) {
        let symbol = model
            .symbols
            .get(&value.target)
            .ok_or_else(|| contract("original inspection specification target absent"))?;
        extent = extent
            .saturating_add(256)
            .saturating_add(path.len())
            .saturating_add(symbol.ty.retained_bytes());
        if extent > 64 << 10 {
            return Ok(extent);
        }
    }
    extent = extent.saturating_add(
        model
            .initial_equations
            .len()
            .saturating_sub(offset)
            .min(limit)
            .saturating_mul(32),
    );
    if let Some(integration) = &fixture.integration {
        extent = extent.saturating_add(
            integration
                .samples
                .len()
                .saturating_sub(offset)
                .min(limit)
                .saturating_mul(16),
        );
        extent = extent.saturating_add(
            integration
                .quadratures
                .len()
                .saturating_sub(offset)
                .min(limit)
                .saturating_mul(128),
        );
        let intervals = integration
            .schedules
            .iter()
            .map(|schedule| schedule.values.len())
            .sum::<usize>();
        extent = extent.saturating_add(
            intervals
                .saturating_sub(offset)
                .min(limit)
                .saturating_mul(256),
        );
    }
    Ok(extent)
}

pub(super) fn project(
    model: &SpecializedModel,
    fixture: &Fixture,
    quantities: &QuantityRegistry,
    offset: usize,
    limit: usize,
) -> Result<InspectionFixture, WorkflowError> {
    let specifications = fixture
        .specifications
        .iter()
        .skip(offset)
        .take(limit)
        .map(|(path, value)| {
            let (quantity_id, unit_id) = physical(model, value.target, quantities)?;
            Ok(InspectionSpecification {
                path: path.clone(),
                target: value.target,
                ty: model.symbols[&value.target].ty.clone(),
                quantity_id,
                unit_id,
                value: value.value,
                fixed: value.fixed,
                lower: value.lower,
                upper: value.upper,
            })
        })
        .collect::<Result<_, WorkflowError>>()?;
    let integration = fixture
        .integration
        .as_ref()
        .map(|original| {
            if model.integrated.len() != 1 {
                return Err(contract(
                    "original integration inspection requires its one admitted time axis",
                ));
            }
            let axis = model
                .integrated
                .values()
                .next()
                .ok_or_else(|| contract("original integration inspection time axis absent"))?;
            let unit_id = quantities
                .quantity_type(axis.quantity)
                .map_err(|error| contract(error.to_string()))?
                .canonical_unit;
            let quadratures = original
                .quadratures
                .iter()
                .skip(offset)
                .take(limit)
                .map(|(target, tolerance)| {
                    let (quantity_id, unit_id) = physical(model, *target, quantities)?;
                    Ok(InspectionQuadrature {
                        target: *target,
                        quantity_id,
                        unit_id,
                        absolute_tolerance: *tolerance,
                    })
                })
                .collect::<Result<_, WorkflowError>>()?;
            let schedule_intervals = original
                .schedules
                .iter()
                .enumerate()
                .flat_map(|(schedule_index, schedule)| {
                    schedule
                        .values
                        .iter()
                        .enumerate()
                        .map(move |(interval, value)| (schedule_index, schedule, interval, value))
                })
                .skip(offset)
                .take(limit)
                .map(|(schedule_index, schedule, interval, value)| {
                    let (quantity_id, unit_id) = physical(model, schedule.target, quantities)?;
                    Ok(InspectionScheduleInterval {
                        schedule: schedule_index,
                        target: schedule.target,
                        quantity_id,
                        unit_id,
                        interval,
                        change_time: interval.checked_sub(1).map(|index| schedule.times[index]),
                        value: *value,
                        control: schedule.control.map(|control| InspectionScheduleControl {
                            lower: control.lower,
                            upper: control.upper,
                        }),
                    })
                })
                .collect::<Result<_, WorkflowError>>()?;
            Ok(InspectionIntegration {
                axis: axis.id,
                quantity_id: axis.quantity,
                unit_id,
                lower: axis.lower,
                upper: axis.upper,
                initial_step: original.initial_step,
                relative_tolerance: original.relative_tolerance,
                normalized_absolute_tolerance: original.normalized_absolute_tolerance,
                quadrature_relative_tolerance: original.quadrature_relative_tolerance,
                samples: original
                    .samples
                    .iter()
                    .skip(offset)
                    .take(limit)
                    .copied()
                    .collect(),
                quadratures,
                schedule_intervals,
            })
        })
        .transpose()?;
    Ok(InspectionFixture {
        declaration: fixture.declaration,
        specifications,
        integration,
        initial_equations: model
            .initial_equations
            .iter()
            .skip(offset)
            .take(limit)
            .copied()
            .collect(),
    })
}
