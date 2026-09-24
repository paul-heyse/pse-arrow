// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Semantic body identity and instance binding are independent of library handles.
use crate::MathError;
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_kernels::Port;
use pse_quantity::{
    QuantityRegistry, UnitConvertSpec, admission::require_same_contract, convert_spec_for_type,
    convert_value,
};
use std::collections::{BTreeMap, BTreeSet};

/// Compiler-owned interpretation; external hashes cannot select numerical behavior.
pub fn guarded_real_policy() -> ContentHash {
    pse_ids::derive_hash(
        "pse.math.guarded-real.v1",
        &[b"real-algebra;physical-first;lazy-domain;exact-library-derivatives"],
    )
}

/// Reuse identity inputs. Ordinary case values and instance identities are excluded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodySpec {
    /// Canonical typed source definition content.
    pub definition: ContentHash,
    /// Consumed finite shape/filter/specialization facts only.
    pub structure: ContentHash,
    /// Complete physical interpretation, including registry declarations.
    pub physical: ContentHash,
    /// Consumed provider contracts, phases and parameter data in stable full-key order.
    pub providers: Vec<ContentHash>,
    /// Real algebra, domain, smoothness and numerical profile.
    pub policy: ContentHash,
}
impl BodySpec {
    /// Versioned key; never hashes printed atoms or process-global library identifiers.
    pub fn key(&self) -> ContentHash {
        let mut h = FramedHasher::new("pse.math.body.v1");
        h.hash(&self.definition)
            .hash(&self.structure)
            .hash(&self.physical)
            .hash(&self.policy)
            .u64(self.providers.len() as u64);
        for provider in &self.providers {
            h.hash(provider);
        }
        h.finish_hash()
    }
}

/// A finite semantic domain in deterministic member order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FiniteDomain {
    /// Domain identity.
    pub id: SemanticId,
    members: Vec<SemanticId>,
}
impl FiniteDomain {
    /// Admit a set, refusing duplicates before canonical ordering.
    /// # Errors
    /// Duplicate members or an exceeded nonzero cardinality bound.
    pub fn new(
        id: SemanticId,
        mut members: Vec<SemanticId>,
        limit: usize,
    ) -> Result<Self, MathError> {
        if limit == 0 || members.len() > limit {
            return Err(MathError::Limit("finite domain cardinality"));
        }
        members.sort_unstable();
        if members.windows(2).any(|w| w[0] == w[1]) {
            return Err(MathError::Contract("duplicate domain member".into()));
        }
        Ok(Self { id, members })
    }
    /// Canonical semantic member order, independent of ingestion order.
    pub fn members(&self) -> &[SemanticId] {
        &self.members
    }
    /// Membership against the admitted finite set.
    pub fn contains(&self, member: SemanticId) -> bool {
        self.members.binary_search(&member).is_ok()
    }
}

/// A variable's structural declaration; its current numerical value lives separately.
#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    /// Global semantic identity and physical contract.
    pub port: Port,
    /// Fixed/free affects the variable layout, not a reusable arithmetic body.
    pub fixed: bool,
    /// Explicit variable domain; integer admission never follows from a numeric value.
    pub domain: VariableDomain,
    /// Declared closed lower bound, if finite.
    pub lower: Option<f64>,
    /// Declared closed upper bound, if finite.
    pub upper: Option<f64>,
}

