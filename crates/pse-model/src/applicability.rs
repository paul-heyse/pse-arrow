// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Evidence claims and scoped permissions, independent of numerical execution.
use crate::generated::enums::{
    ModelingApplicabilityBasis as Basis, ModelingApplicabilityOutcome as Outcome,
    ModelingPermissionTarget as Target, ModelingValidityLayer as Layer,
};
use pse_ids::{FramedHasher, SemanticId};

const CAPTURE_MAX_DEPTH: usize = 512;

/// Checked source-owned populations; inspection is passive and grants no validity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CaptureExtents {
    /// Requested backing of an exact metadata clone, excluding its inline Node.
    pub clone_bytes: usize,
    /// Actual existing owned vector/string capacities, excluding the inline Node.
    pub owned_bytes: usize,
    /// Existing complete observation bound, including nonwinning alternatives.
    pub observation_bytes: usize,
    /// Actual plan nodes, including unions and dependencies.
    pub nodes: usize,
    /// Maximum identity population required by subsequent uniqueness validation.
    pub validation_entries: usize,
}
/// Structural refusal and checked extent overflow are distinct owner outcomes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error, miette::Diagnostic)]
#[diagnostic(code("model.applicability_capture"))]
pub enum CaptureExtentError {
    /// Invalid references or excessive structural nesting/population.
    #[error("invalid applicability capture structure")]
    Structure,
    /// An actual source-owned population/backing is unrepresentable.
    #[error("applicability capture extent overflow")]
    Overflow,
}
fn capture_add(a: usize, b: usize) -> Result<usize, CaptureExtentError> {
    a.checked_add(b).ok_or(CaptureExtentError::Overflow)
}
fn capture_backing<T>(entries: usize) -> Result<usize, CaptureExtentError> {
    std::alloc::Layout::array::<T>(entries)
        .map(|layout| layout.size())
        .map_err(|_| CaptureExtentError::Overflow)
}

