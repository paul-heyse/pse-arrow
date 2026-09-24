// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Canonical framing of actual admitted declarations, never a caller's revision assertion.
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_quantity::{PhysicalPreconditions, QuantityRegistry};
fn id(h: &mut FramedHasher, v: Option<SemanticId>) {
    h.u64(u64::from(v.is_some()));
    if let Some(v) = v {
        h.id(&v);
    }
}
fn number(h: &mut FramedHasher, v: Option<f64>) {
    h.u64(u64::from(v.is_some()));
    if let Some(v) = v {
        h.u64(v.to_bits());
    }
}
fn word(h: &mut FramedHasher, v: Option<&str>) {
    h.u64(u64::from(v.is_some()));
    if let Some(v) = v {
        h.str(v);
    }
}
pub(crate) fn identity(r: &QuantityRegistry, p: &PhysicalPreconditions) -> ContentHash {
    let mut h = FramedHasher::new("pse.math.physical-inventory.v1");
    h.str("units").u64(r.units().len() as u64);
    for v in r.units() {
        h.id(&v.id.as_id())
            .str(&v.symbol)
            .part(&v.dimension.canonical_bytes())
            .u64(v.scale_to_canonical.to_bits())
            .u64(v.offset_to_canonical.to_bits())
            .u64(u64::from(v.is_affine));
        id(&mut h, v.reference_state.map(|x| x.as_id()));
    }
    h.str("kinds").u64(r.kinds().len() as u64);
    for v in r.kinds() {
        h.id(&v.id.as_id())
            .part(&v.dimension.canonical_bytes())
            .u64(u64::from(v.extensive))
            .str(v.addition_kind.as_str());
    }
    h.str("bases").u64(r.bases().len() as u64);
    for v in r.bases() {
        h.id(&v.id.as_id()).str(v.kind.as_str());
        word(&mut h, v.composition_basis.map(|x| x.as_str()));
        word(&mut h, v.rate_basis.map(|x| x.as_str()));
        id(&mut h, v.reference_conditions.map(|x| x.as_id()));
    }
    h.str("references").u64(r.reference_states().len() as u64);
    for v in r.reference_states() {
        h.id(&v.id.as_id())
            .str(v.kind.as_str())
            .u64(u64::from(v.include_enthalpy_of_formation));
        number(&mut h, v.temperature);
        number(&mut h, v.pressure);
        id(&mut h, v.phase);
    }
    h.str("quantities").u64(r.quantity_types().count() as u64);
    for v in r.quantity_types() {
        let k = &v.key;
        h.id(&v.id.as_id())
            .id(&k.kind.as_id())
            .id(&v.canonical_unit.as_id())
            .str(k.scale_kind.as_str());
        id(&mut h, k.basis.map(|x| x.as_id()));
        id(&mut h, k.reference_state.map(|x| x.as_id()));
        word(&mut h, k.subject_kind.map(|x| x.as_str()));
        number(&mut h, v.nominal_magnitude);
        h.u64(k.shape.len() as u64);
        for d in &k.shape {
            h.str(d.as_str());
        }
    }
    h.str("conversions").u64(r.conversions().len() as u64);
    for v in r.conversions() {
        h.id(&v.id.as_id())
            .id(&v.from.as_id())
            .id(&v.to.as_id())
            .str(v.kind.as_str());
        id(&mut h, v.kernel);
        number(&mut h, v.scale);
        number(&mut h, v.offset);
        h.u64(v.required_parameters.len() as u64);
        for n in &v.required_parameters {
            h.str(n);
        }
    }
    h.str("operations").u64(r.operations().len() as u64);
    for v in r.operations() {
        h.id(&v.id.as_id())
            .str(v.opcode.as_str())
            .id(&v.result_kind.as_id())
            .str(v.basis_rule.as_str())
            .str(v.reference_rule.as_str())
            .str(v.scale_rule.as_str())
            .str(v.shape_rule.as_str())
            .str(v.subject_rule.as_str());
        for n in [
            v.basis_source,
            v.reference_source,
            v.scale_source,
            v.shape_source,
            v.subject_source,
        ] {
            h.u64(n.map_or(u64::MAX, u64::from));
        }
        id(&mut h, v.result_basis.map(|x| x.as_id()));
        id(&mut h, v.result_reference_state.map(|x| x.as_id()));
        word(&mut h, v.result_subject_kind.map(|x| x.as_str()));
        h.u64(v.input_kinds.len() as u64);
        for n in &v.input_kinds {
            h.id(&n.as_id());
        }
        h.u64(v.input_conversions.len() as u64);
        for n in &v.input_conversions {
            h.u64(n.operand.into()).id(&n.conversion.as_id());
        }
        h.u64(v.precondition_invariants.len() as u64);
        for n in &v.precondition_invariants {
            h.id(&n.as_id());
        }
    }
    h.str("reductions");
    for (op, d) in r.reduction_domains() {
        h.id(&op.as_id()).str(d.as_str());
    }
    h.str("unit-sets").u64(r.unit_sets().len() as u64);
    for v in r.unit_sets() {
        h.id(&v.id.as_id());
        for n in v.base {
            id(&mut h, n.map(|x| x.as_id()));
        }
    }
    id(&mut h, r.neutral_dimensionless().map(|x| x.as_id()));
    h.str("preconditions").u64(p.declarations().len() as u64);
    for v in p.declarations() {
        h.id(&v.id.as_id()).u64(v.operand_positions.len() as u64);
        for n in &v.operand_positions {
            h.u64((*n).into());
        }
        match v.requirement {
            pse_quantity::PhysicalRequirement::EqualOperandBases { required } => {
                h.str("equal-bases");
                id(&mut h, required.map(|x| x.as_id()));
            }
            pse_quantity::PhysicalRequirement::OperandQuantityContract {
                required,
                match_shape,
            } => {
                h.str("quantity")
                    .id(&required.as_id())
                    .u64(u64::from(match_shape));
            }
        }
    }
    h.finish_hash()
}
