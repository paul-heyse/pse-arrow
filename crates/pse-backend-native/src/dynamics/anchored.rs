// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shooting windows (ADR-0110 Outcome 5). [`Anchored`] wraps a dynamic oracle so that the
//! start of every differential state is a parameter: forward and adjoint sensitivities then
//! reach the node states of multiple shooting, and one window can start where another's
//! node says. It can also integrate the declared quadratures as differential states observed
//! as outputs, so an integral objective becomes a functional of the sampled outputs, the
//! form the adjoint route takes. Both are exact reformulations of the same compiled
//! functions; nothing is integrated or differentiated here.
use super::*;
use std::collections::BTreeMap;
mod layout;

/// A dynamic oracle whose parameters are the inner parameters followed by one anchor per
/// differential state, in state order: the anchors are the differential start values, and
/// the algebraic starts keep the inner guesses for consistent initialization. With observed
/// quadratures, the states are the inner states followed by one state per declared
/// quadrature (zero at the start, rate the quadrature's flux), the outputs gain the
/// quadratures' values, and no quadrature is declared.
///
/// ```
/// # #[cfg(feature = "diffsol")] {
/// use pse_backend_native::dynamics::{Anchored, Profile};
/// # use pse_backend_native::dynamics::{Contract, Evaluation, Function, Oracle, SupportEntry};
/// # use pse_backend_native::ProblemError;
/// # use pse_ids::{ContentHash, SemanticId};
/// # #[derive(Debug)]
/// # struct Decay(Contract);
/// # impl Oracle for Decay {
/// #     fn contract(&self) -> &Contract { &self.0 }
/// #     fn support(&self, _: usize, _: Function) -> Vec<SupportEntry> { vec![] }
/// #     fn evaluate(&mut self, _: usize, f: Function, _: f64, x: &[f64], p: &[f64], _: bool)
/// #         -> Result<Evaluation, ProblemError> {
/// #         let values = match f { Function::Rhs => vec![-p[0] * x[0]], Function::Initial => vec![1.0], _ => x.to_vec() };
/// #         Ok(Evaluation { values, jacobian: None })
/// #     }
/// # }
/// # let id = |n| SemanticId::from_bytes([n; 16]);
/// # let contract = Contract { quadratures: vec![], balances: vec![], identity: ContentHash::from_bytes([1; 32]),
/// #     states: vec![id(1)], differential: vec![true], parameters: vec![id(2)], outputs: vec![id(1)],
/// #     events: vec![vec![]], signs: vec![], derivatives: pse_kernels::DerivativeOrder::First };
/// // One differential state `x' = −k·x`: the window's parameters are `k`, then x's start.
/// let mut window = Anchored::new(Decay(contract), false).unwrap();
/// assert_eq!(window.contract().parameters.len(), 2);
/// let start = window.evaluate(0, Function::Initial, 0.5, &[0.0], &[0.3, 4.0], false).unwrap();
/// assert_eq!(start.values, vec![4.0]);
/// // A window profile gives the anchor a unit scale.
/// let profile = window.profile(&Profile { parameter_scales: vec![1.0], ..Profile::default() });
/// assert_eq!(profile.parameter_scales, vec![1.0, 1.0]);
/// # }
/// ```
#[derive(Debug)]
pub struct Anchored<O> {
    inner: O,
    contract: Contract,
    /// Differential state indices: anchor k starts state `anchors[k]`.
    anchors: Vec<usize>,
    /// Inner states, quadratures, parameters and outputs.
    n: usize,
    nq: usize,
    np: usize,
    m: usize,
    /// Declared quadratures are integrated as states and observed as outputs.
    observed: bool,
    prepared: Option<Arc<AnchoredLayout>>,
    snapshots: BTreeMap<
        (usize, Function),
        pse_math::SharedAllocation<faer::sparse::SparseColMat<usize, f64>>,
    >,
    diagonal: Option<DiagonalScratch>,
}
#[derive(Debug)]
struct DiagonalScratch {
    values: Vec<f64>,
    _owner: Arc<dyn pse_math::AllocationOwner>,
}
/// Source-issued immutable support for one anchored formulation. The source identity
/// and observed-quadrature choice prevent reuse for a different compiled problem.
/// Its allocation owner follows every borrowed native window view.
#[derive(Debug)]
pub struct AnchoredLayout {
    identity: ContentHash,
    observed: bool,
    layout: Arc<PreparedLayout>,
    functions: BTreeMap<(usize, Function), layout::FunctionLayout>,
    _owner: Arc<dyn pse_math::AllocationOwner>,
}
struct Derivative {
    prepared: Arc<AnchoredLayout>,
    matrix: pse_math::SharedAllocation<faer::sparse::SparseColMat<usize, f64>>,
    mode: usize,
    function: Function,
}
impl<O: Oracle> Anchored<O> {
    /// Wrap `inner`; `observe_quadratures` integrates its quadratures as observed states,
    /// which conservation balances over those quadratures would contradict.
    pub fn new(inner: O, observe_quadratures: bool) -> Result<Self, ProblemError> {
        let c = inner.contract().clone();
        c.validate()?;
        if observe_quadratures && !c.balances.is_empty() {
            return Err(contract(
                "observed quadratures cannot carry conservation balances",
            ));
        }
        let anchors = (0..c.states.len())
            .filter(|i| c.differential[*i])
            .collect::<Vec<_>>();
        let observed = observe_quadratures && !c.quadratures.is_empty();
        let mut identity = FramedHasher::new(pse_ids::Frame::ShootingWindowV1);
        identity.hash(&c.identity).bool(observed);
        let mut contract = Contract {
            identity: identity.finish_hash(),
            states: c.states.clone(),
            differential: c.differential.clone(),
            parameters: c.parameters.clone(),
            outputs: c.outputs.clone(),
            quadratures: c.quadratures.clone(),
            balances: c.balances.clone(),
            events: c.events.clone(),
            signs: c.signs.clone(),
            derivatives: c.derivatives,
        };
        contract.parameters.extend(
            anchors
                .iter()
                .map(|i| pse_ids::named_id(c.states[*i], "shooting.anchor")),
        );
        if observed {
            contract.states.extend(c.quadratures.iter().copied());
            contract
                .differential
                .extend(std::iter::repeat_n(true, c.quadratures.len()));
            contract.outputs.extend(c.quadratures.iter().copied());
            // An observed quadrature integrates without a sign of its own.
            if !contract.signs.is_empty() {
                contract
                    .signs
                    .extend(std::iter::repeat_n(StateSign::Free, c.quadratures.len()));
            }
            contract.quadratures.clear();
        }
        contract.validate()?;
        Ok(Self {
            n: c.states.len(),
            nq: c.quadratures.len(),
            np: c.parameters.len(),
            m: c.outputs.len(),
            inner,
            contract,
            anchors,
            observed,
            prepared: None,
            snapshots: BTreeMap::new(),
            diagonal: None,
        })
    }
    /// Prepare the transformed all-mode state layout once, after reserving its
    /// source-issued construction allowance. Numeric window data does not enter it.
    ///
    /// # Errors
    /// Missing allocation authority, insufficient allowance or invalid support.
    pub fn prepare_layout(&self, allowance: usize) -> Result<Arc<AnchoredLayout>, ProblemError> {
        if allowance < size_of::<AnchoredLayout>() {
            return Err(ProblemError::memory("anchored prepared allowance"));
        }
        let owner = self.snapshot_owner(allowance)?;
        let functions = layout::functions(self, allowance)?;
        let function_bytes = functions
            .values()
            .try_fold(size_of::<AnchoredLayout>(), |n, f| n.checked_add(f.bytes()))
            .ok_or_else(|| ProblemError::memory("anchored prepared extent"))?;
        let remaining = allowance
            .checked_sub(function_bytes)
            .ok_or_else(|| ProblemError::memory("anchored prepared allowance"))?;
        let layout = self.prepared_layout(remaining)?;
        if layout.retained_bytes() > remaining {
            return Err(ProblemError::memory("anchored native layout allowance"));
        }
        Ok(Arc::new(AnchoredLayout {
            identity: self.contract.identity,
            observed: self.observed,
            layout,
            functions,
            _owner: owner,
        }))
    }
    /// Conservative finite construction extent from the source's complete total
    /// inner support-entry count across all modes and functions. A smaller declared
    /// count cannot authorize larger construction: preparation checks its real extent.
    ///
    /// # Errors
    /// Arithmetic overflow or an unsupported state/index extent.
    pub fn layout_allocation_bound(&self, inner_entries: usize) -> Result<usize, ProblemError> {
        let modes = self.contract.events.len();
        let events = self
            .contract
            .events
            .iter()
            .try_fold(0usize, |n, e| n.checked_add(e.len()))
            .ok_or_else(|| ProblemError::memory("anchored event extent"))?;
        let roles = modes
            .checked_mul(
                3 + usize::from(self.nq > 0) + usize::from(!self.contract.balances.is_empty()),
            )
            .and_then(|n| {
                n.checked_add(
                    self.contract
                        .events
                        .iter()
                        .filter(|e| !e.is_empty())
                        .count(),
                )
            })
            .and_then(|n| {
                n.checked_add(
                    events.checked_mul(1 + usize::from(!self.contract.balances.is_empty()))?,
                )
            })
            .ok_or_else(|| ProblemError::memory("anchored function extent"))?;
        let entries = inner_entries
            .checked_mul(if self.observed { 2 } else { 1 })
            .and_then(|n| n.checked_add(self.anchors.len().checked_mul(modes)?))
            .and_then(|n| n.checked_add(self.states().checked_mul(modes.checked_add(events)?)?))
            .ok_or_else(|| ProblemError::memory("anchored support extent"))?;
        PreparedLayout::construction_bound(self.contract.states.len(), entries)
            .and_then(|n| n.checked_add(entries.checked_mul(256)?))
            .and_then(|n| {
                n.checked_add(
                    self.width()
                        .checked_add(1)?
                        .checked_mul(roles)?
                        .checked_mul(32)?,
                )
            })
            .and_then(|n| {
                n.checked_add(roles.checked_mul(512 + size_of::<layout::FunctionLayout>())?)
            })
            .and_then(|n| n.checked_add(size_of::<AnchoredLayout>() + 512))
            .and_then(|n| {
                n.checked_add(if self.observed {
                    self.n
                        .checked_mul(size_of::<f64>())?
                        .checked_add(size_of::<DiagonalScratch>() + 2 * size_of::<usize>())?
                } else {
                    0
                })
            })
            .ok_or_else(|| ProblemError::memory("anchored construction extent"))
    }
    /// Bind support prepared by this exact anchored source. The native IVP is still
    /// initialized freshly; this retains only immutable structural destinations.
    ///
    /// # Errors
    /// A different source, formulation or state extent.
    pub fn use_prepared_layout(
        &mut self,
        prepared: Arc<AnchoredLayout>,
    ) -> Result<(), ProblemError> {
        if prepared.identity != self.contract.identity
            || prepared.observed != self.observed
            || prepared.layout.states() != self.contract.states.len()
        {
            return Err(ProblemError::Contract(
                "anchored layout source differs".into(),
            ));
        }
        self.snapshots.clear();
        self.prepared = Some(prepared);
        Ok(())
    }
    /// The differential states the anchors start, in anchor order.
    pub fn anchors(&self) -> &[usize] {
        &self.anchors
    }
    /// The wrapped oracle.
    pub fn into_inner(self) -> O {
        self.inner
    }
    /// A window profile for this oracle: every anchor takes a unit scale (they are
    /// normalized state coordinates), and observed quadratures integrate as states under
    /// their declared absolute output tolerances.
    pub fn profile(&self, window: &Profile) -> Profile {
        let mut p = window.clone();
        p.parameter_scales
            .extend(std::iter::repeat_n(1.0, self.anchors.len()));
        if self.observed {
            p.atol.extend(window.out_atol.iter().copied());
            p.out_rtol = None;
            p.out_atol = Vec::new();
        }
        p
    }
    /// Observed quadrature states.
    fn states(&self) -> usize {
        if self.observed { self.nq } else { 0 }
    }
    /// The wrapper coordinate of an inner coordinate (state, then parameter).
    fn column(&self, inner: usize) -> usize {
        if inner < self.n {
            inner
        } else {
            inner + self.states()
        }
    }
    /// The wrapper coordinate of anchor `k`.
    fn anchor(&self, k: usize) -> usize {
        self.n + self.states() + self.np + k
    }
    fn width(&self) -> usize {
        self.n + self.states() + self.np + self.anchors.len()
    }
    /// Curvature excludes linear anchor and observed-quadrature contributions.
    fn curvature_terms(
        &self,
        mode: usize,
        function: Function,
        weights: &[f64],
    ) -> Result<Vec<(Function, Vec<f64>)>, ProblemError> {
        let n = self.n;
        let rows = match function {
            Function::Rhs | Function::Initial | Function::Reset(_) => self.contract.states.len(),
            Function::Output => self.contract.outputs.len(),
            Function::QuadratureFlux => self.contract.quadratures.len(),
            Function::Roots => self
                .contract
                .events
                .get(mode)
                .ok_or_else(|| ProblemError::Contract("anchored curvature mode".into()))?
                .len(),
            Function::Inventory | Function::Transfer(_) => self.contract.balances.len(),
        };
        if weights.len() != rows || weights.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::Contract("anchored curvature weights".into()));
        }
        let terms = match function {
            Function::Rhs => {
                let mut terms = vec![(Function::Rhs, weights[..n].to_vec())];
                if self.observed {
                    terms.push((Function::QuadratureFlux, weights[n..].to_vec()));
                }
                terms
            }
            Function::Initial => {
                let algebraic = weights[..n]
                    .iter()
                    .zip(&self.contract.differential)
                    .map(|(w, d)| if *d { 0.0 } else { *w })
                    .collect();
                vec![(Function::Initial, algebraic)]
            }
            Function::Output => vec![(Function::Output, weights[..self.m].to_vec())],
            Function::Reset(e) => vec![(Function::Reset(e), weights[..n].to_vec())],
            Function::QuadratureFlux
            | Function::Roots
            | Function::Inventory
            | Function::Transfer(_) => vec![(function, weights.to_vec())],
        };
        Ok(terms)
    }
    /// Inner support entries of `function`, rows offset by `row`, columns mapped.
    fn inner_support(&self, mode: usize, function: Function, row: usize) -> Vec<SupportEntry> {
        self.inner
            .support(mode, function)
            .into_iter()
            .map(|e| {
                SupportEntry::new(
                    OriginalRow::new(e.row.get() + row),
                    OriginalCol::new(self.column(e.col.get())),
                )
            })
            .collect()
    }
    /// Evaluate an inner function at the wrapper's point.
    fn evaluate_inner(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        derivatives: bool,
    ) -> Result<Evaluation, ProblemError> {
        self.inner.evaluate(
            mode,
            function,
            time,
            &state[..self.n],
            &parameters[..self.np],
            derivatives,
        )
    }
    fn derivative(&mut self, mode: usize, function: Function) -> Result<Derivative, ProblemError> {
        let prepared = self.prepared.clone().ok_or_else(|| {
            ProblemError::Contract("anchored derivative layout was not prepared".into())
        })?;
        let layout = prepared
            .functions
            .get(&(mode, function))
            .ok_or_else(|| ProblemError::Contract("anchored function was not prepared".into()))?;
        let mut matrix = self.snapshots.remove(&(mode, function));
        if matrix.as_mut().and_then(|m| m.get_mut()).is_none() {
            let bytes = layout
                .matrix
                .retained_bytes()
                .checked_add(2 * size_of::<usize>())
                .ok_or_else(|| ProblemError::memory("anchored snapshot extent"))?;
            let owner = self.inner.snapshot_owner(bytes)?;
            matrix = Some(
                pse_math::SharedAllocation::from(Arc::new(layout.matrix.matrix().clone()))
                    .with_owner(owner),
            );
        }
        let mut matrix =
            matrix.ok_or_else(|| ProblemError::internal("anchored snapshot absent"))?;
        layout.initialize(
            matrix
                .get_mut()
                .ok_or_else(|| ProblemError::internal("anchored snapshot still borrowed"))?,
        );
        Ok(Derivative {
            prepared,
            matrix,
            mode,
            function,
        })
    }
    /// Refill the canonical prepared wrapper destinations; a source may emit a
    /// subset in another order, but cannot add support after admission.
    fn append(
        &self,
        candidate: &mut Option<Derivative>,
        jacobian: Option<&faer::sparse::SparseColMat<usize, f64>>,
        row: usize,
        keep: impl Fn(usize) -> bool,
    ) -> Result<(), ProblemError> {
        let Some(candidate) = candidate else {
            return Ok(());
        };
        let source = jacobian.ok_or_else(|| ProblemError::internal("anchored inner partials"))?;
        let expected_rows = match candidate.function {
            Function::Rhs if row == self.n => self.nq,
            Function::Rhs | Function::Initial | Function::Reset(_) => self.n,
            Function::Output => self.m,
            Function::QuadratureFlux => self.nq,
            Function::Roots => self.contract.events[candidate.mode].len(),
            Function::Inventory | Function::Transfer(_) => self.contract.balances.len(),
        };
        if source.ncols() != self.n + self.np || source.nrows() != expected_rows {
            return Err(ProblemError::Contract(
                "anchored inner partial dimensions".into(),
            ));
        }
        let layout = candidate
            .prepared
            .functions
            .get(&(candidate.mode, candidate.function))
            .ok_or_else(|| ProblemError::internal("anchored prepared function absent"))?;
        let matrix = candidate
            .matrix
            .get_mut()
            .ok_or_else(|| ProblemError::internal("anchored snapshot still borrowed"))?;
        for column in 0..source.ncols() {
            for k in source.col_range(column) {
                let source_row = source.row_idx()[k];
                if keep(source_row) {
                    let slot = layout
                        .slot(source_row + row, self.column(column))
                        .ok_or_else(|| {
                            ProblemError::Contract(
                                "anchored partial outside prepared support".into(),
                            )
                        })?;
                    let value = matrix.val()[slot] + source.val()[k];
                    if !value.is_finite() {
                        return Err(ProblemError::numerical("nonfinite anchored partial"));
                    }
                    matrix.val_mut()[slot] = value;
                }
            }
        }
        Ok(())
    }
    /// Embed an inner lower-triangle Hessian into the wrapper's coordinates.
    fn embed(
        &self,
        triplets: &mut Vec<faer::sparse::Triplet<usize, usize, f64>>,
        h: &faer::sparse::SparseColMat<usize, f64>,
    ) {
        for col in 0..h.ncols() {
            for k in h.col_range(col) {
                triplets.push(faer::sparse::Triplet::new(
                    self.column(h.row_idx()[k]),
                    self.column(col),
                    h.val()[k],
                ));
            }
        }
    }
}
impl<O: Oracle> Oracle for Anchored<O> {
    fn prepared_layout(&self, allowance: usize) -> Result<Arc<PreparedLayout>, ProblemError> {
        if let Some(prepared) = &self.prepared {
            if prepared.layout.retained_bytes() > allowance {
                return Err(ProblemError::memory("anchored layout exceeds allowance"));
            }
            return Ok(prepared.layout.clone());
        }
        // Anchors add parameters, not state support. Plain windows share the
        // original all-mode state layout without querying support again.
        if !self.observed {
            return self.inner.prepared_layout(allowance);
        }
        let mut supports = Vec::with_capacity(self.contract.events.len());
        let mut support_bytes = supports
            .capacity()
            .checked_mul(size_of::<Vec<SupportEntry>>())
            .ok_or_else(|| ProblemError::memory("anchored support headers extent"))?;
        for mode in 0..self.contract.events.len() {
            let support = self.support(mode, Function::Rhs);
            support_bytes = support_bytes
                .checked_add(
                    support
                        .capacity()
                        .checked_mul(size_of::<SupportEntry>())
                        .ok_or_else(|| ProblemError::memory("anchored source support extent"))?,
                )
                .ok_or_else(|| ProblemError::memory("anchored source support extent"))?;
            if support_bytes > allowance {
                return Err(ProblemError::memory(
                    "anchored source support exceeds allowance",
                ));
            }
            supports.push(support);
        }
        Ok(Arc::new(PreparedLayout::new(
            self.contract.states.len(),
            supports.iter().flat_map(|support| support.iter().copied()),
            allowance - support_bytes,
        )?))
    }
    fn snapshot_owner(
        &self,
        bytes: usize,
    ) -> Result<Arc<dyn pse_math::AllocationOwner>, ProblemError> {
        self.inner.snapshot_owner(bytes)
    }
    fn contract(&self) -> &Contract {
        &self.contract
    }
    fn support(&self, mode: usize, function: Function) -> Vec<SupportEntry> {
        let (n, quadratures) = (self.n, self.states());
        match function {
            Function::Rhs => {
                let mut entries = self.inner_support(mode, Function::Rhs, 0);
                if self.observed {
                    entries.extend(self.inner_support(mode, Function::QuadratureFlux, n));
                }
                entries
            }
            Function::Initial => {
                let differential = &self.contract.differential;
                let mut entries = self
                    .inner_support(mode, Function::Initial, 0)
                    .into_iter()
                    .filter(|e| !differential[e.row.get()])
                    .collect::<Vec<_>>();
                entries.extend(self.anchors.iter().enumerate().map(|(k, i)| {
                    SupportEntry::new(OriginalRow::new(*i), OriginalCol::new(self.anchor(k)))
                }));
                entries
            }
            Function::Output => {
                let mut entries = self.inner_support(mode, Function::Output, 0);
                entries.extend((0..quadratures).map(|q| {
                    SupportEntry::new(OriginalRow::new(self.m + q), OriginalCol::new(n + q))
                }));
                entries
            }
            Function::Reset(e) => {
                let mut entries = self.inner_support(mode, Function::Reset(e), 0);
                entries.extend(
                    (0..quadratures).map(|q| {
                        SupportEntry::new(OriginalRow::new(n + q), OriginalCol::new(n + q))
                    }),
                );
                entries
            }
            Function::QuadratureFlux
            | Function::Roots
            | Function::Inventory
            | Function::Transfer(_) => self.inner_support(mode, function, 0),
        }
    }
    fn rhs_state_diagonal(
        &mut self,
        mode: usize,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        output: &mut [f64],
    ) -> Result<RhsDiagonalSource, ProblemError> {
        if mode >= self.contract.events.len()
            || state.len() != self.contract.states.len()
            || output.len() != state.len()
            || parameters.len() != self.contract.parameters.len()
            || !time.is_finite()
            || state.iter().chain(parameters).any(|v| !v.is_finite())
        {
            return Err(contract("anchored RHS diagonal bindings"));
        }
        if !self.observed {
            return self.inner.rhs_state_diagonal(
                mode,
                time,
                state,
                &parameters[..self.np],
                output,
            );
        }
        if self.diagonal.is_none() {
            let bytes = self
                .n
                .checked_mul(size_of::<f64>())
                .and_then(|n| n.checked_add(size_of::<DiagonalScratch>() + 2 * size_of::<usize>()))
                .ok_or_else(|| ProblemError::memory("anchored diagonal scratch extent"))?;
            let owner = self.inner.snapshot_owner(bytes)?;
            self.diagonal = Some(DiagonalScratch {
                values: vec![f64::NAN; self.n],
                _owner: owner,
            });
        }
        let candidate = self
            .diagonal
            .as_mut()
            .ok_or_else(|| ProblemError::internal("anchored diagonal scratch absent"))?;
        candidate.values.fill(f64::NAN);
        let source = self.inner.rhs_state_diagonal(
            mode,
            time,
            &state[..self.n],
            &parameters[..self.np],
            &mut candidate.values,
        )?;
        if candidate.values.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("nonfinite anchored raw diagonal"));
        }
        // A structural-zero self derivative does not remove the flux's current
        // Value/domain/provider checks. Preserve RHS -> flux order and publish
        // only after both sources complete; no flux First product is requested.
        let flux = self.inner.evaluate(
            mode,
            Function::QuadratureFlux,
            time,
            &state[..self.n],
            &parameters[..self.np],
            false,
        )?;
        if flux.values.len() != self.nq {
            return Err(contract("anchored diagonal flux dimensions"));
        }
        if flux.values.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("nonfinite anchored diagonal flux"));
        }
        output[..self.n].copy_from_slice(&candidate.values);
        output[self.n..].fill(0.);
        Ok(source)
    }
    fn evaluate(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        derivatives: bool,
    ) -> Result<Evaluation, ProblemError> {
        let (n, quadratures) = (self.n, self.states());
        if state.len() != n + quadratures || parameters.len() != self.np + self.anchors.len() {
            return Err(contract("anchored binding dimensions"));
        }
        // Missing preparation and snapshot admission refuse before inner effects.
        let mut candidate = derivatives
            .then(|| self.derivative(mode, function))
            .transpose()?;
        let (values, rows) = match function {
            Function::Rhs => {
                let f =
                    self.evaluate_inner(mode, Function::Rhs, time, state, parameters, derivatives)?;
                let mut values = f.values;
                if derivatives {
                    self.append(&mut candidate, f.jacobian.as_deref(), 0, |_| true)?;
                }
                if self.observed {
                    let g = self.evaluate_inner(
                        mode,
                        Function::QuadratureFlux,
                        time,
                        state,
                        parameters,
                        derivatives,
                    )?;
                    values.extend(g.values);
                    if derivatives {
                        self.append(&mut candidate, g.jacobian.as_deref(), n, |_| true)?;
                    }
                }
                let rows = values.len();
                (values, rows)
            }
            Function::Initial => {
                let guess = self.evaluate_inner(
                    mode,
                    Function::Initial,
                    time,
                    state,
                    parameters,
                    derivatives,
                )?;
                let mut values = guess.values;
                for (k, i) in self.anchors.iter().enumerate() {
                    values[*i] = parameters[self.np + k];
                }
                values.extend(std::iter::repeat_n(0.0, quadratures));
                if derivatives {
                    let differential = self.contract.differential.clone();
                    self.append(&mut candidate, guess.jacobian.as_deref(), 0, |r| {
                        !differential[r]
                    })?;
                }
                (values, n + quadratures)
            }
            Function::Output => {
                let y = self.evaluate_inner(
                    mode,
                    Function::Output,
                    time,
                    state,
                    parameters,
                    derivatives,
                )?;
                let mut values = y.values;
                values.extend_from_slice(&state[n..]);
                if derivatives {
                    self.append(&mut candidate, y.jacobian.as_deref(), 0, |_| true)?;
                }
                (values, self.m + quadratures)
            }
            Function::Reset(e) => {
                let r = self.evaluate_inner(
                    mode,
                    Function::Reset(e),
                    time,
                    state,
                    parameters,
                    derivatives,
                )?;
                let mut values = r.values;
                values.extend_from_slice(&state[n..]);
                if derivatives {
                    self.append(&mut candidate, r.jacobian.as_deref(), 0, |_| true)?;
                }
                (values, n + quadratures)
            }
            Function::QuadratureFlux
            | Function::Roots
            | Function::Inventory
            | Function::Transfer(_) => {
                let v =
                    self.evaluate_inner(mode, function, time, state, parameters, derivatives)?;
                if derivatives {
                    self.append(&mut candidate, v.jacobian.as_deref(), 0, |_| true)?;
                }
                let rows = v.values.len();
                (v.values, rows)
            }
        };
        let jacobian = if let Some(candidate) = candidate {
            if candidate.matrix.nrows() != rows {
                return Err(ProblemError::Contract(
                    "anchored function row dimensions".into(),
                ));
            }
            self.snapshots
                .insert((mode, function), candidate.matrix.clone());
            Some(candidate.matrix)
        } else {
            None
        };
        Ok(Evaluation { values, jacobian })
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "explicit function, physical point and runtime product bindings have distinct scientific meanings"
    )]
    fn jacobian_action(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        direction: &[f64],
        output: &mut [f64],
    ) -> Result<(), ProblemError> {
        if state.len() != self.contract.states.len()
            || parameters.len() != self.contract.parameters.len()
            || direction.len() != self.width()
            || direction.iter().any(|v| !v.is_finite())
        {
            return Err(ProblemError::Contract(
                "anchored Jacobian action binding".into(),
            ));
        }
        let quadratures = self.states();
        let rows = match function {
            Function::Rhs | Function::Initial | Function::Reset(_) => self.n + quadratures,
            Function::Output => self.m + quadratures,
            Function::QuadratureFlux => self.nq,
            Function::Roots => self
                .contract
                .events
                .get(mode)
                .ok_or_else(|| ProblemError::Contract("anchored action mode".into()))?
                .len(),
            Function::Inventory | Function::Transfer(_) => self.contract.balances.len(),
        };
        if output.len() != rows {
            return Err(ProblemError::Contract(
                "anchored Jacobian action output".into(),
            ));
        }
        let seed = (0..self.n + self.np)
            .map(|i| direction[self.column(i)])
            .collect::<Vec<_>>();
        let mut candidate = vec![0.; rows];
        let inner_rows = match function {
            Function::Rhs | Function::Initial | Function::Reset(_) => self.n,
            Function::Output => self.m,
            _ => rows,
        };
        self.inner.jacobian_action(
            mode,
            function,
            time,
            &state[..self.n],
            &parameters[..self.np],
            &seed,
            &mut candidate[..inner_rows],
        )?;
        match function {
            Function::Rhs if self.observed => self.inner.jacobian_action(
                mode,
                Function::QuadratureFlux,
                time,
                &state[..self.n],
                &parameters[..self.np],
                &seed,
                &mut candidate[self.n..],
            )?,
            Function::Initial => {
                for (k, &row) in self.anchors.iter().enumerate() {
                    candidate[row] = direction[self.anchor(k)];
                }
            }
            Function::Output => {
                candidate[self.m..].copy_from_slice(&direction[self.n..self.n + quadratures])
            }
            Function::Reset(_) => {
                candidate[self.n..].copy_from_slice(&direction[self.n..self.n + quadratures])
            }
            _ => {}
        }
        if candidate.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical(
                "nonfinite anchored Jacobian action",
            ));
        }
        output.copy_from_slice(&candidate);
        Ok(())
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "physical point and cotangent bindings are independent scientific inputs"
    )]
    fn jacobian_transpose_action(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        weights: &[f64],
        output: &mut [f64],
    ) -> Result<(), ProblemError> {
        if state.len() != self.contract.states.len()
            || parameters.len() != self.contract.parameters.len()
            || output.len() != self.width()
            || state.iter().chain(parameters).any(|v| !v.is_finite())
        {
            return Err(ProblemError::Contract("anchored transpose binding".into()));
        }
        // The nonlinear terms have the same cotangent restriction as curvature;
        // linear anchors and observed-quadrature identities are joined separately.
        let terms = self.curvature_terms(mode, function, weights)?;
        let mut candidate = vec![0.; self.width()];
        let mut local = vec![f64::NAN; self.n + self.np];
        for (role, cotangent) in terms {
            local.fill(f64::NAN);
            self.inner.jacobian_transpose_action(
                mode,
                role,
                time,
                &state[..self.n],
                &parameters[..self.np],
                &cotangent,
                &mut local,
            )?;
            for (column, value) in local.iter().enumerate() {
                candidate[self.column(column)] += value;
            }
        }
        match function {
            Function::Initial => {
                for (k, &row) in self.anchors.iter().enumerate() {
                    candidate[self.anchor(k)] += weights[row];
                }
            }
            Function::Output => {
                for q in 0..self.states() {
                    candidate[self.n + q] += weights[self.m + q];
                }
            }
            Function::Reset(_) => {
                for q in 0..self.states() {
                    candidate[self.n + q] += weights[self.n + q];
                }
            }
            _ => {}
        }
        if candidate.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical(
                "nonfinite anchored transpose action",
            ));
        }
        output.copy_from_slice(&candidate);
        Ok(())
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "explicit function, physical point and runtime product bindings have distinct scientific meanings"
    )]
    fn weighted_hessian_action(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        weights: &[f64],
        direction: &[f64],
        output: &mut [f64],
    ) -> Result<(), ProblemError> {
        if state.len() != self.contract.states.len()
            || parameters.len() != self.contract.parameters.len()
            || direction.len() != self.width()
            || output.len() != self.width()
            || direction.iter().any(|v| !v.is_finite())
        {
            return Err(ProblemError::Contract(
                "anchored Hessian action direction".into(),
            ));
        }
        let terms = self.curvature_terms(mode, function, weights)?;
        let inner_direction = (0..self.n + self.np)
            .map(|i| direction[self.column(i)])
            .collect::<Vec<_>>();
        let mut candidate = vec![0.; self.width()];
        for (function, weights) in terms {
            if weights.iter().all(|v| *v == 0.) {
                continue;
            }
            let mut product = vec![0.; inner_direction.len()];
            self.inner.weighted_hessian_action(
                mode,
                function,
                time,
                &state[..self.n],
                &parameters[..self.np],
                &weights,
                &inner_direction,
                &mut product,
            )?;
            for (i, value) in product.into_iter().enumerate() {
                candidate[self.column(i)] += value;
            }
        }
        if candidate.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("nonfinite anchored Hessian action"));
        }
        output.copy_from_slice(&candidate);
        Ok(())
    }
    fn weighted_hessian(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        weights: &[f64],
    ) -> Result<faer::sparse::SparseColMat<usize, f64>, ProblemError> {
        let (n, np) = (self.n, self.np);
        let (x, p) = (&state[..n], &parameters[..np]);
        let mut triplets = Vec::new();
        let terms = self.curvature_terms(mode, function, weights)?;
        for (f, w) in terms {
            if w.iter().all(|v| *v == 0.0) {
                continue;
            }
            let h = self.inner.weighted_hessian(mode, f, time, x, p, &w)?;
            self.embed(&mut triplets, &h);
        }
        faer::sparse::SparseColMat::try_new_from_triplets(self.width(), self.width(), &triplets)
            .map_err(|e| ProblemError::memory(format!("anchored Hessian: {e:?}")))
    }
}

#[cfg(test)]
#[cfg(feature = "diffsol")]
#[path = "anchored_tests.rs"]
mod tests;
