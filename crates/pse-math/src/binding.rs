// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Semantic body identity and instance binding are independent of library handles.
use crate::MathError;
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_kernels::Port;
use pse_model::generated::enums::ModelingVariableDomain;
use pse_model::{SemanticEq, SemanticFrame};
use pse_quantity::{CanonicalConversionPlan, QuantityRegistry, admission::require_same_contract};
use std::collections::{BTreeMap, BTreeSet};

/// Compiler-owned interpretation; external hashes cannot select numerical behavior.
pub fn guarded_real_policy() -> ContentHash {
    pse_ids::derive_hash(
        pse_ids::Frame::MathGuardedRealV1,
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
    /// Retained per-node semantic admission and declaration-owned output transitions.
    pub admissions: ContentHash,
    /// Consumed provider contracts, phases and parameter data in stable full-key order.
    pub providers: Vec<ContentHash>,
    /// Real algebra, domain, smoothness and numerical profile.
    pub policy: ContentHash,
}
impl BodySpec {
    /// Versioned key; never hashes printed atoms or process-global library identifiers.
    pub fn key(&self) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::MathBodyV3);
        h.hash(&self.definition)
            .hash(&self.structure)
            .hash(&self.physical)
            .hash(&self.admissions)
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
#[derive(Clone, Debug)]
pub struct Variable {
    /// Global semantic identity and physical contract.
    pub port: Port,
    /// Fixed/free affects the variable layout, not a reusable arithmetic body.
    pub fixed: bool,
    /// Declared registry domain (ADR-0103); integer admission never follows from a value.
    pub domain: ModelingVariableDomain,
    /// Declared closed lower bound, if finite.
    pub lower: Option<f64>,
    /// Declared closed upper bound, if finite.
    pub upper: Option<f64>,
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
#[derive(Clone, Debug)]
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
/// The degradation an earlier lexicographic objective admits while later ones are
/// optimized (ADR-0111 item 3): its value stays within f* + max(absolute, relative·|f*|)
/// of its optimum f*, sense-adjusted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Degradation {
    /// Absolute tolerance, in canonical units of the objective.
    pub absolute: f64,
    /// Dimensionless tolerance relative to the optimum.
    pub relative: f64,
}
impl Degradation {
    /// The admitted degradation at the optimum `optimum`: max(absolute, relative·|f*|).
    pub fn at(self, optimum: f64) -> f64 {
        self.absolute.max(self.relative * optimum.abs())
    }
}
/// An output can contribute to a selected constraint or an objective.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    /// Contribute to a selected constraint row.
    Row(SemanticId),
    /// Contribute to the objective at this position: the only objective, or a
    /// lexicographic level in optimization order (ADR-0111).
    Objective(usize),
}
impl Target {
    /// The primary objective: the only one, or the first lexicographic level.
    pub const PRIMARY: Self = Self::Objective(0);
}
/// One nonzero, dimensionless entry of the row contribution map A.
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct SlotBinding {
    /// Global variable/parameter identity.
    source: SemanticId,
    conversion: CanonicalConversionPlan,
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
        let conversion =
            CanonicalConversionPlan::registered(registry, source.quantity, source.unit)?;
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
        self.conversion.scale()
    }
    /// Affine gather offset in canonical formal coordinates.
    pub fn offset(&self) -> f64 {
        self.conversion.offset()
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
    /// Total body-local checked-member token to actual member attribution, including computed locals.
    pub checked_members: BTreeMap<SemanticId, SemanticId>,
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
                Ok(slot.conversion.apply(value)?.value())
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
    /// No objective, one, or the lexicographic levels in optimization order (ADR-0111).
    objectives: Vec<Objective>,
    /// The degradation of every level but the last while later levels are optimized;
    /// empty unless there are several objectives.
    degradations: Vec<Degradation>,
    /// Constraint forms left to native handlers (ADR-0104); empty for linear lowerings.
    native: Vec<pse_model::forms::NativeConstraint>,
    /// Requirements the lowerings place on the solve route (ADR-0104 §5), in order.
    requirements: Vec<pse_model::generated::enums::ModelingStructuralRequirement>,
}
impl CaseStructure {
    /// Admit identities, closed bounds and resource limits without inspecting values.
    /// # Errors
    /// Duplicate variables/instances/rows, malformed bounds or exceeded budgets.
    pub fn new(
        variables: Vec<Variable>,
        parameters: Vec<Port>,
        instances: Vec<InstanceBinding>,
        rows: Vec<Row>,
        objective: Option<Objective>,
        limits: CaseLimits,
    ) -> Result<Self, MathError> {
        Self::build(
            variables,
            parameters,
            instances,
            rows,
            objective.into_iter().collect(),
            Vec::new(),
            limits,
        )
    }
    /// A structure with several objectives, optimized lexicographically in the given order
    /// (ADR-0111): every level but the last states the degradation it admits while later
    /// levels are optimized. Only a native lexicographic route solves it; every other
    /// route optimizes one level at a time.
    /// # Errors
    /// Fewer than two levels, a degradation on the last level or missing on another, a
    /// negative or nonfinite tolerance, and every refusal of [`Self::new`].
    pub fn lexicographic(
        variables: Vec<Variable>,
        parameters: Vec<Port>,
        instances: Vec<InstanceBinding>,
        rows: Vec<Row>,
        levels: Vec<(Objective, Option<Degradation>)>,
        limits: CaseLimits,
    ) -> Result<Self, MathError> {
        let last = levels
            .len()
            .checked_sub(1)
            .filter(|last| *last > 0)
            .ok_or_else(|| {
                MathError::Contract("a lexicographic structure needs several objectives".into())
            })?;
        let mut objectives = Vec::with_capacity(levels.len());
        let mut degradations = Vec::with_capacity(last);
        for (position, (objective, degradation)) in levels.into_iter().enumerate() {
            match (degradation, position == last) {
                (None, true) => {}
                (Some(d), false)
                    if d.absolute.is_finite()
                        && d.relative.is_finite()
                        && d.absolute >= 0.0
                        && d.relative >= 0.0 =>
                {
                    degradations.push(d);
                }
                _ => {
                    return Err(MathError::Contract(
                        "every earlier lexicographic level states a finite nonnegative degradation, and the last none".into(),
                    ));
                }
            }
            objectives.push(objective);
        }
        Self::build(
            variables,
            parameters,
            instances,
            rows,
            objectives,
            degradations,
            limits,
        )
    }
    /// A structure over other inventories with the objectives and lexicographic
    /// degradations of `source`: the solver structure of a case bound from an admitted one.
    /// # Errors
    /// Every refusal of [`Self::new`].
    pub fn like(
        source: &Self,
        variables: Vec<Variable>,
        parameters: Vec<Port>,
        instances: Vec<InstanceBinding>,
        rows: Vec<Row>,
        limits: CaseLimits,
    ) -> Result<Self, MathError> {
        Self::build(
            variables,
            parameters,
            instances,
            rows,
            source.objectives.clone(),
            source.degradations.clone(),
            limits,
        )
    }
    fn build(
        mut variables: Vec<Variable>,
        mut parameters: Vec<Port>,
        mut instances: Vec<InstanceBinding>,
        mut rows: Vec<Row>,
        objectives: Vec<Objective>,
        degradations: Vec<Degradation>,
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
            // A declared structure may leave the active interval to case bounds (ADR-0103);
            // any declared endpoint keeps it positive and nonempty. Native admission
            // requires both endpoints finite.
            if variable.domain.is_semi()
                && (variable.lower.is_some() && l <= 0.0
                    || l > u
                    || variable.domain.is_integer() && l.ceil() > u.floor())
            {
                return Err(MathError::Contract(
                    "semi-domain needs a nonempty positive interval".into(),
                ));
            }
            if variable.domain == ModelingVariableDomain::Binary
                && (l > 1.0 || u < 0.0 || l.max(0.0).ceil() > u.min(1.0).floor())
                || variable.domain == ModelingVariableDomain::Integer && l.ceil() > u.floor()
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
                .and_then(|count| count.checked_add(instance.checked_members.len()))
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
                        Target::Objective(level) => level >= objectives.len(),
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
            objectives,
            degradations,
            native: Vec::new(),
            requirements: Vec::new(),
        })
    }
    /// Attach native constraint forms. Every row and variable they name must be selected;
    /// a backend without the handlers cannot execute the structure (ADR-0104).
    /// # Errors
    /// A native constraint naming an identity outside this structure.
    pub fn with_native(
        mut self,
        native: Vec<pse_model::forms::NativeConstraint>,
    ) -> Result<Self, MathError> {
        let known = self
            .variables
            .iter()
            .map(|v| v.port.id)
            .chain(self.rows.iter().map(|r| r.id))
            .collect::<BTreeSet<_>>();
        if native
            .iter()
            .flat_map(pse_model::forms::NativeConstraint::identities)
            .any(|id| !known.contains(&id))
        {
            return Err(MathError::Contract(
                "native constraint names an unselected row or variable".into(),
            ));
        }
        self.native = native;
        Ok(self)
    }
    /// Constraint forms that require native handlers.
    pub fn native(&self) -> &[pse_model::forms::NativeConstraint] {
        &self.native
    }
    /// Attach the requirements the lowerings place on the solve route (ADR-0104 §5), such
    /// as the l1 exact-penalty route of an authored `penalty(l1)`. Routing admits only an
    /// adapter whose record and settings honour each of them.
    pub fn with_requirements(
        mut self,
        requirements: impl IntoIterator<
            Item = pse_model::generated::enums::ModelingStructuralRequirement,
        >,
    ) -> Self {
        let mut requirements = requirements.into_iter().collect::<Vec<_>>();
        requirements.sort_by_key(|r| r.as_str());
        requirements.dedup();
        self.requirements = requirements;
        self
    }
    /// Requirements the lowerings place on the solve route.
    pub fn requirements(&self) -> &[pse_model::generated::enums::ModelingStructuralRequirement] {
        &self.requirements
    }
    /// Structural identity includes bindings, physical units, selected inventories and class declarations.
    pub fn key(&self) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::MathCaseStructureV6);
        h.u64(self.variables.len() as u64);
        for v in &self.variables {
            h.id(&v.port.id)
                .id(&v.port.quantity.as_id())
                .id(&v.port.unit.as_id())
                .bool(v.fixed)
                .str(v.domain.as_str())
                .u64(pse_ids::canonical_f64_bits(
                    v.lower.unwrap_or(f64::NEG_INFINITY),
                ))
                .u64(pse_ids::canonical_f64_bits(
                    v.upper.unwrap_or(f64::INFINITY),
                ));
        }
        h.u64(self.parameters.len() as u64);
        for p in &self.parameters {
            h.id(&p.id).id(&p.quantity.as_id()).id(&p.unit.as_id());
        }
        h.u64(self.rows.len() as u64);
        for r in &self.rows {
            h.id(&r.id)
                .id(&r.quantity.as_id())
                .u64(pse_ids::canonical_f64_bits(r.lower))
                .u64(pse_ids::canonical_f64_bits(r.upper));
        }
        h.u64(self.objectives.len() as u64);
        for o in &self.objectives {
            h.id(&o.quantity.as_id()).u64(o.sense as u64);
        }
        for d in &self.degradations {
            h.u64(pse_ids::canonical_f64_bits(d.absolute))
                .u64(pse_ids::canonical_f64_bits(d.relative));
        }
        h.u64(self.native.len() as u64);
        for constraint in &self.native {
            constraint.frame(&mut h);
        }
        h.u64(self.requirements.len() as u64);
        for requirement in &self.requirements {
            h.str(requirement.as_str());
        }
        h.u64(self.instances.len() as u64);
        for b in &self.instances {
            h.id(&b.instance).hash(&b.body).u64(b.slots.len() as u64);
            for s in &b.slots {
                h.id(&s.source())
                    .u64(pse_ids::canonical_f64_bits(s.scale()))
                    .u64(pse_ids::canonical_f64_bits(s.offset()));
            }
            h.u64(b.checked_members.len() as u64);
            for (token, actual) in &b.checked_members {
                h.id(token).id(actual);
            }
            h.u64(b.contributions.len() as u64);
            for c in &b.contributions {
                h.u64(c.output as u64)
                    .u64(pse_ids::canonical_f64_bits(c.scale));
                match c.target {
                    Target::Row(r) => {
                        h.bool(false).id(&r);
                    }
                    Target::Objective(level) => {
                        h.bool(true).u64(level as u64);
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
    /// The primary objective: the only one, or the first lexicographic level.
    pub fn objective(&self) -> Option<&Objective> {
        self.objectives.first()
    }
    /// Every objective in optimization order: none, one, or the lexicographic levels.
    pub fn objectives(&self) -> &[Objective] {
        &self.objectives
    }
    /// The degradation of each lexicographic level but the last (ADR-0111 item 3).
    pub fn degradations(&self) -> &[Degradation] {
        &self.degradations
    }
    /// Several objectives, which only a native lexicographic route optimizes together.
    pub fn lexicographic_levels(&self) -> bool {
        self.objectives.len() > 1
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
    /// Admit only the values frozen into compilation: parameters and fixed variables.
    /// Free coordinates may be absent or outside bounds for structural diagnostics;
    /// this does not admit a numerical trial or supply an implicit initial guess.
    pub fn validate_frozen_values(&self, values: &CaseValues) -> Result<(), MathError> {
        let declared = self
            .variables
            .iter()
            .map(|v| v.port.id)
            .chain(self.parameters.iter().map(|p| p.id))
            .collect::<BTreeSet<_>>();
        if values
            .scalars
            .iter()
            .any(|(id, v)| !declared.contains(id) || !v.is_finite())
        {
            return Err(MathError::Contract(
                "undeclared or nonfinite preparation value".into(),
            ));
        }
        for id in self
            .parameters
            .iter()
            .map(|p| p.id)
            .chain(self.variables.iter().filter(|v| v.fixed).map(|v| v.port.id))
        {
            if !values.scalars.contains_key(&id) {
                return Err(MathError::Contract(format!(
                    "missing frozen preparation value {id}"
                )));
            }
        }
        for variable in self.variables.iter().filter(|v| v.fixed) {
            if !variable.domain.contains(
                values.scalars[&variable.port.id],
                variable.lower.unwrap_or(f64::NEG_INFINITY),
                variable.upper.unwrap_or(f64::INFINITY),
            ) {
                return Err(MathError::Contract(
                    "fixed value outside declared domain".into(),
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
    /// Total bound formal slots and checked-member attribution entries, including aliases.
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

// Equality and framed identities share the canonical floating-point contract.
impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        self.port == other.port
            && self.fixed == other.fixed
            && self.domain == other.domain
            && self.lower.semantic_eq(&other.lower)
            && self.upper.semantic_eq(&other.upper)
    }
}
impl PartialEq for Row {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.quantity == other.quantity
            && self.lower.semantic_eq(&other.lower)
            && self.upper.semantic_eq(&other.upper)
    }
}
impl PartialEq for Contribution {
    fn eq(&self, other: &Self) -> bool {
        self.output == other.output
            && self.target == other.target
            && self.scale.semantic_eq(&other.scale)
    }
}
impl PartialEq for SlotBinding {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.port == other.port
            && self.scale().semantic_eq(&other.scale())
            && self.offset().semantic_eq(&other.offset())
    }
}
impl PartialEq for CaseValues {
    fn eq(&self, other: &Self) -> bool {
        self.scalars.semantic_eq(&other.scalars)
    }
}
impl CaseValues {
    /// Value binding identity, separate from structure and prepared arithmetic.
    pub fn identity(&self) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::MathCaseValuesV1);
        h.u64(self.scalars.len() as u64);
        for (id, value) in &self.scalars {
            h.id(id);
            value.frame(&mut h);
        }
        h.finish_hash()
    }
}