/// Declared decision domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariableDomain {
    /// Real-valued decision.
    Continuous,
    /// Integer-valued decision.
    Integer,
    /// Decision restricted to zero or one.
    Binary,
    /// Zero or a real value in the declared positive interval.
    SemiContinuous,
    /// Zero or an integer value in the declared positive interval.
    SemiInteger,
}
impl VariableDomain {
    /// Integrality applies only to these declared domains.
    pub const fn is_integer(self) -> bool {
        matches!(self, Self::Integer | Self::Binary | Self::SemiInteger)
    }
    /// A semi-variable has a separate zero branch, outside its positive interval.
    pub const fn is_semi(self) -> bool {
        matches!(self, Self::SemiContinuous | Self::SemiInteger)
    }
    /// Exact domain membership for fixed values; no solver tolerance is involved.
    pub fn contains(self, value: f64, lower: f64, upper: f64) -> bool {
        value.is_finite()
            && (self.is_semi() && value == 0.0
                || value >= lower
                    && value <= upper
                    && (!self.is_integer() || value.fract() == 0.0)
                    && (self != Self::Binary || value == 0.0 || value == 1.0))
    }
}
/// Objective orientation retained independently from solver normalization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectiveSense {
    /// Minimize the authored objective.
    Minimize,
    /// Maximize the authored objective.
    Maximize,
}
impl ObjectiveSense {
    /// Multiplier for a minimization oracle.
    pub const fn sign(self) -> f64 {
        match self {
            Self::Minimize => 1.0,
            Self::Maximize => -1.0,
        }
    }
}
/// Complete selected constraint inventory, including isolated rows.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    /// Stable semantic identity.
    pub id: SemanticId,
    /// Canonical physical output contract.
    /// Complete canonical physical contract.
    pub quantity: pse_quantity::QuantityTypeId,
    /// Closed canonical bounds; outward infinities are allowed.
    pub lower: f64,
    /// Closed upper bound; positive infinity is allowed.
    pub upper: f64,
}
/// Explicit scalar objective declaration.
#[derive(Clone, Debug, PartialEq)]
pub struct Objective {
    /// Complete canonical physical contract.
    pub quantity: pse_quantity::QuantityTypeId,
    /// Authored optimization orientation.
    pub sense: ObjectiveSense,
}
/// An output can contribute to a selected constraint or the objective.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// Contribute to a selected constraint row.
    Row(SemanticId),
    /// Contribute to the selected objective.
    Objective,
}
/// One nonzero, dimensionless entry of the row contribution map A.
#[derive(Clone, Debug, PartialEq)]
pub struct Contribution {
    /// Zero-based body output ordinal.
    pub output: usize,
    /// Selected destination.
    pub target: Target,
    /// Finite nonzero dimensionless contribution weight.
    pub scale: f64,
}

/// Case values cannot mutate membership, physical interpretation or fixed/free status.
#[derive(Clone, Debug, Default)]
pub struct CaseValues {
    /// Global scalar values by semantic identity, in declared representation units.
    pub scalars: BTreeMap<SemanticId, f64>,
}

/// One physically checked formal binding. Repeated sources are intentional aliases.
#[derive(Clone, Debug, PartialEq)]
pub struct SlotBinding {
    /// Global variable/parameter identity.
    source: SemanticId,
    conversion: UnitConvertSpec,
    port: Port,
}
impl SlotBinding {
    /// Admit representation conversion without changing kind, basis, datum or scale.
    /// # Errors
    /// Physical mismatch or a composition-dependent conversion masquerading as units.
    pub fn new(
        source: &Port,
        formal: &Port,
        registry: &QuantityRegistry,
    ) -> Result<Self, MathError> {
        require_same_contract(formal.quantity, source.quantity, registry)?;
        let ty = registry.quantity_type(source.quantity)?;
        if formal.unit != registry.quantity_type(formal.quantity)?.canonical_unit {
            return Err(MathError::Contract(
                "body formal coordinates must use their canonical representation".into(),
            ));
        }
        if !ty.key.shape.is_empty() {
            return Err(MathError::Contract(
                "instance slots require scalar physical contracts".into(),
            ));
        }
        let canonical = registry.unit(ty.canonical_unit)?;
        convert_spec_for_type(registry.unit(source.unit)?, canonical, &ty.key)?;
        convert_spec_for_type(registry.unit(formal.unit)?, canonical, &ty.key)?;
        let conversion = convert_spec_for_type(
            registry.unit(source.unit)?,
            registry.unit(formal.unit)?,
            &ty.key,
        )?;
        Ok(Self {
            source: source.id,
            conversion,
            port: source.clone(),
        })
    }
    /// Re-admit a gather against the current actual registry and formal quantity.
    pub fn readmit(
        &self,
        quantity: pse_quantity::QuantityTypeId,
        registry: &QuantityRegistry,
    ) -> Result<Self, MathError> {
        let formal = Port {
            id: self.source,
            quantity,
            unit: registry.quantity_type(quantity)?.canonical_unit,
        };
        Self::new(&self.port, &formal, registry)
    }
    /// Linear gather coefficient in the source representation.
    pub fn scale(&self) -> f64 {
        self.conversion.scale
    }
    /// Affine gather offset in canonical formal coordinates.
    pub fn offset(&self) -> f64 {
        self.conversion.offset
    }
    /// Physically admitted formal contract.
    pub fn quantity(&self) -> pse_quantity::QuantityTypeId {
        self.port.quantity
    }
    /// Bound source identity; the admitted physical interpretation cannot be changed.
    pub const fn source(&self) -> SemanticId {
        self.source
    }
}