pse_contract::projection_struct! {
/// Immutable attribution of an instantiated scientific claim.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "ApplicabilityClaim")]
pub struct Claim {
    projection: { name: "ApplicabilityClaim", rust_path: crate::applicability::Claim, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Absent exactly for undeclared evidence, which is unknown.
    pub id: Option<SemanticId> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Joint whole-interval application, only for a verified declared interval.
    pub coverage: Option<SemanticId> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Declared scientific family or model owner.
    pub owner: SemanticId => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Checked nominal owner and its entity-kind ancestors, in most-derived order.
    /// Non-kind owners have only themselves; callers cannot author this witness.
    pub owner_lineage: Vec<SemanticId> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Evidence source; absent for an undeclared claim.
    pub evidence: Option<SemanticId> => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Form whose actual application demanded this evidence.
    pub form: SemanticId => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Distinguishes actual bound applications, including derived arguments.
    pub call: SemanticId => { tag: 7, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Distinct scientific records selected by this application.
    pub records: Vec<SemanticId> => { tag: 8, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Required claim identities.
    pub dependencies: Vec<SemanticId> => { tag: 9, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Evidence layer, distinct from a mathematical domain.
    pub layer: Layer => { tag: 10, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Fitted, recommended or validated region; none for unknown/unrestricted/union.
    pub basis: Option<Basis> => { tag: 11, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Explicit unknown reason, never a numerical predicate result.
    pub reason: Option<String> => { tag: 12, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// Authorization retains its declaration and scope, with fixed named targets.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "ApplicabilityPermission")]
pub struct Permission {
    projection: { name: "ApplicabilityPermission", rust_path: crate::applicability::Permission, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Authored authorization identity.
    pub id: SemanticId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Declaring lexical consumer scope, not the claim's owner.
    pub scope: SemanticId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Named records or declared scientific families.
    pub target_kind: Target => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Exact admitted target identities; inheritance never widens this set.
    pub targets: Vec<SemanticId> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Independent permission for absence of evidence.
    pub allow_unknown: bool => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Independent permission for leaving a declared region.
    pub allow_extrapolation: bool => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
impl Permission {
    /// A record permission must cover every record used by the claim.
    pub fn covers(&self, claim: &Claim) -> bool {
        match self.target_kind {
            Target::Families => {
                claim.owner_lineage.first() == Some(&claim.owner)
                    && claim
                        .owner_lineage
                        .iter()
                        .any(|id| self.targets.contains(id))
            }
            Target::Records => {
                !claim.records.is_empty()
                    && claim.records.iter().all(|id| self.targets.contains(id))
            }
        }
    }
}
pse_contract::projection_struct! {
/// Numerical inputs are recorded in the lowering's canonical physical units.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "ApplicabilityInput")]
pub struct Input {
    projection: { name: "ApplicabilityInput", rust_path: crate::applicability::Input, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Declared argument name.
    pub name: String => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Actual value at the demanded application; the wire retains explicit IEEE evidence.
    #[serde(
        serialize_with = "crate::diagnostic::serialize_numeric_observation",
        deserialize_with = "crate::diagnostic::deserialize_numeric_observation"
    )]
    #[schemars(with = "crate::diagnostic::NumericObservationSchema")]
    pub value: f64 => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Actual physical quantity type; values use its canonical unit.
    pub quantity_type: SemanticId => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// One typed observation, including alternative evidence which was not required.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "ApplicabilityObservation")]
pub struct Observation {
    projection: { name: "ApplicabilityObservation", rust_path: crate::applicability::Observation, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Complete scientific attribution.
    pub claim: Claim => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Bound assembly instance, populated by the executing consumer.
    pub instance: Option<SemanticId> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Actual inputs of this claim application.
    pub inputs: Vec<Input> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Evidence classification, independent of authorization.
    pub outcome: Outcome => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// This claim was required, rather than a nonwinning union alternative.
    pub required: bool => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Authorizing declarations that matched this claim's exact target.
    pub permissions: Vec<Permission> => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Both permissions remain independent even for a mixed dependency result.
    pub unknown_allowed: bool => { tag: 7, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Whether leaving this particular claim's region was permitted.
    pub extrapolation_allowed: bool => { tag: 8, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Whether the low-level evidence gate admitted this required claim.
    pub admitted: bool => { tag: 9, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
pse_contract::projection_tuple_enum! {
/// A claim plan uses library-evaluated predicate results, not a second numeric engine.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Region {
    projection: { name: "ApplicabilityRegion", rust_path: crate::applicability::Region, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Index into the mathematical program's boolean predicate outputs.
    Predicate [/// Original program predicate output index.
        predicate: usize => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 1;
    /// A declaration explicitly states unrestricted evidence.
    Unrestricted => 2;
    /// Missing evidence or a declared unknown reason.
    Unknown => 3;
    /// Only a declaration can combine alternatives into one evidence union.
    Union [/// Original declared evidence alternatives in native order.
        alternatives: Vec<Node> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 4;
}
}
pse_contract::projection_struct! {
/// Required dependencies are intersections, never implicit fit unions.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Node {
    projection: { name: "ApplicabilityNode", rust_path: crate::applicability::Node, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Attribution of this instantiated claim.
    pub claim: Claim => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Its declaration-owned evidence form.
    pub region: Region => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Independently required evidence.
    pub dependencies: Vec<Node> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Indices of actual numerical argument values to retain as observations.
    pub inputs: Vec<(String, usize, SemanticId)> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Exact permissions visible in the application's consumer scope.
    pub permissions: Vec<Permission> => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
/// Full demand-time observation set, including refusals.
#[derive(Clone, Debug, PartialEq)]
pub struct Assessment {
    /// Every demanded claim and declared alternative.
    pub observations: Vec<Observation>,
    /// Indices of inadmissible required observations.
    pub refused: Vec<usize>,
}
impl Node {
    /// Preserve full declaration/selection/permission identity in compiled caches.
    pub fn frame(&self, h: &mut FramedHasher) {
        h.str("applicability-node-v2");
        h.bool(self.claim.id.is_some());
        if let Some(id) = self.claim.id {
            h.id(&id);
        }
        h.bool(self.claim.coverage.is_some());
        if let Some(id) = self.claim.coverage {
            h.id(&id);
        }
        h.id(&self.claim.owner)
            .id(&self.claim.form)
            .id(&self.claim.call);
        h.bool(self.claim.evidence.is_some());
        if let Some(id) = self.claim.evidence {
            h.id(&id);
        }
        h.str(self.claim.layer.as_str())
            .str(self.claim.basis.map_or("", |v| v.as_str()))
            .str(self.claim.reason.as_deref().unwrap_or(""));
        for ids in [
            &self.claim.owner_lineage,
            &self.claim.records,
            &self.claim.dependencies,
        ] {
            h.u64(ids.len() as u64);
            for id in ids {
                h.id(id);
            }
        }
        match &self.region {
            Region::Predicate(i) => {
                h.str("predicate").u64(*i as u64);
            }
            Region::Unrestricted => {
                h.str("unrestricted");
            }
            Region::Unknown => {
                h.str("unknown");
            }
            Region::Union(children) => {
                h.str("union").u64(children.len() as u64);
                for child in children {
                    child.frame(h);
                }
            }
        }
        h.u64(self.dependencies.len() as u64);
        for child in &self.dependencies {
            child.frame(h);
        }
        h.u64(self.inputs.len() as u64);
        for (name, index, quantity) in &self.inputs {
            h.str(name).u64(*index as u64).id(quantity);
        }
        h.u64(self.permissions.len() as u64);
        for p in &self.permissions {
            h.id(&p.id)
                .id(&p.scope)
                .str(p.target_kind.as_str())
                .bool(p.allow_unknown)
                .bool(p.allow_extrapolation)
                .u64(p.targets.len() as u64);
            for id in &p.targets {
                h.id(id);
            }
        }
    }
    /// Validate capture references before any permission can affect evaluation.
    pub fn valid_capture(&self, predicates: usize, inputs: usize, budget: usize) -> bool {
        let Some(entries) = self.capture_validation_scratch(predicates, inputs, budget) else {
            return false;
        };
        let mut scratch = Vec::new();
        if scratch.try_reserve_exact(entries).is_err() {
            return false;
        }
        scratch.resize(entries, SemanticId::NIL);
        self.valid_capture_with_scratch(predicates, inputs, budget, &mut scratch)
    }
    /// Allocation-free structural screening and actual identity scratch population.
    /// Duplicate lineage validation remains outstanding until scratch is supplied.
    /// The existing node/depth budget and all reference rules apply to both phases.
    pub fn capture_validation_scratch(
        &self,
        predicates: usize,
        inputs: usize,
        mut budget: usize,
    ) -> Option<usize> {
        let mut entries = 0;
        self.visit_capture(predicates, inputs, &mut budget, 0, &mut |lineage| {
            entries = entries.max(lineage.len());
            true
        })
        .then_some(entries)
    }
    /// Validate using admitted caller storage; insufficient scratch refuses without
    /// a fallback allocation. Sorting changes only the supplied scratch, not the plan.
    pub fn valid_capture_with_scratch(
        &self,
        predicates: usize,
        inputs: usize,
        mut budget: usize,
        scratch: &mut [SemanticId],
    ) -> bool {
        self.visit_capture(predicates, inputs, &mut budget, 0, &mut |lineage| {
            let Some(target) = scratch.get_mut(..lineage.len()) else {
                return false;
            };
            target.copy_from_slice(lineage);
            target.sort_unstable();
            target.windows(2).all(|pair| pair[0] != pair[1])
        })
    }
    fn visit_capture(
        &self,
        predicates: usize,
        inputs: usize,
        left: &mut usize,
        depth: usize,
        lineage: &mut impl FnMut(&[SemanticId]) -> bool,
    ) -> bool {
        fn visit(
            n: &Node,
            predicates: usize,
            inputs: usize,
            left: &mut usize,
            depth: usize,
            lineage: &mut impl FnMut(&[SemanticId]) -> bool,
        ) -> bool {
            if *left == 0 || depth > CAPTURE_MAX_DEPTH {
                return false;
            }
            *left -= 1;
            if n.claim.owner_lineage.first() != Some(&n.claim.owner)
                || matches!(n.region,Region::Predicate(i) if i>=predicates)
                || n.inputs.iter().any(|(_, i, _)| *i >= inputs)
                || !lineage(&n.claim.owner_lineage)
            {
                return false;
            }
            if let Region::Union(children) = &n.region
                && !children
                    .iter()
                    .all(|n| visit(n, predicates, inputs, left, depth + 1, lineage))
            {
                return false;
            }
            n.dependencies
                .iter()
                .all(|n| visit(n, predicates, inputs, left, depth + 1, lineage))
        }
        visit(self, predicates, inputs, left, depth, lineage)
    }
    /// Owned plan metadata, separate from shared expression/library storage.
    pub fn retained_bytes(&self) -> usize {
        let mut remaining = usize::MAX;
        self.capture_sizes(&mut remaining, 0)
            .map_or(usize::MAX, |extent| extent.owned_bytes)
    }
    /// Conservative retained evaluation extent, including nonwinning alternatives.
    pub fn observation_bytes(&self) -> usize {
        let mut remaining = usize::MAX;
        self.capture_sizes(&mut remaining, 0)
            .map_or(usize::MAX, |extent| extent.observation_bytes)
    }
    /// Allocation-free structural screening and checked native populations. Duplicate
    /// lineage validation remains required using admitted caller scratch afterward.
    pub fn checked_capture_extents(
        &self,
        predicates: usize,
        inputs: usize,
        mut budget: usize,
    ) -> Result<CaptureExtents, CaptureExtentError> {
        self.capture_validation_scratch(predicates, inputs, budget)
            .ok_or(CaptureExtentError::Structure)?;
        self.capture_sizes(&mut budget, 0)
    }
    fn capture_sizes(
        &self,
        left: &mut usize,
        depth: usize,
    ) -> Result<CaptureExtents, CaptureExtentError> {
        if *left == 0 || depth > CAPTURE_MAX_DEPTH {
            return Err(CaptureExtentError::Structure);
        }
        *left -= 1;
        let c = &self.claim;
        let mut clone = capture_backing::<SemanticId>(capture_add(
            capture_add(c.owner_lineage.len(), c.records.len())?,
            c.dependencies.len(),
        )?)?;
        clone = capture_add(
            clone,
            capture_backing::<u8>(c.reason.as_ref().map_or(0, String::len))?,
        )?;
        let mut owned = capture_backing::<SemanticId>(capture_add(
            capture_add(c.owner_lineage.capacity(), c.records.capacity())?,
            c.dependencies.capacity(),
        )?)?;
        owned = capture_add(
            owned,
            capture_backing::<u8>(c.reason.as_ref().map_or(0, String::capacity))?,
        )?;
        let mut observation = capture_add(size_of::<Observation>(), clone)?;
        clone = capture_add(
            clone,
            capture_backing::<(String, usize, SemanticId)>(self.inputs.len())?,
        )?;
        owned = capture_add(
            owned,
            capture_backing::<(String, usize, SemanticId)>(self.inputs.capacity())?,
        )?;
        observation = capture_add(observation, capture_backing::<Input>(self.inputs.len())?)?;
        for (name, _, _) in &self.inputs {
            let bytes = capture_backing::<u8>(name.len())?;
            clone = capture_add(clone, bytes)?;
            observation = capture_add(observation, bytes)?;
            owned = capture_add(owned, capture_backing::<u8>(name.capacity())?)?;
        }
        let permission_bytes = capture_backing::<Permission>(self.permissions.len())?;
        clone = capture_add(clone, permission_bytes)?;
        observation = capture_add(observation, permission_bytes)?;
        owned = capture_add(
            owned,
            capture_backing::<Permission>(self.permissions.capacity())?,
        )?;
        for permission in &self.permissions {
            let bytes = capture_backing::<SemanticId>(permission.targets.len())?;
            clone = capture_add(clone, bytes)?;
            observation = capture_add(observation, bytes)?;
            owned = capture_add(
                owned,
                capture_backing::<SemanticId>(permission.targets.capacity())?,
            )?;
        }
        let mut result = CaptureExtents {
            clone_bytes: clone,
            owned_bytes: owned,
            observation_bytes: observation,
            nodes: 1,
            validation_entries: c.owner_lineage.len(),
        };
        let mut children =
            |children: &Vec<Node>, copies: usize| -> Result<(), CaptureExtentError> {
                result.clone_bytes =
                    capture_add(result.clone_bytes, capture_backing::<Node>(children.len())?)?;
                result.owned_bytes = capture_add(
                    result.owned_bytes,
                    capture_backing::<Node>(children.capacity())?,
                )?;
                for child in children {
                    let extent = child.capture_sizes(left, depth + 1)?;
                    result.clone_bytes = capture_add(result.clone_bytes, extent.clone_bytes)?;
                    result.owned_bytes = capture_add(result.owned_bytes, extent.owned_bytes)?;
                    result.observation_bytes = capture_add(
                        result.observation_bytes,
                        extent
                            .observation_bytes
                            .checked_mul(copies)
                            .ok_or(CaptureExtentError::Overflow)?,
                    )?;
                    result.nodes = capture_add(result.nodes, extent.nodes)?;
                    result.validation_entries =
                        result.validation_entries.max(extent.validation_entries);
                }
                Ok(())
            };
        if let Region::Union(alternatives) = &self.region {
            children(alternatives, 2)?;
        }
        children(&self.dependencies, 1)?;
        Ok(result)
    }
    /// Assess only already evaluated library predicate results and actual input values.
    /// A declared union prefers applicable evidence, then unknown, then outside evidence.
    pub fn assess(&self, predicates: &[bool], values: &[f64]) -> Assessment {
        let mut assessment = Assessment {
            observations: Vec::new(),
            refused: Vec::new(),
        };
        self.observe(predicates, values, true, &mut assessment);
        assessment
    }
    fn classification(&self, predicates: &[bool]) -> Outcome {
        if self.claim.id.is_none() {
            return Outcome::UnknownEvidence;
        }
        match &self.region {
            Region::Predicate(i) => {
                if predicates.get(*i) == Some(&true) {
                    Outcome::Applicable
                } else {
                    Outcome::OutsideRegion
                }
            }
            Region::Unrestricted => Outcome::Applicable,
            Region::Unknown => Outcome::UnknownEvidence,
            Region::Union(children) => {
                let outcomes = children
                    .iter()
                    .map(|n| n.effective_classification(predicates))
                    .collect::<Vec<_>>();
                if outcomes.contains(&Outcome::Applicable) {
                    Outcome::Applicable
                } else if outcomes.contains(&Outcome::UnknownEvidence) || children.is_empty() {
                    Outcome::UnknownEvidence
                } else {
                    Outcome::OutsideRegion
                }
            }
        }
    }
    fn effective_classification(&self, predicates: &[bool]) -> Outcome {
        let own = self.classification(predicates);
        let dependencies = self
            .dependencies
            .iter()
            .map(|n| n.effective_classification(predicates))
            .collect::<Vec<_>>();
        if own == Outcome::UnknownEvidence || dependencies.contains(&Outcome::UnknownEvidence) {
            Outcome::UnknownEvidence
        } else if own == Outcome::OutsideRegion || dependencies.contains(&Outcome::OutsideRegion) {
            Outcome::OutsideRegion
        } else {
            Outcome::Applicable
        }
    }
    /// Test the same required-claim gates as full assessment without cloning
    /// declaration metadata. Numerical callers retain inputs for later hydration.
    pub fn admits(&self, predicates: &[bool]) -> bool {
        let mut admitted = true;
        self.visit(predicates, true, &mut |_, _, _, _, _, allowed| {
            admitted &= allowed
        });
        admitted
    }
    fn observe(
        &self,
        predicates: &[bool],
        values: &[f64],
        required: bool,
        assessment: &mut Assessment,
    ) {
        self.visit(
            predicates,
            required,
            &mut |node, outcome, required, unknown_allowed, extrapolation_allowed, admitted| {
                if !admitted {
                    assessment.refused.push(assessment.observations.len());
                }
                assessment.observations.push(Observation {
                    claim: node.claim.clone(),
                    instance: None,
                    inputs: node
                        .inputs
                        .iter()
                        .filter_map(|(name, i, quantity_type)| {
                            values.get(*i).map(|value| Input {
                                name: name.clone(),
                                value: *value,
                                quantity_type: *quantity_type,
                            })
                        })
                        .collect(),
                    outcome,
                    required,
                    permissions: node
                        .permissions
                        .iter()
                        .filter(|p| p.covers(&node.claim))
                        .cloned()
                        .collect(),
                    unknown_allowed,
                    extrapolation_allowed,
                    admitted,
                });
            },
        );
    }
    fn visit(
        &self,
        predicates: &[bool],
        required: bool,
        visit: &mut impl FnMut(&Self, Outcome, bool, bool, bool, bool),
    ) {
        let outcome = self.classification(predicates);
        let (unknown_allowed, extrapolation_allowed) = self
            .permissions
            .iter()
            .filter(|p| p.covers(&self.claim))
            .fold((false, false), |(unknown, outside), permission| {
                (
                    unknown || permission.allow_unknown,
                    outside || permission.allow_extrapolation,
                )
            });
        let admitted = !required
            || match outcome {
                Outcome::Applicable => true,
                Outcome::UnknownEvidence => unknown_allowed,
                Outcome::OutsideRegion => extrapolation_allowed,
            };
        visit(
            self,
            outcome,
            required,
            unknown_allowed,
            extrapolation_allowed,
            admitted,
        );
        if let Region::Union(children) = &self.region {
            let winner = children
                .iter()
                .position(|n| n.effective_classification(predicates) == outcome);
            for (i, child) in children.iter().enumerate() {
                child.visit(predicates, required && winner == Some(i), visit);
            }
        }
        for dependency in &self.dependencies {
            dependency.visit(predicates, required, visit);
        }
    }
}

impl Assessment {
    /// Owned observation extent for runtime reservation and retained failures.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.refused.capacity() * size_of::<usize>()
            + self.observations.capacity() * size_of::<Observation>()
            + self
                .observations
                .iter()
                .map(Observation::retained_bytes)
                .sum::<usize>()
    }
}
impl Observation {
    /// Additional owned capacities beyond the inline observation value.
    pub fn retained_bytes(&self) -> usize {
        self.claim.owner_lineage.capacity() * size_of::<SemanticId>()
            + self.claim.records.capacity() * size_of::<SemanticId>()
            + self.claim.dependencies.capacity() * size_of::<SemanticId>()
            + self.claim.reason.as_ref().map_or(0, String::capacity)
            + self.inputs.capacity() * size_of::<Input>()
            + self.inputs.iter().map(|i| i.name.capacity()).sum::<usize>()
            + self.permissions.capacity() * size_of::<Permission>()
            + self
                .permissions
                .iter()
                .map(|p| p.targets.capacity() * size_of::<SemanticId>())
                .sum::<usize>()
    }
}

#[cfg(test)]
mod capture_extent_tests {
    use super::*;
    #[test]
    fn checked_population_and_backing_overflow_have_a_distinct_owner_class() {
        assert_eq!(
            capture_add(usize::MAX, 1),
            Err(CaptureExtentError::Overflow)
        );
        assert_eq!(
            capture_backing::<Node>(usize::MAX),
            Err(CaptureExtentError::Overflow)
        );
    }
}
