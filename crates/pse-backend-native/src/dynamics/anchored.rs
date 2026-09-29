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
/// #     events: vec![vec![]], derivatives: pse_kernels::DerivativeOrder::First };
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
        let mut identity = pse_ids::FramedHasher::new(pse_ids::Frame::ShootingWindowV1);
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
        })
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
    /// Append an inner Jacobian's entries, rows offset by `row` and columns mapped.
    fn append(
        &self,
        triplets: &mut Vec<faer::sparse::Triplet<usize, usize, f64>>,
        jacobian: Option<&faer::sparse::SparseColMat<usize, f64>>,
        row: usize,
        keep: impl Fn(usize) -> bool,
    ) -> Result<(), ProblemError> {
        let j = jacobian.ok_or_else(|| ProblemError::internal("anchored inner partials"))?;
        for col in 0..j.ncols() {
            for k in j.col_range(col) {
                let r = j.row_idx()[k];
                if keep(r) {
                    triplets.push(faer::sparse::Triplet::new(
                        r + row,
                        self.column(col),
                        j.val()[k],
                    ));
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
    fn matrix(
        &self,
        rows: usize,
        triplets: &[faer::sparse::Triplet<usize, usize, f64>],
    ) -> Result<faer::sparse::SparseColMat<usize, f64>, ProblemError> {
        faer::sparse::SparseColMat::try_new_from_triplets(rows, self.width(), triplets)
            .map_err(|e| ProblemError::memory(format!("anchored partials: {e:?}")))
    }
}
impl<O: Oracle> Oracle for Anchored<O> {
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
                entries.extend((0..quadratures).map(|q| {
                    SupportEntry::new(OriginalRow::new(n + q), OriginalCol::new(n + q))
                }));
                entries
            }
            Function::QuadratureFlux | Function::Roots => self.inner_support(mode, function, 0),
        }
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
        let mut triplets = Vec::new();
        let (values, rows) = match function {
            Function::Rhs => {
                let f = self.evaluate_inner(mode, Function::Rhs, time, state, parameters, derivatives)?;
                let mut values = f.values;
                if derivatives {
                    self.append(&mut triplets, f.jacobian.as_ref(), 0, |_| true)?;
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
                        self.append(&mut triplets, g.jacobian.as_ref(), n, |_| true)?;
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
                    self.append(&mut triplets, guess.jacobian.as_ref(), 0, |r| {
                        !differential[r]
                    })?;
                    for (k, i) in self.anchors.iter().enumerate() {
                        triplets.push(faer::sparse::Triplet::new(*i, self.anchor(k), 1.0));
                    }
                }
                (values, n + quadratures)
            }
            Function::Output => {
                let y = self.evaluate_inner(mode, Function::Output, time, state, parameters, derivatives)?;
                let mut values = y.values;
                values.extend_from_slice(&state[n..]);
                if derivatives {
                    self.append(&mut triplets, y.jacobian.as_ref(), 0, |_| true)?;
                    for q in 0..quadratures {
                        triplets.push(faer::sparse::Triplet::new(self.m + q, n + q, 1.0));
                    }
                }
                (values, self.m + quadratures)
            }
            Function::Reset(e) => {
                let r = self.evaluate_inner(mode, Function::Reset(e), time, state, parameters, derivatives)?;
                let mut values = r.values;
                values.extend_from_slice(&state[n..]);
                if derivatives {
                    self.append(&mut triplets, r.jacobian.as_ref(), 0, |_| true)?;
                    for q in 0..quadratures {
                        triplets.push(faer::sparse::Triplet::new(n + q, n + q, 1.0));
                    }
                }
                (values, n + quadratures)
            }
            Function::QuadratureFlux | Function::Roots => {
                let v = self.evaluate_inner(mode, function, time, state, parameters, derivatives)?;
                if derivatives {
                    self.append(&mut triplets, v.jacobian.as_ref(), 0, |_| true)?;
                }
                let rows = v.values.len();
                (v.values, rows)
            }
        };
        let jacobian = derivatives
            .then(|| self.matrix(rows, &triplets))
            .transpose()?;
        Ok(Evaluation { values, jacobian })
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
        // Anchors, observed quadrature states and their outputs enter linearly.
        let terms: Vec<(Function, Vec<f64>)> = match function {
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
            Function::QuadratureFlux | Function::Roots => vec![(function, weights.to_vec())],
        };
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

#[cfg(all(test, feature = "diffsol"))]
#[path = "anchored_tests.rs"]
mod tests;