/// One semantic instance of a reusable local body.
#[derive(Clone, Debug, PartialEq)]
pub struct InstanceBinding {
    /// Instance identity, excluded from body identity.
    pub instance: SemanticId,
    /// Reusable body identity.
    pub body: ContentHash,
    /// Inputs in formal order, preserving multiplicity and aliasing.
    pub slots: Vec<SlotBinding>,
    /// Output contributions; repeated targets deliberately accumulate.
    pub contributions: Vec<Contribution>,
}
impl InstanceBinding {
    /// Bind one trial into local canonical coordinates.
    /// # Errors
    /// Missing or nonfinite current value; no defaulting absent ragged members to zero.
    pub fn values(&self, case: &CaseValues) -> Result<Vec<f64>, MathError> {
        self.slots
            .iter()
            .map(|slot| {
                let value = *case
                    .scalars
                    .get(&slot.source)
                    .ok_or_else(|| MathError::Contract("missing case scalar".into()))?;
                let converted = convert_value(&slot.conversion, value);
                if !value.is_finite() || !converted.is_finite() {
                    return Err(MathError::Contract("nonfinite bound scalar".into()));
                }
                Ok(converted)
            })
            .collect()
    }
}

/// Bounded prepared structure. Adding a known body shape only adds bindings.
#[derive(Clone, Debug, PartialEq)]
pub struct CaseStructure {
    /// Canonical global variable declarations, including fixed values.
    variables: Vec<Variable>,
    parameters: Vec<Port>,
    /// Canonical semantic instance order.
    instances: Vec<InstanceBinding>,
    rows: Vec<Row>,
    objective: Option<Objective>,
}
impl CaseStructure {
    /// Admit identities, closed bounds and resource limits without inspecting values.
    /// # Errors
    /// Duplicate variables/instances/rows, malformed bounds or exceeded budgets.
    pub fn new(
        mut variables: Vec<Variable>,
        mut parameters: Vec<Port>,
        mut instances: Vec<InstanceBinding>,
        mut rows: Vec<Row>,
        objective: Option<Objective>,
        limits: CaseLimits,
    ) -> Result<Self, MathError> {
        if limits.instances == 0
            || limits.rows == 0
            || limits.bodies == 0
            || limits.scalars == 0
            || limits.slots == 0
            || instances.len() > limits.instances
            || variables
                .len()
                .checked_add(parameters.len())
                .is_none_or(|n| n > limits.scalars)
        {
            return Err(MathError::Limit("case instances"));
        }
        variables.sort_by_key(|v| v.port.id);
        parameters.sort_by_key(|p| p.id);
        let mut ports = variables
            .iter()
            .map(|v| (v.port.id, &v.port))
            .collect::<BTreeMap<_, _>>();
        for parameter in &parameters {
            if ports.insert(parameter.id, parameter).is_some() {
                return Err(MathError::Contract(
                    "duplicate variable or parameter identity".into(),
                ));
            }
        }
        instances.sort_by_key(|v| v.instance);
        if variables.windows(2).any(|w| w[0].port.id == w[1].port.id)
            || instances.windows(2).any(|w| w[0].instance == w[1].instance)
        {
            return Err(MathError::Contract("duplicate case identity".into()));
        }
        for variable in &variables {
            let l = variable.lower.unwrap_or(f64::NEG_INFINITY);
            let u = variable.upper.unwrap_or(f64::INFINITY);
            if variable.domain.is_semi()
                && (!l.is_finite()
                    || !u.is_finite()
                    || l <= 0.0
                    || l > u
                    || variable.domain.is_integer() && l.ceil() > u.floor())
            {
                return Err(MathError::Contract(
                    "semi-domain needs a nonempty finite positive interval".into(),
                ));
            }
            if variable.domain == VariableDomain::Binary
                && (l > 1.0 || u < 0.0 || l.max(0.0).ceil() > u.min(1.0).floor())
                || variable.domain == VariableDomain::Integer && l.ceil() > u.floor()
            {
                return Err(MathError::Contract("empty declared integer domain".into()));
            }
            if variable
                .lower
                .iter()
                .chain(&variable.upper)
                .any(|v| !v.is_finite())
                || variable
                    .lower
                    .zip(variable.upper)
                    .is_some_and(|(l, u)| l > u)
            {
                return Err(MathError::Contract("invalid variable bounds".into()));
            }
        }
        rows.sort_by_key(|r| r.id);
        if rows.len() > limits.rows
            || rows.windows(2).any(|w| w[0].id == w[1].id)
            || rows.iter().any(|r| {
                r.lower.is_nan()
                    || r.upper.is_nan()
                    || r.lower > r.upper
                    || r.lower == f64::INFINITY
                    || r.upper == f64::NEG_INFINITY
            })
        {
            return Err(MathError::Contract("invalid selected row inventory".into()));
        }
        let row_ids: BTreeSet<_> = rows.iter().map(|r| r.id).collect();
        let mut bodies = BTreeSet::new();
        let mut slot_count = 0usize;
        for instance in &instances {
            slot_count = slot_count
                .checked_add(instance.slots.len())
                .ok_or(MathError::Limit("case slots"))?;
            if slot_count > limits.slots || instance.contributions.is_empty() {
                return Err(MathError::Limit("case slots or empty output layout"));
            }
            for slot in &instance.slots {
                if ports.get(&slot.source).is_none_or(|p| **p != slot.port) {
                    return Err(MathError::Contract(
                        "slot source has no matching declared physical port".into(),
                    ));
                }
            }
            bodies.insert(instance.body);
            for contribution in &instance.contributions {
                if !contribution.scale.is_finite()
                    || contribution.scale == 0.0
                    || match contribution.target {
                        Target::Row(id) => !row_ids.contains(&id),
                        Target::Objective => objective.is_none(),
                    }
                {
                    return Err(MathError::Contract("invalid output contribution".into()));
                }
            }
            if bodies.len() > limits.bodies {
                return Err(MathError::Limit("case specializations"));
            }
        }
        Ok(Self {
            variables,
            parameters,
            instances,
            rows,
            objective,
        })
    }
    /// Structural identity includes bindings, physical units, selected inventories and class declarations.
    pub fn key(&self) -> ContentHash {
        let mut h = FramedHasher::new("pse.math.case-structure.v1");
        h.u64(self.variables.len() as u64);
        for v in &self.variables {
            h.id(&v.port.id)
                .id(&v.port.quantity.as_id())
                .id(&v.port.unit.as_id())
                .bool(v.fixed)
                .u64(v.domain as u64)
                .u64(v.lower.unwrap_or(f64::NEG_INFINITY).to_bits())
                .u64(v.upper.unwrap_or(f64::INFINITY).to_bits());
        }
        h.u64(self.parameters.len() as u64);
        for p in &self.parameters {
            h.id(&p.id).id(&p.quantity.as_id()).id(&p.unit.as_id());
        }
        h.u64(self.rows.len() as u64);
        for r in &self.rows {
            h.id(&r.id)
                .id(&r.quantity.as_id())
                .u64(r.lower.to_bits())
                .u64(r.upper.to_bits());
        }
        h.bool(self.objective.is_some());
        if let Some(o) = &self.objective {
            h.id(&o.quantity.as_id()).u64(o.sense as u64);
        }
        h.u64(self.instances.len() as u64);
        for b in &self.instances {
            h.id(&b.instance).hash(&b.body).u64(b.slots.len() as u64);
            for s in &b.slots {
                h.id(&s.source())
                    .u64(s.scale().to_bits())
                    .u64(s.offset().to_bits());
            }
            h.u64(b.contributions.len() as u64);
            for c in &b.contributions {
                h.u64(c.output as u64).u64(c.scale.to_bits());
                match c.target {
                    Target::Row(r) => {
                        h.bool(false).id(&r);
                    }
                    Target::Objective => {
                        h.bool(true);
                    }
                }
            }
        }
        h.finish_hash()
    }
    /// Complete selected rows, including rows with no numerical support.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }
    /// Explicit objective, when selected.
    pub fn objective(&self) -> Option<&Objective> {
        self.objective.as_ref()
    }
    /// Declared source variables in semantic order.
    pub fn variables(&self) -> &[Variable] {
        &self.variables
    }
    /// Ordinary value parameters, distinct from structural specialization.
    pub fn parameters(&self) -> &[Port] {
        &self.parameters
    }
    /// Bound body occurrences in semantic order.
    pub fn instances(&self) -> &[InstanceBinding] {
        &self.instances
    }
    /// Admit a complete trial value vector; supplied values cannot add structure.
    pub fn validate_values(&self, values: &CaseValues) -> Result<(), MathError> {
        if values.scalars.len() != self.variables.len() + self.parameters.len() {
            return Err(MathError::Contract(
                "case values differ from declared scalar inventory".into(),
            ));
        }
        for port in self
            .variables
            .iter()
            .map(|v| &v.port)
            .chain(&self.parameters)
        {
            if values.scalars.get(&port.id).is_none_or(|v| !v.is_finite()) {
                return Err(MathError::Contract(
                    "missing or nonfinite case value".into(),
                ));
            }
        }
        for variable in &self.variables {
            let value = values.scalars[&variable.port.id];
            if variable.fixed
                && !variable.domain.contains(
                    value,
                    variable.lower.unwrap_or(f64::NEG_INFINITY),
                    variable.upper.unwrap_or(f64::INFINITY),
                )
            {
                return Err(MathError::Contract(
                    "fixed value outside declared integer domain".into(),
                ));
            }
            if !(variable.domain.is_semi() && value == 0.0)
                && (variable.lower.is_some_and(|l| value < l)
                    || variable.upper.is_some_and(|u| value > u))
            {
                return Err(MathError::Contract(
                    "case value outside declared bounds".into(),
                ));
            }
        }
        Ok(())
    }
    /// Stable free-variable layout. Body formal layouts do not change with fixed/free edits.
    pub fn free_variables(&self) -> impl Iterator<Item = SemanticId> + '_ {
        self.variables
            .iter()
            .filter(|v| !v.fixed)
            .map(|v| v.port.id)
    }
}

/// Finite case preparation limits, independent of local arithmetic-body width.
#[derive(Clone, Copy, Debug)]
pub struct CaseLimits {
    /// Variables plus ordinary parameters.
    pub scalars: usize,
    /// Bound body occurrences.
    pub instances: usize,
    /// Semantic result rows.
    pub rows: usize,
    /// Distinct specializations.
    pub bodies: usize,
    /// Total bound formal slots, including aliases.
    pub slots: usize,
}
impl Default for CaseLimits {
    fn default() -> Self {
        Self {
            scalars: 100_000,
            instances: 100_000,
            rows: 100_000,
            bodies: 1024,
            slots: 1_000_000,
        }
    }
}
