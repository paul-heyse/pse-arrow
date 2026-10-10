// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Enumerated numerical families over admitted original oracles. Family preparation
//! owns correspondence and incidence; binding owns numerical anchors and steps.
//! No auxiliary native success is scientific permission for the original problem.
use crate::{
    MathError,
    assembly::CasePlan,
    binding::Target,
    index::{Addend, Entry, GlobalCol, GlobalRow},
    sparse::AssemblyMatrix,
};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_kernels::DerivativeOrder;
use pse_model::strategy::{AccuracyClass, AccuracyDemand, AccuracyEvidence};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, atomic::AtomicBool},
};

/// A coordinate's original interval, retained in native column order.
#[derive(Clone, Debug, PartialEq)]
pub struct Coordinate {
    /// Original semantic variable.
    pub id: SemanticId,
    /// Original closed lower bound, possibly negative infinity.
    pub lower: f64,
    /// Original closed upper bound, possibly positive infinity.
    pub upper: f64,
}
/// A constraint's original interval, retained in oracle row order.
#[derive(Clone, Debug, PartialEq)]
pub struct Constraint {
    /// Original semantic row.
    pub id: SemanticId,
    /// Original closed lower bound.
    pub lower: f64,
    /// Original closed upper bound.
    pub upper: f64,
}
/// Actual supplier support, independent of structural incidence and smoothness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DerivativeSupport {
    /// Highest admitted local derivative order. This does not certify numeric accuracy.
    pub order: DerivativeOrder,
    /// An admitted first derivative action is callable; its supplier owns whether it
    /// internally assembles or computes a directional action.
    pub jacobian_product: bool,
    /// Identity of the derivative producer, including its admitted numerical meaning.
    pub source: ContentHash,
}
/// Obligations retained from the original rather than reinterpreted by a native method.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OriginalObligations {
    /// Original guards and applicability policy, including physical validity.
    pub guards: ContentHash,
    /// Selected-function and branch/sheet meaning.
    pub selection: ContentHash,
    /// Original objective interpretation, when present. Feasibility does not optimize it.
    pub objective: Option<ContentHash>,
}
/// Complete immutable original inventory. Structural zeros survive Value preparation.
#[derive(Clone, Debug, PartialEq)]
pub struct OriginalContract {
    identity: ContentHash,
    normalization: ContentHash,
    coordinates: Vec<Coordinate>,
    constraints: Vec<Constraint>,
    incidence: Vec<Entry<GlobalRow, GlobalCol>>,
    support: DerivativeSupport,
    obligations: OriginalObligations,
}
impl OriginalContract {
    /// Admit a complete inventory supplied by its preparation owner.
    /// # Errors
    /// Empty coordinates, invalid intervals, duplicate identities or out-of-scope edges.
    pub fn new(
        identity: ContentHash,
        normalization: ContentHash,
        coordinates: Vec<Coordinate>,
        constraints: Vec<Constraint>,
        mut incidence: Vec<Entry<GlobalRow, GlobalCol>>,
        support: DerivativeSupport,
        obligations: OriginalObligations,
    ) -> Result<Self, MathError> {
        let valid_interval = |l: f64, u: f64| {
            !l.is_nan() && !u.is_nan() && l <= u && l != f64::INFINITY && u != f64::NEG_INFINITY
        };
        if coordinates.is_empty()
            || coordinates
                .iter()
                .any(|c| !valid_interval(c.lower, c.upper))
            || constraints
                .iter()
                .any(|c| !valid_interval(c.lower, c.upper))
            || coordinates
                .iter()
                .map(|c| c.id)
                .collect::<BTreeSet<_>>()
                .len()
                != coordinates.len()
            || constraints
                .iter()
                .map(|c| c.id)
                .collect::<BTreeSet<_>>()
                .len()
                != constraints.len()
            || incidence
                .iter()
                .any(|e| e.row.get() >= constraints.len() || e.col.get() >= coordinates.len())
            || support.jacobian_product && support.order < DerivativeOrder::First
        {
            return Err(MathError::Contract(
                "invalid original derived-family inventory or derivative support".into(),
            ));
        }
        incidence.sort_unstable();
        incidence.dedup();
        Ok(Self {
            identity,
            normalization,
            coordinates,
            constraints,
            incidence,
            support,
            obligations,
        })
    }
    /// Derive complete all-branch incidence from the existing library preparation owner,
    /// even when the numerical plan is Value-only. Parametric columns retain their edges.
    /// # Errors
    /// Failed incidence preparation or a coordinate without an original variable interval.
    pub fn from_case(
        plan: &CasePlan,
        normalization: ContentHash,
        derivative_source: ContentHash,
        obligations: OriginalObligations,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        let variable_map: BTreeMap<_, _> = plan
            .structure()
            .variables()
            .iter()
            .map(|v| (v.port.id, v))
            .collect();
        let columns: BTreeMap<_, _> = plan
            .columns()
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, GlobalCol::new(i)))
            .collect();
        let rows: BTreeMap<_, _> = plan
            .structure()
            .rows()
            .iter()
            .enumerate()
            .map(|(i, r)| (r.id, GlobalRow::new(i)))
            .collect();
        let coordinates = plan
            .columns()
            .iter()
            .map(|id| {
                let v = variable_map.get(id).ok_or_else(|| {
                    MathError::Contract(
                        "derived solve coordinate is a parameter without a variable interval"
                            .into(),
                    )
                })?;
                Ok(Coordinate {
                    id: *id,
                    lower: v.lower.unwrap_or(f64::NEG_INFINITY),
                    upper: v.upper.unwrap_or(f64::INFINITY),
                })
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        let mut incidence = Vec::new();
        for binding in plan.structure().instances() {
            let outputs = binding
                .contributions
                .iter()
                .map(|c| c.output)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let slots = binding
                .slots
                .iter()
                .enumerate()
                .filter_map(|(i, s)| columns.contains_key(&s.source()).then_some(i))
                .collect::<Vec<_>>();
            let body = plan.bodies().get(&binding.body).ok_or_else(|| {
                MathError::Contract("derived family lacks its admitted body".into())
            })?;
            let structural = body.incidence(&outputs, &slots, cancel)?;
            for contribution in &binding.contributions {
                if let Target::Row(row) = contribution.target {
                    let support = structural
                        .first_for_output(contribution.output)
                        .ok_or_else(|| {
                            MathError::Contract("missing original structural row support".into())
                        })?;
                    for &slot in support {
                        let source = binding
                            .slots
                            .get(slot)
                            .ok_or_else(|| {
                                MathError::Contract(
                                    "structural support references an invalid slot".into(),
                                )
                            })?
                            .source();
                        if let Some(&column) = columns.get(&source) {
                            incidence.push(Entry::new(rows[&row], column));
                        }
                    }
                }
            }
        }
        Self::new(
            plan.structure().key(),
            normalization,
            coordinates,
            plan.structure()
                .rows()
                .iter()
                .map(|r| Constraint {
                    id: r.id,
                    lower: r.lower,
                    upper: r.upper,
                })
                .collect(),
            incidence,
            DerivativeSupport {
                order: plan.order(),
                jacobian_product: plan.order() >= DerivativeOrder::First,
                source: derivative_source,
            },
            obligations,
        )
    }
    /// Complete owned inventory extent, excluding shared external source owners.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.coordinates.capacity() * size_of::<Coordinate>()
            + self.constraints.capacity() * size_of::<Constraint>()
            + self.incidence.capacity() * size_of::<Entry<GlobalRow, GlobalCol>>()
    }
    /// Original identity, never an auxiliary result identity.
    pub fn identity(&self) -> ContentHash {
        self.identity
    }
    /// Original error and coordinate normalization.
    pub fn normalization(&self) -> ContentHash {
        self.normalization
    }
    /// All original variable bounds.
    pub fn coordinates(&self) -> &[Coordinate] {
        &self.coordinates
    }
    /// All original constraint bounds, including inequalities and isolated rows.
    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }
    /// Complete original structural edges, independent of numerical derivative order.
    pub fn incidence(&self) -> &[Entry<GlobalRow, GlobalCol>] {
        &self.incidence
    }
    /// Actual available derivative action support.
    pub fn support(&self) -> DerivativeSupport {
        self.support
    }
    /// Original physical and selected-function obligations.
    pub fn obligations(&self) -> OriginalObligations {
        self.obligations
    }
    /// Exact numerical view obtained by subtracting each named original equality's
    /// finite bound. Physical metadata stays with the original supplier.
    /// # Errors
    /// An original row is not a finite equality with one exact declared offset.
    pub fn zero_residual_view(&self) -> Result<Arc<Self>, MathError> {
        if self
            .constraints
            .iter()
            .any(|row| !row.lower.is_finite() || row.lower.to_bits() != row.upper.to_bits())
        {
            return Err(MathError::Contract(
                "zero residual view requires finite original equality bounds".into(),
            ));
        }
        let mut identity = FramedHasher::new(pse_ids::Frame::DerivedBindingV2);
        identity
            .str("explicit-authored-equality-residual-view")
            .hash(&self.identity);
        for row in &self.constraints {
            identity.id(&row.id).f64(row.lower);
        }
        Ok(Arc::new(Self::new(
            identity.finish_hash(),
            self.normalization,
            self.coordinates.clone(),
            self.constraints
                .iter()
                .map(|row| Constraint {
                    id: row.id,
                    lower: 0.,
                    upper: 0.,
                })
                .collect(),
            self.incidence.clone(),
            self.support,
            self.obligations,
        )?))
    }
    pub(crate) fn frame(&self, h: &mut FramedHasher) {
        h.hash(&self.identity)
            .hash(&self.normalization)
            .hash(&self.support.source)
            .u64(order_tag(self.support.order))
            .bool(self.support.jacobian_product)
            .hash(&self.obligations.guards)
            .hash(&self.obligations.selection)
            .bool(self.obligations.objective.is_some());
        if let Some(objective) = self.obligations.objective {
            h.hash(&objective);
        }
        h.u64(self.coordinates.len() as u64);
        for c in &self.coordinates {
            h.id(&c.id).f64(c.lower).f64(c.upper);
        }
        h.u64(self.constraints.len() as u64);
        for c in &self.constraints {
            h.id(&c.id).f64(c.lower).f64(c.upper);
        }
        frame_edges(h, &self.incidence);
    }
}
/// Opaque mathematical supplier. Errors retain their actual owning type.
/// Guards, provider and selected-sheet checks remain part of these evaluations.
pub trait Oracle: std::fmt::Debug {
    /// Concrete boundary failure; derivative failures are not rewritten as success.
    type Error: From<MathError>;
    /// Full admitted original structure and actual action support.
    fn contract(&self) -> &OriginalContract;
    /// Original constraint values in declared order.
    fn values(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), Self::Error>;
    /// Admitted derivative action, with its original producer's numerical meaning.
    fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), Self::Error>;
}
/// Optional admitted original objective. Projected problems do not acquire an
/// independent block objective merely by selecting rows or coordinates.
pub trait ObjectiveOracle: Oracle {
    /// Actual objective derivative producer and available order.
    fn objective_support(&self) -> DerivativeSupport;
    /// Evaluate the original objective, including its own guards and selected meaning.
    fn objective(&mut self, x: &[f64]) -> Result<f64, Self::Error>;
    /// Apply its actual first derivative to an original-coordinate direction.
    fn objective_product(&mut self, x: &[f64], direction: &[f64]) -> Result<f64, Self::Error>;
}
/// Immutable contract of an admitted reconstruction, usually supplied by an existing
/// implicit selected-function owner. Incidence is from reconstructed original
/// coordinates (rows) to retained solve coordinates (columns), independent of Value.
#[derive(Clone, Debug, PartialEq)]
pub struct ReconstructionContract {
    original: Arc<OriginalContract>,
    source: ContentHash,
    validity: ContentHash,
    retained: Vec<GlobalCol>,
    eliminated: Vec<GlobalRow>,
    incidence: Vec<Entry<GlobalCol, GlobalCol>>,
    support: DerivativeSupport,
    key: ContentHash,
}
/// Actual reconstruction producer and the validity/derivative obligations it owns.
#[derive(Clone, Copy, Debug)]
pub struct ReconstructionProducer {
    /// Actual value reconstruction program/provider identity.
    pub source: ContentHash,
    /// Explicit domain/regularity/selected-function obligations.
    pub validity: ContentHash,
    /// Supplied derivative/action support, independent of elimination admission.
    pub support: DerivativeSupport,
}
impl ReconstructionContract {
    /// Declare the reconstruction owner's actual program, validity obligations and
    /// derivative support. This inventory does not establish regularity or elimination.
    /// The supplier's `admit` operation must establish these obligations at each point.
    /// # Errors
    /// Invalid coordinate/row maps, missing identity edges, invalid incidence/support,
    /// or attempted elimination of a nonzero equality or inequality.
    pub fn new(
        original: Arc<OriginalContract>,
        source: ContentHash,
        validity: ContentHash,
        retained: Vec<GlobalCol>,
        eliminated: Vec<GlobalRow>,
        incidence: Vec<Entry<GlobalCol, GlobalCol>>,
        support: DerivativeSupport,
    ) -> Result<Self, MathError> {
        Self::new_inner(
            original,
            ReconstructionProducer {
                source,
                validity,
                support,
            },
            retained,
            eliminated,
            incidence,
            false,
        )
    }
    /// Admit explicit original equality subtraction metadata supplied by the actual
    /// reconstruction producer. Every named offset must equal the physical RHS;
    /// this metadata alone proves neither source arithmetic nor regularity.
    pub fn new_with_offsets(
        original: Arc<OriginalContract>,
        producer: ReconstructionProducer,
        retained: Vec<GlobalCol>,
        eliminated: Vec<GlobalRow>,
        incidence: Vec<Entry<GlobalCol, GlobalCol>>,
        offsets: &[(GlobalRow, f64)],
    ) -> Result<Self, MathError> {
        if offsets.len() != eliminated.len()
            || offsets
                .iter()
                .zip(&eliminated)
                .any(|((row, offset), expected)| {
                    row != expected
                        || original.constraints.get(row.get()).is_none_or(|r| {
                            !offset.is_finite()
                                || r.lower.to_bits() != offset.to_bits()
                                || r.upper.to_bits() != offset.to_bits()
                        })
                })
        {
            return Err(MathError::Contract(
                "reconstruction eliminated residual offsets differ from original equalities".into(),
            ));
        }
        Self::new_inner(original, producer, retained, eliminated, incidence, true)
    }
    fn new_inner(
        original: Arc<OriginalContract>,
        producer: ReconstructionProducer,
        retained: Vec<GlobalCol>,
        eliminated: Vec<GlobalRow>,
        mut incidence: Vec<Entry<GlobalCol, GlobalCol>>,
        offsets_checked: bool,
    ) -> Result<Self, MathError> {
        let ReconstructionProducer {
            source,
            validity,
            support,
        } = producer;
        if eliminated.is_empty()
            || retained
                .iter()
                .any(|c| c.get() >= original.coordinates.len())
            || eliminated
                .iter()
                .any(|r| r.get() >= original.constraints.len())
            || retained.iter().collect::<BTreeSet<_>>().len() != retained.len()
            || eliminated.iter().collect::<BTreeSet<_>>().len() != eliminated.len()
        {
            return Err(MathError::Contract(
                "invalid reconstruction projection".into(),
            ));
        }
        if retained.len() >= original.coordinates.len()
            || eliminated.iter().any(|r| {
                let c = &original.constraints[r.get()];
                !c.lower.is_finite()
                    || c.lower.to_bits() != c.upper.to_bits()
                    || !offsets_checked && c.lower != 0.0
            })
            || incidence
                .iter()
                .any(|e| e.row.get() >= original.coordinates.len() || e.col.get() >= retained.len())
            || support.jacobian_product && support.order < DerivativeOrder::First
        {
            return Err(MathError::Contract(
                "invalid admitted reduced reconstruction inventory".into(),
            ));
        }
        incidence.sort_unstable();
        incidence.dedup();
        for (local, original_column) in retained.iter().enumerate() {
            if incidence
                .iter()
                .filter(|e| e.row == *original_column)
                .map(|e| e.col.get())
                .collect::<Vec<_>>()
                != [local]
            {
                return Err(MathError::Contract(
                    "retained reduced coordinates must reconstruct by the identity map".into(),
                ));
            }
        }
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedFamilyV1);
        h.str("admitted-reconstruction");
        original.frame(&mut h);
        h.hash(&source)
            .hash(&validity)
            .hash(&support.source)
            .u64(order_tag(support.order))
            .bool(support.jacobian_product);
        frame_projection(&mut h, &retained, &eliminated);
        h.u64(incidence.len() as u64);
        for e in &incidence {
            h.u64(e.row.get() as u64).u64(e.col.get() as u64);
        }
        Ok(Self {
            original,
            source,
            validity,
            retained,
            eliminated,
            incidence,
            support,
            key: h.finish_hash(),
        })
    }
    /// Owned maps/support extent; the original source inventory remains separately owned.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.retained.capacity() * size_of::<GlobalCol>()
            + self.eliminated.capacity() * size_of::<GlobalRow>()
            + self.incidence.capacity() * size_of::<Entry<GlobalCol, GlobalCol>>()
    }
    /// Actual named RHS offsets, mechanically derived from the original authority.
    pub fn residual_offsets(&self) -> Vec<(GlobalRow, f64)> {
        self.eliminated
            .iter()
            .map(|&r| (r, self.original.constraints[r.get()].lower))
            .collect()
    }
    /// Complete immutable reconstruction identity, including validity and actual source.
    pub fn key(&self) -> ContentHash {
        self.key
    }
    /// Original contract including every bound, guard, row and objective obligation.
    pub fn original(&self) -> &OriginalContract {
        &self.original
    }
    /// Actual value reconstruction program/provider identity.
    pub fn source(&self) -> ContentHash {
        self.source
    }
    /// Explicit domain/regularity/selected-function obligations owned by admission.
    pub fn validity(&self) -> ContentHash {
        self.validity
    }
    /// Original coordinates retained as reduced solve coordinates in declared order.
    pub fn retained(&self) -> &[GlobalCol] {
        &self.retained
    }
    /// Original finite equality rows supplied by the admitted reconstruction.
    pub fn eliminated(&self) -> &[GlobalRow] {
        &self.eliminated
    }
    /// Full structural dependence of reconstructed coordinates on reduced coordinates.
    pub fn incidence(&self) -> &[Entry<GlobalCol, GlobalCol>] {
        &self.incidence
    }
    /// Actual reconstruction derivative/action support, not an inferred inverse.
    pub fn support(&self) -> DerivativeSupport {
        self.support
    }
}
/// Actual full local point used to establish reconstruction conditions and selected
/// sheet lineage. This admission readout is unqualified: it supplies no forward
/// accuracy evidence, certificate or consumer refinement guarantee.
#[derive(Clone, Debug)]
pub struct ReconstructionAdmission {
    /// Admitted point in the supplier's full original coordinate order.
    pub values: Vec<f64>,
}
/// A realized reconstructed point or direction with evidence for the exact consumed
/// product. A small eliminated residual is not this forward error evidence.
#[derive(Clone, Debug)]
pub struct ReconstructionObservation {
    /// Values in full original coordinate order.
    pub values: Vec<f64>,
    /// Actual point/action error evidence; estimates are never upgraded to certificates.
    pub accuracy: AccuracyEvidence,
}
/// Producer-issued refusal for one consumed reconstruction product. These are
/// bounded acceleration outcomes, distinct from enclosing task/pool exhaustion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefinementRefusal {
    /// The actual verifier did not establish the requested capability/evidence.
    Unavailable(crate::implicit::SelectionProofRefusal),
    /// The explicitly supplied product round allowance was consumed.
    Rounds,
    /// The explicitly supplied product proof-cell allowance was consumed.
    ProofCells,
    /// Actual controls or interval uncertainty cannot represent this allowance.
    Precision,
}
/// Explicit consumer allowance for one point or one realized action product.
/// Proof cells are shared across its rounds; the original task clock is unchanged.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct RefinementLimits {
    /// Maximum complete enclosure/correction rounds for this consumed product.
    pub rounds: usize,
    /// Maximum actual interval proof cells across all those rounds.
    pub proof_cells: u64,
}
impl RefinementLimits {
    /// Refuse absent finite work rather than choose implicit numerical defaults.
    pub fn validate(self) -> Result<(), MathError> {
        if self.rounds == 0 || self.proof_cells == 0 {
            return Err(MathError::Contract(
                "positive explicit reconstruction refinement allowances required".into(),
            ));
        }
        Ok(())
    }
}
/// Opaque reconstruction boundary. Existing implicit preparation owns the numerical
/// solve, selected root sheet, chart validity and actual derivatives behind this API.
/// This transformation provides neither a nonlinear solver nor an inverse approximation.
pub trait ReconstructionOracle: std::fmt::Debug {
    /// Same concrete failure boundary as its consumed original oracle.
    type Error: From<MathError>;
    /// Actual immutable source, structural map and explicit validity obligations.
    fn contract(&self) -> &ReconstructionContract;
    /// Consumed parameters/provider realization and selected root-sheet lineage.
    /// Numerical cache mutations alone must not change this mathematical identity.
    fn realization(&self) -> ContentHash;
    /// Actual last consumed refinement refusal, for enclosing consumer rebinding.
    /// Success clears this observation; it grants no capability or proof.
    fn refinement_refusal(&self) -> Option<RefinementRefusal> {
        None
    }
    /// Whether the supplier owns a uniform incoming-uncertainty operation.
    fn supports_uncertainty(&self) -> bool {
        false
    }
    /// Establish the declared reconstruction conditions at this actual reduced point.
    /// Unknown regularity or an unavailable/disconnected selected sheet must refuse.
    /// Return the actual local point so dependent suppliers can establish their
    /// conditions before consumed-product identity is captured. This point is not
    /// qualified accuracy evidence and cannot satisfy a consumer demand.
    fn admit(&mut self, x: &[f64]) -> Result<ReconstructionAdmission, Self::Error>;
    /// Reconstruct a full original point using actual consumed forward accuracy.
    fn point(
        &mut self,
        x: &[f64],
        demand: &AccuracyDemand,
        refinement: RefinementLimits,
    ) -> Result<ReconstructionObservation, Self::Error>;
    /// Apply the admitted reconstruction derivative to this actual reduced direction.
    fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        demand: &AccuracyDemand,
        refinement: RefinementLimits,
    ) -> Result<ReconstructionObservation, Self::Error>;
    /// Consume predecessor uncertainty in the same normalized coordinate scales.
    /// The default supplier has only an exact-input contract, so nonzero incoming
    /// error requires a distinct uniform enclosure capability.
    fn uncertain(
        &mut self,
        x: &[f64],
        direction: Option<&[f64]>,
        incoming: ReconstructionUncertainty<'_>,
        demand: &AccuracyDemand,
        refinement: RefinementLimits,
    ) -> Result<ReconstructionObservation, Self::Error> {
        if incoming.point.len() != x.len()
            || incoming.action.is_some_and(|a| a.len() != x.len())
            || incoming
                .point
                .iter()
                .chain(incoming.action.into_iter().flatten())
                .any(|e| !e.is_finite() || *e < 0.0)
        {
            return Err(MathError::Contract(
                "incoming reconstruction uncertainty extent/value".into(),
            )
            .into());
        }
        if incoming
            .point
            .iter()
            .chain(incoming.action.into_iter().flatten())
            .any(|e| *e != 0.0)
        {
            return Err(MathError::Refinement {
                product: demand.product,
                source_key: self.contract().source(),
                validity: self.contract().validity(),
                reason: RefinementRefusal::Unavailable(
                    crate::implicit::SelectionProofRefusal::Unsupported,
                ),
            }
            .into());
        }
        match direction {
            Some(v) => self.jacobian_product(x, v, demand, refinement),
            None => self.point(x, demand, refinement),
        }
    }
}
/// Actual predecessor errors and their consumed evidence class, without upgrading
/// an estimate to a uniform neighborhood certificate.
#[derive(Clone, Copy, Debug)]
pub struct ReconstructionUncertainty<'a> {
    /// Per-input normalized point error.
    pub point: &'a [f64],
    /// Per-input normalized direction error when an action is consumed.
    pub action: Option<&'a [f64]>,
    /// Weakest evidence class of the actual predecessors.
    pub class: AccuracyClass,
}
/// Inner observations consumed by an outer value/action. They retain their original
/// products and numerical class; outer forward accuracy requires its own propagated
/// control (for example `ErrorAmplification`) and original supplier observations.
#[derive(Clone, Debug)]
pub struct ConsumedReconstruction {
    /// Full original coordinates actually consumed by this value/action, in the
    /// original contract order. This moves the existing reconstructed point storage.
    pub original_point: Vec<f64>,
    /// Actual reconstructed point accuracy.
    pub point: AccuracyEvidence,
    /// Actual reconstruction action accuracy, when an action was consumed.
    pub action: Option<AccuracyEvidence>,
}
impl ConsumedReconstruction {
    /// Owned receipt extent. Coordinates reuse the reconstruction's admitted storage;
    /// callers retaining the receipt keep this existing allocation charged.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>() + self.original_point.capacity() * size_of::<f64>()
    }
}
/// Pseudo-time construction declares how mass depends on the current trial state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MassStructure {
    /// Matrix is frozen at the accepted anchor throughout trials and retries.
    Frozen {
        /// Complete nonzero-possible matrix pattern, in original row/column order.
        incidence: Vec<Entry<GlobalRow, GlobalCol>>,
    },
    /// State-dependent mass includes an explicitly supplied derivative action.
    StateDependent {
        /// Actual mass expression/provider identity.
        source: ContentHash,
        /// Complete nonzero-possible matrix pattern.
        incidence: Vec<Entry<GlobalRow, GlobalCol>>,
        /// Support of `DM(x)[v] * velocity`, not just the support of `M(x)`.
        derivative_incidence: Vec<Entry<GlobalRow, GlobalCol>>,
    },
}
/// Actual state-dependent mass supplier. No wrapper invents `DM` from a mass value.
pub trait MassOracle: std::fmt::Debug {
    /// Same concrete boundary failure as the consumed residual supplier.
    type Error: From<MathError>;
    /// Complete supplied mass and derivative structure.
    fn structure(&self) -> &MassStructure;
    /// Consumed mass parameter values and branch/provider realization, distinct from
    /// its immutable structure. Rebinding them must change the bound step's identity.
    fn realization(&self) -> ContentHash;
    /// Evaluate `M(x) * vector`.
    fn apply(&mut self, x: &[f64], vector: &[f64], out: &mut [f64]) -> Result<(), Self::Error>;
    /// Evaluate the bilinear action `DM(x)[direction] * velocity`.
    fn derivative_action(
        &mut self,
        x: &[f64],
        direction: &[f64],
        velocity: &[f64],
        out: &mut [f64],
    ) -> Result<(), Self::Error>;
}
/// Explicit correspondence granted by this family, independently of solver status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Correspondence {
    /// Constraints/bounds are unchanged; original objective and permission remain required.
    ConstraintIdentity,
    /// The terminal homotopy parameter is exactly one; all original equations remain.
    /// Objective stationarity, physical permission and accuracy still require assessment.
    TerminalEquationIdentity,
    /// The fixed parameter and reconstruction map identify a slice of the original.
    /// This grants neither connected-path evidence nor satisfaction at other parameters.
    ReconstructionSlice,
    /// Reconstruction equivalence requires the supplier's explicit validity admission
    /// and consumed point/action accuracy. Original assessment remains necessary.
    ReconstructionUnderValidity,
    /// Selected rows only. A collection of successful block trials does not establish
    /// simultaneous original satisfaction at their reconstructed aggregate point.
    BlockCoverage,
    /// A finite artificial step is only a candidate for original correction/assessment.
    Approximate,
}
/// Enumerated transformations exposed to execution owners.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FamilyClass {
    /// Original constraints with a constant feasibility objective.
    BoundedFeasibility,
    /// An artificial time step over an original zero-equality system.
    ShiftedPseudoTime,
    /// Fix one original coordinate and solve the remaining coordinate slice.
    ParameterContinuation,
    /// Subtract a frozen original residual, fading the subtraction to zero.
    AnchoredHomotopy,
    /// Affinely blend two admitted residual suppliers with a common coordinate domain.
    AffineHomotopy,
    /// Project original rows and columns while freezing external coordinates.
    BlockSubsystem,
    /// Compose original rows/objective with an admitted selected reconstruction.
    ReducedSpace,
}
#[derive(Clone, Debug)]
enum FamilyKind {
    Feasibility,
    PseudoTime {
        pairing: Vec<GlobalCol>,
        sign: f64,
        mass: MassStructure,
    },
    Parameter {
        parameter: GlobalCol,
        columns: Arc<Vec<GlobalCol>>,
        rows: Arc<Vec<GlobalRow>>,
    },
    AnchoredHomotopy,
    AffineHomotopy {
        start: Arc<OriginalContract>,
    },
    Block {
        columns: Arc<Vec<GlobalCol>>,
        rows: Arc<Vec<GlobalRow>>,
        external: Vec<Entry<GlobalRow, GlobalCol>>,
    },
    Reduced {
        reconstruction: Arc<ReconstructionContract>,
        rows: Arc<Vec<GlobalRow>>,
    },
}
/// Immutable enumerated family. Numerical values cannot mutate its structure.
#[derive(Clone, Debug)]
pub struct DerivedFamily {
    original: Arc<OriginalContract>,
    kind: FamilyKind,
    incidence: Vec<Entry<GlobalRow, GlobalCol>>,
    key: ContentHash,
}
impl DerivedFamily {
    /// Retain every original bound, guard, row and selected function under a zero objective.
    pub fn bounded_feasibility(original: Arc<OriginalContract>) -> Self {
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedFamilyV1);
        h.str("bounded-feasibility");
        original.frame(&mut h);
        Self {
            incidence: original.incidence.clone(),
            original,
            kind: FamilyKind::Feasibility,
            key: h.finish_hash(),
        }
    }
    /// Prepare `M(x) * (x-anchor)/step + sign * F(x)` in original normalized units.
    /// Pairing names the artificial flow's row/state correspondence, not attraction.
    /// # Errors
    /// Non-square/nonzero-equality originals, invalid pairing, sign, mass edges or action support.
    pub fn shifted_pseudo_time(
        original: Arc<OriginalContract>,
        pairing: Vec<GlobalCol>,
        sign: f64,
        mut mass: MassStructure,
    ) -> Result<Self, MathError> {
        let n = original.coordinates.len();
        if original.constraints.len() != n
            || original
                .constraints
                .iter()
                .any(|c| c.lower != 0.0 || c.upper != 0.0)
            || original.obligations.objective.is_some()
            || pairing.len() != n
            || pairing.iter().map(|c| c.get()).collect::<BTreeSet<_>>() != (0..n).collect()
            || sign != -1.0 && sign != 1.0
            || !original.support.jacobian_product
        {
            return Err(MathError::Contract("shifted pseudo-time needs a square zero-equality original, explicit row/state pairing, sign and actual JVP".into()));
        }
        let mut incidence = original.incidence.clone();
        let mut normalize =
            |edges: &mut Vec<Entry<GlobalRow, GlobalCol>>| -> Result<(), MathError> {
                if edges.iter().any(|e| e.row.get() >= n || e.col.get() >= n) {
                    return Err(MathError::Contract("mass support dimensions".into()));
                }
                edges.sort_unstable();
                edges.dedup();
                incidence.extend_from_slice(edges);
                Ok(())
            };
        match &mut mass {
            MassStructure::Frozen { incidence } => normalize(incidence)?,
            MassStructure::StateDependent {
                incidence,
                derivative_incidence,
                ..
            } => {
                normalize(incidence)?;
                normalize(derivative_incidence)?;
            }
        }
        incidence.sort_unstable();
        incidence.dedup();
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedFamilyV1);
        h.str("shifted-pseudo-time");
        original.frame(&mut h);
        h.f64(sign).u64(pairing.len() as u64);
        for column in &pairing {
            h.u64(column.get() as u64);
        }
        match &mass {
            MassStructure::Frozen { incidence } => {
                h.str("frozen");
                frame_edges(&mut h, incidence);
            }
            MassStructure::StateDependent {
                source,
                incidence,
                derivative_incidence,
            } => {
                h.str("state-dependent").hash(source);
                frame_edges(&mut h, incidence);
                frame_edges(&mut h, derivative_incidence);
            }
        }
        Ok(Self {
            original,
            kind: FamilyKind::PseudoTime {
                pairing,
                sign,
                mass,
            },
            incidence,
            key: h.finish_hash(),
        })
    }
    /// Prepare a natural parameter slice. Its incidence uses free coordinates first,
    /// followed by the continuation parameter; Value preparation retains every edge.
    /// This is not an arclength tracker and makes no fold or connected-sheet claim.
    /// # Errors
    /// Invalid parameter or no remaining solve coordinate.
    pub fn parameter_continuation(
        original: Arc<OriginalContract>,
        parameter: GlobalCol,
    ) -> Result<Self, MathError> {
        let n = original.coordinates.len();
        if n < 2 || parameter.get() >= n {
            return Err(MathError::Contract("parameter continuation needs an original coordinate and a remaining solve coordinate".into()));
        }
        let columns = (0..n)
            .filter(|&c| c != parameter.get())
            .map(GlobalCol::new)
            .collect::<Vec<_>>();
        let rows = (0..original.constraints.len())
            .map(GlobalRow::new)
            .collect::<Vec<_>>();
        let mut incidence = projected_edges(&original, &columns, &rows);
        incidence.extend(
            original
                .incidence
                .iter()
                .filter(|e| e.col == parameter)
                .map(|e| Entry::new(e.row, GlobalCol::new(columns.len()))),
        );
        incidence.sort_unstable();
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedFamilyV1);
        h.str("natural-parameter-continuation");
        original.frame(&mut h);
        h.u64(parameter.get() as u64);
        Ok(Self {
            original,
            kind: FamilyKind::Parameter {
                parameter,
                columns: Arc::new(columns),
                rows: Arc::new(rows),
            },
            incidence,
            key: h.finish_hash(),
        })
    }
    /// Prepare `F(x) - (1-t)*F(anchor)`. The anchor residual is supplied once when
    /// binding, then frozen. At `t=1` the equations are exactly the original equations.
    /// # Errors
    /// A shifted inequality would change its feasible interval rather than construct
    /// a root homotopy; only zero-equality systems are admitted here.
    pub fn anchored_homotopy(original: Arc<OriginalContract>) -> Result<Self, MathError> {
        zero_equalities(&original)?;
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedFamilyV1);
        h.str("anchored-residual-homotopy");
        original.frame(&mut h);
        let incidence = homotopy_edges(&original, &[]);
        Ok(Self {
            original,
            kind: FamilyKind::AnchoredHomotopy,
            incidence,
            key: h.finish_hash(),
        })
    }
    /// Prepare `(1-t)*G(x) + t*F(x)` using admitted original and start suppliers.
    /// No derivative is inferred from opaque values; available actions are intersected.
    /// # Errors
    /// Incompatible coordinates, normalization or zero-equality row identities.
    pub fn affine_homotopy(
        original: Arc<OriginalContract>,
        start: Arc<OriginalContract>,
    ) -> Result<Self, MathError> {
        zero_equalities(&original)?;
        zero_equalities(&start)?;
        if original.coordinates != start.coordinates
            || original.constraints != start.constraints
            || original.normalization != start.normalization
        {
            return Err(MathError::Contract("affine homotopy suppliers need identical coordinates, zero-equality rows and normalization".into()));
        }
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedFamilyV1);
        h.str("affine-residual-homotopy");
        original.frame(&mut h);
        start.frame(&mut h);
        let incidence = homotopy_edges(&original, &start.incidence);
        Ok(Self {
            original,
            kind: FamilyKind::AffineHomotopy { start },
            incidence,
            key: h.finish_hash(),
        })
    }
    /// Prepare an explicit subsystem, preserving original ordering through declared maps.
    /// External dependencies are retained separately, never inferred to be independent
    /// from a structural matching or erased because current derivatives are zero.
    /// # Errors
    /// Empty, duplicate or out-of-range row/column maps.
    pub fn block_subsystem(
        original: Arc<OriginalContract>,
        columns: Vec<GlobalCol>,
        rows: Vec<GlobalRow>,
    ) -> Result<Self, MathError> {
        validate_projection(&original, &columns, &rows)?;
        let incidence = projected_edges(&original, &columns, &rows);
        let external = original
            .incidence
            .iter()
            .filter(|e| rows.contains(&e.row) && !columns.contains(&e.col))
            .copied()
            .collect();
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedFamilyV1);
        h.str("block-subsystem");
        original.frame(&mut h);
        frame_projection(&mut h, &columns, &rows);
        Ok(Self {
            original,
            kind: FamilyKind::Block {
                columns: Arc::new(columns),
                rows: Arc::new(rows),
                external,
            },
            incidence,
            key: h.finish_hash(),
        })
    }
    /// Compose retained original rows with an admitted reconstruction. Structural
    /// incidence is the Boolean composition of the two actual support inventories;
    /// no numerical matching or current derivative establishes elimination.
    /// # Errors
    /// Original contract mismatch. A zero-row target retains original objective,
    /// bounds and independent assessment obligations.
    pub fn reduced_space(
        original: Arc<OriginalContract>,
        reconstruction: Arc<ReconstructionContract>,
    ) -> Result<Self, MathError> {
        if original.as_ref() != reconstruction.original.as_ref() {
            return Err(MathError::Contract(
                "reduced reconstruction original mismatch".into(),
            ));
        }
        let rows = (0..original.constraints.len())
            .map(GlobalRow::new)
            .filter(|r| !reconstruction.eliminated.contains(r))
            .collect::<Vec<_>>();
        let mut incidence = Vec::new();
        for edge in &original.incidence {
            if let Some(row) = rows.iter().position(|r| r == &edge.row) {
                incidence.extend(
                    reconstruction
                        .incidence
                        .iter()
                        .filter(|map| map.row == edge.col)
                        .map(|map| Entry::new(GlobalRow::new(row), map.col)),
                );
            }
        }
        incidence.sort_unstable();
        incidence.dedup();
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedFamilyV1);
        h.str("selected-reduced-space");
        original.frame(&mut h);
        h.hash(&reconstruction.key);
        frame_projection(&mut h, &reconstruction.retained, &rows);
        Ok(Self {
            original,
            kind: FamilyKind::Reduced {
                reconstruction,
                rows: Arc::new(rows),
            },
            incidence,
            key: h.finish_hash(),
        })
    }
    /// Additional owned family/map extent; original/reconstruction/shared start
    /// contracts are retained and charged by their explicit owners separately.
    pub fn retained_bytes(&self) -> usize {
        let maps = match &self.kind {
            FamilyKind::Feasibility
            | FamilyKind::AnchoredHomotopy
            | FamilyKind::AffineHomotopy { .. } => 0,
            FamilyKind::PseudoTime { pairing, mass, .. } => {
                pairing.capacity() * size_of::<GlobalCol>()
                    + match mass {
                        MassStructure::Frozen { incidence } => {
                            incidence.capacity() * size_of::<Entry<GlobalRow, GlobalCol>>()
                        }
                        MassStructure::StateDependent {
                            incidence,
                            derivative_incidence,
                            ..
                        } => {
                            (incidence.capacity() + derivative_incidence.capacity())
                                * size_of::<Entry<GlobalRow, GlobalCol>>()
                        }
                    }
            }
            FamilyKind::Parameter { columns, rows, .. } => {
                columns.capacity() * size_of::<GlobalCol>()
                    + rows.capacity() * size_of::<GlobalRow>()
                    + 2 * (size_of::<Vec<GlobalRow>>() + 2 * size_of::<usize>())
            }
            FamilyKind::Block {
                columns,
                rows,
                external,
            } => {
                columns.capacity() * size_of::<GlobalCol>()
                    + rows.capacity() * size_of::<GlobalRow>()
                    + external.capacity() * size_of::<Entry<GlobalRow, GlobalCol>>()
                    + 2 * (size_of::<Vec<GlobalRow>>() + 2 * size_of::<usize>())
            }
            FamilyKind::Reduced { rows, .. } => {
                rows.capacity() * size_of::<GlobalRow>()
                    + size_of::<Vec<GlobalRow>>()
                    + 2 * size_of::<usize>()
            }
        };
        size_of::<Self>()
            + self.incidence.capacity() * size_of::<Entry<GlobalRow, GlobalCol>>()
            + maps
    }
    /// Enumerated family class, independent of numerical realization.
    pub fn class(&self) -> FamilyClass {
        match self.kind {
            FamilyKind::Feasibility => FamilyClass::BoundedFeasibility,
            FamilyKind::PseudoTime { .. } => FamilyClass::ShiftedPseudoTime,
            FamilyKind::Parameter { .. } => FamilyClass::ParameterContinuation,
            FamilyKind::AnchoredHomotopy => FamilyClass::AnchoredHomotopy,
            FamilyKind::AffineHomotopy { .. } => FamilyClass::AffineHomotopy,
            FamilyKind::Block { .. } => FamilyClass::BlockSubsystem,
            FamilyKind::Reduced { .. } => FamilyClass::ReducedSpace,
        }
    }
    /// Original coordinates consumed as solve coordinates by a projected binding.
    pub fn coordinate_map(&self) -> Option<&[GlobalCol]> {
        match &self.kind {
            FamilyKind::Parameter { columns, .. } | FamilyKind::Block { columns, .. } => {
                Some(columns)
            }
            FamilyKind::Reduced { reconstruction, .. } => Some(&reconstruction.retained),
            _ => None,
        }
    }
    /// Original rows consumed by a projected binding, in its declared output order.
    pub fn row_map(&self) -> Option<&[GlobalRow]> {
        match &self.kind {
            FamilyKind::Parameter { rows, .. }
            | FamilyKind::Block { rows, .. }
            | FamilyKind::Reduced { rows, .. } => Some(rows),
            _ => None,
        }
    }
    /// Original dependencies on frozen external coordinates for a block subsystem.
    /// These edges use original row/column indices, unlike the local incidence.
    pub fn external_incidence(&self) -> &[Entry<GlobalRow, GlobalCol>] {
        match &self.kind {
            FamilyKind::Block { external, .. } => external,
            _ => &[],
        }
    }
    /// Stable structure identity, independent of anchor, step and frozen matrix values.
    pub fn key(&self) -> ContentHash {
        self.key
    }
    /// Complete original inventory retained by this family.
    pub fn original(&self) -> &OriginalContract {
        &self.original
    }
    /// Family's actual structural incidence. Natural parameter and homotopy families
    /// append their continuation parameter after solve coordinates. Block edges use
    /// local row/column order; external edges remain available independently.
    pub fn incidence(&self) -> &[Entry<GlobalRow, GlobalCol>] {
        &self.incidence
    }
    /// Declared artificial-flow residual sign; absent for every other family.
    pub fn pseudo_sign(&self) -> Option<f64> {
        match &self.kind {
            FamilyKind::PseudoTime { sign, .. } => Some(*sign),
            _ => None,
        }
    }
    /// Prepared mass support and its frozen versus state-dependent meaning.
    pub fn mass_structure(&self) -> Option<&MassStructure> {
        match &self.kind {
            FamilyKind::PseudoTime { mass, .. } => Some(mass),
            _ => None,
        }
    }
    /// Admitted reconstruction owner for a reduced family, when present.
    pub fn reconstruction_contract(&self) -> Option<&ReconstructionContract> {
        match &self.kind {
            FamilyKind::Reduced { reconstruction, .. } => Some(reconstruction),
            _ => None,
        }
    }
    /// Actual support exposed by these adapters. They consume first derivative actions;
    /// original Second support does not manufacture a derived Hessian or mass second derivative.
    pub fn support(&self) -> DerivativeSupport {
        let action = self.original.support.jacobian_product
            && match &self.kind {
                FamilyKind::AffineHomotopy { start } => start.support.jacobian_product,
                FamilyKind::Reduced { reconstruction, .. } => {
                    reconstruction.support.jacobian_product
                }
                _ => true,
            };
        DerivativeSupport {
            order: if action {
                DerivativeOrder::First
            } else {
                DerivativeOrder::Value
            },
            jacobian_product: action,
            source: self.key,
        }
    }
    /// Granted correspondence; neither variant transfers auxiliary objective/status/duals.
    pub fn correspondence(&self) -> Correspondence {
        match self.kind {
            FamilyKind::Feasibility => Correspondence::ConstraintIdentity,
            FamilyKind::PseudoTime { .. } => Correspondence::Approximate,
            FamilyKind::Parameter { .. } => Correspondence::ReconstructionSlice,
            FamilyKind::Block { .. } => Correspondence::BlockCoverage,
            FamilyKind::Reduced { .. } => Correspondence::ReconstructionUnderValidity,
            FamilyKind::AnchoredHomotopy | FamilyKind::AffineHomotopy { .. } => {
                Correspondence::TerminalEquationIdentity
            }
        }
    }
    /// Identity reconstruction in original coordinates, requiring original assessment.
    /// # Errors
    /// Incorrect dimensions or nonfinite coordinates. Bounds are assessed by the original.
    pub fn reconstruct(&self, candidate: &[f64]) -> Result<Vec<f64>, MathError> {
        if self.coordinate_map().is_some() {
            return Err(MathError::Contract(
                "projected reconstruction needs its bound parameter or external coordinates".into(),
            ));
        }
        trial(candidate, self.original.coordinates.len())?;
        Ok(candidate.to_vec())
    }
    /// Bind a feasibility oracle without changing its original intervals or obligations.
    /// # Errors
    /// Wrong family kind or supplier contract.
    pub fn bind_feasibility<O: Oracle>(
        self: &Arc<Self>,
        oracle: O,
    ) -> Result<BoundFeasibility<O>, MathError> {
        if !matches!(self.kind, FamilyKind::Feasibility)
            || oracle.contract() != self.original.as_ref()
        {
            return Err(MathError::Contract(
                "feasibility family/supplier mismatch".into(),
            ));
        }
        Ok(BoundFeasibility {
            family: self.clone(),
            oracle,
        })
    }
    /// Bind one artificial step. Rejected retries reuse the same frozen mass/anchor.
    /// # Errors
    /// Wrong family/supplier, invalid step/anchor or mismatched supplied mass structure.
    pub fn bind_pseudo_time<O: Oracle>(
        self: &Arc<Self>,
        oracle: O,
        anchor: Vec<f64>,
        step: f64,
        mass: MassBinding<O::Error>,
    ) -> Result<BoundPseudoTime<O>, MathError> {
        let FamilyKind::PseudoTime {
            mass: expected,
            sign,
            ..
        } = &self.kind
        else {
            return Err(MathError::Contract("not a pseudo-time family".into()));
        };
        let n = self.original.coordinates.len();
        trial(&anchor, n)?;
        if oracle.contract() != self.original.as_ref() || !step.is_finite() || step <= 0.0 {
            return Err(MathError::Contract("pseudo-time supplier or step".into()));
        }
        match (&mass, expected) {
            (MassBinding::Frozen(matrix), MassStructure::Frozen { incidence })
                if matrix.matrix().nrows() == n
                    && matrix.matrix().ncols() == n
                    && matrix.matrix().val().iter().all(|v| v.is_finite())
                    && matrix_edges(matrix) == *incidence => {}
            (
                MassBinding::StateDependent(supplier),
                expected @ MassStructure::StateDependent { .. },
            ) if supplier.structure() == expected => {}
            _ => {
                return Err(MathError::Contract(
                    "bound mass differs from prepared mass support or semantics".into(),
                ));
            }
        }
        let key = binding_key(self.key, &anchor, step, &mass);
        Ok(BoundPseudoTime {
            family: self.clone(),
            sign: *sign,
            oracle,
            anchor,
            step,
            mass,
            key,
        })
    }
    /// Fix the natural parameter without recompiling the immutable family. Realization
    /// identifies consumed provider/branch/parameter state beyond the explicit value.
    /// # Errors
    /// Wrong family/supplier or parameter outside its original interval.
    pub fn bind_parameter<O: Oracle>(
        self: &Arc<Self>,
        oracle: O,
        value: f64,
        realization: ContentHash,
    ) -> Result<BoundProjection<O>, MathError> {
        let FamilyKind::Parameter {
            parameter,
            columns,
            rows,
        } = &self.kind
        else {
            return Err(MathError::Contract("not a natural parameter family".into()));
        };
        if oracle.contract() != self.original.as_ref() {
            return Err(MathError::Contract("parameter supplier mismatch".into()));
        }
        coordinate_value(&self.original.coordinates[parameter.get()], value)?;
        let mut anchor = vec![0.0; self.original.coordinates.len()];
        anchor[parameter.get()] = value;
        let key = projection_binding_key(self.key, &anchor, realization);
        Ok(BoundProjection {
            family: self.clone(),
            columns: columns.clone(),
            rows: rows.clone(),
            oracle,
            anchor,
            realization,
            key,
        })
    }
    /// Freeze the full original external state for a block trial. Other block trials
    /// cannot mutate it through this binding; retries keep this admitted anchor.
    /// # Errors
    /// Wrong family/supplier, malformed anchor or original bound violation.
    pub fn bind_block<O: Oracle>(
        self: &Arc<Self>,
        oracle: O,
        anchor: Vec<f64>,
        realization: ContentHash,
    ) -> Result<BoundProjection<O>, MathError> {
        let FamilyKind::Block { columns, rows, .. } = &self.kind else {
            return Err(MathError::Contract("not a block family".into()));
        };
        if oracle.contract() != self.original.as_ref() {
            return Err(MathError::Contract("block family/supplier mismatch".into()));
        }
        original_point(&self.original, &anchor)?;
        let key = projection_binding_key(self.key, &anchor, realization);
        Ok(BoundProjection {
            family: self.clone(),
            columns: columns.clone(),
            rows: rows.clone(),
            oracle,
            anchor,
            realization,
            key,
        })
    }
    /// Bind the actual reconstruction supplier without a duplicate implicit solver.
    /// Numerical realization and consumed accuracy remain checked at each operation.
    /// # Errors
    /// Wrong family, original supplier or admitted reconstruction contract.
    pub fn bind_reduced<O: Oracle, R: ReconstructionOracle<Error = O::Error>>(
        self: &Arc<Self>,
        oracle: O,
        reconstruction: R,
        realization: ContentHash,
    ) -> Result<BoundReduced<O, R>, MathError> {
        let FamilyKind::Reduced {
            reconstruction: expected,
            rows,
        } = &self.kind
        else {
            return Err(MathError::Contract("not a reduced family".into()));
        };
        if oracle.contract() != self.original.as_ref()
            || reconstruction.contract() != expected.as_ref()
        {
            return Err(MathError::Contract(
                "reduced supplier contract mismatch".into(),
            ));
        }
        Ok(BoundReduced {
            family: self.clone(),
            rows: rows.clone(),
            oracle,
            reconstruction,
            realization,
        })
    }
    /// Evaluate and freeze the anchor residual using the actual original supplier.
    /// Binding does not infer anchor validity from structural matching.
    pub fn bind_anchored_homotopy<O: Oracle>(
        self: &Arc<Self>,
        mut oracle: O,
        anchor: Vec<f64>,
        parameter: f64,
        realization: ContentHash,
    ) -> Result<BoundHomotopy<O>, O::Error> {
        if !matches!(self.kind, FamilyKind::AnchoredHomotopy)
            || oracle.contract() != self.original.as_ref()
        {
            return Err(
                MathError::Contract("anchored homotopy family/supplier mismatch".into()).into(),
            );
        }
        original_point(&self.original, &anchor)?;
        homotopy_parameter(parameter)?;
        let mut residual = vec![0.0; self.original.constraints.len()];
        oracle.values(&anchor, &mut residual)?;
        finite(&residual)?;
        let data = HomotopyData::Anchored { anchor, residual };
        let key = homotopy_binding_key(self.key, parameter, realization, &data);
        Ok(BoundHomotopy {
            family: self.clone(),
            oracle,
            data,
            parameter,
            realization,
            key,
        })
    }
    /// Bind the actual start supplier with the same concrete failure boundary.
    /// Both suppliers' guard and selection checks execute during residual evaluations.
    /// # Errors
    /// Wrong prepared source, original contract or parameter interval.
    pub fn bind_affine_homotopy<O: Oracle, S: Oracle<Error = O::Error> + 'static>(
        self: &Arc<Self>,
        oracle: O,
        start: S,
        parameter: f64,
        realization: ContentHash,
    ) -> Result<BoundHomotopy<O>, MathError> {
        let FamilyKind::AffineHomotopy { start: expected } = &self.kind else {
            return Err(MathError::Contract("not an affine homotopy family".into()));
        };
        if oracle.contract() != self.original.as_ref() || start.contract() != expected.as_ref() {
            return Err(MathError::Contract(
                "affine homotopy supplier mismatch".into(),
            ));
        }
        homotopy_parameter(parameter)?;
        let data = HomotopyData::Affine {
            start: Box::new(start),
        };
        let key = homotopy_binding_key(self.key, parameter, realization, &data);
        Ok(BoundHomotopy {
            family: self.clone(),
            oracle,
            data,
            parameter,
            realization,
            key,
        })
    }
    /// Explicit row/state correspondence of the artificial flow.
    pub fn pairing(&self) -> Option<&[GlobalCol]> {
        match &self.kind {
            FamilyKind::PseudoTime { pairing, .. } => Some(pairing),
            _ => None,
        }
    }
}
/// Numerical mass binding, separate from the immutable family structure.
#[derive(Debug)]
pub enum MassBinding<E> {
    /// Accepted-anchor matrix, frozen throughout nonlinear trials and step retries.
    Frozen(AssemblyMatrix),
    /// Explicit state-dependent supplier with actual `DM` support.
    StateDependent(Box<dyn MassOracle<Error = E>>),
}
/// Executable bounded feasibility view. Its objective is constant zero.
#[derive(Debug)]
pub struct BoundFeasibility<O> {
    family: Arc<DerivedFamily>,
    oracle: O,
}
impl<O: Oracle> BoundFeasibility<O> {
    /// Retained family and original correspondence.
    pub fn family(&self) -> &Arc<DerivedFamily> {
        &self.family
    }
    /// A feasibility objective carries no original stationarity permission.
    /// # Errors
    /// Malformed trial.
    pub fn objective(&self, x: &[f64]) -> Result<f64, MathError> {
        trial(x, self.family.original.coordinates.len())?;
        Ok(0.0)
    }
    /// Exact gradient of the auxiliary constant objective.
    /// # Errors
    /// Malformed trial or gradient extent.
    pub fn gradient(&self, x: &[f64], out: &mut [f64]) -> Result<(), MathError> {
        self.objective(x)?;
        extent(out, x.len())?;
        out.fill(0.0);
        Ok(())
    }
    /// All original constraints, including inequalities, evaluated by their actual supplier.
    pub fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), O::Error> {
        trial(x, self.family.original.coordinates.len())?;
        extent(out, self.family.original.constraints.len())?;
        self.oracle.values(x, out)?;
        finite(out)?;
        Ok(())
    }
    /// Actual original constraint action; absent support is a refusal, never a zero derivative.
    pub fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), O::Error> {
        action_inputs(self.family.original(), x, direction, out)?;
        self.oracle.jacobian_product(x, direction, out)?;
        finite(out)?;
        Ok(())
    }
    /// Consume the wrapper when returning to original correction/assessment.
    pub fn into_original(self) -> O {
        self.oracle
    }
}
/// Executable natural parameter slice or block subsystem. The supplier always sees
/// complete original points and directions, so opaque guards and sheets stay owned.
#[derive(Debug)]
pub struct BoundProjection<O> {
    family: Arc<DerivedFamily>,
    columns: Arc<Vec<GlobalCol>>,
    rows: Arc<Vec<GlobalRow>>,
    oracle: O,
    anchor: Vec<f64>,
    realization: ContentHash,
    key: ContentHash,
}
impl<O: Oracle> BoundProjection<O> {
    /// Immutable family and complete original obligations.
    pub fn family(&self) -> &Arc<DerivedFamily> {
        &self.family
    }
    /// Identity of this fixed parameter/external state and consumed realization.
    pub fn key(&self) -> ContentHash {
        self.key
    }
    /// Original variable intervals in local solve-coordinate order.
    pub fn coordinates(&self) -> Vec<Coordinate> {
        self.columns()
            .iter()
            .map(|c| self.family.original.coordinates[c.get()].clone())
            .collect()
    }
    /// Original row intervals in local output order, including inequalities.
    pub fn constraints(&self) -> Vec<Constraint> {
        self.rows()
            .iter()
            .map(|r| self.family.original.constraints[r.get()].clone())
            .collect()
    }
    fn columns(&self) -> &[GlobalCol] {
        &self.columns
    }
    fn rows(&self) -> &[GlobalRow] {
        &self.rows
    }
    /// Reconstruct every original coordinate, preserving the bound external state.
    /// Original variable intervals are enforced before any supplier evaluation.
    /// # Errors
    /// Incorrect dimensions, nonfinite values or violated original bounds.
    pub fn reconstruct(&self, x: &[f64]) -> Result<Vec<f64>, MathError> {
        trial(x, self.columns().len())?;
        let mut original = self.anchor.clone();
        for (c, &value) in self.columns().iter().zip(x) {
            original[c.get()] = value;
        }
        original_point(&self.family.original, &original)?;
        Ok(original)
    }
    fn lift_direction(&self, direction: &[f64]) -> Result<Vec<f64>, MathError> {
        trial(direction, self.columns().len())?;
        let mut original = vec![0.0; self.anchor.len()];
        for (c, &value) in self.columns().iter().zip(direction) {
            original[c.get()] = value;
        }
        Ok(original)
    }
    fn project_rows(&self, original: &[f64], out: &mut [f64]) -> Result<(), MathError> {
        extent(out, self.rows().len())?;
        for (r, value) in self.rows().iter().zip(out) {
            *value = original[r.get()];
        }
        Ok(())
    }
    /// Evaluate selected rows at their fully reconstructed original point.
    /// Full original evaluation deliberately retains coupled guard obligations.
    pub fn values(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), O::Error> {
        extent(out, self.rows().len())?;
        let original = self.reconstruct(x)?;
        let mut values = vec![0.0; self.family.original.constraints.len()];
        self.oracle.values(&original, &mut values)?;
        finite(&values)?;
        self.project_rows(&values, out)?;
        Ok(())
    }
    /// Exact projection of the admitted original action, including all original
    /// contributions to selected rows. The wrapper performs no finite differencing.
    pub fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), O::Error> {
        let original = self.reconstruct(x)?;
        let lifted = self.lift_direction(direction)?;
        extent(out, self.rows().len())?;
        let mut values = vec![0.0; self.family.original.constraints.len()];
        action_inputs(&self.family.original, &original, &lifted, &values)?;
        self.oracle
            .jacobian_product(&original, &lifted, &mut values)?;
        finite(&values)?;
        self.project_rows(&values, out)?;
        Ok(())
    }
    /// Apply the actual derivative with respect to the fixed natural parameter.
    /// This is the continuation column, distinct from the free-coordinate action.
    pub fn parameter_product(
        &mut self,
        x: &[f64],
        direction: f64,
        out: &mut [f64],
    ) -> Result<(), O::Error> {
        let FamilyKind::Parameter { parameter, .. } = self.family.kind else {
            return Err(
                MathError::Contract("block binding has no continuation parameter".into()).into(),
            );
        };
        let original = self.reconstruct(x)?;
        finite(&[direction])?;
        extent(out, self.rows().len())?;
        let mut lifted = vec![0.0; self.anchor.len()];
        lifted[parameter.get()] = direction;
        let mut values = vec![0.0; self.family.original.constraints.len()];
        action_inputs(&self.family.original, &original, &lifted, &values)?;
        self.oracle
            .jacobian_product(&original, &lifted, &mut values)?;
        finite(&values)?;
        self.project_rows(&values, out)?;
        Ok(())
    }
    /// Rebind the natural parameter while preserving the prepared family/supplier.
    /// # Errors
    /// Block binding or value outside the original parameter interval.
    pub fn rebind_parameter(mut self, value: f64) -> Result<Self, MathError> {
        let FamilyKind::Parameter { parameter, .. } = self.family.kind else {
            return Err(MathError::Contract(
                "block binding has no continuation parameter".into(),
            ));
        };
        coordinate_value(&self.family.original.coordinates[parameter.get()], value)?;
        self.anchor[parameter.get()] = value;
        self.key = projection_binding_key(self.family.key, &self.anchor, self.realization);
        Ok(self)
    }
    /// Evaluate all original constraints after reconstruction. A block's own successful
    /// rows cannot replace this simultaneous original observation or its assessment.
    pub fn original_values(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), O::Error> {
        let original = self.reconstruct(x)?;
        extent(out, self.family.original.constraints.len())?;
        self.oracle.values(&original, out)?;
        finite(out)?;
        Ok(())
    }
    /// Return the original supplier for original correction/assessment.
    pub fn into_original(self) -> O {
        self.oracle
    }
}
impl<O: ObjectiveOracle> BoundProjection<O> {
    /// Evaluate the coupled original objective at the reconstructed full state.
    /// This is not a proof that block optimization is independent.
    pub fn original_objective(&mut self, x: &[f64]) -> Result<f64, O::Error> {
        if self.family.original.obligations.objective.is_none() {
            return Err(MathError::Contract("original has no declared objective".into()).into());
        }
        let original = self.reconstruct(x)?;
        let value = self.oracle.objective(&original)?;
        finite(&[value])?;
        Ok(value)
    }
    /// Apply the admitted original objective derivative to a lifted local direction.
    pub fn original_objective_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
    ) -> Result<f64, O::Error> {
        let support = self.oracle.objective_support();
        if self.family.original.obligations.objective.is_none()
            || !support.jacobian_product
            || support.order < DerivativeOrder::First
        {
            return Err(
                MathError::Contract("original objective action is unavailable".into()).into(),
            );
        }
        let original = self.reconstruct(x)?;
        let lifted = self.lift_direction(direction)?;
        let value = self.oracle.objective_product(&original, &lifted)?;
        finite(&[value])?;
        Ok(value)
    }
}
/// Executable reduced composition with explicit inner accuracy consumption. Methods
/// return inner observations, not a certificate for the outer product. The execution
/// owner propagates them into each actual consumed outer accuracy demand.
#[derive(Debug)]
pub struct BoundReduced<O, R> {
    family: Arc<DerivedFamily>,
    rows: Arc<Vec<GlobalRow>>,
    oracle: O,
    reconstruction: R,
    realization: ContentHash,
}
impl<O: Oracle, R: ReconstructionOracle<Error = O::Error>> BoundReduced<O, R> {
    /// Immutable reduced family and retained original obligations.
    pub fn family(&self) -> &Arc<DerivedFamily> {
        &self.family
    }
    /// Actual reconstruction contract used for full-coordinate bound rows.
    pub fn reconstruction_contract(&self) -> &ReconstructionContract {
        self.reconstruction.contract()
    }
    /// Bound original provider/parameter realization and selected reconstruction lineage.
    pub fn key(&self) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str("selected-reduced-space")
            .hash(&self.family.key)
            .hash(&self.realization)
            .hash(&self.reconstruction.realization());
        h.finish_hash()
    }
    /// Actual retained original bounds, in reduced solve-coordinate order.
    pub fn coordinates(&self) -> Vec<Coordinate> {
        self.reconstruction
            .contract()
            .retained
            .iter()
            .map(|c| self.family.original.coordinates[c.get()].clone())
            .collect()
    }
    /// Every remaining original row interval, including all inequalities.
    pub fn constraints(&self) -> Vec<Constraint> {
        self.rows()
            .iter()
            .map(|r| self.family.original.constraints[r.get()].clone())
            .collect()
    }
    fn rows(&self) -> &[GlobalRow] {
        &self.rows
    }
    fn input(&self, x: &[f64]) -> Result<(), MathError> {
        let FamilyKind::Reduced {
            reconstruction: expected,
            ..
        } = &self.family.kind
        else {
            return Err(MathError::Contract("not a reduced binding".into()));
        };
        if self.reconstruction.contract() != expected.as_ref() {
            return Err(MathError::Contract(
                "immutable reconstruction source/validity contract changed after binding".into(),
            ));
        }
        let retained = &self.reconstruction.contract().retained;
        extent(x, retained.len())?;
        for (c, &value) in retained.iter().zip(x) {
            coordinate_value(&self.family.original.coordinates[c.get()], value)?;
        }
        Ok(())
    }
    fn realization(&self, x: &[f64]) -> ContentHash {
        projection_binding_key(self.family.key, x, self.key())
    }
    /// Bind the actual initial selected-root lineage before constructing consumed
    /// product demands. Later admissions must preserve this lineage through the
    /// supplier's certified transport rather than changing a demand during evaluation.
    pub fn prepare_reconstruction(&mut self, x: &[f64]) -> Result<(), O::Error> {
        self.input(x)?;
        self.reconstruction.admit(x)?;
        Ok(())
    }
    /// Exact scope of the reconstructed-point product. The caller supplies its actual
    /// consumer allowance/class using this identity, never a universal inner tolerance.
    /// # Errors
    /// Malformed reduced point or original retained bound violation.
    pub fn point_product(&self, x: &[f64]) -> Result<AccuracyProduct, MathError> {
        self.input(x)?;
        Ok(AccuracyProduct {
            problem: self.family.key,
            realization: self.realization(x),
            source: self.reconstruction.contract().source,
            normalization: self.family.original.normalization,
            outputs: self
                .family
                .original
                .coordinates
                .iter()
                .map(|c| c.id)
                .collect(),
            action: AccuracyAction::Values,
        })
    }
    /// Exact scope of the actual reconstruction action, including realized direction.
    /// # Errors
    /// Malformed point/direction or unavailable actual reconstruction action.
    pub fn action_product(
        &self,
        x: &[f64],
        direction: &[f64],
    ) -> Result<AccuracyProduct, MathError> {
        self.input(x)?;
        trial(direction, x.len())?;
        if !self.reconstruction.contract().support.jacobian_product {
            return Err(MathError::Contract(
                "reconstruction derivative action is unavailable".into(),
            ));
        }
        Ok(AccuracyProduct {
            problem: self.family.key,
            realization: self.realization(x),
            source: self.reconstruction.contract().support.source,
            normalization: self.family.original.normalization,
            outputs: self
                .family
                .original
                .coordinates
                .iter()
                .map(|c| c.id)
                .collect(),
            action: AccuracyAction::JacobianProduct {
                direction: projection_binding_key(
                    self.family.key,
                    direction,
                    self.reconstruction.realization(),
                ),
            },
        })
    }
    fn demand(&self, product: &AccuracyProduct, demand: &AccuracyDemand) -> Result<(), MathError> {
        demand
            .validate()
            .map_err(|e| MathError::Contract(e.to_string()))?;
        if demand.product != product.key()? || demand.normalization != product.normalization {
            return Err(MathError::Contract(
                "reconstruction accuracy demand does not cover the actual consumed product".into(),
            ));
        }
        Ok(())
    }
    /// Admit and reconstruct using the existing supplier, checking its actual accuracy,
    /// retained identity coordinates and every original variable interval.
    /// Unavailable validity or forward accuracy refuses before original execution.
    pub fn reconstruct(
        &mut self,
        x: &[f64],
        demand: &AccuracyDemand,
        refinement: RefinementLimits,
    ) -> Result<ReconstructionObservation, O::Error> {
        refinement.validate()?;
        let product = self.point_product(x)?;
        self.demand(&product, demand)?;
        self.reconstruction.admit(x)?;
        let point = self.reconstruction.point(x, demand, refinement)?;
        if self.point_product(x)?.key()? != product.key()? {
            return Err(MathError::Contract(
                "reconstruction changed consumed parameters or selected sheet during evaluation"
                    .into(),
            )
            .into());
        }
        original_point(&self.family.original, &point.values)?;
        for (c, &value) in self.reconstruction.contract().retained.iter().zip(x) {
            if point.values[c.get()] != value {
                return Err(MathError::Contract(
                    "reconstruction violated its retained coordinate identity map".into(),
                )
                .into());
            }
        }
        observed_accuracy(point.accuracy, demand)?;
        Ok(point)
    }
    fn action(
        &mut self,
        x: &[f64],
        direction: &[f64],
        demand: &AccuracyDemand,
        refinement: RefinementLimits,
    ) -> Result<ReconstructionObservation, O::Error> {
        refinement.validate()?;
        let product = self.action_product(x, direction)?;
        self.demand(&product, demand)?;
        self.reconstruction.admit(x)?;
        let action = self
            .reconstruction
            .jacobian_product(x, direction, demand, refinement)?;
        if self.action_product(x, direction)?.key()? != product.key()? {
            return Err(MathError::Contract(
                "reconstruction changed consumed parameters or selected sheet during action".into(),
            )
            .into());
        }
        trial(&action.values, self.family.original.coordinates.len())?;
        for (c, &value) in self
            .reconstruction
            .contract()
            .retained
            .iter()
            .zip(direction)
        {
            if action.values[c.get()] != value {
                return Err(MathError::Contract(
                    "reconstruction action violated its retained identity derivative".into(),
                )
                .into());
            }
        }
        observed_accuracy(action.accuracy, demand)?;
        Ok(action)
    }
    fn project_rows(&self, values: &[f64], out: &mut [f64]) {
        for (r, value) in self.rows().iter().zip(out) {
            *value = values[r.get()];
        }
    }
    /// Evaluate retained rows at the actual reconstructed full state. The original
    /// supplier evaluates all rows so removed rows' guards/sheets are not discarded.
    pub fn values(
        &mut self,
        x: &[f64],
        demand: &AccuracyDemand,
        refinement: RefinementLimits,
        out: &mut [f64],
    ) -> Result<ConsumedReconstruction, O::Error> {
        extent(out, self.rows().len())?;
        let point = self.reconstruct(x, demand, refinement)?;
        let mut values = vec![0.0; self.family.original.constraints.len()];
        self.oracle.values(&point.values, &mut values)?;
        finite(&values)?;
        self.project_rows(&values, out);
        Ok(ConsumedReconstruction {
            original_point: point.values,
            point: point.accuracy,
            action: None,
        })
    }
    /// Compose `DF(reconstruct(x)) * Dreconstruct(x)[direction]` using both actual
    /// actions. No finite inverse, finite difference or bespoke implicit solver is used.
    pub fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        point_demand: &AccuracyDemand,
        action_demand: &AccuracyDemand,
        refinement: RefinementLimits,
        out: &mut [f64],
    ) -> Result<ConsumedReconstruction, O::Error> {
        extent(out, self.rows().len())?;
        if !self.family.support().jacobian_product {
            return Err(MathError::Contract(
                "reduced composition derivative action is unavailable".into(),
            )
            .into());
        }
        let point = self.reconstruct(x, point_demand, refinement)?;
        let action = self.action(x, direction, action_demand, refinement)?;
        let mut values = vec![0.0; self.family.original.constraints.len()];
        action_inputs(
            &self.family.original,
            &point.values,
            &action.values,
            &values,
        )?;
        self.oracle
            .jacobian_product(&point.values, &action.values, &mut values)?;
        finite(&values)?;
        self.project_rows(&values, out);
        Ok(ConsumedReconstruction {
            original_point: point.values,
            point: point.accuracy,
            action: Some(action.accuracy),
        })
    }
    /// Full original row/action composition and its realized reconstruction action.
    /// Native projected rows and reconstructed coordinate bounds consume this same
    /// product so a second interval action proof is not needed for bound rows.
    pub fn original_composition_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        point_demand: &AccuracyDemand,
        action_demand: &AccuracyDemand,
        refinement: RefinementLimits,
    ) -> Result<(Vec<f64>, ReconstructionObservation, ConsumedReconstruction), O::Error> {
        if !self.family.support().jacobian_product {
            return Err(MathError::Contract(
                "reduced composition derivative action is unavailable".into(),
            )
            .into());
        }
        let point = self.reconstruct(x, point_demand, refinement)?;
        let action = self.action(x, direction, action_demand, refinement)?;
        let mut values = vec![0.0; self.family.original.constraints.len()];
        action_inputs(
            &self.family.original,
            &point.values,
            &action.values,
            &values,
        )?;
        self.oracle
            .jacobian_product(&point.values, &action.values, &mut values)?;
        finite(&values)?;
        let consumed = ConsumedReconstruction {
            original_point: point.values,
            point: point.accuracy,
            action: Some(action.accuracy),
        };
        Ok((values, action, consumed))
    }
    /// Observe all original rows simultaneously, including eliminated equalities.
    /// Conditional reconstruction never substitutes for final original assessment.
    pub fn original_values(
        &mut self,
        x: &[f64],
        demand: &AccuracyDemand,
        refinement: RefinementLimits,
        out: &mut [f64],
    ) -> Result<ConsumedReconstruction, O::Error> {
        extent(out, self.family.original.constraints.len())?;
        let point = self.reconstruct(x, demand, refinement)?;
        self.oracle.values(&point.values, out)?;
        finite(out)?;
        Ok(ConsumedReconstruction {
            original_point: point.values,
            point: point.accuracy,
            action: None,
        })
    }
    /// Consume the wrappers before original correction/assessment or owner reuse.
    pub fn into_suppliers(self) -> (O, R) {
        (self.oracle, self.reconstruction)
    }
}
impl<O: ObjectiveOracle, R: ReconstructionOracle<Error = O::Error>> BoundReduced<O, R> {
    /// Evaluate the coupled original objective after full reconstruction.
    pub fn original_objective(
        &mut self,
        x: &[f64],
        demand: &AccuracyDemand,
        refinement: RefinementLimits,
    ) -> Result<(f64, ConsumedReconstruction), O::Error> {
        if self.family.original.obligations.objective.is_none() {
            return Err(MathError::Contract("original has no declared objective".into()).into());
        }
        let point = self.reconstruct(x, demand, refinement)?;
        let value = self.oracle.objective(&point.values)?;
        finite(&[value])?;
        Ok((
            value,
            ConsumedReconstruction {
                original_point: point.values,
                point: point.accuracy,
                action: None,
            },
        ))
    }
    /// Compose the actual objective and reconstruction actions, retaining inner errors.
    pub fn original_objective_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        point_demand: &AccuracyDemand,
        action_demand: &AccuracyDemand,
        refinement: RefinementLimits,
    ) -> Result<(f64, ConsumedReconstruction), O::Error> {
        let support = self.oracle.objective_support();
        if self.family.original.obligations.objective.is_none()
            || !support.jacobian_product
            || support.order < DerivativeOrder::First
        {
            return Err(
                MathError::Contract("original objective action is unavailable".into()).into(),
            );
        }
        let point = self.reconstruct(x, point_demand, refinement)?;
        let action = self.action(x, direction, action_demand, refinement)?;
        let value = self
            .oracle
            .objective_product(&point.values, &action.values)?;
        finite(&[value])?;
        Ok((
            value,
            ConsumedReconstruction {
                original_point: point.values,
                point: point.accuracy,
                action: Some(action.accuracy),
            },
        ))
    }
}
#[derive(Debug)]
enum HomotopyData<E> {
    Anchored {
        anchor: Vec<f64>,
        residual: Vec<f64>,
    },
    Affine {
        start: Box<dyn Oracle<Error = E>>,
    },
}
/// Bound residual homotopy. First actions compose admitted suppliers; Second support
/// is not synthesized. Only parameter exactly one grants terminal equation identity.
#[derive(Debug)]
pub struct BoundHomotopy<O: Oracle> {
    family: Arc<DerivedFamily>,
    oracle: O,
    data: HomotopyData<O::Error>,
    parameter: f64,
    realization: ContentHash,
    key: ContentHash,
}
impl<O: Oracle> BoundHomotopy<O> {
    /// Immutable family and original obligations.
    pub fn family(&self) -> &Arc<DerivedFamily> {
        &self.family
    }
    /// Consumed parameter, anchor residual and provider/branch realization identity.
    pub fn key(&self) -> ContentHash {
        self.key
    }
    /// Current homotopy parameter in the closed interval zero to one.
    pub fn parameter(&self) -> f64 {
        self.parameter
    }
    /// Exact terminal condition; near-one numerical progress is still auxiliary.
    pub fn is_terminal(&self) -> bool {
        self.parameter == 1.0
    }
    /// Actual current guarantee. Equation identity never transfers stationarity/duals.
    pub fn correspondence(&self) -> Correspondence {
        if self.is_terminal() {
            Correspondence::TerminalEquationIdentity
        } else {
            Correspondence::Approximate
        }
    }
    /// Rebind a homotopy parameter without rebuilding its programs or anchor residual.
    /// # Errors
    /// Parameter outside the finite closed unit interval.
    pub fn rebind_parameter(mut self, parameter: f64) -> Result<Self, MathError> {
        homotopy_parameter(parameter)?;
        self.parameter = parameter;
        self.key = homotopy_binding_key(self.family.key, parameter, self.realization, &self.data);
        Ok(self)
    }
    /// Preserve every original coordinate and its bounds before original assessment.
    /// # Errors
    /// Malformed point or original bound violation.
    pub fn reconstruct(&self, x: &[f64]) -> Result<Vec<f64>, MathError> {
        original_point(&self.family.original, x)?;
        Ok(x.to_vec())
    }
    /// Evaluate actual residual suppliers with their original guards and sheets.
    /// At terminal one, the start supplier is not consumed and cannot obstruct `F(x)`.
    pub fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), O::Error> {
        original_point(&self.family.original, x)?;
        extent(out, self.family.original.constraints.len())?;
        let mut original = vec![0.0; out.len()];
        self.oracle.values(x, &mut original)?;
        finite(&original)?;
        if self.is_terminal() {
            out.copy_from_slice(&original);
            return Ok(());
        }
        let mut result = original;
        match &mut self.data {
            HomotopyData::Anchored { residual, .. } => {
                for (value, anchor) in result.iter_mut().zip(residual) {
                    *value -= (1.0 - self.parameter) * *anchor;
                }
            }
            HomotopyData::Affine { start } => {
                let mut initial = vec![0.0; out.len()];
                start.values(x, &mut initial)?;
                finite(&initial)?;
                for (value, initial) in result.iter_mut().zip(initial) {
                    *value = self.parameter * *value + (1.0 - self.parameter) * initial;
                }
            }
        }
        finite(&result)?;
        out.copy_from_slice(&result);
        Ok(())
    }
    /// Apply the composed first derivative with respect to original coordinates.
    pub fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), O::Error> {
        original_point(&self.family.original, x)?;
        action_inputs(&self.family.original, x, direction, out)?;
        let mut result = vec![0.0; out.len()];
        self.oracle.jacobian_product(x, direction, &mut result)?;
        finite(&result)?;
        if !self.is_terminal()
            && let HomotopyData::Affine { start } = &mut self.data
        {
            if !start.contract().support.jacobian_product {
                return Err(MathError::Contract(
                    "homotopy start derivative action is unavailable".into(),
                )
                .into());
            }
            let mut initial = vec![0.0; out.len()];
            start.jacobian_product(x, direction, &mut initial)?;
            finite(&initial)?;
            for (value, initial) in result.iter_mut().zip(initial) {
                *value = self.parameter * *value + (1.0 - self.parameter) * initial;
            }
        }
        finite(&result)?;
        out.copy_from_slice(&result);
        Ok(())
    }
    /// Apply the parameter column: frozen `F(anchor)`, or `F(x)-G(x)`.
    /// It consumes values, so its availability does not invent an original state JVP.
    pub fn parameter_product(
        &mut self,
        x: &[f64],
        direction: f64,
        out: &mut [f64],
    ) -> Result<(), O::Error> {
        original_point(&self.family.original, x)?;
        finite(&[direction])?;
        extent(out, self.family.original.constraints.len())?;
        let mut original = vec![0.0; out.len()];
        self.oracle.values(x, &mut original)?;
        finite(&original)?;
        let mut result = match &mut self.data {
            HomotopyData::Anchored { residual, .. } => residual.clone(),
            HomotopyData::Affine { start } => {
                let mut initial = vec![0.0; out.len()];
                start.values(x, &mut initial)?;
                finite(&initial)?;
                original.iter().zip(initial).map(|(f, g)| f - g).collect()
            }
        };
        for value in &mut result {
            *value *= direction;
        }
        finite(&result)?;
        out.copy_from_slice(&result);
        Ok(())
    }
    /// All original constraints, independent of homotopy progress or residual zeros.
    pub fn original_values(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), O::Error> {
        original_point(&self.family.original, x)?;
        extent(out, self.family.original.constraints.len())?;
        self.oracle.values(x, out)?;
        finite(out)?;
        Ok(())
    }
    /// Consume the wrapper before original correction/assessment.
    pub fn into_original(self) -> O {
        self.oracle
    }
}
/// Executable artificial step; successful evaluation does not imply an original root.
#[derive(Debug)]
pub struct BoundPseudoTime<O: Oracle> {
    family: Arc<DerivedFamily>,
    sign: f64,
    oracle: O,
    anchor: Vec<f64>,
    step: f64,
    mass: MassBinding<O::Error>,
    key: ContentHash,
}
impl<O: Oracle> BoundPseudoTime<O> {
    /// Family structure and retained original obligations.
    pub fn family(&self) -> &Arc<DerivedFamily> {
        &self.family
    }
    /// Anchor/step/mass realization identity, distinct from stable family structure.
    pub fn key(&self) -> ContentHash {
        self.key
    }
    /// Accepted original-coordinate anchor, immutable during trials/retries.
    pub fn anchor(&self) -> &[f64] {
        &self.anchor
    }
    /// Current positive artificial step.
    pub fn step(&self) -> f64 {
        self.step
    }
    /// Retry with another step; accepted anchor and frozen mass are preserved.
    /// # Errors
    /// Nonpositive/nonfinite step.
    pub fn rebind_step(mut self, step: f64) -> Result<Self, MathError> {
        if !step.is_finite() || step <= 0.0 {
            return Err(MathError::Contract("pseudo-time retry step".into()));
        }
        self.step = step;
        self.key = binding_key(self.family.key, &self.anchor, step, &self.mass);
        Ok(self)
    }
    fn sign(&self) -> f64 {
        self.sign
    }
    fn velocity(&self, x: &[f64]) -> Result<Vec<f64>, MathError> {
        trial(x, self.anchor.len())?;
        let v = x
            .iter()
            .zip(&self.anchor)
            .map(|(x, anchor)| (x - anchor) / self.step)
            .collect::<Vec<_>>();
        finite(&v)?;
        Ok(v)
    }
    fn mass_apply(&mut self, x: &[f64], vector: &[f64], out: &mut [f64]) -> Result<(), O::Error> {
        match &mut self.mass {
            MassBinding::Frozen(matrix) => matrix.product(vector, out)?,
            MassBinding::StateDependent(supplier) => supplier.apply(x, vector, out)?,
        }
        finite(out)?;
        Ok(())
    }
    /// Evaluate the artificial residual with actual original guard/provider checks.
    pub fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), O::Error> {
        let velocity = self.velocity(x)?;
        extent(out, velocity.len())?;
        let mut original = vec![0.0; out.len()];
        self.oracle.values(x, &mut original)?;
        finite(&original)?;
        let mut shifted = vec![0.0; out.len()];
        self.mass_apply(x, &velocity, &mut shifted)?;
        let sign = self.sign();
        for (v, f) in shifted.iter_mut().zip(original) {
            *v += sign * f;
        }
        finite(&shifted)?;
        out.copy_from_slice(&shifted);
        Ok(())
    }
    /// Apply `M(x)*v/step + sign*DF(x)[v] + DM(x)[v]*(x-anchor)/step`.
    /// Frozen-mass bindings omit `DM` by construction. The wrapper calls the supplier
    /// action directly; supplier-internal assembly remains that producer's responsibility.
    pub fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), O::Error> {
        action_inputs(self.family.original(), x, direction, out)?;
        let velocity = self.velocity(x)?;
        let mut original = vec![0.0; out.len()];
        self.oracle.jacobian_product(x, direction, &mut original)?;
        finite(&original)?;
        let scaled = direction.iter().map(|v| v / self.step).collect::<Vec<_>>();
        finite(&scaled)?;
        let mut shifted = vec![0.0; out.len()];
        self.mass_apply(x, &scaled, &mut shifted)?;
        if let MassBinding::StateDependent(supplier) = &mut self.mass {
            let mut dm = vec![0.0; out.len()];
            supplier.derivative_action(x, direction, &velocity, &mut dm)?;
            finite(&dm)?;
            for (v, d) in shifted.iter_mut().zip(dm) {
                *v += d;
            }
        }
        let sign = self.sign();
        for (v, f) in shifted.iter_mut().zip(original) {
            *v += sign * f;
        }
        finite(&shifted)?;
        out.copy_from_slice(&shifted);
        Ok(())
    }
    /// Materialize the admitted action into the family's canonical CSC structure on an
    /// explicit assembly demand. Calls one supplier action per supported column.
    /// # Errors
    /// Supplier failure, nonfinite derivative, missing support edge or native index limit.
    pub fn jacobian(&mut self, x: &[f64], native_index: usize) -> Result<AssemblyMatrix, O::Error> {
        let n = self.anchor.len();
        trial(x, n)?;
        let mut matrix = AssemblyMatrix::new(n, n, &self.family.incidence, native_index)?;
        let mut direction = vec![0.0; n];
        let mut product = vec![0.0; n];
        for col in 0..n {
            direction[col] = 1.0;
            self.jacobian_product(x, &direction, &mut product)?;
            direction[col] = 0.0;
            for (row, value) in product.iter().enumerate() {
                if *value != 0.0
                    && self
                        .family
                        .incidence
                        .binary_search(&Entry::new(GlobalRow::new(row), GlobalCol::new(col)))
                        .is_err()
                {
                    return Err(MathError::Contract(
                        "derivative action lies outside prepared family incidence".into(),
                    )
                    .into());
                }
            }
            for (k, edge) in self
                .family
                .incidence
                .iter()
                .enumerate()
                .filter(|(_, edge)| edge.col.get() == col)
            {
                matrix.add(Addend::new(k), product[edge.row.get()])?;
            }
        }
        Ok(matrix)
    }
    /// Consume the auxiliary realization before original correction/assessment.
    pub fn into_original(self) -> O {
        self.oracle
    }
}

