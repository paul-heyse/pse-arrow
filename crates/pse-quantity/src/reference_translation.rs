// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Scientific datum translation with composition and paired reference anchors.
use crate::{CanonicalConversionPlan, CanonicalMagnitude, QuantityError, QuantityRegistry, QuantityTypeId, ScaleKind};
use pse_ids::SemanticId;
use std::collections::BTreeMap;

/// One scientific value at explicit physical reference conditions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScientificAnchor {
    /// Canonical source or target point value.
    pub value: CanonicalMagnitude,
    /// Actual common reference temperature.
    pub temperature: CanonicalMagnitude,
    /// Actual common reference pressure.
    pub pressure: CanonicalMagnitude,
    /// Scientific declaration/source identities; never an implicit default.
    pub provenance: Vec<SemanticId>,
}

/// The admitted transformation between two complete datum contracts.
/// Its anchors are evaluated at the same physical state for each actual component.
#[derive(Clone, Debug)]
pub struct ReferenceTranslation {
    source: CanonicalConversionPlan,
    target: CanonicalConversionPlan,
    scale: f64,
    anchors: BTreeMap<SemanticId, (ScientificAnchor, ScientificAnchor)>,
}
impl ReferenceTranslation {
    /// Admit the contextual contract before scientific anchor functions are evaluated.
    /// Mathematical composition retains those actual function calls and their provenance.
    /// # Errors
    /// Incompatible datum points, incomplete membership/provenance or invalid conditions.
    pub fn admit_context(registry:&QuantityRegistry,source:QuantityTypeId,target:QuantityTypeId,temperature:CanonicalMagnitude,pressure:CanonicalMagnitude,members:&[SemanticId],provenance:&[SemanticId])->Result<(),QuantityError> {
        let a=registry.quantity_type(source)?;let b=registry.quantity_type(target)?;
        let mut expected=a.key.clone();expected.reference_state=b.key.reference_state;
        if expected!=b.key || a.key.scale_kind!=ScaleKind::Point || a.key.reference_state.is_none() || b.key.reference_state.is_none() || members.is_empty() || members.iter().collect::<std::collections::BTreeSet<_>>().len()!=members.len() || members.contains(&SemanticId::NIL) || provenance.is_empty() || provenance.contains(&SemanticId::NIL) {
            return Err(refusal("translation requires compatible datum points, distinct actual members and scientific provenance"));
        }
        for (condition,is_temperature) in [(temperature,true),(pressure,false)] {
            let contract=registry.quantity_type(condition.quantity())?;
            let declared=registry.reference_states().flat_map(|reference|if is_temperature {reference.temperature}else{reference.pressure}).any(|value|value.quantity_type==condition.quantity());
            if !declared || contract.key.reference_state.is_some() || contract.key.scale_kind!=ScaleKind::Point || condition.value()<=0.0 {return Err(refusal("reference anchors require positive absolute temperature and pressure"));}
        }
        Ok(())
    }
    /// Admit one selected scientific mapping; units cannot change its basis or subject.
    /// # Errors
    /// Non-point/incompatible contracts, empty context, missing provenance, mismatched
    /// anchor contracts or different physical reference conditions.
    pub fn admit(
        registry: &QuantityRegistry,
        source: QuantityTypeId,
        target: QuantityTypeId,
        anchors: BTreeMap<SemanticId, (ScientificAnchor, ScientificAnchor)>,
    ) -> Result<Self, QuantityError> {
        let a = registry.quantity_type(source)?;
        let b = registry.quantity_type(target)?;
        let mut expected = a.key.clone();
        expected.reference_state = b.key.reference_state;
        if expected != b.key || a.key.scale_kind != ScaleKind::Point
            || a.key.reference_state.is_none() || b.key.reference_state.is_none()
            || anchors.is_empty()
        { return Err(refusal("translation requires compatible referenced point contracts and an explicit composition context")); }
        let common=anchors.values().next().map(|(first,_)|(first.temperature,first.pressure));
        if let Some((temperature,pressure))=common {
            let provenance=anchors.values().flat_map(|(a,b)|a.provenance.iter().chain(&b.provenance).copied()).collect::<Vec<_>>();
            Self::admit_context(registry,source,target,temperature,pressure,&anchors.keys().copied().collect::<Vec<_>>(),&provenance)?;
        }
        for (first, second) in anchors.values() {
            if first.value.quantity() != source || second.value.quantity() != target
                || first.temperature != second.temperature || first.pressure != second.pressure
                || first.provenance.is_empty() || second.provenance.is_empty()
                || first.provenance.iter().chain(&second.provenance).any(|id| *id == SemanticId::NIL)
                || common!=Some((first.temperature,first.pressure))
            { return Err(refusal("paired anchors require source/target contracts, identical physical conditions and explicit provenance")); }
        }
        let scale = registry.unit(a.canonical_unit)?.scale_to_canonical / registry.unit(b.canonical_unit)?.scale_to_canonical;
        if !scale.is_finite() || scale <= 0.0 { return Err(refusal("reference translation canonical scale is invalid")); }
        Ok(Self { source: CanonicalConversionPlan::canonical(registry, source)?, target: CanonicalConversionPlan::canonical(registry, target)?, scale, anchors })
    }
    /// Source contract, before the explicit datum change.
    pub const fn source(&self) -> QuantityTypeId { self.source.quantity() }
    /// Target contract, after the explicit datum change.
    pub const fn target(&self) -> QuantityTypeId { self.target.quantity() }
    /// Representation-only multiplier; the contextual anchor offset remains separate.
    pub const fn scale(&self) -> f64 { self.scale }
    /// Exact admitted component anchor pairs, retaining conditions and provenance.
    pub fn anchors(&self) -> &BTreeMap<SemanticId, (ScientificAnchor, ScientificAnchor)> { &self.anchors }
    /// Translate with explicit nonnegative composition weights, normalized by their total.
    /// Missing/extra species cannot be silently dropped. Runtime lowering uses the same
    /// weighted source/target anchor expression so composition derivatives remain visible.
    /// # Errors
    /// Wrong value contract, different component context, invalid weights, or overflow.
    pub fn apply(&self, value: CanonicalMagnitude, composition: &BTreeMap<SemanticId, f64>) -> Result<CanonicalMagnitude, QuantityError> {
        if value.quantity() != self.source() || !composition.keys().eq(self.anchors.keys()) {
            return Err(refusal("reference translation requires the exact source and component context"));
        }
        let (mut total, mut source, mut target) = (0.0, 0.0, 0.0);
        for (member, weight) in composition {
            if !weight.is_finite() || *weight < 0.0 { return Err(refusal("composition weights must be finite and nonnegative")); }
            let (a, b) = &self.anchors[member];
            total += weight;
            source += weight * a.value.value();
            target += weight * b.value.value();
        }
        if !total.is_finite() || total <= 0.0 || !source.is_finite() || !target.is_finite() {
            return Err(refusal("composition total or anchor accumulation is invalid"));
        }
        let translated = (value.value() - source / total) * self.scale + target / total;
        self.target.apply(translated)
    }
}
fn refusal(detail: &str) -> QuantityError {
    QuantityError::InferencePrecondition { rule: "reference.translation", detail: detail.into() }
}
