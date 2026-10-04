// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Concrete reduced reconstruction over the existing selected implicit worker.
use crate::{
    MathError,
    derived::{
        Coordinate, DerivativeSupport, OriginalContract, ReconstructionAdmission,
        ReconstructionContract, ReconstructionObservation, ReconstructionOracle,
        ReconstructionProducer,
    },
    implicit::{RegimeFactory, RegimeSelection, RootActionEvidence, SelectionProofRefusal},
    index::{Entry, GlobalCol, GlobalRow},
    normalization::Normalization,
};
use pse_ids::{ContentHash, FramedHasher};
use pse_kernels::{DerivativeOrder, ExecutionScope, ProviderFactory};
use pse_model::strategy::{AccuracyClass, AccuracyDemand, AccuracyEvidence};
use std::{
    collections::BTreeSet,
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Checked realization of named authored equality values as zero residuals.
/// The factory's named rows compute authored row values; every selected alternative
/// receives the identical subtraction in numerical evaluation and verifier projection.
#[derive(Clone, Debug, PartialEq)]
pub enum SelectedResidualRealization {
    /// Actual factory outputs are authored row values; subtract checked RHS values.
    AuthoredValues,
    /// Actual compiler producer already subtracted the named authored RHS values.
    /// Every declared offset is checked mechanically against the original row bounds.
    ZeroResiduals {
        /// Named RHS values already subtracted by the actual residual producer.
        authored_offsets: Vec<(pse_ids::SemanticId, f64)>,
    },
}
/// Checked source realization and exact named original equality correspondence.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedResidualBinding {
    original: ContentHash,
    source: ContentHash,
    rows: Vec<(GlobalRow, pse_ids::SemanticId, f64)>,
    realization: SelectedResidualRealization,
    key: ContentHash,
}
impl SelectedResidualBinding {
    /// Derive offsets mechanically from the actual original equality intervals.
    pub fn for_original(
        factory: &RegimeFactory,
        original: &OriginalContract,
        eliminated: &[GlobalRow],
    ) -> Result<Self, MathError> {
        Self::for_source(
            factory,
            original,
            eliminated,
            SelectedResidualRealization::AuthoredValues,
        )
    }
    /// Bind an explicit compiler/producer realization, without inferring subtraction
    /// from equation IDs or applying an original offset twice.
    pub fn for_source(
        factory: &RegimeFactory,
        original: &OriginalContract,
        eliminated: &[GlobalRow],
        realization: SelectedResidualRealization,
    ) -> Result<Self, MathError> {
        let rows = eliminated
            .iter()
            .map(|&row| {
                let constraint = original.constraints().get(row.get()).ok_or_else(|| {
                    MathError::Contract("selected residual row outside original".into())
                })?;
                if !constraint.lower.is_finite()
                    || constraint.lower.to_bits() != constraint.upper.to_bits()
                {
                    return Err(MathError::Contract(
                        "selected residual offset requires an authored finite equality".into(),
                    ));
                }
                Ok((row, constraint.id, constraint.lower))
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        if rows.is_empty()
            || rows
                .iter()
                .map(|(r, _, _)| r)
                .collect::<BTreeSet<_>>()
                .len()
                != rows.len()
            || factory.alternatives.iter().any(|b| {
                b.residual.rows.len() != rows.len()
                    || b.residual
                        .rows
                        .iter()
                        .zip(&rows)
                        .any(|(id, (_, expected, _))| id != expected)
            })
        {
            return Err(MathError::Contract(
                "selected residual named row coverage".into(),
            ));
        }
        if let SelectedResidualRealization::ZeroResiduals { authored_offsets } = &realization
            && (authored_offsets.len() != rows.len()
                || authored_offsets.iter().zip(&rows).any(
                    |((id, offset), (_, expected, value))| {
                        id != expected || offset.to_bits() != value.to_bits()
                    },
                ))
        {
            return Err(MathError::Contract(
                "realized zero residual offsets differ from authored named equalities".into(),
            ));
        }
        let source = factory.configuration_key();
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str("selected-authored-equality-offset-binding");
        original.frame(&mut h);
        h.hash(&source);
        for (row, id, offset) in &rows {
            h.u64(row.get() as u64).id(id).f64(*offset);
        }
        h.str(match realization {
            SelectedResidualRealization::AuthoredValues => "authored-values",
            SelectedResidualRealization::ZeroResiduals { .. } => "already-realized-zero-residuals",
        });
        Ok(Self {
            original: original.identity(),
            source,
            rows,
            realization,
            key: h.finish_hash(),
        })
    }
    /// Additional owned binding metadata, excluding source factory/contracts.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.rows.capacity() * size_of::<(GlobalRow, pse_ids::SemanticId, f64)>()
            + match &self.realization {
                SelectedResidualRealization::AuthoredValues => 0,
                SelectedResidualRealization::ZeroResiduals { authored_offsets } => {
                    authored_offsets.capacity() * size_of::<(pse_ids::SemanticId, f64)>()
                }
            }
    }
    /// Actual factory, original contract, row ordering and offsets identity.
    pub fn key(&self) -> ContentHash {
        self.key
    }
    /// Preserved named authored subtraction, in selected residual output order.
    pub fn rows(&self) -> &[(GlobalRow, pse_ids::SemanticId, f64)] {
        &self.rows
    }
    /// Actual admitted producer realization, including any already-applied offsets.
    pub fn realization(&self) -> &SelectedResidualRealization {
        &self.realization
    }
    fn offsets(&self) -> Option<Vec<f64>> {
        match self.realization {
            SelectedResidualRealization::AuthoredValues => {
                Some(self.rows.iter().map(|(_, _, value)| *value).collect())
            }
            SelectedResidualRealization::ZeroResiduals { .. } => None,
        }
    }
    fn check(
        &self,
        factory: &RegimeFactory,
        original: &OriginalContract,
        eliminated: &[GlobalRow],
    ) -> Result<(), MathError> {
        if self != &Self::for_source(factory, original, eliminated, self.realization.clone())? {
            return Err(MathError::Contract(
                "selected residual binding differs from actual original source/bounds".into(),
            ));
        }
        Ok(())
    }
}

/// Actual selected-root reconstruction. Native iteration and IFT jets stay owned by
/// the admitted implicit factory; interval evidence stays owned by its verifier.
/// Runtime admission must retain the original factory lease and this wrapper's extent.
#[derive(Debug)]
pub struct SelectedImplicitReconstruction<E = MathError> {
    contract: Arc<ReconstructionContract>,
    selection: RegimeSelection,
    unknown_columns: Vec<GlobalCol>,
    normalization: Normalization,
    cancel: Arc<AtomicBool>,
    marker: PhantomData<fn() -> E>,
    last_refusal: Option<crate::derived::RefinementRefusal>,
}
impl<E> SelectedImplicitReconstruction<E> {
    /// Source identity of the actual factory's compiled interpretation/configuration.
    pub fn source(factory: &RegimeFactory) -> ContentHash {
        factory.configuration_key()
    }
    /// Actual numerical IFT/action-enclosure producer identity, separately framed.
    pub fn derivative_source(factory: &RegimeFactory) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str("selected-implicit-ift-with-interval-action")
            .hash(&Self::source(factory));
        h.finish_hash()
    }
    /// Derive reconstruction structure from admitted original coordinates and actual
    /// residual support. POUNCE matching/DM/BTF owns structural inverse dependencies;
    /// numerical rank, regularity, root selection and accuracy are established later.
    /// Every branch must represent the same declared original eliminated equalities.
    pub fn prepare_contract(
        factory: &RegimeFactory,
        original: Arc<OriginalContract>,
        retained: Vec<GlobalCol>,
        eliminated: Vec<GlobalRow>,
        validity: ContentHash,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Arc<ReconstructionContract>, MathError> {
        if eliminated.iter().any(|r| {
            original
                .constraints()
                .get(r.get())
                .is_none_or(|c| c.lower != 0.0 || c.upper != 0.0)
        }) {
            return Err(MathError::Contract(
                "nonzero selected equality requires an explicit residual offset binding".into(),
            ));
        }
        Self::prepare_contract_inner(
            factory, original, retained, eliminated, validity, cancel, None,
        )
    }
    /// Prepare reconstruction with mechanically checked nonzero equality offsets.
    pub fn prepare_contract_with_binding(
        factory: &RegimeFactory,
        original: Arc<OriginalContract>,
        retained: Vec<GlobalCol>,
        eliminated: Vec<GlobalRow>,
        validity: ContentHash,
        cancel: &Arc<AtomicBool>,
        binding: &SelectedResidualBinding,
    ) -> Result<Arc<ReconstructionContract>, MathError> {
        binding.check(factory, &original, &eliminated)?;
        Self::prepare_contract_inner(
            factory,
            original,
            retained,
            eliminated,
            validity,
            cancel,
            Some(binding),
        )
    }
    fn prepared_source(
        factory: &RegimeFactory,
        binding: Option<&SelectedResidualBinding>,
    ) -> ContentHash {
        match binding {
            None => Self::source(factory),
            Some(binding) => {
                let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
                h.str("selected-offset-residual-source")
                    .hash(&Self::source(factory))
                    .hash(&binding.key());
                h.finish_hash()
            }
        }
    }
    fn prepared_derivative_source(
        factory: &RegimeFactory,
        binding: Option<&SelectedResidualBinding>,
    ) -> ContentHash {
        match binding {
            None => Self::derivative_source(factory),
            Some(binding) => {
                let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
                h.str("selected-offset-residual-ift-source")
                    .hash(&Self::prepared_source(factory, Some(binding)));
                h.finish_hash()
            }
        }
    }
    fn prepare_contract_inner(
        factory: &RegimeFactory,
        original: Arc<OriginalContract>,
        retained: Vec<GlobalCol>,
        eliminated: Vec<GlobalRow>,
        validity: ContentHash,
        cancel: &Arc<AtomicBool>,
        binding: Option<&SelectedResidualBinding>,
    ) -> Result<Arc<ReconstructionContract>, MathError> {
        let unknown_columns = maps(factory, &original, &retained, &eliminated)?;
        let n = unknown_columns.len();
        let p = retained.len();
        let mut incidence = retained
            .iter()
            .enumerate()
            .map(|(j, &c)| Entry::new(c, GlobalCol::new(j)))
            .collect::<Vec<_>>();
        for branch in &factory.alternatives {
            if cancel.load(Ordering::Acquire) {
                return Err(MathError::Cancelled);
            }
            let structural = branch.residual.body.incidence(cancel)?;
            let mut unknown_support = Vec::with_capacity(n);
            let mut parameter_support = Vec::with_capacity(n);
            for output in structural.outputs() {
                let support = structural.first_for_output(*output).ok_or_else(|| {
                    MathError::Contract("reconstruction residual lacks structural support".into())
                })?;
                unknown_support.push(
                    support
                        .iter()
                        .copied()
                        .filter(|i| *i < n)
                        .collect::<Vec<_>>(),
                );
                parameter_support.push(
                    support
                        .iter()
                        .copied()
                        .filter_map(|i| i.checked_sub(n).filter(|j| *j < p))
                        .collect::<BTreeSet<_>>(),
                );
            }
            if unknown_support.len() != n {
                return Err(MathError::Contract(
                    "reconstruction residual structural row coverage".into(),
                ));
            }
            let mut adj_ptr = vec![0];
            let mut vars = Vec::new();
            for row in &unknown_support {
                vars.extend_from_slice(row);
                adj_ptr.push(vars.len());
            }
            let input = pounce_presolve::EqualityIncidence {
                n_vars: n,
                eq_row_inner_idx: (0..n).collect(),
                adj_ptr,
                vars,
            };
            let matching = pounce_presolve::matching::hopcroft_karp(&input);
            if matching.size != n {
                return Err(MathError::Contract(
                    "reconstruction has no complete structural unknown matching".into(),
                ));
            }
            let dm = pounce_presolve::DulmageMendelsohnPartition::from_matching(&input, &matching);
            let components =
                pounce_presolve::SquareComponents::of_square_part(&input, &matching, &dm);
            let mut dependencies = vec![BTreeSet::<usize>::new(); n];
            let mut covered = BTreeSet::new();
            for component in &components.components {
                for block in
                    pounce_presolve::BlockTriangularForm::of_component(&input, &matching, component)
                        .blocks
                {
                    let mut inputs = BTreeSet::new();
                    for &row in &block.eq_rows {
                        inputs.extend(parameter_support[row].iter().copied());
                        for &column in &unknown_support[row] {
                            if !block.cols.contains(&column) {
                                inputs.extend(dependencies[column].iter().copied());
                            }
                        }
                    }
                    for column in block.cols {
                        covered.insert(column);
                        dependencies[column] = inputs.clone();
                    }
                }
            }
            if covered.len() != n {
                return Err(MathError::Contract(
                    "reconstruction BTF did not cover every unknown".into(),
                ));
            }
            for (unknown, columns) in dependencies.iter().enumerate() {
                for &column in columns {
                    incidence.push(Entry::new(unknown_columns[unknown], GlobalCol::new(column)));
                }
            }
        }
        let source = Self::prepared_source(factory, binding);
        let support = DerivativeSupport {
            order: DerivativeOrder::First,
            jacobian_product: true,
            source: Self::prepared_derivative_source(factory, binding),
        };
        let contract = match binding {
            None => ReconstructionContract::new(
                original, source, validity, retained, eliminated, incidence, support,
            )?,
            Some(binding) => ReconstructionContract::new_with_offsets(
                original,
                ReconstructionProducer {
                    source,
                    validity,
                    support,
                },
                retained,
                eliminated,
                incidence,
                &binding
                    .rows()
                    .iter()
                    .map(|(row, _, offset)| (*row, *offset))
                    .collect::<Vec<_>>(),
            )?,
        };
        Ok(Arc::new(contract))
    }
    /// Bind one existing factory worker to exact original maps and normalized error
    /// coordinates. A reconstruction cannot infer elimination from a matching alone.
    pub fn new(
        factory: &RegimeFactory,
        contract: Arc<ReconstructionContract>,
        normalization: Normalization,
        scope: ExecutionScope,
    ) -> Result<Self, MathError> {
        if contract
            .eliminated()
            .iter()
            .any(|r| contract.original().constraints()[r.get()].lower != 0.0)
        {
            return Err(MathError::Contract(
                "nonzero selected equality requires explicit residual binding".into(),
            ));
        }
        Self::new_inner(factory, contract, normalization, scope, None)
    }
    /// Bind the exact same checked offsets to actual native residual and verifier DAG.
    pub fn new_with_binding(
        factory: &RegimeFactory,
        contract: Arc<ReconstructionContract>,
        normalization: Normalization,
        scope: ExecutionScope,
        binding: &SelectedResidualBinding,
    ) -> Result<Self, MathError> {
        binding.check(factory, contract.original(), contract.eliminated())?;
        Self::new_inner(factory, contract, normalization, scope, Some(binding))
    }
    fn new_inner(
        factory: &RegimeFactory,
        contract: Arc<ReconstructionContract>,
        normalization: Normalization,
        scope: ExecutionScope,
        binding: Option<&SelectedResidualBinding>,
    ) -> Result<Self, MathError> {
        normalization.validate(
            contract.original().coordinates().len(),
            contract.original().constraints().len(),
        )?;
        if normalization.key() != contract.original().normalization()
            || contract.source() != Self::prepared_source(factory, binding)
            || contract.support().source != Self::prepared_derivative_source(factory, binding)
            || !contract.support().jacobian_product
            || contract.support().order != DerivativeOrder::First
        {
            return Err(MathError::Contract(
                "selected reconstruction source/support/normalization mismatch".into(),
            ));
        }
        let unknown_columns = maps(
            factory,
            contract.original(),
            contract.retained(),
            contract.eliminated(),
        )?;
        let cancel = scope.cancellation().clone();
        let offsets = binding.and_then(SelectedResidualBinding::offsets);
        let selection = factory
            .prepare_selection_offsets(scope, offsets.as_deref())
            .map_err(|cause| MathError::Provider {
                source_id: factory.spec.id,
                provider: factory.spec.id,
                cause,
            })?;
        Ok(Self {
            contract,
            selection,
            unknown_columns,
            normalization,
            cancel,
            marker: PhantomData,
            last_refusal: None,
        })
    }
    /// Additional retained wrapper extent. Original contracts/factory programs and
    /// native verifier workspace remain charged by their existing owners, not twice.
    pub fn retained_bytes(&self) -> Result<usize, MathError> {
        size_of::<Self>()
            .checked_add(
                self.unknown_columns
                    .capacity()
                    .checked_mul(size_of::<GlobalCol>())
                    .ok_or(MathError::Limit("reconstruction map extent"))?,
            )
            .and_then(|n| {
                n.checked_add(
                    (self.normalization.variables.capacity() + self.normalization.rows.capacity())
                        * size_of::<f64>(),
                )
            })
            .ok_or(MathError::Limit("reconstruction wrapper extent"))
    }
    /// Actual selected worker for observational work accounting and final owner reuse.
    pub fn selection(&self) -> &RegimeSelection {
        &self.selection
    }
    fn check_input(&self, x: &[f64]) -> Result<(), MathError> {
        if x.len() != self.contract.retained().len() {
            return Err(MathError::Contract(
                "selected reconstruction retained extent".into(),
            ));
        }
        for (&column, &value) in self.contract.retained().iter().zip(x) {
            let Coordinate { lower, upper, id } =
                self.contract.original().coordinates()[column.get()];
            if !value.is_finite() || value < lower || value > upper {
                return Err(MathError::Domain {
                    source_id: id,
                    requirement: "retained reconstruction coordinate violates its original interval",
                });
            }
        }
        Ok(())
    }
    fn full(&self, x: &[f64], unknowns: &[f64]) -> Result<Vec<f64>, MathError> {
        if unknowns.len() != self.unknown_columns.len() {
            return Err(MathError::Contract(
                "selected reconstruction unknown extent".into(),
            ));
        }
        let mut result = vec![0.0; self.contract.original().coordinates().len()];
        for (&column, &value) in self.contract.retained().iter().zip(x) {
            result[column.get()] = value;
        }
        for (&column, &value) in self.unknown_columns.iter().zip(unknowns) {
            result[column.get()] = value;
        }
        if result.iter().any(|v| !v.is_finite()) {
            return Err(MathError::Contract(
                "nonfinite selected reconstruction".into(),
            ));
        }
        Ok(result)
    }
    fn accuracy(
        &self,
        values: &[f64],
        intervals: &[super::ProofInterval],
        demand: &AccuracyDemand,
    ) -> Result<AccuracyEvidence, MathError> {
        demand
            .validate()
            .map_err(|e| MathError::Contract(e.to_string()))?;
        if demand.normalization != self.normalization.key()
            || intervals.len() != self.unknown_columns.len()
        {
            return Err(MathError::Contract(
                "selected reconstruction accuracy scope/extent".into(),
            ));
        }
        let mut error = 0.0_f64;
        for ((&column, &value), interval) in self.unknown_columns.iter().zip(values).zip(intervals)
        {
            if !interval.valid() || !value.is_finite() {
                return Err(MathError::Contract(
                    "invalid actual root/action enclosure".into(),
                ));
            }
            // Each positive primitive is widened separately. Certified library interval
            // endpoints and declared binary64 coordinate scales are treated exactly;
            // a rounded-to-nearest subtraction/division is never used as an upper bound.
            let distance = (value - interval.lower)
                .abs()
                .next_up()
                .max((value - interval.upper).abs().next_up());
            let normalized = (distance / self.normalization.variables[column.get()]).next_up();
            error = error.max(normalized);
        }
        let evidence = AccuracyEvidence {
            product: demand.product,
            normalization: demand.normalization,
            class: AccuracyClass::Certified,
            error: Some(error),
        };
        Ok(evidence)
    }
    // The interval inverse is uniform over the entire candidate/root hull. Thus
    // K already covers the nonlinear mean-value map; no local linearization
    // remainder is added. This only chooses controls, never certifies a product.
    fn backward_allowance(
        &self,
        selected: &super::SelectedRegime,
        demand: &AccuracyDemand,
        inverse: f64,
    ) -> Result<f64, MathError> {
        let mut h = FramedHasher::new(pse_ids::Frame::AccuracyProductV1);
        h.str("selected-normalized-residual")
            .hash(&self.contract.source())
            .hash(&demand.product);
        for r in self.contract.eliminated() {
            h.id(&self.contract.original().constraints()[r.get()].id);
        }
        for v in &selected.values {
            h.f64(*v);
        }
        let representable = demand.allowance / inverse;
        if !representable.is_finite() || representable <= 0. {
            return Err(self.refusal(demand, crate::derived::RefinementRefusal::Precision));
        }
        crate::derived::ErrorAmplification {
            input_product: h.finish_hash(),
            output_product: demand.product,
            normalization: demand.normalization,
            inverse_norm: inverse,
            remainder: 0.0,
            class: AccuracyClass::Certified,
        }
        .inner_allowance(demand)
    }
    fn refusal(
        &self,
        demand: &AccuracyDemand,
        reason: crate::derived::RefinementRefusal,
    ) -> MathError {
        MathError::Refinement {
            product: demand.product,
            source_key: self.contract.source(),
            validity: self.contract.validity(),
            reason,
        }
    }
    fn refined(
        &mut self,
        x: &[f64],
        direction: Option<&[f64]>,
        demand: &AccuracyDemand,
        limits: crate::derived::RefinementLimits,
    ) -> Result<ReconstructionObservation, MathError> {
        limits.validate()?;
        self.check_input(x)?;
        if direction.is_some_and(|v| v.len() != x.len() || v.iter().any(|a| !a.is_finite())) {
            return Err(MathError::Contract(
                "selected reconstruction direction extent/value".into(),
            ));
        }
        demand
            .validate()
            .map_err(|e| MathError::Contract(e.to_string()))?;
        let deadline = self.selection.refinement_deadline()?;
        self.selection
            .refinement_checkpoint(deadline, &self.cancel)?;
        let mut selected = self
            .selection
            .evaluate(x, DerivativeOrder::First, &self.cancel)?;
        self.selection
            .refinement_checkpoint(deadline, &self.cancel)?;
        let scales = self
            .unknown_columns
            .iter()
            .map(|c| self.normalization.variables[c.get()])
            .collect::<Vec<_>>();
        let rows = self
            .contract
            .eliminated()
            .iter()
            .map(|r| self.normalization.rows[r.get()])
            .collect::<Vec<_>>();
        let mut remaining = limits.proof_cells;
        for round in 0..limits.rounds {
            self.selection
                .refinement_checkpoint(deadline, &self.cancel)?;
            let (intervals, inverse, cells) = match self.selection.refine_point(
                x,
                &scales,
                &rows,
                remaining,
                deadline,
                &self.cancel,
            )? {
                super::RootPointEvidence::Enclosed {
                    intervals,
                    inverse_norm_upper,
                    proof_cells,
                } => (intervals, inverse_norm_upper, proof_cells),
                super::RootPointEvidence::Interrupted { .. } => return Err(MathError::Cancelled),
                super::RootPointEvidence::Incomplete {
                    reason: SelectionProofRefusal::Resource,
                    proof_cells,
                } => {
                    return Err(self.refusal(
                        demand,
                        if proof_cells >= remaining {
                            crate::derived::RefinementRefusal::ProofCells
                        } else {
                            crate::derived::RefinementRefusal::Unavailable(
                                SelectionProofRefusal::Resource,
                            )
                        },
                    ));
                }
                super::RootPointEvidence::Incomplete { reason, .. } => {
                    return Err(self.refusal(
                        demand,
                        crate::derived::RefinementRefusal::Unavailable(reason),
                    ));
                }
            };
            remaining = remaining.checked_sub(cells).ok_or_else(|| {
                MathError::Contract("point verifier exceeded refinement allowance".into())
            })?;
            let mut action_uncertainty = 0.0_f64;
            let (values, evidence) = if let Some(direction) = direction {
                if selected.jacobian.len() != self.unknown_columns.len() * x.len() {
                    return Err(MathError::Contract(
                        "selected reconstruction actual IFT extent".into(),
                    ));
                }
                let values = (0..self.unknown_columns.len())
                    .map(|i| {
                        selected.jacobian[i * x.len()..(i + 1) * x.len()]
                            .iter()
                            .zip(direction)
                            .map(|(a, v)| a * v)
                            .sum::<f64>()
                    })
                    .collect::<Vec<_>>();
                let (action, cells) = match self.selection.enclose_action_bounded(
                    x,
                    direction,
                    remaining,
                    deadline,
                    &self.cancel,
                )? {
                    RootActionEvidence::Enclosed {
                        intervals,
                        proof_cells,
                    } => (intervals, proof_cells),
                    RootActionEvidence::Interrupted { .. } => return Err(MathError::Cancelled),
                    RootActionEvidence::Incomplete {
                        reason: SelectionProofRefusal::Resource,
                        proof_cells,
                    } => {
                        return Err(self.refusal(
                            demand,
                            if proof_cells >= remaining {
                                crate::derived::RefinementRefusal::ProofCells
                            } else {
                                crate::derived::RefinementRefusal::Unavailable(
                                    SelectionProofRefusal::Resource,
                                )
                            },
                        ));
                    }
                    RootActionEvidence::Incomplete { reason, .. } => {
                        return Err(self.refusal(
                            demand,
                            crate::derived::RefinementRefusal::Unavailable(reason),
                        ));
                    }
                };
                remaining = remaining.checked_sub(cells).ok_or_else(|| {
                    MathError::Contract("action verifier exceeded refinement allowance".into())
                })?;
                for (interval, scale) in action.iter().zip(&scales) {
                    action_uncertainty = action_uncertainty
                        .max(((interval.upper - interval.lower).next_up() / 2.0 / scale).next_up());
                }
                let evidence = self.accuracy(&values, &action, demand)?;
                (values, evidence)
            } else {
                (
                    selected.values.clone(),
                    self.accuracy(&selected.values, &intervals, demand)?,
                )
            };
            if evidence.satisfies(demand) {
                self.selection
                    .refinement_checkpoint(deadline, &self.cancel)?;
                return Ok(ReconstructionObservation {
                    values: self.full(direction.unwrap_or(x), &values)?,
                    accuracy: evidence,
                });
            }
            if round + 1 == limits.rounds {
                return Err(self.refusal(demand, crate::derived::RefinementRefusal::Rounds));
            }
            if remaining == 0 {
                return Err(self.refusal(demand, crate::derived::RefinementRefusal::ProofCells));
            }
            let mut root_allowance = demand.allowance;
            // Action error is not inferred from K. The observed action enclosure error
            // supplies a numerical tightening ratio only; the next action enclosure
            // remains the sole certifier. No estimate becomes accuracy evidence.
            if direction.is_some() {
                root_allowance *= demand.allowance
                    / evidence.error.ok_or_else(|| {
                        MathError::Contract("missing action enclosure error".into())
                    })?;
            }
            if !root_allowance.is_finite() || root_allowance <= 0. {
                return Err(self.refusal(demand, crate::derived::RefinementRefusal::Precision));
            }
            let allowance_demand = AccuracyDemand {
                allowance: root_allowance,
                ..*demand
            };
            let residual_allowance =
                self.backward_allowance(&selected, &allowance_demand, inverse)?;
            let linear = if let Some(direction) = direction {
                let slack = (demand.allowance - action_uncertainty).next_down();
                if slack <= 0. {
                    return Err(self.refusal(demand, crate::derived::RefinementRefusal::Precision));
                }
                let linear_demand = AccuracyDemand {
                    allowance: slack,
                    ..*demand
                };
                Some((
                    direction,
                    self.backward_allowance(&selected, &linear_demand, inverse)?,
                ))
            } else {
                None
            };
            selected = self
                .selection
                .refine_numerical(crate::implicit::NumericalRefinement {
                    parameters: x,
                    unknown_scales: &scales,
                    row_scales: &rows,
                    root_allowance,
                    residual_allowance,
                    linear,
                    product: (
                        demand.product,
                        self.contract.source(),
                        self.contract.validity(),
                    ),
                    deadline,
                    cancel: &self.cancel,
                })?;
        }
        Err(self.refusal(demand, crate::derived::RefinementRefusal::Rounds))
    }
}
impl<E: From<MathError> + std::fmt::Debug> ReconstructionOracle
    for SelectedImplicitReconstruction<E>
{
    type Error = E;
    fn contract(&self) -> &ReconstructionContract {
        &self.contract
    }
    fn supports_uncertainty(&self) -> bool {
        true
    }
    fn refinement_refusal(&self) -> Option<crate::derived::RefinementRefusal> {
        self.last_refusal
    }
    fn realization(&self) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str("actual-selected-reconstruction")
            .hash(&self.contract.source());
        match self.selection.sheet_identity() {
            Some(sheet) => {
                h.bool(true).hash(&sheet);
            }
            None => {
                h.bool(false);
            }
        }
        h.finish_hash()
    }
    fn admit(&mut self, x: &[f64]) -> Result<ReconstructionAdmission, E> {
        self.check_input(x)?;
        let selected = self
            .selection
            .evaluate(x, DerivativeOrder::First, &self.cancel)?;
        if self.selection.sheet_identity().is_none() || self.selection.selected_chart().is_none() {
            return Err(MathError::Contract(
                "selected reconstruction regular root sheet is unestablished".into(),
            )
            .into());
        }
        Ok(ReconstructionAdmission {
            values: self.full(x, &selected.values)?,
        })
    }
    fn point(
        &mut self,
        x: &[f64],
        demand: &AccuracyDemand,
        refinement: crate::derived::RefinementLimits,
    ) -> Result<ReconstructionObservation, E> {
        let result = self.refined(x, None, demand, refinement);
        self.last_refusal = match &result {
            Err(MathError::Refinement { reason, .. }) => Some(*reason),
            _ => None,
        };
        result.map_err(E::from)
    }
    fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        demand: &AccuracyDemand,
        refinement: crate::derived::RefinementLimits,
    ) -> Result<ReconstructionObservation, E> {
        let result = self.refined(x, Some(direction), demand, refinement);
        self.last_refusal = match &result {
            Err(MathError::Refinement { reason, .. }) => Some(*reason),
            _ => None,
        };
        result.map_err(E::from)
    }
    fn uncertain(
        &mut self,
        x: &[f64],
        direction: Option<&[f64]>,
        incoming: crate::derived::ReconstructionUncertainty<'_>,
        demand: &AccuracyDemand,
        limits: crate::derived::RefinementLimits,
    ) -> Result<ReconstructionObservation, E> {
        let result = (|| -> Result<_, MathError> {
            if incoming.point.len() != x.len()
                || incoming.action.is_some_and(|v| v.len() != x.len())
                || incoming
                    .point
                    .iter()
                    .chain(incoming.action.into_iter().flatten())
                    .any(|e| !e.is_finite() || *e < 0.0)
                || direction.is_some() != incoming.action.is_some()
            {
                return Err(MathError::Contract(
                    "selected incoming uncertainty extent/value".into(),
                ));
            }
            if incoming
                .point
                .iter()
                .chain(incoming.action.into_iter().flatten())
                .all(|e| *e == 0.0)
            {
                return self.refined(x, direction, demand, limits);
            }
            if incoming.class != AccuracyClass::Certified
                && demand.class == AccuracyClass::Certified
            {
                return Err(self.refusal(
                    demand,
                    crate::derived::RefinementRefusal::Unavailable(SelectionProofRefusal::Coverage),
                ));
            }
            limits.validate()?;
            let deadline = self.selection.refinement_deadline()?;
            let before = self
                .selection
                .observed_point_cells()
                .checked_add(self.selection.observed_action_cells())
                .ok_or(MathError::Limit("incoming uncertainty proof observation"))?;
            let exact = self.refined(x, direction, demand, limits)?;
            let after = self
                .selection
                .observed_point_cells()
                .checked_add(self.selection.observed_action_cells())
                .ok_or(MathError::Limit("incoming uncertainty proof observation"))?;
            let remaining = limits
                .proof_cells
                .checked_sub(after.saturating_sub(before))
                .ok_or(MathError::Limit("incoming uncertainty proof allowance"))?;
            if remaining == 0 {
                return Err(self.refusal(demand, crate::derived::RefinementRefusal::ProofCells));
            }
            let interval = |value: f64,
                            error: f64,
                            column: &GlobalCol|
             -> Result<super::ProofInterval, MathError> {
                let radius = (error * self.normalization.variables[column.get()]).next_up();
                let interval = super::ProofInterval {
                    lower: (value - radius).next_down(),
                    upper: (value + radius).next_up(),
                };
                if !interval.valid() {
                    return Err(MathError::Contract(
                        "incoming physical uncertainty interval".into(),
                    ));
                }
                Ok(interval)
            };
            let parameters = x
                .iter()
                .zip(incoming.point)
                .zip(self.contract.retained())
                .map(|((x, error), c)| interval(*x, *error, c))
                .collect::<Result<Vec<_>, _>>()?;
            let directions = direction
                .zip(incoming.action)
                .map(|(v, errors)| {
                    v.iter()
                        .zip(errors)
                        .zip(self.contract.retained())
                        .map(|((v, error), c)| interval(*v, *error, c))
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?;
            let (points, actions) = match self.selection.enclose_neighborhood(
                x,
                &parameters,
                directions.as_deref(),
                remaining,
                deadline,
                &self.cancel,
            )? {
                super::RootNeighborhoodEvidence::Enclosed {
                    points, actions, ..
                } => (points, actions),
                super::RootNeighborhoodEvidence::Incomplete { reason, .. } => {
                    return Err(self.refusal(
                        demand,
                        crate::derived::RefinementRefusal::Unavailable(reason),
                    ));
                }
                super::RootNeighborhoodEvidence::Interrupted { .. } => {
                    return Err(MathError::Cancelled);
                }
            };
            let intervals = if direction.is_some() {
                actions
                    .as_deref()
                    .ok_or_else(|| MathError::Contract("uniform incoming action absent".into()))?
            } else {
                &points
            };
            let unknowns: Vec<_> = self
                .unknown_columns
                .iter()
                .map(|c| exact.values[c.get()])
                .collect();
            let mut evidence = self.accuracy(&unknowns, intervals, demand)?;
            if incoming.class != AccuracyClass::Certified {
                evidence.class = AccuracyClass::Estimated;
            }
            if !evidence.satisfies(demand) {
                return Err(self.refusal(demand, crate::derived::RefinementRefusal::Precision));
            }
            self.selection
                .refinement_checkpoint(deadline, &self.cancel)?;
            Ok(ReconstructionObservation {
                values: exact.values,
                accuracy: evidence,
            })
        })();
        self.last_refusal = match &result {
            Err(MathError::Refinement { reason, .. }) => Some(*reason),
            _ => None,
        };
        result.map_err(E::from)
    }
}
fn maps(
    factory: &RegimeFactory,
    original: &OriginalContract,
    retained: &[GlobalCol],
    eliminated: &[GlobalRow],
) -> Result<Vec<GlobalCol>, MathError> {
    if factory.alternatives.is_empty()
        || factory.spec.inputs.len() != retained.len()
        || retained
            .iter()
            .any(|c| c.get() >= original.coordinates().len())
        || eliminated
            .iter()
            .any(|r| r.get() >= original.constraints().len())
        || factory
            .spec
            .inputs
            .iter()
            .zip(retained)
            .any(|(p, c)| p.id != original.coordinates()[c.get()].id)
        || factory.spec.derivatives < DerivativeOrder::First
        || factory.spec.smoothness < DerivativeOrder::First
    {
        return Err(MathError::Contract(
            "selected reconstruction actual input/support map".into(),
        ));
    }
    let unknown_columns = factory
        .spec
        .outputs
        .iter()
        .map(|p| {
            original
                .coordinates()
                .iter()
                .position(|c| c.id == p.id)
                .map(GlobalCol::new)
                .ok_or_else(|| {
                    MathError::Contract("selected unknown is not an original coordinate".into())
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut covered = retained.iter().copied().collect::<BTreeSet<_>>();
    if unknown_columns.is_empty()
        || unknown_columns.iter().any(|c| !covered.insert(*c))
        || covered.len() != original.coordinates().len()
        || factory.alternatives.iter().any(|b| {
            b.residual.rows.len() != eliminated.len()
                || b.residual
                    .rows
                    .iter()
                    .zip(eliminated)
                    .any(|(id, r)| *id != original.constraints()[r.get()].id)
                || b.residual.unknowns.len() != unknown_columns.len()
                || b.residual
                    .unknowns
                    .iter()
                    .zip(&unknown_columns)
                    .any(|(u, c)| {
                        u.id != original.coordinates()[c.get()].id
                            || u.lower.to_bits() != original.coordinates()[c.get()].lower.to_bits()
                            || u.upper.to_bits() != original.coordinates()[c.get()].upper.to_bits()
                    })
                || b.residual.requirements.requested_output < DerivativeOrder::First
        })
    {
        return Err(MathError::Contract(
            "selected reconstruction original equality/unknown/bound correspondence".into(),
        ));
    }
    Ok(unknown_columns)
}