/// Numerical control combining local inverse amplification and nonlinear remainder.
/// A condition-number estimate alone is dimensionless and cannot replace inverse norm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ErrorAmplification {
    /// Backward-error product to which the inverse/remainder applies.
    pub input_product: ContentHash,
    /// Consumed forward-error product covered by this control.
    pub output_product: ContentHash,
    /// Shared normalization of the supplied inverse and error observations.
    pub normalization: ContentHash,
    /// Norm of the local inverse in the demand's actual normalized coordinates.
    pub inverse_norm: f64,
    /// Admitted contraction bound/estimate on the nonlinear remainder, strictly below one.
    pub remainder: f64,
    /// Established class for these numbers. Estimated library conditioning stays estimated.
    pub class: AccuracyClass,
}
impl ErrorAmplification {
    /// Amplify the supplied backward error into root/output error; numerical provenance
    /// and a small residual alone cannot yield a certified forward bound.
    /// # Errors
    /// Invalid amplification, unresolved evidence or mismatched normalization.
    pub fn propagate(
        self,
        input: AccuracyEvidence,
        demand: &AccuracyDemand,
    ) -> Result<AccuracyEvidence, MathError> {
        self.validate()?;
        demand
            .validate()
            .map_err(|e| MathError::Contract(e.to_string()))?;
        if input.product != self.input_product
            || demand.product != self.output_product
            || input.normalization != self.normalization
            || demand.normalization != self.normalization
        {
            return Err(MathError::Contract(
                "backward/forward accuracy product or normalization mismatch".into(),
            ));
        }
        let class = combined_class(self.class, input.class);
        let error = input
            .error
            .filter(|e| e.is_finite() && *e >= 0.0)
            .map(|e| {
                if class == AccuracyClass::Certified && e > 0.0 {
                    // Upper error / lower positive contraction slack. Ordinary rounding
                    // cannot establish a conservative certified forward allowance.
                    ((e * self.inverse_norm).next_up() / (1.0 - self.remainder).next_down())
                        .next_up()
                } else {
                    e * self.inverse_norm / (1.0 - self.remainder)
                }
            })
            .filter(|e| e.is_finite());
        Ok(AccuracyEvidence {
            product: demand.product,
            normalization: demand.normalization,
            class: if error.is_some() {
                class
            } else {
                AccuracyClass::Unresolved
            },
            error,
        })
    }
    /// Required inner backward-error allowance derived from consumed forward accuracy.
    /// # Errors
    /// Unavailable evidence class or invalid quantities. No fixed tightening factor.
    pub fn inner_allowance(self, demand: &AccuracyDemand) -> Result<f64, MathError> {
        self.validate()?;
        demand
            .validate()
            .map_err(|e| MathError::Contract(e.to_string()))?;
        if demand.product != self.output_product || demand.normalization != self.normalization {
            return Err(MathError::Contract(
                "inner allowance product or normalization mismatch".into(),
            ));
        }
        if demand.class == AccuracyClass::Certified && self.class != AccuracyClass::Certified {
            return Err(MathError::Contract(
                "certified accuracy requires certified inverse/remainder evidence".into(),
            ));
        }
        let allowance = if demand.class == AccuracyClass::Certified && demand.allowance > 0.0 {
            // Demands round down: returning a larger inner allowance would weaken
            // the consumed certified contract. Clamp representational underflow to zero.
            ((demand.allowance * (1.0 - self.remainder).next_down()).next_down()
                / self.inverse_norm)
                .next_down()
                .max(0.0)
        } else {
            demand.allowance * (1.0 - self.remainder) / self.inverse_norm
        };
        finite(&[allowance])?;
        Ok(allowance)
    }
    fn validate(self) -> Result<(), MathError> {
        if !self.inverse_norm.is_finite()
            || self.inverse_norm <= 0.0
            || !self.remainder.is_finite()
            || !(0.0..1.0).contains(&self.remainder)
            || self.class == AccuracyClass::Unresolved
        {
            Err(MathError::Contract(
                "unresolved or invalid local inverse/remainder accuracy control".into(),
            ))
        } else {
            Ok(())
        }
    }
}
/// Mathematical product consumed by an accuracy demand. Source provenance remains
/// separate from the numerical error evidence covering the realized product.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccuracyAction {
    /// Selected output values, without a derivative claim.
    Values,
    /// Partials in explicit ordered semantic derivative coordinates.
    Partials {
        /// Admitted first or second order.
        order: DerivativeOrder,
        /// Actual derivative coordinate identities.
        coordinates: Vec<SemanticId>,
    },
    /// First derivative applied to this realized direction.
    JacobianProduct {
        /// Canonical actual direction value identity.
        direction: ContentHash,
    },
}
/// Complete product scope used by consumed accuracy, not a substitute for evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccuracyProduct {
    /// Original or bound-derived problem identity.
    pub problem: ContentHash,
    /// Consumed current point, parameters and selected branch/sheet realization.
    pub realization: ContentHash,
    /// Actual mathematical/derivative source identity.
    pub source: ContentHash,
    /// Coordinate and error normalization.
    pub normalization: ContentHash,
    /// Selected semantic outputs in consumer order.
    pub outputs: Vec<SemanticId>,
    /// Actual action and derivative demand.
    pub action: AccuracyAction,
}
impl AccuracyProduct {
    /// Frame exact consumed scope with the dedicated product identity owner.
    /// # Errors
    /// Missing/duplicated outputs, Value-labelled partials, empty/duplicate coordinates.
    pub fn key(&self) -> Result<ContentHash, MathError> {
        if self.outputs.is_empty()
            || self.outputs.iter().collect::<BTreeSet<_>>().len() != self.outputs.len()
        {
            return Err(MathError::Contract(
                "accuracy product needs explicit distinct outputs".into(),
            ));
        }
        let mut h = FramedHasher::new(pse_ids::Frame::AccuracyProductV1);
        h.hash(&self.problem)
            .hash(&self.realization)
            .hash(&self.source)
            .hash(&self.normalization)
            .u64(self.outputs.len() as u64);
        for output in &self.outputs {
            h.id(output);
        }
        match &self.action {
            AccuracyAction::Values => {
                h.str("values");
            }
            AccuracyAction::Partials { order, coordinates } => {
                if *order < DerivativeOrder::First
                    || coordinates.is_empty()
                    || coordinates.iter().collect::<BTreeSet<_>>().len() != coordinates.len()
                {
                    return Err(MathError::Contract(
                        "accuracy partials need derivative order and distinct coordinates".into(),
                    ));
                }
                h.str("partials")
                    .u64(order_tag(*order))
                    .u64(coordinates.len() as u64);
                for coordinate in coordinates {
                    h.id(coordinate);
                }
            }
            AccuracyAction::JacobianProduct { direction } => {
                h.str("jacobian-product").hash(direction);
            }
        }
        Ok(h.finish_hash())
    }
}
fn combined_class(a: AccuracyClass, b: AccuracyClass) -> AccuracyClass {
    match (a, b) {
        (AccuracyClass::Certified, AccuracyClass::Certified) => AccuracyClass::Certified,
        (AccuracyClass::Unresolved, _) | (_, AccuracyClass::Unresolved) => {
            AccuracyClass::Unresolved
        }
        _ => AccuracyClass::Estimated,
    }
}
fn order_tag(order: DerivativeOrder) -> u64 {
    match order {
        DerivativeOrder::Value => 0,
        DerivativeOrder::First => 1,
        DerivativeOrder::Second => 2,
    }
}
fn frame_edges(h: &mut FramedHasher, edges: &[Entry<GlobalRow, GlobalCol>]) {
    h.u64(edges.len() as u64);
    for e in edges {
        h.u64(e.row.get() as u64).u64(e.col.get() as u64);
    }
}
fn matrix_edges(matrix: &AssemblyMatrix) -> Vec<Entry<GlobalRow, GlobalCol>> {
    let m = matrix.matrix();
    let mut edges = (0..m.ncols())
        .flat_map(|col| {
            m.row_idx_of_col(col)
                .map(move |row| Entry::new(GlobalRow::new(row), GlobalCol::new(col)))
        })
        .collect::<Vec<_>>();
    edges.sort_unstable();
    edges
}
fn binding_key<E: From<MathError>>(
    family: ContentHash,
    anchor: &[f64],
    step: f64,
    mass: &MassBinding<E>,
) -> ContentHash {
    let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
    h.hash(&family).u64(anchor.len() as u64);
    for value in anchor {
        h.f64(*value);
    }
    h.f64(step);
    match mass {
        MassBinding::Frozen(matrix) => {
            h.str("frozen").u64(matrix.matrix().val().len() as u64);
            for value in matrix.matrix().val() {
                h.f64(*value);
            }
        }
        MassBinding::StateDependent(supplier) => {
            h.str("state-dependent").hash(&supplier.realization());
        }
    }
    h.finish_hash()
}
fn projection_binding_key(
    family: ContentHash,
    anchor: &[f64],
    realization: ContentHash,
) -> ContentHash {
    let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
    h.hash(&family).hash(&realization).u64(anchor.len() as u64);
    for value in anchor {
        h.f64(*value);
    }
    h.finish_hash()
}
fn homotopy_binding_key<E>(
    family: ContentHash,
    parameter: f64,
    realization: ContentHash,
    data: &HomotopyData<E>,
) -> ContentHash {
    let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
    h.hash(&family).hash(&realization).f64(parameter);
    match data {
        HomotopyData::Anchored { anchor, residual } => {
            h.str("anchor").u64(anchor.len() as u64);
            for value in anchor {
                h.f64(*value);
            }
            h.u64(residual.len() as u64);
            for value in residual {
                h.f64(*value);
            }
        }
        HomotopyData::Affine { .. } => {
            h.str("affine");
        }
    }
    h.finish_hash()
}
fn zero_equalities(original: &OriginalContract) -> Result<(), MathError> {
    if original.constraints.is_empty()
        || original
            .constraints
            .iter()
            .any(|c| c.lower != 0.0 || c.upper != 0.0)
    {
        Err(MathError::Contract("residual homotopy requires original zero equalities; inequalities remain original obligations".into()))
    } else {
        Ok(())
    }
}
fn homotopy_parameter(parameter: f64) -> Result<(), MathError> {
    if parameter.is_finite() && (0.0..=1.0).contains(&parameter) {
        Ok(())
    } else {
        Err(MathError::Contract(
            "homotopy parameter must be in the closed unit interval".into(),
        ))
    }
}
fn homotopy_edges(
    original: &OriginalContract,
    extra: &[Entry<GlobalRow, GlobalCol>],
) -> Vec<Entry<GlobalRow, GlobalCol>> {
    let mut edges = original.incidence.clone();
    edges.extend_from_slice(extra);
    edges.extend((0..original.constraints.len()).map(|r| {
        Entry::new(
            GlobalRow::new(r),
            GlobalCol::new(original.coordinates.len()),
        )
    }));
    edges.sort_unstable();
    edges.dedup();
    edges
}
fn validate_projection(
    original: &OriginalContract,
    columns: &[GlobalCol],
    rows: &[GlobalRow],
) -> Result<(), MathError> {
    if columns.is_empty()
        || rows.is_empty()
        || columns
            .iter()
            .any(|c| c.get() >= original.coordinates.len())
        || rows.iter().any(|r| r.get() >= original.constraints.len())
        || columns.iter().collect::<BTreeSet<_>>().len() != columns.len()
        || rows.iter().collect::<BTreeSet<_>>().len() != rows.len()
    {
        Err(MathError::Contract(
            "invalid original block row/coordinate projection".into(),
        ))
    } else {
        Ok(())
    }
}
fn projected_edges(
    original: &OriginalContract,
    columns: &[GlobalCol],
    rows: &[GlobalRow],
) -> Vec<Entry<GlobalRow, GlobalCol>> {
    let mut edges = original
        .incidence
        .iter()
        .filter_map(|e| {
            Some(Entry::new(
                GlobalRow::new(rows.iter().position(|r| r == &e.row)?),
                GlobalCol::new(columns.iter().position(|c| c == &e.col)?),
            ))
        })
        .collect::<Vec<_>>();
    edges.sort_unstable();
    edges
}
fn frame_projection(h: &mut FramedHasher, columns: &[GlobalCol], rows: &[GlobalRow]) {
    h.u64(columns.len() as u64);
    for c in columns {
        h.u64(c.get() as u64);
    }
    h.u64(rows.len() as u64);
    for r in rows {
        h.u64(r.get() as u64);
    }
}
fn coordinate_value(coordinate: &Coordinate, value: f64) -> Result<(), MathError> {
    if value.is_finite() && value >= coordinate.lower && value <= coordinate.upper {
        Ok(())
    } else {
        Err(MathError::Domain {
            source_id: coordinate.id,
            requirement: "reconstructed coordinate violates its original interval",
        })
    }
}
fn original_point(original: &OriginalContract, x: &[f64]) -> Result<(), MathError> {
    extent(x, original.coordinates.len())?;
    for (coordinate, &value) in original.coordinates.iter().zip(x) {
        coordinate_value(coordinate, value)?;
    }
    Ok(())
}
fn observed_accuracy(evidence: AccuracyEvidence, demand: &AccuracyDemand) -> Result<(), MathError> {
    if evidence.satisfies(demand) {
        Ok(())
    } else {
        Err(MathError::Contract(
            "actual reconstruction accuracy is unavailable or does not satisfy its consumed demand"
                .into(),
        ))
    }
}
fn trial(x: &[f64], n: usize) -> Result<(), MathError> {
    extent(x, n)?;
    finite(x)
}
fn extent(x: &[f64], n: usize) -> Result<(), MathError> {
    if x.len() == n {
        Ok(())
    } else {
        Err(MathError::Contract("derived oracle dimensions".into()))
    }
}
fn finite(x: &[f64]) -> Result<(), MathError> {
    if x.iter().all(|v| v.is_finite()) {
        Ok(())
    } else {
        Err(MathError::Contract(
            "nonfinite derived oracle value/action".into(),
        ))
    }
}
fn action_inputs(
    original: &OriginalContract,
    x: &[f64],
    direction: &[f64],
    out: &[f64],
) -> Result<(), MathError> {
    trial(x, original.coordinates.len())?;
    trial(direction, x.len())?;
    extent(out, original.constraints.len())?;
    if !original.support.jacobian_product {
        return Err(MathError::Contract(
            "original derivative action is unavailable".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn hash(n: u8) -> ContentHash {
        ContentHash::from_bytes([n; 32])
    }
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    fn edge(r: usize, c: usize) -> Entry<GlobalRow, GlobalCol> {
        Entry::new(r.into(), c.into())
    }
    fn original(
        order: DerivativeOrder,
        objective: bool,
        inequality: bool,
    ) -> Arc<OriginalContract> {
        Arc::new(
            OriginalContract::new(
                hash(1),
                hash(2),
                vec![Coordinate {
                    id: id(1),
                    lower: -2.0,
                    upper: 3.0,
                }],
                vec![Constraint {
                    id: id(2),
                    lower: 0.0,
                    upper: if inequality { 5.0 } else { 0.0 },
                }],
                vec![edge(0, 0)],
                DerivativeSupport {
                    order,
                    jacobian_product: order >= DerivativeOrder::First,
                    source: hash(3),
                },
                OriginalObligations {
                    guards: hash(4),
                    selection: hash(5),
                    objective: objective.then_some(hash(6)),
                },
            )
            .unwrap(),
        )
    }
    #[derive(Debug)]
    struct Quadratic {
        contract: Arc<OriginalContract>,
        jvp_calls: usize,
    }
    impl Oracle for Quadratic {
        type Error = MathError;
        fn contract(&self) -> &OriginalContract {
            &self.contract
        }
        fn values(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), MathError> {
            out[0] = x[0] * x[0] - 1.0;
            Ok(())
        }
        fn jacobian_product(
            &mut self,
            x: &[f64],
            direction: &[f64],
            out: &mut [f64],
        ) -> Result<(), MathError> {
            self.jvp_calls += 1;
            out[0] = 2.0 * x[0] * direction[0];
            Ok(())
        }
    }
    fn frozen(value: f64) -> AssemblyMatrix {
        let mut m = AssemblyMatrix::new(1, 1, &[edge(0, 0)], 100).unwrap();
        m.add(Addend::new(0), value).unwrap();
        m
    }
    fn oracle(contract: &Arc<OriginalContract>) -> Quadratic {
        Quadratic {
            contract: contract.clone(),
            jvp_calls: 0,
        }
    }
    #[test]
    fn feasibility_retains_bounds_objective_and_reconstruction_without_terminal_permission() {
        let original = original(DerivativeOrder::First, true, true);
        let family = Arc::new(DerivedFamily::bounded_feasibility(original.clone()));
        let mut bound = family.bind_feasibility(oracle(&original)).unwrap();
        assert_eq!(family.original().coordinates()[0].lower, -2.0);
        assert_eq!(family.original().constraints()[0].upper, 5.0);
        assert_eq!(family.original().obligations().objective, Some(hash(6)));
        assert_eq!(family.correspondence(), Correspondence::ConstraintIdentity);
        assert_eq!(bound.objective(&[2.0]).unwrap(), 0.0);
        let mut out = [99.0];
        bound.gradient(&[2.0], &mut out).unwrap();
        assert_eq!(out, [0.0]);
        bound.constraints(&[2.0], &mut out).unwrap();
        assert_eq!(out, [3.0]);
        bound.jacobian_product(&[2.0], &[3.0], &mut out).unwrap();
        assert_eq!(out, [12.0]);
        assert_eq!(family.reconstruct(&[2.0]).unwrap(), vec![2.0]);
        assert!(family.reconstruct(&[f64::NAN]).is_err());
    }
    #[test]
    fn value_incidence_is_retained_but_missing_jvp_is_refused() {
        let original = original(DerivativeOrder::Value, false, false);
        let family = Arc::new(DerivedFamily::bounded_feasibility(original.clone()));
        assert_eq!(family.incidence(), [edge(0, 0)]);
        let mut bound = family.bind_feasibility(oracle(&original)).unwrap();
        assert!(bound.jacobian_product(&[1.0], &[1.0], &mut [0.0]).is_err());
        assert!(
            DerivedFamily::shifted_pseudo_time(
                original,
                vec![GlobalCol::new(0)],
                1.0,
                MassStructure::Frozen {
                    incidence: vec![edge(0, 0)]
                }
            )
            .is_err()
        );
    }
    #[test]
    fn value_case_plan_uses_library_incidence_without_preparing_unused_second_derivatives() {
        use crate::{
            assembly::AssemblyLimits,
            binding::{
                CaseLimits, CaseStructure, Contribution, InstanceBinding, Row, SlotBinding,
                Variable,
            },
            typed::{Binary, BodyBuilder, BodyLimits},
        };
        use pse_kernels::Port;
        use pse_model::generated::enums::ModelingVariableDomain;
        use pse_quantity::{
            IndexSet,
            standard::{StandardInvariantChecker, ids, standard_registry},
        };
        let registry = standard_registry().unwrap();
        let quantity = ids::quantity("neutral");
        let port = |n| Port {
            id: id(n),
            quantity,
            unit: registry.quantity_type(quantity).unwrap().canonical_unit,
        };
        let mut body = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            1,
            BodyLimits::default(),
        )
        .unwrap();
        let x = body.input(0, quantity, IndexSet::new(), id(30)).unwrap();
        let square = body
            .binary(Binary::Mul, x.clone(), x, None, id(31))
            .unwrap();
        let prepared = Arc::new(body.prepare(&[square]).unwrap());
        let structure = Arc::new(
            CaseStructure::new(
                vec![Variable {
                    port: port(1),
                    fixed: false,
                    domain: ModelingVariableDomain::Continuous,
                    lower: Some(-2.0),
                    upper: Some(3.0),
                }],
                vec![],
                vec![InstanceBinding {
                    checked_members: Default::default(),
                    instance: id(4),
                    body: hash(40),
                    slots: (vec![SlotBinding::new(&port(1), &port(1), &registry).unwrap()]).into(),
                    contributions: vec![Contribution {
                        output: 0,
                        target: Target::Row(id(2)),
                        scale: 1.0,
                    }],
                }],
                vec![Row {
                    id: id(2),
                    quantity,
                    lower: 0.0,
                    upper: 0.0,
                }],
                None,
                CaseLimits::default(),
            )
            .unwrap(),
        );
        let cancel = Arc::new(AtomicBool::new(false));
        let plan = CasePlan::prepare(
            structure,
            BTreeMap::from([(hash(40), prepared)]),
            &registry,
            DerivativeOrder::Value,
            AssemblyLimits::default(),
            &cancel,
        )
        .unwrap();
        assert!(plan.jacobian_pattern().row_idx().is_empty());
        let contract = OriginalContract::from_case(
            &plan,
            hash(2),
            hash(3),
            OriginalObligations {
                guards: hash(4),
                selection: hash(5),
                objective: None,
            },
            &cancel,
        )
        .unwrap();
        assert_eq!(contract.incidence(), [edge(0, 0)]);
        assert_eq!(contract.support().order, DerivativeOrder::Value);
        assert!(!contract.support().jacobian_product);
        assert_eq!(plan.order(), DerivativeOrder::Value);
    }
    #[test]
    fn frozen_mass_step_retries_preserve_anchor_and_mass_and_auxiliary_root_is_not_original() {
        let original = original(DerivativeOrder::First, false, false);
        let family = Arc::new(
            DerivedFamily::shifted_pseudo_time(
                original.clone(),
                vec![0.into()],
                1.0,
                MassStructure::Frozen {
                    incidence: vec![edge(0, 0)],
                },
            )
            .unwrap(),
        );
        let mut bound = family
            .bind_pseudo_time(
                oracle(&original),
                vec![0.0],
                1.0,
                MassBinding::Frozen(frozen(2.0)),
            )
            .unwrap();
        let mut out = [0.0];
        bound.residual(&[0.5], &mut out).unwrap();
        assert_eq!(out, [0.25]);
        bound.jacobian_product(&[0.5], &[3.0], &mut out).unwrap();
        assert_eq!(out, [9.0]);
        let jacobian = bound.jacobian(&[0.5], 100).unwrap();
        assert_eq!(jacobian.matrix().val(), [3.0]);
        let old_key = bound.key();
        let family_key = bound.family().key();
        let mut retry = bound.rebind_step(0.5).unwrap();
        assert_ne!(old_key, retry.key());
        assert_eq!(family_key, retry.family().key());
        assert_eq!(retry.anchor(), [0.0]);
        retry.jacobian_product(&[0.5], &[3.0], &mut out).unwrap();
        assert_eq!(out, [15.0]);
        assert_eq!(retry.family().correspondence(), Correspondence::Approximate);
        let auxiliary_root = 2.0f64.sqrt() - 1.0;
        let mut original_oracle = retry.into_original();
        original_oracle.values(&[auxiliary_root], &mut out).unwrap();
        assert!(out[0].abs() > 0.5);
    }
    #[derive(Debug)]
    struct VariableMass {
        structure: MassStructure,
        dm_calls: usize,
    }
    impl MassOracle for VariableMass {
        type Error = MathError;
        fn structure(&self) -> &MassStructure {
            &self.structure
        }
        fn realization(&self) -> ContentHash {
            hash(90)
        }
        fn apply(&mut self, x: &[f64], vector: &[f64], out: &mut [f64]) -> Result<(), MathError> {
            out[0] = (1.0 + x[0] * x[0]) * vector[0];
            Ok(())
        }
        fn derivative_action(
            &mut self,
            x: &[f64],
            direction: &[f64],
            velocity: &[f64],
            out: &mut [f64],
        ) -> Result<(), MathError> {
            self.dm_calls += 1;
            out[0] = 2.0 * x[0] * direction[0] * velocity[0];
            Ok(())
        }
    }
    #[test]
    fn state_dependent_mass_uses_actual_dm_action_and_matches_independent_derivative() {
        let original = original(DerivativeOrder::First, false, false);
        let mass = MassStructure::StateDependent {
            source: hash(9),
            incidence: vec![edge(0, 0)],
            derivative_incidence: vec![edge(0, 0)],
        };
        let family = Arc::new(
            DerivedFamily::shifted_pseudo_time(original.clone(), vec![0.into()], 1.0, mass.clone())
                .unwrap(),
        );
        let mut bound = family
            .bind_pseudo_time(
                oracle(&original),
                vec![1.0],
                0.5,
                MassBinding::StateDependent(Box::new(VariableMass {
                    structure: mass,
                    dm_calls: 0,
                })),
            )
            .unwrap();
        let mut out = [0.0];
        bound.residual(&[2.0], &mut out).unwrap();
        assert_eq!(out, [13.0]);
        bound.jacobian_product(&[2.0], &[3.0], &mut out).unwrap();
        assert_eq!(out, [66.0]);
        // Independent derivative of (1+x^2)*(x-1)/0.5 + x^2-1 at x=2.
        let exact = 2.0 * (1.0 + 3.0 * 2.0 * 2.0 - 2.0 * 2.0) + 2.0 * 2.0;
        assert_eq!(out[0], 3.0 * exact);
        let epsilon = 1e-6;
        let mut plus = [0.0];
        let mut minus = [0.0];
        bound.residual(&[2.0 + epsilon], &mut plus).unwrap();
        bound.residual(&[2.0 - epsilon], &mut minus).unwrap();
        assert!(((plus[0] - minus[0]) / (2.0 * epsilon) - exact).abs() < 1e-7);
        assert_eq!(bound.jacobian(&[2.0], 100).unwrap().matrix().val(), [22.0]);
    }
    #[test]
    fn preparation_rejects_lost_constraint_class_objective_pairing_and_mass_support() {
        assert!(
            DerivedFamily::shifted_pseudo_time(
                original(DerivativeOrder::First, false, true),
                vec![0.into()],
                1.0,
                MassStructure::Frozen {
                    incidence: vec![edge(0, 0)]
                }
            )
            .is_err()
        );
        assert!(
            DerivedFamily::shifted_pseudo_time(
                original(DerivativeOrder::First, true, false),
                vec![0.into()],
                1.0,
                MassStructure::Frozen {
                    incidence: vec![edge(0, 0)]
                }
            )
            .is_err()
        );
        assert!(
            DerivedFamily::shifted_pseudo_time(
                original(DerivativeOrder::First, false, false),
                vec![1.into()],
                1.0,
                MassStructure::Frozen {
                    incidence: vec![edge(0, 0)]
                }
            )
            .is_err()
        );
        let original = original(DerivativeOrder::First, false, false);
        let family = Arc::new(
            DerivedFamily::shifted_pseudo_time(
                original.clone(),
                vec![0.into()],
                1.0,
                MassStructure::Frozen { incidence: vec![] },
            )
            .unwrap(),
        );
        assert!(
            family
                .bind_pseudo_time(
                    oracle(&original),
                    vec![0.0],
                    1.0,
                    MassBinding::Frozen(frozen(1.0))
                )
                .is_err()
        );
    }
    #[test]
    fn engineering_certified_amplification_rounds_bounds_outward_and_demands_down() {
        let demand = AccuracyDemand {
            product: hash(11),
            normalization: hash(2),
            allowance: 6.0,
            class: AccuracyClass::Certified,
        };
        let control = ErrorAmplification {
            input_product: hash(10),
            output_product: demand.product,
            normalization: demand.normalization,
            inverse_norm: 3.0,
            remainder: 0.5,
            class: AccuracyClass::Certified,
        };
        let input = AccuracyEvidence {
            product: hash(10),
            normalization: hash(2),
            class: AccuracyClass::Certified,
            error: Some(1.0),
        };
        // Independent exact arithmetic: 1 * 3 / (1 - 1/2) = 6;
        // the greatest admissible exact backward error is 1.
        assert!(control.propagate(input, &demand).unwrap().error.unwrap() >= 6.0);
        let allowance = control.inner_allowance(&demand).unwrap();
        assert!(allowance > 0.0 && allowance <= 1.0);
        let zero = control
            .propagate(
                AccuracyEvidence {
                    error: Some(0.0),
                    ..input
                },
                &demand,
            )
            .unwrap();
        assert_eq!(zero.error, Some(0.0));
        assert_eq!(
            control
                .inner_allowance(&AccuracyDemand {
                    allowance: 0.0,
                    ..demand
                })
                .unwrap(),
            0.0
        );
        let underflow = control
            .inner_allowance(&AccuracyDemand {
                allowance: f64::from_bits(1),
                ..demand
            })
            .unwrap();
        assert_eq!(underflow, 0.0);
    }
    #[test]
    fn inverse_amplification_controls_ill_conditioned_nested_accuracy_without_certification() {
        let demand = AccuracyDemand {
            product: hash(11),
            normalization: hash(2),
            allowance: 1e-5,
            class: AccuracyClass::Estimated,
        };
        let control = ErrorAmplification {
            input_product: hash(10),
            output_product: demand.product,
            normalization: demand.normalization,
            inverse_norm: 1e6,
            remainder: 0.5,
            class: AccuracyClass::Estimated,
        };
        assert!((control.inner_allowance(&demand).unwrap() / 5e-12 - 1.0).abs() < 1e-15);
        let backward = AccuracyEvidence {
            product: hash(10),
            normalization: hash(2),
            class: AccuracyClass::Estimated,
            error: Some(1e-8),
        };
        let forward = control.propagate(backward, &demand).unwrap();
        assert_eq!(forward.error, Some(0.02));
        assert!(!forward.satisfies(&demand));
        let tight = control
            .propagate(
                AccuracyEvidence {
                    error: Some(4e-12),
                    ..backward
                },
                &demand,
            )
            .unwrap();
        assert!(tight.satisfies(&demand));
        let certified = AccuracyDemand {
            class: AccuracyClass::Certified,
            ..demand
        };
        assert!(!tight.satisfies(&certified));
        assert!(control.inner_allowance(&certified).is_err());
        let nested_demand = AccuracyDemand {
            product: hash(12),
            allowance: 1e-4,
            ..demand
        };
        let nested = ErrorAmplification {
            input_product: hash(11),
            output_product: hash(12),
            inverse_norm: 10.0,
            remainder: 0.0,
            ..control
        };
        assert!(
            nested
                .propagate(tight, &nested_demand)
                .unwrap()
                .satisfies(&nested_demand)
        );
        assert!(
            control
                .propagate(
                    AccuracyEvidence {
                        product: hash(99),
                        ..backward
                    },
                    &demand
                )
                .is_err()
        );
    }
    #[test]
    fn accuracy_identity_includes_realization_direction_outputs_order_and_normalization() {
        let product = AccuracyProduct {
            problem: hash(1),
            realization: hash(2),
            source: hash(3),
            normalization: hash(4),
            outputs: vec![id(1)],
            action: AccuracyAction::JacobianProduct { direction: hash(5) },
        };
        let key = product.key().unwrap();
        assert_ne!(
            key,
            AccuracyProduct {
                realization: hash(6),
                ..product.clone()
            }
            .key()
            .unwrap()
        );
        assert_ne!(
            key,
            AccuracyProduct {
                normalization: hash(6),
                ..product.clone()
            }
            .key()
            .unwrap()
        );
        assert_ne!(
            key,
            AccuracyProduct {
                outputs: vec![id(2)],
                ..product.clone()
            }
            .key()
            .unwrap()
        );
        assert_ne!(
            key,
            AccuracyProduct {
                action: AccuracyAction::JacobianProduct { direction: hash(6) },
                ..product.clone()
            }
            .key()
            .unwrap()
        );
        assert_ne!(
            key,
            AccuracyProduct {
                action: AccuracyAction::Partials {
                    order: DerivativeOrder::First,
                    coordinates: vec![id(1)]
                },
                ..product.clone()
            }
            .key()
            .unwrap()
        );
        assert!(
            AccuracyProduct {
                action: AccuracyAction::Partials {
                    order: DerivativeOrder::Value,
                    coordinates: vec![id(1)]
                },
                ..product
            }
            .key()
            .is_err()
        );
    }
    fn coupled_contract(order: DerivativeOrder) -> Arc<OriginalContract> {
        Arc::new(
            OriginalContract::new(
                hash(20),
                hash(2),
                vec![
                    Coordinate {
                        id: id(1),
                        lower: -10.0,
                        upper: 10.0,
                    },
                    Coordinate {
                        id: id(2),
                        lower: 0.0,
                        upper: 10.0,
                    },
                ],
                vec![
                    Constraint {
                        id: id(3),
                        lower: 0.0,
                        upper: 0.0,
                    },
                    Constraint {
                        id: id(4),
                        lower: -1.0,
                        upper: 5.0,
                    },
                ],
                vec![edge(0, 0), edge(0, 1), edge(1, 0), edge(1, 1)],
                DerivativeSupport {
                    order,
                    jacobian_product: order >= DerivativeOrder::First,
                    source: hash(3),
                },
                OriginalObligations {
                    guards: hash(4),
                    selection: hash(5),
                    objective: Some(hash(6)),
                },
            )
            .unwrap(),
        )
    }
    #[derive(Debug)]
    struct Coupled {
        contract: Arc<OriginalContract>,
        jvp_calls: usize,
        fail_above: Option<f64>,
    }
    impl Oracle for Coupled {
        type Error = MathError;
        fn contract(&self) -> &OriginalContract {
            &self.contract
        }
        fn values(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), MathError> {
            if self.fail_above.is_some_and(|limit| x[1] > limit) {
                return Err(MathError::Domain {
                    source_id: id(9),
                    requirement: "coupled source trial failed",
                });
            }
            out[0] = x[0] * x[0] + x[1] - 1.0;
            out[1] = x[0] * x[1];
            Ok(())
        }
        fn jacobian_product(
            &mut self,
            x: &[f64],
            v: &[f64],
            out: &mut [f64],
        ) -> Result<(), MathError> {
            self.jvp_calls += 1;
            out[0] = 2.0 * x[0] * v[0] + v[1];
            out[1] = x[1] * v[0] + x[0] * v[1];
            Ok(())
        }
    }
    impl ObjectiveOracle for Coupled {
        fn objective_support(&self) -> DerivativeSupport {
            self.contract.support
        }
        fn objective(&mut self, x: &[f64]) -> Result<f64, MathError> {
            Ok((x[0] - x[1]).powi(2))
        }
        fn objective_product(&mut self, x: &[f64], v: &[f64]) -> Result<f64, MathError> {
            Ok(2.0 * (x[0] - x[1]) * (v[0] - v[1]))
        }
    }
    fn coupled(contract: &Arc<OriginalContract>) -> Coupled {
        Coupled {
            contract: contract.clone(),
            jvp_calls: 0,
            fail_above: None,
        }
    }
    #[test]
    fn natural_parameter_reconstructs_original_and_composes_actual_coordinate_and_parameter_actions()
     {
        let original = coupled_contract(DerivativeOrder::Second);
        let family = Arc::new(
            DerivedFamily::parameter_continuation(original.clone(), GlobalCol::new(1)).unwrap(),
        );
        assert_eq!(family.class(), FamilyClass::ParameterContinuation);
        assert_eq!(
            family.incidence(),
            [edge(0, 0), edge(0, 1), edge(1, 0), edge(1, 1)]
        );
        assert_eq!(family.support().order, DerivativeOrder::First);
        assert!(family.reconstruct(&[3.0]).is_err());
        let mut bound = family
            .bind_parameter(coupled(&original), 2.0, hash(70))
            .unwrap();
        assert_eq!(bound.reconstruct(&[3.0]).unwrap(), [3.0, 2.0]);
        assert_eq!(bound.coordinates(), vec![original.coordinates()[0].clone()]);
        assert_eq!(bound.constraints(), original.constraints());
        let mut out = [0.0; 2];
        bound.values(&[3.0], &mut out).unwrap();
        assert_eq!(out, [10.0, 6.0]);
        bound.jacobian_product(&[3.0], &[2.0], &mut out).unwrap();
        assert_eq!(out, [12.0, 4.0]);
        bound.parameter_product(&[3.0], 3.0, &mut out).unwrap();
        assert_eq!(out, [3.0, 9.0]);
        assert_eq!(bound.original_objective(&[3.0]).unwrap(), 1.0);
        assert_eq!(
            bound.original_objective_product(&[3.0], &[2.0]).unwrap(),
            4.0
        );
        assert_eq!(
            bound.family().correspondence(),
            Correspondence::ReconstructionSlice
        );
        let key = bound.key();
        let mut rebound = bound.rebind_parameter(1.0).unwrap();
        assert_ne!(key, rebound.key());
        assert_eq!(family.key(), rebound.family().key());
        rebound.values(&[3.0], &mut out).unwrap();
        assert_eq!(out, [9.0, 3.0]);
        assert!(rebound.reconstruct(&[11.0]).is_err());
        assert!(
            family
                .bind_parameter(coupled(&original), -1.0, hash(70))
                .is_err()
        );
        assert_ne!(
            family.key(),
            DerivedFamily::parameter_continuation(original.clone(), GlobalCol::new(0))
                .unwrap()
                .key()
        );
        let source_changed = family
            .bind_parameter(coupled(&original), 1.0, hash(71))
            .unwrap();
        assert_ne!(rebound.key(), source_changed.key());
        assert_eq!(rebound.into_original().jvp_calls, 2);
    }
    #[test]
    fn block_keeps_external_coupling_objective_inequalities_and_requires_simultaneous_original_assessment()
     {
        let original = coupled_contract(DerivativeOrder::First);
        let family = Arc::new(
            DerivedFamily::block_subsystem(
                original.clone(),
                vec![GlobalCol::new(1)],
                vec![GlobalRow::new(1)],
            )
            .unwrap(),
        );
        assert_eq!(family.incidence(), [edge(0, 0)]);
        assert_eq!(family.external_incidence(), [edge(1, 0)]);
        assert_eq!(family.correspondence(), Correspondence::BlockCoverage);
        let mut bound = family
            .bind_block(coupled(&original), vec![2.0, 1.0], hash(70))
            .unwrap();
        assert_eq!(bound.coordinates(), vec![original.coordinates()[1].clone()]);
        assert_eq!(bound.constraints(), vec![original.constraints()[1].clone()]);
        let mut out = [0.0];
        bound.values(&[0.0], &mut out).unwrap();
        assert_eq!(out, [0.0]);
        // The block interval is satisfied, but the omitted original equality is not.
        let mut all = [0.0; 2];
        bound.original_values(&[0.0], &mut all).unwrap();
        assert_eq!(all, [3.0, 0.0]);
        bound.jacobian_product(&[0.0], &[3.0], &mut out).unwrap();
        assert_eq!(out, [6.0]);
        assert_eq!(bound.original_objective(&[0.0]).unwrap(), 4.0);
        assert_eq!(
            bound.original_objective_product(&[0.0], &[3.0]).unwrap(),
            -12.0
        );
        assert_eq!(bound.reconstruct(&[4.0]).unwrap(), [2.0, 4.0]);
        assert_eq!(bound.reconstruct(&[0.0]).unwrap(), [2.0, 0.0]);
        assert!(bound.reconstruct(&[-1.0]).is_err());
        assert!(bound.parameter_product(&[0.0], 1.0, &mut out).is_err());
        assert_eq!(bound.into_original().jvp_calls, 1);
        assert!(
            DerivedFamily::block_subsystem(
                original.clone(),
                vec![1.into(), 1.into()],
                vec![0.into()]
            )
            .is_err()
        );
        assert!(
            family
                .bind_block(coupled(&original), vec![11.0, 0.0], hash(70))
                .is_err()
        );
    }
    #[test]
    fn block_failed_trial_keeps_frozen_state_and_outputs_and_value_structure_has_no_fabricated_action()
     {
        let original = coupled_contract(DerivativeOrder::Value);
        let family = Arc::new(
            DerivedFamily::block_subsystem(
                original.clone(),
                vec![1.into()],
                vec![0.into(), 1.into()],
            )
            .unwrap(),
        );
        assert_eq!(family.incidence(), [edge(0, 0), edge(1, 0)]);
        assert_eq!(family.external_incidence(), [edge(0, 0), edge(1, 0)]);
        let mut supplier = coupled(&original);
        supplier.fail_above = Some(2.0);
        let mut bound = family
            .bind_block(supplier, vec![2.0, 1.0], hash(70))
            .unwrap();
        let key = bound.key();
        let mut out = [99.0, 98.0];
        assert!(matches!(
            bound.values(&[3.0], &mut out),
            Err(MathError::Domain { .. })
        ));
        assert_eq!(out, [99.0, 98.0]);
        assert_eq!(bound.key(), key);
        assert_eq!(bound.reconstruct(&[1.0]).unwrap(), [2.0, 1.0]);
        assert!(bound.jacobian_product(&[1.0], &[1.0], &mut out).is_err());
        assert_eq!(bound.into_original().jvp_calls, 0);
    }
    #[test]
    fn block_declared_order_is_respected_by_incidence_and_direction_reconstruction() {
        let original = coupled_contract(DerivativeOrder::First);
        let family = Arc::new(
            DerivedFamily::block_subsystem(
                original.clone(),
                vec![1.into(), 0.into()],
                vec![1.into(), 0.into()],
            )
            .unwrap(),
        );
        let ordered = DerivedFamily::block_subsystem(
            original.clone(),
            vec![0.into(), 1.into()],
            vec![0.into(), 1.into()],
        )
        .unwrap();
        assert_ne!(family.key(), ordered.key());
        let mut bound = family
            .bind_block(coupled(&original), vec![0.0, 0.0], hash(70))
            .unwrap();
        let mut out = [0.0; 2];
        bound.values(&[3.0, 2.0], &mut out).unwrap();
        assert_eq!(out, [6.0, 6.0]);
        bound
            .jacobian_product(&[3.0, 2.0], &[5.0, 7.0], &mut out)
            .unwrap();
        assert_eq!(out, [31.0, 33.0]);
        assert_eq!(bound.reconstruct(&[3.0, 2.0]).unwrap(), [2.0, 3.0]);
    }
    #[test]
    fn anchored_homotopy_freezes_actual_anchor_residual_and_only_exact_terminal_is_original() {
        let original = original(DerivativeOrder::Second, true, false);
        let family = Arc::new(DerivedFamily::anchored_homotopy(original.clone()).unwrap());
        assert_eq!(family.incidence(), [edge(0, 0), edge(0, 1)]);
        assert_eq!(family.original().obligations().objective, Some(hash(6)));
        assert_eq!(family.support().order, DerivativeOrder::First);
        let mut bound = family
            .bind_anchored_homotopy(oracle(&original), vec![0.0], 0.0, hash(70))
            .unwrap();
        let mut out = [0.0];
        bound.residual(&[0.0], &mut out).unwrap();
        assert_eq!(out, [0.0]);
        bound.original_values(&[0.0], &mut out).unwrap();
        assert_eq!(out, [-1.0]);
        bound.jacobian_product(&[2.0], &[3.0], &mut out).unwrap();
        assert_eq!(out, [12.0]);
        bound.parameter_product(&[2.0], 3.0, &mut out).unwrap();
        assert_eq!(out, [-3.0]);
        let key = bound.key();
        let mut halfway = bound.rebind_parameter(0.5).unwrap();
        assert_ne!(key, halfway.key());
        halfway.residual(&[2.0], &mut out).unwrap();
        assert_eq!(out, [3.5]);
        assert_eq!(halfway.correspondence(), Correspondence::Approximate);
        let near = halfway.rebind_parameter(1.0 - f64::EPSILON).unwrap();
        assert!(!near.is_terminal());
        assert_eq!(near.correspondence(), Correspondence::Approximate);
        let mut terminal = near.rebind_parameter(1.0).unwrap();
        assert_eq!(
            terminal.correspondence(),
            Correspondence::TerminalEquationIdentity
        );
        terminal.residual(&[2.0], &mut out).unwrap();
        assert_eq!(out, [3.0]);
        assert_eq!(terminal.reconstruct(&[2.0]).unwrap(), [2.0]);
        assert_eq!(terminal.into_original().jvp_calls, 1);
        assert!(
            family
                .bind_anchored_homotopy(oracle(&original), vec![4.0], 0.0, hash(70))
                .is_err()
        );
        assert!(
            DerivedFamily::anchored_homotopy(self::original(DerivativeOrder::First, false, true))
                .is_err()
        );
    }
    #[derive(Debug)]
    struct LinearStart {
        contract: Arc<OriginalContract>,
    }
    impl Oracle for LinearStart {
        type Error = MathError;
        fn contract(&self) -> &OriginalContract {
            &self.contract
        }
        fn values(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), MathError> {
            if x[0] < 0.0 {
                return Err(MathError::Domain {
                    source_id: id(8),
                    requirement: "start supplier domain",
                });
            }
            out[0] = x[0] - 2.0;
            Ok(())
        }
        fn jacobian_product(
            &mut self,
            _x: &[f64],
            v: &[f64],
            out: &mut [f64],
        ) -> Result<(), MathError> {
            out[0] = v[0];
            Ok(())
        }
    }
    #[test]
    fn affine_homotopy_composes_actual_actions_and_terminal_no_longer_consumes_start_domain() {
        let original = original(DerivativeOrder::First, false, false);
        let mut start_contract = (*original).clone();
        start_contract.identity = hash(80);
        start_contract.support.source = hash(81);
        let start = Arc::new(start_contract);
        let family =
            Arc::new(DerivedFamily::affine_homotopy(original.clone(), start.clone()).unwrap());
        let mut bound = family
            .bind_affine_homotopy(
                oracle(&original),
                LinearStart {
                    contract: start.clone(),
                },
                0.25,
                hash(70),
            )
            .unwrap();
        let mut out = [0.0];
        bound.residual(&[2.0], &mut out).unwrap();
        assert_eq!(out, [0.75]);
        bound.jacobian_product(&[2.0], &[3.0], &mut out).unwrap();
        assert_eq!(out, [5.25]);
        bound.parameter_product(&[2.0], 2.0, &mut out).unwrap();
        assert_eq!(out, [6.0]);
        assert!(bound.residual(&[-1.0], &mut out).is_err());
        let mut terminal = bound.rebind_parameter(1.0).unwrap();
        terminal.residual(&[-1.0], &mut out).unwrap();
        assert_eq!(out, [0.0]);
        terminal
            .jacobian_product(&[-1.0], &[3.0], &mut out)
            .unwrap();
        assert_eq!(out, [-6.0]);
        // The parameter derivative still consumes G, even at the terminal parameter.
        assert!(terminal.parameter_product(&[-1.0], 1.0, &mut out).is_err());
        let mut value_start = (*start).clone();
        value_start.support.order = DerivativeOrder::Value;
        value_start.support.jacobian_product = false;
        let value_start = Arc::new(value_start);
        let value_family = Arc::new(
            DerivedFamily::affine_homotopy(original.clone(), value_start.clone()).unwrap(),
        );
        assert!(!value_family.support().jacobian_product);
        assert_eq!(value_family.support().order, DerivativeOrder::Value);
        let mut missing = value_family
            .bind_affine_homotopy(
                oracle(&original),
                LinearStart {
                    contract: value_start,
                },
                0.5,
                hash(70),
            )
            .unwrap();
        assert!(missing.jacobian_product(&[2.0], &[1.0], &mut out).is_err());
        assert_ne!(family.key(), value_family.key());
        assert!(
            family
                .bind_affine_homotopy(
                    oracle(&original),
                    LinearStart { contract: start },
                    1.01,
                    hash(70)
                )
                .is_err()
        );
    }
    fn reconstruction_contract(
        original: &Arc<OriginalContract>,
        order: DerivativeOrder,
    ) -> Arc<ReconstructionContract> {
        Arc::new(
            ReconstructionContract::new(
                original.clone(),
                hash(90),
                hash(91),
                vec![0.into()],
                vec![0.into()],
                vec![
                    Entry::new(0.into(), 0.into()),
                    Entry::new(1.into(), 0.into()),
                ],
                DerivativeSupport {
                    order,
                    jacobian_product: order >= DerivativeOrder::First,
                    source: hash(92),
                },
            )
            .unwrap(),
        )
    }
    #[test]
    fn reduced_zero_row_objective_and_full_reconstruction_retain_original_obligations() {
        let base = coupled_contract(DerivativeOrder::First);
        let original = Arc::new(
            OriginalContract::new(
                base.identity(),
                base.normalization(),
                base.coordinates().to_vec(),
                vec![base.constraints()[0].clone()],
                vec![edge(0, 0), edge(0, 1)],
                base.support(),
                base.obligations(),
            )
            .unwrap(),
        );
        let map = Arc::new(
            ReconstructionContract::new(
                original.clone(),
                hash(90),
                hash(91),
                vec![0.into()],
                vec![0.into()],
                vec![
                    Entry::new(0.into(), 0.into()),
                    Entry::new(1.into(), 0.into()),
                ],
                original.support(),
            )
            .unwrap(),
        );
        let family = DerivedFamily::reduced_space(original.clone(), map).unwrap();
        assert_eq!(family.row_map().unwrap(), []);
        assert_eq!(
            family.original().obligations().objective,
            original.obligations().objective
        );
        let complete = Arc::new(
            ReconstructionContract::new(
                original.clone(),
                hash(90),
                hash(91),
                vec![],
                vec![0.into()],
                vec![],
                original.support(),
            )
            .unwrap(),
        );
        let family = DerivedFamily::reduced_space(original.clone(), complete).unwrap();
        assert_eq!(family.coordinate_map().unwrap(), []);
        assert_eq!(family.original().coordinates(), original.coordinates());
    }
    #[derive(Debug)]
    struct SelectedReconstruction {
        contract: Arc<ReconstructionContract>,
        realization: ContentHash,
        admitted: bool,
        point_error: Option<f64>,
        action_error: Option<f64>,
        point_calls: usize,
        action_calls: usize,
        corrupt_identity: bool,
        corrupt_action: bool,
    }
    impl ReconstructionOracle for SelectedReconstruction {
        type Error = MathError;
        fn contract(&self) -> &ReconstructionContract {
            &self.contract
        }
        fn realization(&self) -> ContentHash {
            self.realization
        }
        fn admit(&mut self, x: &[f64]) -> Result<ReconstructionAdmission, MathError> {
            if self.admitted {
                Ok(ReconstructionAdmission {
                    values: vec![x[0], 1.0 - x[0] * x[0]],
                })
            } else {
                Err(MathError::Domain {
                    source_id: id(10),
                    requirement: "selected reconstruction validity is unresolved",
                })
            }
        }
        fn point(
            &mut self,
            x: &[f64],
            demand: &AccuracyDemand,
            _refinement: RefinementLimits,
        ) -> Result<ReconstructionObservation, MathError> {
            self.point_calls += 1;
            Ok(ReconstructionObservation {
                values: vec![
                    x[0] + if self.corrupt_identity { 1.0 } else { 0.0 },
                    1.0 - x[0] * x[0],
                ],
                accuracy: AccuracyEvidence {
                    product: demand.product,
                    normalization: demand.normalization,
                    class: AccuracyClass::Estimated,
                    error: self.point_error,
                },
            })
        }
        fn jacobian_product(
            &mut self,
            x: &[f64],
            v: &[f64],
            demand: &AccuracyDemand,
            _refinement: RefinementLimits,
        ) -> Result<ReconstructionObservation, MathError> {
            self.action_calls += 1;
            Ok(ReconstructionObservation {
                values: vec![
                    v[0] + if self.corrupt_action { 1.0 } else { 0.0 },
                    -2.0 * x[0] * v[0],
                ],
                accuracy: AccuracyEvidence {
                    product: demand.product,
                    normalization: demand.normalization,
                    class: AccuracyClass::Estimated,
                    error: self.action_error,
                },
            })
        }
    }
    fn selected_reconstruction(contract: &Arc<ReconstructionContract>) -> SelectedReconstruction {
        SelectedReconstruction {
            contract: contract.clone(),
            realization: hash(93),
            admitted: true,
            point_error: Some(1e-9),
            action_error: Some(2e-9),
            point_calls: 0,
            action_calls: 0,
            corrupt_identity: false,
            corrupt_action: false,
        }
    }
    fn product_demand(
        product: AccuracyProduct,
        allowance: f64,
        class: AccuracyClass,
    ) -> AccuracyDemand {
        AccuracyDemand {
            product: product.key().unwrap(),
            normalization: product.normalization,
            allowance,
            class,
        }
    }
    fn refinement() -> RefinementLimits {
        RefinementLimits {
            rounds: 8,
            proof_cells: 64,
        }
    }
    #[test]
    fn reduced_composition_preserves_full_bounds_inequality_objective_and_uses_actual_chain_rule() {
        let original = coupled_contract(DerivativeOrder::Second);
        let map = reconstruction_contract(&original, DerivativeOrder::Second);
        let family = Arc::new(DerivedFamily::reduced_space(original.clone(), map.clone()).unwrap());
        assert_eq!(family.class(), FamilyClass::ReducedSpace);
        assert_eq!(
            family.correspondence(),
            Correspondence::ReconstructionUnderValidity
        );
        assert_eq!(family.incidence(), [edge(0, 0)]);
        assert_eq!(family.support().order, DerivativeOrder::First);
        let mut bound = family
            .bind_reduced(coupled(&original), selected_reconstruction(&map), hash(70))
            .unwrap();
        assert_eq!(bound.coordinates(), vec![original.coordinates()[0].clone()]);
        assert_eq!(bound.constraints(), vec![original.constraints()[1].clone()]);
        let point_demand = product_demand(
            bound.point_product(&[0.5]).unwrap(),
            1e-8,
            AccuracyClass::Estimated,
        );
        let action_demand = product_demand(
            bound.action_product(&[0.5], &[2.0]).unwrap(),
            1e-8,
            AccuracyClass::Estimated,
        );
        assert_ne!(point_demand.product, action_demand.product);
        let point = bound
            .reconstruct(&[0.5], &point_demand, refinement())
            .unwrap();
        assert_eq!(point.values, [0.5, 0.75]);
        let mut out = [0.0];
        let consumed = bound
            .values(&[0.5], &point_demand, refinement(), &mut out)
            .unwrap();
        assert_eq!(out, [0.375]);
        assert_eq!(consumed.original_point, point.values);
        assert!(
            consumed.retained_bytes() >= size_of::<ConsumedReconstruction>() + 2 * size_of::<f64>()
        );
        assert_eq!(consumed.point.error, Some(1e-9));
        assert!(consumed.action.is_none());
        let consumed = bound
            .jacobian_product(
                &[0.5],
                &[2.0],
                &point_demand,
                &action_demand,
                refinement(),
                &mut out,
            )
            .unwrap();
        assert_eq!(out, [0.5]);
        assert_eq!(consumed.original_point, point.values);
        assert_eq!(consumed.action.unwrap().error, Some(2e-9));
        let (objective, _) = bound
            .original_objective(&[0.5], &point_demand, refinement())
            .unwrap();
        assert_eq!(objective, 0.0625);
        let (derivative, consumed) = bound
            .original_objective_product(&[0.5], &[2.0], &point_demand, &action_demand, refinement())
            .unwrap();
        assert_eq!(derivative, -2.0);
        assert_eq!(consumed.original_point, point.values);
        assert_eq!(consumed.action.unwrap().product, action_demand.product);
        let mut all = [99.0; 2];
        bound
            .original_values(&[0.5], &point_demand, refinement(), &mut all)
            .unwrap();
        assert_eq!(all, [0.0, 0.375]);
        // Eliminated y has an original nonnegative interval: z alone is in bounds,
        // but reconstruction at z=2 violates y's interval and must refuse.
        let outside_demand = product_demand(
            bound.point_product(&[2.0]).unwrap(),
            1e-8,
            AccuracyClass::Estimated,
        );
        assert!(matches!(
            bound.reconstruct(&[2.0], &outside_demand, refinement()),
            Err(MathError::Domain { .. })
        ));
        let (original_supplier, reconstruction_supplier) = bound.into_suppliers();
        assert_eq!(original_supplier.jvp_calls, 1);
        assert_eq!(reconstruction_supplier.action_calls, 2);
    }
    #[test]
    fn reduced_validity_and_consumed_forward_accuracy_refuse_without_manufacturing_certificates() {
        let original = coupled_contract(DerivativeOrder::First);
        let map = reconstruction_contract(&original, DerivativeOrder::First);
        let family = Arc::new(DerivedFamily::reduced_space(original.clone(), map.clone()).unwrap());
        let mut bound = family
            .bind_reduced(coupled(&original), selected_reconstruction(&map), hash(70))
            .unwrap();
        let demand = product_demand(
            bound.point_product(&[0.5]).unwrap(),
            1e-8,
            AccuracyClass::Estimated,
        );
        let certified = AccuracyDemand {
            class: AccuracyClass::Certified,
            ..demand
        };
        assert!(bound.reconstruct(&[0.5], &certified, refinement()).is_err());
        let too_tight = AccuracyDemand {
            allowance: 1e-10,
            ..demand
        };
        assert!(bound.reconstruct(&[0.5], &too_tight, refinement()).is_err());
        bound.reconstruction.point_error = None;
        assert!(bound.reconstruct(&[0.5], &demand, refinement()).is_err());
        bound.reconstruction.point_error = Some(1e-9);
        bound.reconstruction.admitted = false;
        let calls = bound.reconstruction.point_calls;
        assert!(matches!(
            bound.reconstruct(&[0.5], &demand, refinement()),
            Err(MathError::Domain { .. })
        ));
        assert_eq!(bound.reconstruction.point_calls, calls);
        bound.reconstruction.admitted = true;
        let action = product_demand(
            bound.action_product(&[0.5], &[2.0]).unwrap(),
            1e-10,
            AccuracyClass::Estimated,
        );
        let mut out = [99.0];
        assert!(
            bound
                .jacobian_product(&[0.5], &[2.0], &demand, &action, refinement(), &mut out)
                .is_err()
        );
        assert_eq!(out, [99.0]);
        assert_eq!(bound.oracle.jvp_calls, 0);
    }
    #[test]
    fn reduced_product_scope_includes_actual_point_direction_source_validity_sheet_and_outer_realization()
     {
        let original = coupled_contract(DerivativeOrder::First);
        let map = reconstruction_contract(&original, DerivativeOrder::First);
        let family = Arc::new(DerivedFamily::reduced_space(original.clone(), map.clone()).unwrap());
        let mut bound = family
            .bind_reduced(coupled(&original), selected_reconstruction(&map), hash(70))
            .unwrap();
        let demand = product_demand(
            bound.point_product(&[0.5]).unwrap(),
            1e-8,
            AccuracyClass::Estimated,
        );
        assert_ne!(
            demand.product,
            bound.point_product(&[0.25]).unwrap().key().unwrap()
        );
        assert_ne!(
            bound.action_product(&[0.5], &[2.0]).unwrap().key().unwrap(),
            bound.action_product(&[0.5], &[3.0]).unwrap().key().unwrap()
        );
        assert!(bound.reconstruct(&[0.25], &demand, refinement()).is_err());
        assert_eq!(bound.reconstruction.point_calls, 0);
        let initial_key = bound.key();
        bound.reconstruction.realization = hash(94);
        assert_ne!(initial_key, bound.key());
        assert!(bound.reconstruct(&[0.5], &demand, refinement()).is_err());
        let rebound = family
            .bind_reduced(coupled(&original), selected_reconstruction(&map), hash(71))
            .unwrap();
        assert_ne!(
            demand.product,
            rebound.point_product(&[0.5]).unwrap().key().unwrap()
        );
        let mut changed = (*map).clone();
        changed.validity = hash(95); // A changed supplier contract is refused even if its source is unchanged.
        bound.reconstruction.contract = Arc::new(changed);
        assert!(bound.point_product(&[0.5]).is_err());
        let changed = Arc::new(
            ReconstructionContract::new(
                original.clone(),
                hash(96),
                hash(97),
                map.retained.clone(),
                map.eliminated.clone(),
                map.incidence.clone(),
                map.support,
            )
            .unwrap(),
        );
        assert_ne!(
            family.key(),
            DerivedFamily::reduced_space(original, changed)
                .unwrap()
                .key()
        );
    }
    #[test]
    fn reduced_inventory_and_retained_identities_are_explicit_and_value_does_not_acquire_an_action()
    {
        let original = coupled_contract(DerivativeOrder::Value);
        let map = reconstruction_contract(&original, DerivativeOrder::Value);
        let family = Arc::new(DerivedFamily::reduced_space(original.clone(), map.clone()).unwrap());
        assert_eq!(family.incidence(), [edge(0, 0)]);
        assert!(!family.support().jacobian_product);
        let mut bound = family
            .bind_reduced(coupled(&original), selected_reconstruction(&map), hash(70))
            .unwrap();
        assert!(bound.action_product(&[0.5], &[2.0]).is_err());
        let demand = product_demand(
            bound.point_product(&[0.5]).unwrap(),
            1e-8,
            AccuracyClass::Estimated,
        );
        bound.reconstruction.corrupt_identity = true;
        assert!(bound.reconstruct(&[0.5], &demand, refinement()).is_err());
        bound.reconstruction.corrupt_identity = false;
        assert!(
            ReconstructionContract::new(
                original.clone(),
                hash(90),
                hash(91),
                vec![0.into()],
                vec![1.into()],
                map.incidence.clone(),
                map.support
            )
            .is_err()
        );
        assert!(
            ReconstructionContract::new(
                original.clone(),
                hash(90),
                hash(91),
                vec![0.into()],
                vec![0.into()],
                vec![Entry::new(1.into(), 0.into())],
                map.support
            )
            .is_err()
        );
        let original = coupled_contract(DerivativeOrder::First);
        let map = reconstruction_contract(&original, DerivativeOrder::First);
        let family = Arc::new(DerivedFamily::reduced_space(original.clone(), map.clone()).unwrap());
        let mut bound = family
            .bind_reduced(coupled(&original), selected_reconstruction(&map), hash(70))
            .unwrap();
        bound.reconstruction.corrupt_action = true;
        let point_demand = product_demand(
            bound.point_product(&[0.5]).unwrap(),
            1e-8,
            AccuracyClass::Estimated,
        );
        let action_demand = product_demand(
            bound.action_product(&[0.5], &[2.0]).unwrap(),
            1e-8,
            AccuracyClass::Estimated,
        );
        let mut out = [99.0];
        assert!(
            bound
                .jacobian_product(
                    &[0.5],
                    &[2.0],
                    &point_demand,
                    &action_demand,
                    refinement(),
                    &mut out
                )
                .is_err()
        );
        assert_eq!(out, [99.0]);
        assert_eq!(bound.oracle.jvp_calls, 0);
    }
}
