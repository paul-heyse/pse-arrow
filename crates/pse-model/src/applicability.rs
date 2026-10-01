// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Evidence claims and scoped permissions, independent of numerical execution.
use crate::generated::enums::{
    ModelingApplicabilityBasis as Basis, ModelingApplicabilityOutcome as Outcome,
    ModelingPermissionTarget as Target, ModelingValidityLayer as Layer,
};
use pse_ids::{FramedHasher, SemanticId};

/// Immutable attribution of an instantiated scientific claim.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Claim {
    /// Absent exactly for undeclared evidence, which is unknown.
    pub id: Option<SemanticId>,
    /// Joint whole-interval application, only for a verified declared interval.
    pub coverage: Option<SemanticId>,
    /// Declared scientific family or model owner.
    pub owner: SemanticId,
    /// Checked nominal owner and its entity-kind ancestors, in most-derived order.
    /// Non-kind owners have only themselves; callers cannot author this witness.
    pub owner_lineage: Vec<SemanticId>,
    /// Evidence source; absent for an undeclared claim.
    pub evidence: Option<SemanticId>,
    /// Form whose actual application demanded this evidence.
    pub form: SemanticId,
    /// Distinguishes actual bound applications, including derived arguments.
    pub call: SemanticId,
    /// Distinct scientific records selected by this application.
    pub records: Vec<SemanticId>,
    /// Required claim identities.
    pub dependencies: Vec<SemanticId>,
    /// Evidence layer, distinct from a mathematical domain.
    pub layer: Layer,
    /// Fitted, recommended or validated region; none for unknown/unrestricted/union.
    pub basis: Option<Basis>,
    /// Explicit unknown reason, never a numerical predicate result.
    pub reason: Option<String>,
}
/// Authorization retains its declaration and scope, with fixed named targets.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Permission {
    /// Authored authorization identity.
    pub id: SemanticId,
    /// Declaring lexical consumer scope, not the claim's owner.
    pub scope: SemanticId,
    /// Named records or declared scientific families.
    pub target_kind: Target,
    /// Exact admitted target identities; inheritance never widens this set.
    pub targets: Vec<SemanticId>,
    /// Independent permission for absence of evidence.
    pub allow_unknown: bool,
    /// Independent permission for leaving a declared region.
    pub allow_extrapolation: bool,
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
/// Numerical inputs are recorded in the lowering's canonical physical units.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Input {
    /// Declared argument name.
    pub name: String,
    /// Actual value at the demanded application.
    pub value: f64,
    /// Actual physical quantity type; values use its canonical unit.
    pub quantity_type: SemanticId,
}
/// One typed observation, including alternative evidence which was not required.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Observation {
    /// Complete scientific attribution.
    pub claim: Claim,
    /// Bound assembly instance, populated by the executing consumer.
    pub instance: Option<SemanticId>,
    /// Actual inputs of this claim application.
    pub inputs: Vec<Input>,
    /// Evidence classification, independent of authorization.
    pub outcome: Outcome,
    /// This claim was required, rather than a nonwinning union alternative.
    pub required: bool,
    /// Authorizing declarations that matched this claim's exact target.
    pub permissions: Vec<Permission>,
    /// Both permissions remain independent even for a mixed dependency result.
    pub unknown_allowed: bool,
    /// Whether leaving this particular claim's region was permitted.
    pub extrapolation_allowed: bool,
    /// Whether the low-level evidence gate admitted this required claim.
    pub admitted: bool,
}
/// A claim plan uses library-evaluated predicate results, not a second numeric engine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Region {
    /// Index into the mathematical program's boolean predicate outputs.
    Predicate(usize),
    /// A declaration explicitly states unrestricted evidence.
    Unrestricted,
    /// Missing evidence or a declared unknown reason.
    Unknown,
    /// Only a declaration can combine alternatives into one evidence union.
    Union(Vec<Node>),
}
/// Required dependencies are intersections, never implicit fit unions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    /// Attribution of this instantiated claim.
    pub claim: Claim,
    /// Its declaration-owned evidence form.
    pub region: Region,
    /// Independently required evidence.
    pub dependencies: Vec<Node>,
    /// Indices of actual numerical argument values to retain as observations.
    pub inputs: Vec<(String, usize, SemanticId)>,
    /// Exact permissions visible in the application's consumer scope.
    pub permissions: Vec<Permission>,
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
        fn visit(
            n: &Node,
            predicates: usize,
            inputs: usize,
            left: &mut usize,
            depth: usize,
        ) -> bool {
            if *left == 0 || depth > 512 {
                return false;
            }
            *left -= 1;
            if n.claim.owner_lineage.first() != Some(&n.claim.owner)
                || n.claim
                    .owner_lineage
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    != n.claim.owner_lineage.len()
                || matches!(n.region,Region::Predicate(i) if i>=predicates)
                || n.inputs.iter().any(|(_, i, _)| *i >= inputs)
            {
                return false;
            }
            if let Region::Union(children) = &n.region
                && !children
                    .iter()
                    .all(|n| visit(n, predicates, inputs, left, depth + 1))
            {
                return false;
            }
            n.dependencies
                .iter()
                .all(|n| visit(n, predicates, inputs, left, depth + 1))
        }
        {
            let mut remaining = budget;
            visit(self, predicates, inputs, &mut remaining, 0)
        }
    }
    /// Owned plan metadata, separate from shared expression/library storage.
    pub fn retained_bytes(&self) -> usize {
        let own = self.claim.owner_lineage.capacity() * size_of::<SemanticId>()
            + self.claim.records.capacity() * size_of::<SemanticId>()
            + self.claim.dependencies.capacity() * size_of::<SemanticId>()
            + self.claim.reason.as_ref().map_or(0, String::capacity)
            + self.inputs.capacity() * size_of::<(String, usize, SemanticId)>()
            + self
                .inputs
                .iter()
                .map(|(n, _, _)| n.capacity())
                .sum::<usize>()
            + self.permissions.capacity() * size_of::<Permission>()
            + self
                .permissions
                .iter()
                .map(|p| p.targets.capacity() * size_of::<SemanticId>())
                .sum::<usize>();
        let alternatives = match &self.region {
            Region::Union(c) => {
                c.capacity() * size_of::<Node>() + c.iter().map(Node::retained_bytes).sum::<usize>()
            }
            _ => 0,
        };
        own + alternatives
            + self.dependencies.capacity() * size_of::<Node>()
            + self
                .dependencies
                .iter()
                .map(Node::retained_bytes)
                .sum::<usize>()
    }
    /// Conservative retained evaluation extent, including nonwinning alternatives.
    pub fn observation_bytes(&self) -> usize {
        let own = size_of::<Observation>()
            + self.claim.owner_lineage.len() * size_of::<SemanticId>()
            + self.claim.records.len() * size_of::<SemanticId>()
            + self.claim.dependencies.len() * size_of::<SemanticId>()
            + self.claim.reason.as_ref().map_or(0, String::len)
            + self.inputs.len() * size_of::<Input>()
            + self.inputs.iter().map(|(n, _, _)| n.len()).sum::<usize>()
            + self.permissions.len() * size_of::<Permission>()
            + self
                .permissions
                .iter()
                .map(|p| p.targets.len() * size_of::<SemanticId>())
                .sum::<usize>();
        let alternatives = match &self.region {
            Region::Union(c) => c.iter().fold(0usize, |b, n| {
                b.saturating_add(n.observation_bytes().saturating_mul(2))
            }),
            _ => 0,
        };
        self.dependencies
            .iter()
            .fold(own.saturating_add(alternatives), |b, n| {
                b.saturating_add(n.observation_bytes())
            })
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
    fn observe(
        &self,
        predicates: &[bool],
        values: &[f64],
        required: bool,
        assessment: &mut Assessment,
    ) {
        let outcome = self.classification(predicates);
        let permissions = self
            .permissions
            .iter()
            .filter(|p| p.covers(&self.claim))
            .cloned()
            .collect::<Vec<_>>();
        let unknown_allowed = permissions.iter().any(|p| p.allow_unknown);
        let extrapolation_allowed = permissions.iter().any(|p| p.allow_extrapolation);
        let admitted = !required
            || match outcome {
                Outcome::Applicable => true,
                Outcome::UnknownEvidence => unknown_allowed,
                Outcome::OutsideRegion => extrapolation_allowed,
            };
        if !admitted {
            assessment.refused.push(assessment.observations.len());
        }
        assessment.observations.push(Observation {
            claim: self.claim.clone(),
            instance: None,
            inputs: self
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
            permissions,
            unknown_allowed,
            extrapolation_allowed,
            admitted,
        });
        if let Region::Union(children) = &self.region {
            // Nonwinning fit regions are retained, but do not individually gate a union.
            let winner = children
                .iter()
                .position(|n| n.effective_classification(predicates) == outcome);
            for (i, child) in children.iter().enumerate() {
                child.observe(
                    predicates,
                    values,
                    required && winner == Some(i),
                    assessment,
                );
            }
        }
        for dependency in &self.dependencies {
            dependency.observe(predicates, values, required, assessment);
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
