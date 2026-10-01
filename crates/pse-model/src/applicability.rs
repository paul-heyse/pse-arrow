// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Evidence claims and scoped permissions, independent of numerical execution.
use pse_ids::{FramedHasher, SemanticId};
use crate::generated::enums::{ModelingApplicabilityBasis as Basis, ModelingApplicabilityOutcome as Outcome, ModelingPermissionTarget as Target, ModelingValidityLayer as Layer};

/// Immutable attribution of an instantiated scientific claim.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Claim {
    /// Absent exactly for undeclared evidence, which is unknown.
    pub id: Option<SemanticId>,
    /// Joint whole-interval application, only for a verified declared interval.
    pub coverage: Option<SemanticId>,
    /// Declared scientific family or model owner.
    pub owner: SemanticId,
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
            Target::Families => self.targets.contains(&claim.owner),
            Target::Records => !claim.records.is_empty() && claim.records.iter().all(|id| self.targets.contains(id)),
        }
    }
}
/// Numerical inputs are recorded in the lowering's canonical physical units.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Input { /// Declared argument name.
    pub name: String, /// Actual value at the demanded application.
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
pub struct Assessment { /// Every demanded claim and declared alternative.
    pub observations: Vec<Observation>, /// Indices of inadmissible required observations.
    pub refused: Vec<usize>,
}
impl Node {
    /// Preserve full declaration/selection/permission identity in compiled caches.
    pub fn frame(&self, h: &mut FramedHasher) {
        h.str("applicability-node-v1");
        h.bool(self.claim.id.is_some()); if let Some(id)=self.claim.id { h.id(&id); }
        h.bool(self.claim.coverage.is_some()); if let Some(id)=self.claim.coverage { h.id(&id); }
        h.id(&self.claim.owner).id(&self.claim.form).id(&self.claim.call);
        h.bool(self.claim.evidence.is_some()); if let Some(id)=self.claim.evidence { h.id(&id); }
        h.str(self.claim.layer.as_str()).str(self.claim.basis.map_or("",|v|v.as_str())).str(self.claim.reason.as_deref().unwrap_or(""));
        for ids in [&self.claim.records,&self.claim.dependencies] { h.u64(ids.len() as u64); for id in ids { h.id(id); } }
        match &self.region { Region::Predicate(i)=>{h.str("predicate").u64(*i as u64);},Region::Unrestricted=>{h.str("unrestricted");},Region::Unknown=>{h.str("unknown");},Region::Union(children)=>{h.str("union").u64(children.len() as u64);for child in children {child.frame(h);}} }
        h.u64(self.dependencies.len() as u64);for child in &self.dependencies {child.frame(h);}
        h.u64(self.inputs.len() as u64);for (name,index,quantity) in &self.inputs {h.str(name).u64(*index as u64).id(quantity);}
        h.u64(self.permissions.len() as u64);for p in &self.permissions {h.id(&p.id).id(&p.scope).str(p.target_kind.as_str()).bool(p.allow_unknown).bool(p.allow_extrapolation).u64(p.targets.len() as u64);for id in &p.targets {h.id(id);}}
    }
    /// Assess only already evaluated library predicate results and actual input values.
    /// A declared union prefers applicable evidence, then unknown, then outside evidence.
    pub fn assess(&self, predicates: &[bool], values: &[f64]) -> Assessment {
        let mut assessment=Assessment { observations:Vec::new(),refused:Vec::new() };
        self.observe(predicates,values,true,&mut assessment);
        assessment
    }
    fn classification(&self, predicates:&[bool]) -> Outcome {
        if self.claim.id.is_none() {return Outcome::UnknownEvidence;}
        match &self.region {
            Region::Predicate(i) => if predicates.get(*i)==Some(&true) {Outcome::Applicable} else {Outcome::OutsideRegion},
            Region::Unrestricted=>Outcome::Applicable,
            Region::Unknown=>Outcome::UnknownEvidence,
            Region::Union(children)=> {
                let outcomes=children.iter().map(|n|n.effective_classification(predicates)).collect::<Vec<_>>();
                if outcomes.contains(&Outcome::Applicable) {Outcome::Applicable} else if outcomes.contains(&Outcome::UnknownEvidence) || children.is_empty() {Outcome::UnknownEvidence} else {Outcome::OutsideRegion}
            }
        }
    }
    fn effective_classification(&self,predicates:&[bool])->Outcome {
        let own=self.classification(predicates);
        let dependencies=self.dependencies.iter().map(|n|n.effective_classification(predicates)).collect::<Vec<_>>();
        if own==Outcome::UnknownEvidence || dependencies.contains(&Outcome::UnknownEvidence) {Outcome::UnknownEvidence}
        else if own==Outcome::OutsideRegion || dependencies.contains(&Outcome::OutsideRegion) {Outcome::OutsideRegion}
        else {Outcome::Applicable}
    }
    fn observe(&self,predicates:&[bool],values:&[f64],required:bool,assessment:&mut Assessment) {
        let outcome=self.classification(predicates);
        let permissions=self.permissions.iter().filter(|p|p.covers(&self.claim)).cloned().collect::<Vec<_>>();
        let unknown_allowed=permissions.iter().any(|p|p.allow_unknown);
        let extrapolation_allowed=permissions.iter().any(|p|p.allow_extrapolation);
        let admitted=!required || match outcome {Outcome::Applicable=>true,Outcome::UnknownEvidence=>unknown_allowed,Outcome::OutsideRegion=>extrapolation_allowed};
        if !admitted {assessment.refused.push(assessment.observations.len());}
        assessment.observations.push(Observation {claim:self.claim.clone(),instance:None,inputs:self.inputs.iter().filter_map(|(name,i,quantity_type)|values.get(*i).map(|value|Input{name:name.clone(),value:*value,quantity_type:*quantity_type})).collect(),outcome,required,permissions,unknown_allowed,extrapolation_allowed,admitted});
        if let Region::Union(children)=&self.region {
            // Nonwinning fit regions are retained, but do not individually gate a union.
            let winner=children.iter().position(|n|n.effective_classification(predicates)==outcome);
            for (i,child) in children.iter().enumerate() {
                child.observe(predicates,values,required && winner==Some(i),assessment);
            }
        }
        for dependency in &self.dependencies {dependency.observe(predicates,values,required,assessment);}
    }
}

impl Assessment {
    /// Owned observation extent for runtime reservation and retained failures.
    pub fn retained_bytes(&self)->usize {
        std::mem::size_of::<Self>()+self.refused.capacity()*std::mem::size_of::<usize>()+
        self.observations.capacity()*std::mem::size_of::<Observation>()+
        self.observations.iter().map(Observation::retained_bytes).sum::<usize>()
    }
}
impl Observation {
    /// Additional owned capacities beyond the inline observation value.
    pub fn retained_bytes(&self)->usize {
        self.claim.records.capacity()*std::mem::size_of::<SemanticId>()+
        self.claim.dependencies.capacity()*std::mem::size_of::<SemanticId>()+
        self.claim.reason.as_ref().map_or(0,String::capacity)+
        self.inputs.capacity()*std::mem::size_of::<Input>()+self.inputs.iter().map(|i|i.name.capacity()).sum::<usize>()+
        self.permissions.capacity()*std::mem::size_of::<Permission>()+
        self.permissions.iter().map(|p|p.targets.capacity()*std::mem::size_of::<SemanticId>()).sum::<usize>()
    }
}
