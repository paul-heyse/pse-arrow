// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native views of library-owned mathematical derived families.
use crate::{NleOracle, NlpOracle, OracleContract, ProblemError, RootOperations, Variable};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::DerivativeOrder;
use pse_math::{
    derived::{
        self as math, BoundFeasibility, BoundHomotopy, BoundProjection, BoundPseudoTime,
        BoundReduced, DerivativeSupport, ObjectiveOracle, Oracle, OriginalContract,
        ReconstructionOracle,
    },
    index::{Entry, GlobalCol, GlobalRow},
    sparse::AssemblyMatrix,
};
use pse_model::strategy::{AccuracyClass, AccuracyDemand};
use std::sync::Arc;

/// Native NLP supplier in the original authored representation. Its action assembles
/// the existing native Jacobian; the bridge claims no demanded directional program.
#[derive(Debug)]
pub struct NlpBridge {
    original: Arc<OriginalContract>,
    oracle: Box<dyn NlpOracle>,
}
impl NlpBridge {
    /// Bind exactly the original native physical intervals and coordinate order.
    pub fn new(
        oracle: Box<dyn NlpOracle>,
        original: Arc<OriginalContract>,
    ) -> Result<Self, ProblemError> {
        inventory(oracle.contract(), &original)?;
        oracle.contract().validate(original.support().order)?;
        if oracle
            .constraint_bounds()
            .iter()
            .zip(original.constraints())
            .any(|((l, u), r)| l.to_bits() != r.lower.to_bits() || u.to_bits() != r.upper.to_bits())
            || oracle.constraint_bounds().len() != original.constraints().len()
            || original.support().order > oracle.contract().derivatives
            || original.support().jacobian_product
                && oracle.contract().derivatives < DerivativeOrder::First
        {
            return Err(ProblemError::Contract(
                "derived NLP source bounds or actual derivative support".into(),
            ));
        }
        Ok(Self { original, oracle })
    }
    /// Return the original supplier for final correction and scientific assessment.
    pub fn into_original(self) -> Box<dyn NlpOracle> {
        self.oracle
    }
}
impl Oracle for NlpBridge {
    type Error = ProblemError;
    fn contract(&self) -> &OriginalContract {
        &self.original
    }
    fn values(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.oracle.constraints(x, out)
    }
    fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        if !self.original.support().jacobian_product {
            return Err(ProblemError::Unsupported(
                "original NLP action unavailable".into(),
            ));
        }
        let mut coefficients = vec![0.0; self.oracle.jacobian_pattern().compute_nnz()];
        self.oracle.jacobian(x, &mut coefficients)?;
        sparse_product(
            self.oracle.jacobian_pattern(),
            &coefficients,
            direction,
            out,
        )
    }
}
impl ObjectiveOracle for NlpBridge {
    fn objective_support(&self) -> DerivativeSupport {
        DerivativeSupport {
            order: self
                .oracle
                .contract()
                .derivatives
                .min(DerivativeOrder::First),
            jacobian_product: self.oracle.contract().derivatives >= DerivativeOrder::First,
            source: self
                .original
                .obligations()
                .objective
                .unwrap_or(self.original.support().source),
        }
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        self.oracle.objective(x)
    }
    fn objective_product(&mut self, x: &[f64], direction: &[f64]) -> Result<f64, ProblemError> {
        let mut gradient = vec![0.0; self.original.coordinates().len()];
        self.oracle.gradient(x, &mut gradient)?;
        if direction.len() != gradient.len() {
            return Err(ProblemError::Contract(
                "original objective action extent".into(),
            ));
        }
        let result = gradient
            .iter()
            .zip(direction)
            .map(|(a, v)| a * v)
            .sum::<f64>();
        finite(&[result])?;
        Ok(result)
    }
}
/// An explicitly offset original equality representation. Metadata remains available
/// to the final original assessment; zero residuals never replace authored bounds.
#[derive(Debug)]
pub struct RootBridge {
    original: Arc<OriginalContract>,
    oracle: Box<dyn NleOracle>,
    offsets: Vec<f64>,
}
impl RootBridge {
    /// The native supplier computes `authored_value - offset` in each named row.
    /// Every supplied offset must equal that original finite equality's authored bound.
    pub fn new(
        oracle: Box<dyn NleOracle>,
        original: Arc<OriginalContract>,
        offsets: Vec<f64>,
    ) -> Result<Self, ProblemError> {
        inventory(oracle.contract(), &original)?;
        if original.support().jacobian_product {
            oracle.operations().admit(oracle.contract(), true)?;
        }
        if offsets.len() != original.constraints().len()
            || offsets.iter().zip(original.constraints()).any(|(v, r)| {
                !v.is_finite()
                    || v.to_bits() != r.lower.to_bits()
                    || r.lower.to_bits() != r.upper.to_bits()
            })
            || original.support().order > oracle.contract().derivatives
                && original.support().order > DerivativeOrder::First
            || original.support().jacobian_product && !oracle.operations().jacobian_product
        {
            return Err(ProblemError::Contract(
                "explicit original equality offsets or actual root operation mismatch".into(),
            ));
        }
        Ok(Self {
            original,
            oracle,
            offsets,
        })
    }
    /// Original authored equality offsets, preserved in declared row order.
    pub fn offsets(&self) -> &[f64] {
        &self.offsets
    }
    /// Explicitly create a numerical zero-equality view with separately retained
    /// physical source/offset metadata and a distinct consumed numerical identity.
    pub fn zero_residuals(self) -> Result<ZeroResidualBridge, ProblemError> {
        let numerical = self.original.zero_residual_view()?;
        Ok(ZeroResidualBridge {
            physical: self,
            numerical,
        })
    }
    /// Return the actual root supplier and its explicit original offsets together.
    pub fn into_original(self) -> (Box<dyn NleOracle>, Vec<f64>) {
        (self.oracle, self.offsets)
    }
}
impl Oracle for RootBridge {
    type Error = ProblemError;
    fn contract(&self) -> &OriginalContract {
        &self.original
    }
    fn values(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let mut result = vec![0.0; self.offsets.len()];
        self.oracle.residual(x, &mut result)?;
        if out.len() != result.len() {
            return Err(ProblemError::Contract("authored root value extent".into()));
        }
        for (value, offset) in result.iter_mut().zip(&self.offsets) {
            *value += offset;
        }
        finite(&result)?;
        out.copy_from_slice(&result);
        Ok(())
    }
    fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.oracle.jacobian_product(x, direction, out)
    }
}
/// Zero residual mathematical view retaining all original authored source metadata.
#[derive(Debug)]
pub struct ZeroResidualBridge {
    physical: RootBridge,
    numerical: Arc<OriginalContract>,
}
impl ZeroResidualBridge {
    /// Numerical zero-equality inventory; distinct from the retained authored inventory.
    pub fn numerical_contract(&self) -> &Arc<OriginalContract> {
        &self.numerical
    }
    /// Full authored bounds, identities and obligations before residual subtraction.
    pub fn physical_original(&self) -> &OriginalContract {
        &self.physical.original
    }
    /// Explicit subtraction metadata used by this numerical view.
    pub fn offsets(&self) -> &[f64] {
        self.physical.offsets()
    }
    /// Restore the original representation and supplier for final assessment.
    pub fn into_original(self) -> RootBridge {
        self.physical
    }
}
impl Oracle for ZeroResidualBridge {
    type Error = ProblemError;
    fn contract(&self) -> &OriginalContract {
        &self.numerical
    }
    fn values(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.physical.oracle.residual(x, out)
    }
    fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.physical.oracle.jacobian_product(x, direction, out)
    }
}
/// Consumer-issued normalized allowances. Each actual point/action creates its own
/// fully scoped demand; no default tolerance or default certificate is supplied.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReconstructionAccuracy {
    /// Actual consumer allowance for normalized reconstructed point error.
    pub point: f64,
    /// Actual consumer allowance for each realized normalized action error.
    pub action: f64,
    /// Required evidence class, independently supplied by the consumer.
    pub class: AccuracyClass,
    /// Explicit bounded work for each consumed point/action refinement.
    pub refinement: math::RefinementLimits,
}
impl ReconstructionAccuracy {
    /// Validate finite consumer allowances and explicit bounded refinement before preparation.
    /// # Errors
    /// Invalid error budgets or absent refinement work.
    pub fn validate(self) -> Result<(), ProblemError> {
        if !self.point.is_finite()
            || self.point < 0.0
            || !self.action.is_finite()
            || self.action < 0.0
        {
            return Err(ProblemError::Contract(
                "reconstruction allowances must be finite and nonnegative".into(),
            ));
        }
        self.refinement.validate().map_err(ProblemError::Math)
    }
    fn demand(
        self,
        product: math::AccuracyProduct,
        action: bool,
    ) -> Result<AccuracyDemand, ProblemError> {
        let demand = AccuracyDemand {
            product: product.key()?,
            normalization: product.normalization,
            allowance: if action { self.action } else { self.point },
            class: self.class,
        };
        demand
            .validate()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        Ok(demand)
    }
}
/// Native row correspondence; coordinate bounds retain their original physical values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReducedRow {
    /// Remaining original authored equality or inequality, in original row order.
    Constraint(GlobalRow),
    /// Physical bound of an eliminated original coordinate, using its actual action.
    CoordinateBound(GlobalCol),
}
/// First-order reduced native NLP, including coupled objectives, original inequalities
/// and every finite eliminated-coordinate bound with its actual reconstruction action.
#[derive(Debug)]
pub struct ReducedOracle<O, R> {
    bound: BoundReduced<O, R>,
    contract: OracleContract,
    rows: Vec<ReducedRow>,
    bounds: Vec<(f64, f64)>,
    pattern: AssemblyMatrix,
    accuracy: ReconstructionAccuracy,
    last_point: Option<ContentHash>,
    last_action: Option<ContentHash>,
    applied_action: Option<(ContentHash, pse_model::strategy::AccuracyEvidence)>,
}
impl<O: ObjectiveOracle<Error = ProblemError>, R: ReconstructionOracle<Error = ProblemError>>
    ReducedOracle<O, R>
{
    /// Admit initial lineage before native layout is frozen. `smoothness` is the
    /// actual original source admission, not inferred from a structural graph.
    pub fn new(
        mut bound: BoundReduced<O, R>,
        anchor: &[f64],
        accuracy: ReconstructionAccuracy,
        smoothness: DerivativeOrder,
        native_index: usize,
    ) -> Result<Self, ProblemError> {
        accuracy.refinement.validate()?;
        bound.prepare_reconstruction(anchor)?;
        accuracy.demand(bound.point_product(anchor)?, false)?;
        accuracy.demand(
            bound.action_product(anchor, &vec![0.0; anchor.len()])?,
            true,
        )?;
        if !bound.family().support().jacobian_product || smoothness < DerivativeOrder::First {
            return Err(ProblemError::Unsupported(
                "reduced native First action/smoothness unavailable".into(),
            ));
        }
        let original = bound.family().original();
        let map = bound.reconstruction_contract();
        let mut rows = bound
            .family()
            .row_map()
            .ok_or_else(|| ProblemError::Contract("reduced family row map".into()))?
            .iter()
            .copied()
            .map(ReducedRow::Constraint)
            .collect::<Vec<_>>();
        let mut row_ids = rows
            .iter()
            .map(|r| match r {
                ReducedRow::Constraint(r) => original.constraints()[r.get()].id,
                ReducedRow::CoordinateBound(_) => SemanticId::NIL,
            })
            .collect::<Vec<_>>();
        let mut bounds = bound
            .constraints()
            .iter()
            .map(|r| (r.lower, r.upper))
            .collect::<Vec<_>>();
        let mut entries = bound.family().incidence().to_vec();
        for (i, c) in original.coordinates().iter().enumerate() {
            let column = GlobalCol::new(i);
            if map.retained().contains(&column) || !c.lower.is_finite() && !c.upper.is_finite() {
                continue;
            }
            let row = rows.len();
            rows.push(ReducedRow::CoordinateBound(column));
            row_ids.push(pse_ids::named_id(c.id, "reconstructed-original-bound"));
            bounds.push((c.lower, c.upper));
            entries.extend(
                map.incidence()
                    .iter()
                    .filter(|e| e.row == column)
                    .map(|e| Entry::new(GlobalRow::new(row), e.col)),
            );
        }
        let coordinates = bound.coordinates();
        let pattern = AssemblyMatrix::new(rows.len(), coordinates.len(), &entries, native_index)?;
        let contract = OracleContract {
            identity: bound.key(),
            variables: coordinates
                .into_iter()
                .map(|c| Variable {
                    id: c.id,
                    lower: c.lower,
                    upper: c.upper,
                })
                .collect(),
            rows: row_ids,
            derivatives: DerivativeOrder::First,
            smoothness: DerivativeOrder::First,
        };
        contract.validate(DerivativeOrder::First)?;
        Ok(Self {
            bound,
            contract,
            rows,
            bounds,
            pattern,
            accuracy,
            last_point: None,
            last_action: None,
            applied_action: None,
        })
    }
    /// Additional facade allocations; bound suppliers and their existing leases remain
    /// charged separately by preparation. No hidden retained factor is claimed.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.rows.capacity() * size_of::<ReducedRow>()
            + self.bounds.capacity() * size_of::<(f64, f64)>()
            + self.contract.variables.capacity() * size_of::<Variable>()
            + self.contract.rows.capacity() * size_of::<SemanticId>()
            + self.pattern.retained_bytes()
    }
    /// Whether this worker actually issued the point/action demand most recently consumed.
    pub fn accepts_refinement_product(&self, product: ContentHash) -> bool {
        self.last_point == Some(product) || self.last_action == Some(product)
    }
    /// Last actual successful reconstruction action at the full original point it
    /// consumed. The accuracy product additionally binds the exact reduced direction.
    pub fn applied_action(&self) -> Option<&(ContentHash, pse_model::strategy::AccuracyEvidence)> {
        self.applied_action.as_ref()
    }
    /// Native-to-original correspondence, including original reconstructed bounds.
    pub fn row_map(&self) -> &[ReducedRow] {
        &self.rows
    }
    /// Reconstruct a screened-start proposal with this actual native worker's retained
    /// selected sheet and the same consumed point accuracy used during iteration.
    pub fn original_proposal(
        &mut self,
        x: &[f64],
    ) -> Result<math::ReconstructionObservation, ProblemError> {
        let demand = self.point_demand(x)?;
        self.bound.reconstruct(x, &demand, self.accuracy.refinement)
    }
    /// Retained actual mathematical suppliers for final original correction/assessment.
    pub fn into_bound(self) -> BoundReduced<O, R> {
        self.bound
    }
    fn point_demand(&mut self, x: &[f64]) -> Result<AccuracyDemand, ProblemError> {
        self.bound.prepare_reconstruction(x)?;
        let demand = self.accuracy.demand(self.bound.point_product(x)?, false)?;
        self.last_point = Some(demand.product);
        self.last_action = None;
        Ok(demand)
    }
    fn product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        let point = self.point_demand(x)?;
        let product = self.bound.action_product(x, direction)?;
        let action = self.accuracy.demand(product, true)?;
        self.last_action = Some(action.product);
        let (rows, reconstruction, consumed) = self.bound.original_composition_product(
            x,
            direction,
            &point,
            &action,
            self.accuracy.refinement,
        )?;
        if out.len() != self.rows.len() {
            return Err(ProblemError::Contract(
                "reduced native row action extent".into(),
            ));
        }
        let result = self
            .rows
            .iter()
            .map(|r| match r {
                ReducedRow::Constraint(r) => rows[r.get()],
                ReducedRow::CoordinateBound(c) => reconstruction.values[c.get()],
            })
            .collect::<Vec<_>>();
        finite(&result)?;
        self.applied_action = Some((
            crate::square_response::point_key(&consumed.original_point),
            reconstruction.accuracy,
        ));
        out.copy_from_slice(&result);
        Ok(())
    }
}
impl<O: ObjectiveOracle<Error = ProblemError>, R: ReconstructionOracle<Error = ProblemError>>
    NlpOracle for ReducedOracle<O, R>
{
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.pattern.matrix().symbolic()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        None
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        let demand = self.point_demand(x)?;
        if self
            .bound
            .family()
            .original()
            .obligations()
            .objective
            .is_none()
        {
            return Ok(0.0);
        }
        self.bound
            .original_objective(x, &demand, self.accuracy.refinement)
            .map(|v| v.0)
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let demand = self.point_demand(x)?;
        let point = self
            .bound
            .reconstruct(x, &demand, self.accuracy.refinement)?;
        let mut original = vec![0.0; self.bound.family().original().constraints().len()];
        self.bound
            .original_values(x, &demand, self.accuracy.refinement, &mut original)?;
        if out.len() != self.rows.len() {
            return Err(ProblemError::Contract(
                "reduced native constraint extent".into(),
            ));
        }
        let result = self
            .rows
            .iter()
            .map(|r| match r {
                ReducedRow::Constraint(r) => original[r.get()],
                ReducedRow::CoordinateBound(c) => point.values[c.get()],
            })
            .collect::<Vec<_>>();
        finite(&result)?;
        out.copy_from_slice(&result);
        Ok(())
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let point = self.point_demand(x)?;
        let n = self.contract.variables.len();
        if out.len() != n {
            return Err(ProblemError::Contract(
                "reduced native gradient extent".into(),
            ));
        }
        let mut result = vec![0.0; n];
        let mut direction = vec![0.0; n];
        let mut applied_action = None;
        if self
            .bound
            .family()
            .original()
            .obligations()
            .objective
            .is_some()
        {
            for (column, value) in result.iter_mut().enumerate() {
                direction[column] = 1.0;
                let product = self.bound.action_product(x, &direction)?;
                let action = self.accuracy.demand(product, true)?;
                self.last_action = Some(action.product);
                let (actual, consumed) = self.bound.original_objective_product(
                    x,
                    &direction,
                    &point,
                    &action,
                    self.accuracy.refinement,
                )?;
                *value = actual;
                if let Some(evidence) = consumed.action {
                    applied_action = Some((
                        crate::square_response::point_key(&consumed.original_point),
                        evidence,
                    ));
                }
                direction[column] = 0.0;
            }
        }
        finite(&result)?;
        if applied_action.is_some() {
            self.applied_action = applied_action;
        }
        out.copy_from_slice(&result);
        Ok(())
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let pattern = self
            .pattern
            .matrix()
            .symbolic()
            .to_owned()
            .map_err(|e| ProblemError::Contract(format!("derived sparse ownership: {e:?}")))?;
        assemble(pattern.as_ref(), out, |direction, values| {
            self.product(x, direction, values)
        })
    }
    fn hessian(&mut self, _: &[f64], _: f64, _: &[f64], _: &mut [f64]) -> Result<(), ProblemError> {
        Err(ProblemError::Unsupported(
            "reduced exact Hessian is not prepared; select a First profile".into(),
        ))
    }
}
fn inventory(native: &OracleContract, original: &OriginalContract) -> Result<(), ProblemError> {
    native.validate(DerivativeOrder::Value)?;
    if native.variables.len() != original.coordinates().len()
        || native.rows.len() != original.constraints().len()
        || native
            .variables
            .iter()
            .zip(original.coordinates())
            .any(|(v, c)| {
                v.id != c.id
                    || v.lower.to_bits() != c.lower.to_bits()
                    || v.upper.to_bits() != c.upper.to_bits()
            })
        || native
            .rows
            .iter()
            .zip(original.constraints())
            .any(|(id, r)| *id != r.id)
    {
        return Err(ProblemError::Contract(
            "native/math original coordinate or row inventory mismatch".into(),
        ));
    }
    Ok(())
}
fn sparse_product(
    pattern: faer::sparse::SymbolicSparseColMatRef<'_, usize>,
    coefficients: &[f64],
    direction: &[f64],
    out: &mut [f64],
) -> Result<(), ProblemError> {
    if direction.len() != pattern.ncols()
        || out.len() != pattern.nrows()
        || coefficients.len() != pattern.compute_nnz()
    {
        return Err(ProblemError::Contract(
            "original sparse action extent".into(),
        ));
    }
    let mut result = vec![0.0; out.len()];
    let mut k = 0;
    for (column, &v) in direction.iter().enumerate() {
        for row in pattern.row_idx_of_col(column) {
            result[row] += coefficients[k] * v;
            k += 1;
        }
    }
    finite(&result)?;
    out.copy_from_slice(&result);
    Ok(())
}
fn assemble(
    pattern: faer::sparse::SymbolicSparseColMatRef<'_, usize>,
    out: &mut [f64],
    mut product: impl FnMut(&[f64], &mut [f64]) -> Result<(), ProblemError>,
) -> Result<(), ProblemError> {
    if out.len() != pattern.compute_nnz() {
        return Err(ProblemError::Contract(
            "derived native sparse Jacobian extent".into(),
        ));
    }
    let mut result = vec![0.0; out.len()];
    let mut direction = vec![0.0; pattern.ncols()];
    let mut values = vec![0.0; pattern.nrows()];
    let mut k = 0;
    for column in 0..pattern.ncols() {
        direction[column] = 1.0;
        values.fill(0.0);
        product(&direction, &mut values)?;
        direction[column] = 0.0;
        finite(&values)?;
        let mut rows = pattern.row_idx_of_col(column).peekable();
        for (row, &value) in values.iter().enumerate() {
            if rows.peek().copied() == Some(row) {
                let _ = rows.next();
                result[k] = value;
                k += 1;
            } else if value != 0.0 {
                return Err(ProblemError::Contract(
                    "derived action lies outside original structural incidence".into(),
                ));
            }
        }
    }
    finite(&result)?;
    out.copy_from_slice(&result);
    Ok(())
}
fn finite(values: &[f64]) -> Result<(), ProblemError> {
    if values.iter().any(|v| !v.is_finite()) {
        Err(ProblemError::Contract(
            "nonfinite derived native product".into(),
        ))
    } else {
        Ok(())
    }
}

/// Mathematical residual binding consumed by a native root view. The mathematical
/// family owns its transform; this facade performs canonical sparse assembly only.
pub trait ResidualBinding: std::fmt::Debug {
    /// Immutable enumerated family.
    fn family(&self) -> &Arc<math::DerivedFamily>;
    /// Actual anchor/step/parameter/provider/branch binding identity.
    fn binding_key(&self) -> ContentHash;
    /// Current solve coordinate intervals.
    fn coordinates(&self) -> Vec<math::Coordinate>;
    /// Current consumed original row intervals (must be zero equalities for this view).
    fn constraints(&self) -> Vec<math::Constraint>;
    /// Actual mathematical residual, including all source guards and sheets.
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError>;
    /// Actual admitted mathematical derivative action.
    fn action(&mut self, x: &[f64], direction: &[f64], out: &mut [f64])
    -> Result<(), ProblemError>;
}
macro_rules! full_residual_binding {
    ($binding:ident) => {
        impl<O: Oracle<Error = ProblemError>> ResidualBinding for $binding<O> {
            fn family(&self) -> &Arc<math::DerivedFamily> {
                self.family()
            }
            fn binding_key(&self) -> ContentHash {
                self.key()
            }
            fn coordinates(&self) -> Vec<math::Coordinate> {
                self.family().original().coordinates().to_vec()
            }
            fn constraints(&self) -> Vec<math::Constraint> {
                self.family().original().constraints().to_vec()
            }
            fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
                self.residual(x, out)
            }
            fn action(
                &mut self,
                x: &[f64],
                direction: &[f64],
                out: &mut [f64],
            ) -> Result<(), ProblemError> {
                self.jacobian_product(x, direction, out)
            }
        }
    };
}
full_residual_binding!(BoundHomotopy);
full_residual_binding!(BoundPseudoTime);
impl<O: Oracle<Error = ProblemError>> ResidualBinding for BoundProjection<O> {
    fn family(&self) -> &Arc<math::DerivedFamily> {
        self.family()
    }
    fn binding_key(&self) -> ContentHash {
        self.key()
    }
    fn coordinates(&self) -> Vec<math::Coordinate> {
        self.coordinates()
    }
    fn constraints(&self) -> Vec<math::Constraint> {
        self.constraints()
    }
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.values(x, out)
    }
    fn action(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.jacobian_product(x, direction, out)
    }
}
/// Actual square native view of a fixed homotopy/shifted/block/parameter binding.
/// Artificial success remains auxiliary; final assessment consumes the original owner.
#[derive(Debug)]
pub struct FamilyRoot<F> {
    bound: F,
    contract: OracleContract,
    pattern: AssemblyMatrix,
}
impl<F: ResidualBinding> FamilyRoot<F> {
    /// Freeze exact binding metadata. Continuation/retry owners consume `into_binding`
    /// and construct a new view after rebinding rather than changing a live layout.
    pub fn new(
        bound: F,
        smoothness: DerivativeOrder,
        native_index: usize,
    ) -> Result<Self, ProblemError> {
        let coordinates = bound.coordinates();
        let rows = bound.constraints();
        if coordinates.len() != rows.len()
            || rows.iter().any(|r| r.lower != 0.0 || r.upper != 0.0)
            || !bound.family().support().jacobian_product
            || smoothness < DerivativeOrder::First
        {
            return Err(ProblemError::Unsupported("derived root requires a smooth square zero-residual binding with actual action support".into()));
        }
        let entries = bound
            .family()
            .incidence()
            .iter()
            .copied()
            .filter(|e| e.col.get() < coordinates.len())
            .collect::<Vec<_>>();
        let pattern = AssemblyMatrix::new(rows.len(), coordinates.len(), &entries, native_index)?;
        let contract = OracleContract {
            identity: bound.binding_key(),
            variables: coordinates
                .into_iter()
                .map(|c| Variable {
                    id: c.id,
                    lower: c.lower,
                    upper: c.upper,
                })
                .collect(),
            rows: rows.into_iter().map(|r| r.id).collect(),
            derivatives: DerivativeOrder::First,
            smoothness: DerivativeOrder::First,
        };
        contract.square()?;
        Ok(Self {
            bound,
            contract,
            pattern,
        })
    }
    /// Current actual mathematical binding, including auxiliary correspondence.
    pub fn binding(&self) -> &F {
        &self.bound
    }
    /// Return the actual supplier for parameter/step rebinding or original assessment.
    pub fn into_binding(self) -> F {
        self.bound
    }
    /// Additional owned facade allocation, excluding the already admitted suppliers.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.pattern.retained_bytes()
            + self.contract.variables.capacity() * size_of::<Variable>()
            + self.contract.rows.capacity() * size_of::<SemanticId>()
    }
}
impl<F: ResidualBinding> NleOracle for FamilyRoot<F> {
    fn operations(&self) -> RootOperations {
        RootOperations {
            jacobian: true,
            jacobian_product: true,
        }
    }
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.bound.residual(x, out)
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.pattern.matrix().symbolic()
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let pattern = self
            .pattern
            .matrix()
            .symbolic()
            .to_owned()
            .map_err(|e| ProblemError::Contract(format!("derived sparse ownership: {e:?}")))?;
        assemble(pattern.as_ref(), out, |direction, values| {
            self.bound.action(x, direction, values)
        })
    }
    fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.bound.action(x, direction, out)
    }
}
/// Native zero-objective feasibility preserves every original authored row/bound.
#[derive(Debug)]
pub struct FeasibilityOracle<O> {
    bound: BoundFeasibility<O>,
    contract: OracleContract,
    bounds: Vec<(f64, f64)>,
    pattern: AssemblyMatrix,
}
impl<O: Oracle<Error = ProblemError>> FeasibilityOracle<O> {
    /// Actual smoothness is supplied by original preparation; no Hessian is fabricated.
    pub fn new(
        bound: BoundFeasibility<O>,
        smoothness: DerivativeOrder,
        native_index: usize,
    ) -> Result<Self, ProblemError> {
        let original = bound.family().original();
        let order = bound.family().support().order;
        let contract = OracleContract {
            identity: bound.family().key(),
            variables: original
                .coordinates()
                .iter()
                .map(|c| Variable {
                    id: c.id,
                    lower: c.lower,
                    upper: c.upper,
                })
                .collect(),
            rows: original.constraints().iter().map(|r| r.id).collect(),
            derivatives: order,
            smoothness: smoothness.min(DerivativeOrder::First),
        };
        let bounds = original
            .constraints()
            .iter()
            .map(|r| (r.lower, r.upper))
            .collect();
        let pattern = AssemblyMatrix::new(
            contract.rows.len(),
            contract.variables.len(),
            bound.family().incidence(),
            native_index,
        )?;
        contract.validate(order)?;
        Ok(Self {
            bound,
            contract,
            bounds,
            pattern,
        })
    }
    /// Return the actual original supplier after auxiliary feasibility work.
    pub fn into_bound(self) -> BoundFeasibility<O> {
        self.bound
    }
}
impl<O: Oracle<Error = ProblemError>> NlpOracle for FeasibilityOracle<O> {
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.pattern.matrix().symbolic()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        None
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        Ok(self.bound.objective(x)?)
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.bound.constraints(x, out)
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        Ok(self.bound.gradient(x, out)?)
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let pattern = self
            .pattern
            .matrix()
            .symbolic()
            .to_owned()
            .map_err(|e| ProblemError::Contract(format!("derived sparse ownership: {e:?}")))?;
        assemble(pattern.as_ref(), out, |direction, values| {
            self.bound.jacobian_product(x, direction, values)
        })
    }
    fn hessian(&mut self, _: &[f64], _: f64, _: &[f64], _: &mut [f64]) -> Result<(), ProblemError> {
        Err(ProblemError::Unsupported(
            "feasibility exact Hessian unavailable; select First profile".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_math::derived::{Constraint, Coordinate, OriginalObligations};
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    fn hash(n: u8) -> ContentHash {
        ContentHash::from_bytes([n; 32])
    }
    #[test]
    fn assembled_actual_action_refuses_undeclared_incidence_transactionally() {
        let pattern = AssemblyMatrix::new(
            2,
            1,
            &[Entry::new(GlobalRow::new(0), GlobalCol::new(0))],
            10,
        )
        .unwrap();
        let mut output = [19.0];
        assert!(matches!(
            assemble(pattern.matrix().symbolic(), &mut output, |_, values| {
                values.copy_from_slice(&[3.0, 2.0]);
                Ok(())
            }),
            Err(ProblemError::Contract(_))
        ));
        assert_eq!(output, [19.0]);
        assemble(pattern.matrix().symbolic(), &mut output, |_, values| {
            values.copy_from_slice(&[3.0, 0.0]);
            Ok(())
        })
        .unwrap();
        assert_eq!(output, [3.0]);
    }
    fn original(lower: f64, upper: f64) -> Arc<OriginalContract> {
        Arc::new(
            OriginalContract::new(
                hash(1),
                pse_math::normalization::Normalization::identity(1, 1).key(),
                vec![Coordinate {
                    id: id(1),
                    lower: -10.0,
                    upper: 10.0,
                }],
                vec![Constraint {
                    id: id(2),
                    lower,
                    upper,
                }],
                vec![Entry::new(GlobalRow::new(0), GlobalCol::new(0))],
                DerivativeSupport {
                    order: DerivativeOrder::First,
                    jacobian_product: true,
                    source: hash(2),
                },
                OriginalObligations {
                    guards: hash(3),
                    selection: hash(4),
                    objective: Some(hash(5)),
                },
            )
            .unwrap(),
        )
    }
    #[derive(Debug)]
    struct Source {
        contract: OracleContract,
        pattern: AssemblyMatrix,
        bounds: Vec<(f64, f64)>,
    }
    impl Source {
        fn new(lower: f64, upper: f64, value_only: bool) -> Self {
            Self {
                contract: OracleContract {
                    identity: hash(1),
                    variables: vec![Variable {
                        id: id(1),
                        lower: -10.0,
                        upper: 10.0,
                    }],
                    rows: vec![id(2)],
                    derivatives: if value_only {
                        DerivativeOrder::Value
                    } else {
                        DerivativeOrder::First
                    },
                    smoothness: DerivativeOrder::First,
                },
                pattern: AssemblyMatrix::new(
                    1,
                    1,
                    &[Entry::new(GlobalRow::new(0), GlobalCol::new(0))],
                    10,
                )
                .unwrap(),
                bounds: vec![(lower, upper)],
            }
        }
    }
    impl NleOracle for Source {
        fn operations(&self) -> RootOperations {
            RootOperations {
                jacobian: false,
                jacobian_product: true,
            }
        }
        fn contract(&self) -> &OracleContract {
            &self.contract
        }
        fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
            self.pattern.matrix().symbolic()
        }
        fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            out[0] = x[0] - 3.0;
            Ok(())
        }
        fn jacobian(&mut self, _: &[f64], _: &mut [f64]) -> Result<(), ProblemError> {
            Err(ProblemError::Unsupported(
                "full root Jacobian deliberately unavailable".into(),
            ))
        }
        fn jacobian_product(
            &mut self,
            _: &[f64],
            v: &[f64],
            out: &mut [f64],
        ) -> Result<(), ProblemError> {
            out[0] = v[0];
            Ok(())
        }
    }
    impl NlpOracle for Source {
        fn contract(&self) -> &OracleContract {
            &self.contract
        }
        fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
            self.pattern.matrix().symbolic()
        }
        fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
            None
        }
        fn constraint_bounds(&self) -> &[(f64, f64)] {
            &self.bounds
        }
        fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
            Ok(x[0] * x[0])
        }
        fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            out[0] = x[0] * x[0];
            Ok(())
        }
        fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            out[0] = 2.0 * x[0];
            Ok(())
        }
        fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            out[0] = 2.0 * x[0];
            Ok(())
        }
        fn hessian(
            &mut self,
            _: &[f64],
            _: f64,
            _: &[f64],
            _: &mut [f64],
        ) -> Result<(), ProblemError> {
            Err(ProblemError::Unsupported("test First source".into()))
        }
    }
    #[test]
    fn explicit_offset_root_bridge_preserves_authored_bounds_and_true_directional_source() {
        let original = original(2.0, 2.0);
        let mut bridge = RootBridge::new(
            Box::new(Source::new(2.0, 2.0, true)),
            original.clone(),
            vec![2.0],
        )
        .unwrap();
        let mut out = [0.0];
        bridge.values(&[3.0], &mut out).unwrap();
        assert_eq!(out, [2.0]);
        assert_eq!(bridge.offsets(), [2.0]);
        let zero = bridge.zero_residuals().unwrap();
        assert_ne!(zero.contract().identity(), original.identity());
        assert_eq!(zero.physical_original().constraints()[0].lower, 2.0);
        let family = Arc::new(
            math::DerivedFamily::anchored_homotopy(zero.numerical_contract().clone()).unwrap(),
        );
        let bound = family
            .bind_anchored_homotopy(zero, vec![0.0], 0.5, hash(7))
            .unwrap();
        let mut native = FamilyRoot::new(bound, DerivativeOrder::First, 10).unwrap();
        native.residual(&[1.0], &mut out).unwrap();
        assert_eq!(out, [-0.5]);
        native.jacobian_product(&[1.0], &[2.0], &mut out).unwrap();
        assert_eq!(out, [2.0]);
        native.jacobian(&[1.0], &mut out).unwrap();
        assert_eq!(out, [1.0]);
        let zero = native.into_binding().into_original();
        assert_eq!(zero.offsets(), [2.0]);
        assert_eq!(zero.physical_original().constraints()[0].upper, 2.0);
        assert!(
            RootBridge::new(Box::new(Source::new(2.0, 2.0, true)), original, vec![0.0]).is_err()
        );
    }
    #[test]
    fn native_feasibility_bridge_keeps_nonzero_equality_and_inequality_bounds() {
        for (lower, upper) in [(2.0, 2.0), (-1.0, 5.0)] {
            let original = original(lower, upper);
            let bridge =
                NlpBridge::new(Box::new(Source::new(lower, upper, false)), original.clone())
                    .unwrap();
            let family = Arc::new(math::DerivedFamily::bounded_feasibility(original));
            let bound = family.bind_feasibility(bridge).unwrap();
            let mut native = FeasibilityOracle::new(bound, DerivativeOrder::First, 10).unwrap();
            assert_eq!(native.constraint_bounds(), [(lower, upper)]);
            assert_eq!(native.objective(&[2.0]).unwrap(), 0.0);
            let mut out = [99.0];
            native.gradient(&[2.0], &mut out).unwrap();
            assert_eq!(out, [0.0]);
            native.constraints(&[2.0], &mut out).unwrap();
            assert_eq!(out, [4.0]);
            native.jacobian(&[2.0], &mut out).unwrap();
            assert_eq!(out, [4.0]);
            assert!(native.hessian(&[2.0], 1.0, &[1.0], &mut out).is_err());
        }
    }

    #[cfg(all(feature = "root-isolation", feature = "kinsol"))]
    #[test]
    fn concrete_selected_reconstruction_native_objective_inequality_and_original_bounds() {
        use pse_math::{
            implicit::reconstruction::SelectedImplicitReconstruction,
            implicit::{
                Configuration, Factory, Options, RegimeFactory, RegimeFactoryBranch, Selection,
                Unknown,
            },
            library::Optimization,
            normalization::Normalization,
            typed::{Binary, BodyBuilder, BodyLimits},
        };
        use std::{collections::BTreeMap, sync::atomic::AtomicBool, time::Duration};
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let q = registry.neutral_dimensionless().unwrap();
        let unit_id = registry.quantity_type(q).unwrap().canonical_unit;
        let unit = registry.unit(unit_id).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let build = |kind: u8| {
            let mut b = BodyBuilder::new(
                pse_math::initialize().unwrap(),
                &registry,
                &pse_quantity::standard::StandardInvariantChecker,
                2,
                BodyLimits::default(),
            )
            .unwrap();
            let y = b
                .input(0, q, pse_quantity::IndexSet::new(), id(12))
                .unwrap();
            let p = b
                .input(1, q, pse_quantity::IndexSet::new(), id(11))
                .unwrap();
            let literal = |b: &mut BodyBuilder<'_>, value| {
                b.literal(
                    value,
                    unit,
                    pse_quantity::literal::LiteralContext::Explicit { quantity_type: q },
                    id(20),
                )
                .unwrap()
            };
            let outputs = match kind {
                0 => {
                    let square = b.binary(Binary::Mul, y.clone(), y, None, id(20)).unwrap();
                    vec![b.binary(Binary::Sub, square, p, None, id(20)).unwrap()]
                }
                1 => vec![literal(&mut b, 1.0)],
                _ => vec![literal(&mut b, 0.0), literal(&mut b, 0.0)],
            };
            b.prepare(&outputs).unwrap()
        };
        let residual = build(0);
        let eligibility = build(1);
        let criterion = build(2);
        let projection = pse_math::factorable::root_isolation_program(
            id(20),
            &residual,
            &eligibility,
            &criterion,
            &cancel,
            10_000,
        )
        .unwrap()
        .unwrap();
        let body = Arc::new(
            residual
                .compile(
                    &[0],
                    &[0, 1],
                    DerivativeOrder::First,
                    Optimization::default(),
                    pse_math::jets::EvaluationLimits::default(),
                    &cancel,
                )
                .unwrap(),
        );
        let eligibility = Arc::new(
            eligibility
                .compile(
                    &[0],
                    &[0, 1],
                    DerivativeOrder::First,
                    Optimization::default(),
                    pse_math::jets::EvaluationLimits::default(),
                    &cancel,
                )
                .unwrap(),
        );
        let criterion = Arc::new(
            criterion
                .compile(
                    &[0, 1],
                    &[0, 1],
                    DerivativeOrder::First,
                    Optimization::default(),
                    pse_math::jets::EvaluationLimits::default(),
                    &cancel,
                )
                .unwrap(),
        );
        let port = |id| pse_kernels::Port {
            id,
            quantity: q,
            unit: unit_id,
        };
        let spec = pse_kernels::ProviderSpec {
            shapes: Default::default(),
            derivative_source: pse_kernels::DerivativeSource::Implicit,
            id: id(20),
            revision: hash(20),
            data: hash(21),
            inputs: vec![port(id(11))],
            outputs: vec![port(id(12))],
            derivatives: DerivativeOrder::First,
            smoothness: DerivativeOrder::First,
        };
        #[derive(Debug)]
        struct Counted(std::sync::atomic::AtomicUsize);
        impl pse_math::implicit::InnerSolver for Counted {
            fn minimum_order(&self) -> DerivativeOrder {
                DerivativeOrder::First
            }
            fn identity(&self) -> ContentHash {
                pse_math::implicit::solver_identity("test.native-kinsol-admitted-start.v1")
            }
            fn solve(
                &self,
                problem: Arc<pse_math::implicit::Problem>,
                parameters: &[f64],
                options: &Options,
                cancel: &Arc<AtomicBool>,
            ) -> Result<Vec<f64>, pse_math::MathError> {
                self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let point = pse_math::implicit::InnerSolver::solve(
                    &crate::implicit::Kinsol,
                    problem.clone(),
                    parameters,
                    options,
                    cancel,
                )?;
                // Exercise a supplier that retains an already admitted start at the
                // requested coarse precision. Every call still executes real KINSOL;
                // original residual validation decides whether the start is lawful.
                if problem
                    .verify(parameters, &options.start, options, cancel)
                    .is_ok()
                {
                    Ok(options.start.clone())
                } else {
                    Ok(point)
                }
            }
        }
        let counted = Arc::new(Counted(std::sync::atomic::AtomicUsize::new(0)));
        let unknowns = vec![Unknown {
            id: id(12),
            lower: 0.01,
            upper: 3.5,
        }];
        let residual = Factory {
            selection: Selection::default(),
            requirements: pse_kernels::DerivativeRequirements::new(
                DerivativeOrder::First,
                DerivativeOrder::First,
                DerivativeOrder::First,
                DerivativeOrder::First,
                DerivativeOrder::First,
            )
            .unwrap(),
            spec: spec.clone(),
            body,
            unknowns: unknowns.clone(),
            rows: vec![id(21)],
            configuration: Configuration::Fixed(
                unknowns,
                Options {
                    start: vec![2.000000001],
                    variable_nominals: vec![1.0],
                    variable_tolerance: vec![1e-8],
                    residual_tolerance: vec![1e-8],
                    iterations: 100,
                    time_limit: Duration::from_secs(5),
                    derivative_tolerance: 1e-10,
                },
            ),
            hints: None,
            terms: None,
            solver: counted.clone(),
            cancel: cancel.clone(),
            max_entries: 100,
            providers: BTreeMap::new(),
        };
        let factory = RegimeFactory {
            spec,
            alternatives: vec![RegimeFactoryBranch {
                residual,
                eligibility,
                criterion,
                isolation: Some(Arc::new(projection)),
            }],
            maximum_regimes: 1,
            time_limit: Duration::from_secs(5),
            cancel: cancel.clone(),
            verifier: Some(Arc::new(crate::root_isolation::Ibex)),
        };
        let normalization = Normalization::identity(2, 2);
        let original = Arc::new(
            OriginalContract::new(
                hash(23),
                normalization.key(),
                vec![
                    Coordinate {
                        id: id(11),
                        lower: 0.1,
                        upper: 9.0,
                    },
                    Coordinate {
                        id: id(12),
                        lower: 0.01,
                        upper: 3.5,
                    },
                ],
                vec![
                    Constraint {
                        id: id(21),
                        lower: 0.0,
                        upper: 0.0,
                    },
                    Constraint {
                        id: id(22),
                        lower: -1.0,
                        upper: 10.0,
                    },
                ],
                vec![
                    Entry::new(GlobalRow::new(0), GlobalCol::new(0)),
                    Entry::new(GlobalRow::new(0), GlobalCol::new(1)),
                    Entry::new(GlobalRow::new(1), GlobalCol::new(0)),
                    Entry::new(GlobalRow::new(1), GlobalCol::new(1)),
                ],
                DerivativeSupport {
                    order: DerivativeOrder::First,
                    jacobian_product: true,
                    source: hash(24),
                },
                OriginalObligations {
                    guards: hash(25),
                    selection: hash(26),
                    objective: Some(hash(27)),
                },
            )
            .unwrap(),
        );
        #[derive(Debug)]
        struct Coupled {
            contract: Arc<OriginalContract>,
        }
        impl Oracle for Coupled {
            type Error = ProblemError;
            fn contract(&self) -> &OriginalContract {
                &self.contract
            }
            fn values(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
                out[0] = x[1] * x[1] - x[0];
                out[1] = x[0] + x[1];
                Ok(())
            }
            fn jacobian_product(
                &mut self,
                x: &[f64],
                v: &[f64],
                out: &mut [f64],
            ) -> Result<(), ProblemError> {
                out[0] = 2.0 * x[1] * v[1] - v[0];
                out[1] = v[0] + v[1];
                Ok(())
            }
        }
        impl ObjectiveOracle for Coupled {
            fn objective_support(&self) -> DerivativeSupport {
                self.contract.support()
            }
            fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
                Ok(x[0] * x[1])
            }
            fn objective_product(&mut self, x: &[f64], v: &[f64]) -> Result<f64, ProblemError> {
                Ok(x[1] * v[0] + x[0] * v[1])
            }
        }
        let map = SelectedImplicitReconstruction::<ProblemError>::prepare_contract(
            &factory,
            original.clone(),
            vec![GlobalCol::new(0)],
            vec![GlobalRow::new(0)],
            hash(28),
            &cancel,
        )
        .unwrap();
        assert_eq!(
            map.incidence(),
            [
                Entry::new(GlobalCol::new(0), GlobalCol::new(0)),
                Entry::new(GlobalCol::new(1), GlobalCol::new(0))
            ]
        );
        let mut reconstruction = SelectedImplicitReconstruction::<ProblemError>::new(
            &factory,
            map.clone(),
            normalization.clone(),
            pse_kernels::ExecutionScope::new(cancel.clone(), None),
        )
        .unwrap();
        ReconstructionOracle::admit(&mut reconstruction, &[4.0]).unwrap();
        assert_eq!(counted.0.load(std::sync::atomic::Ordering::Relaxed), 1);
        let chart = reconstruction.selection().selected_chart().unwrap().clone();
        let demand = AccuracyDemand {
            product: hash(46),
            normalization: normalization.key(),
            allowance: 1e-12,
            class: AccuracyClass::Certified,
        };
        let observed = ReconstructionOracle::jacobian_product(
            &mut reconstruction,
            &[4.0],
            &[1.0],
            &demand,
            math::RefinementLimits {
                rounds: 8,
                proof_cells: 64,
            },
        )
        .unwrap();
        assert!(observed.accuracy.satisfies(&demand));
        assert!((observed.values[1] - 0.25).abs() < 1e-12);
        let retained_chart = reconstruction.selection().selected_chart().unwrap();
        assert_eq!(retained_chart.parameters, chart.parameters);
        assert_eq!(retained_chart.existence, chart.existence);
        assert_eq!(retained_chart.uniqueness, chart.uniqueness);
        assert_eq!(retained_chart.verifier_identity, chart.verifier_identity);
        assert_eq!(retained_chart.order, chart.order);
        assert!(reconstruction.selection().observed_point_cells() >= 4);
        assert!(reconstruction.selection().observed_action_cells() >= 4);
        let family = Arc::new(math::DerivedFamily::reduced_space(original.clone(), map).unwrap());
        let bound = family
            .bind_reduced(Coupled { contract: original }, reconstruction, hash(29))
            .unwrap();
        let mut native = ReducedOracle::new(
            bound,
            &[4.0],
            ReconstructionAccuracy {
                point: 1e-12,
                action: 1e-12,
                class: AccuracyClass::Certified,
                refinement: math::RefinementLimits {
                    rounds: 8,
                    proof_cells: 64,
                },
            },
            DerivativeOrder::First,
            100,
        )
        .unwrap();
        assert!(counted.0.load(std::sync::atomic::Ordering::Relaxed) > 1);
        assert_eq!(
            native.row_map(),
            [
                ReducedRow::Constraint(GlobalRow::new(1)),
                ReducedRow::CoordinateBound(GlobalCol::new(1))
            ]
        );
        assert_eq!(native.constraint_bounds(), [(-1.0, 10.0), (0.01, 3.5)]);
        assert!((native.objective(&[4.0]).unwrap() - 8.0).abs() < 1e-7);
        let mut gradient = [0.0];
        native.gradient(&[4.0], &mut gradient).unwrap();
        assert!((gradient[0] - 3.0).abs() < 1e-7);
        let mut values = [0.0; 2];
        native.constraints(&[4.0], &mut values).unwrap();
        assert!((values[0] - 6.0).abs() < 1e-7);
        assert!((values[1] - 2.0).abs() < 1e-7);
        let mut jacobian = [0.0; 2];
        native.jacobian(&[4.0], &mut jacobian).unwrap();
        assert!((jacobian[0] - 1.25).abs() < 1e-7);
        assert!((jacobian[1] - 0.25).abs() < 1e-7);
        assert!(native.hessian(&[4.0], 1.0, &[0.0; 2], &mut []).is_err());
        assert!(
            counted.0.load(std::sync::atomic::Ordering::Relaxed) > 1,
            "tight consumed accuracy must actually correct the loose numerical root"
        );
        let corrected_calls = counted.0.load(std::sync::atomic::Ordering::Relaxed);
        native.objective(&[4.0]).unwrap();
        assert_eq!(
            counted.0.load(std::sync::atomic::Ordering::Relaxed),
            corrected_calls
        );
        let mut bound = native.into_bound();
        let point = bound.point_product(&[4.0]).unwrap();
        let impossible = AccuracyDemand {
            product: point.key().unwrap(),
            normalization: point.normalization,
            allowance: f64::from_bits(1),
            class: AccuracyClass::Certified,
        };
        assert!(
            bound
                .reconstruct(
                    &[4.0],
                    &impossible,
                    math::RefinementLimits {
                        rounds: 8,
                        proof_cells: 64
                    }
                )
                .is_err()
        );
        let mut full = [0.0; 2];
        let demand = AccuracyDemand {
            allowance: 1e-6,
            ..impossible
        };
        bound
            .original_values(
                &[4.0],
                &demand,
                math::RefinementLimits {
                    rounds: 8,
                    proof_cells: 64,
                },
                &mut full,
            )
            .unwrap();
        assert!(full[0].abs() < 1e-8);
        assert!((full[1] - 6.0).abs() < 1e-7);
        let (_, reconstruction) = bound.into_suppliers();
        assert!(reconstruction.selection().observed_action_cells() > 0);
        assert!(reconstruction.selection().observed_point_cells() > 0);
    }
    #[cfg(all(feature = "root-isolation", feature = "kinsol"))]
    #[test]
    fn concrete_nonzero_selected_reconstruction_matches_authored_offset_and_verified_action() {
        use pse_math::{
            implicit::reconstruction::SelectedImplicitReconstruction,
            implicit::{
                Configuration, Factory, Options, RegimeFactory, RegimeFactoryBranch, Selection,
                Unknown,
            },
            library::Optimization,
            normalization::Normalization,
            typed::{Binary, BodyBuilder, BodyLimits},
        };
        use std::{collections::BTreeMap, sync::atomic::AtomicBool, time::Duration};
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let q = registry.neutral_dimensionless().unwrap();
        let unit_id = registry.quantity_type(q).unwrap().canonical_unit;
        let unit = registry.unit(unit_id).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let build = |kind: u8| {
            let mut b = BodyBuilder::new(
                pse_math::initialize().unwrap(),
                &registry,
                &pse_quantity::standard::StandardInvariantChecker,
                2,
                BodyLimits::default(),
            )
            .unwrap();
            let y = b
                .input(0, q, pse_quantity::IndexSet::new(), id(12))
                .unwrap();
            let p = b
                .input(1, q, pse_quantity::IndexSet::new(), id(11))
                .unwrap();
            let literal = |b: &mut BodyBuilder<'_>, value| {
                b.literal(
                    value,
                    unit,
                    pse_quantity::literal::LiteralContext::Explicit { quantity_type: q },
                    id(20),
                )
                .unwrap()
            };
            let outputs = match kind {
                0 => {
                    let square = b.binary(Binary::Mul, y.clone(), y, None, id(20)).unwrap();
                    vec![b.binary(Binary::Sub, square, p, None, id(20)).unwrap()]
                }
                1 => vec![literal(&mut b, 1.0)],
                _ => vec![literal(&mut b, 0.0), literal(&mut b, 0.0)],
            };
            b.prepare(&outputs).unwrap()
        };
        let residual = build(0);
        let eligibility = build(1);
        let criterion = build(2);
        let projection = pse_math::factorable::root_isolation_program(
            id(20),
            &residual,
            &eligibility,
            &criterion,
            &cancel,
            10_000,
        )
        .unwrap()
        .unwrap();
        let body = Arc::new(
            residual
                .compile(
                    &[0],
                    &[0, 1],
                    DerivativeOrder::First,
                    Optimization::default(),
                    pse_math::jets::EvaluationLimits::default(),
                    &cancel,
                )
                .unwrap(),
        );
        let eligibility = Arc::new(
            eligibility
                .compile(
                    &[0],
                    &[0, 1],
                    DerivativeOrder::First,
                    Optimization::default(),
                    pse_math::jets::EvaluationLimits::default(),
                    &cancel,
                )
                .unwrap(),
        );
        let criterion = Arc::new(
            criterion
                .compile(
                    &[0, 1],
                    &[0, 1],
                    DerivativeOrder::First,
                    Optimization::default(),
                    pse_math::jets::EvaluationLimits::default(),
                    &cancel,
                )
                .unwrap(),
        );
        let port = |id| pse_kernels::Port {
            id,
            quantity: q,
            unit: unit_id,
        };
        let spec = pse_kernels::ProviderSpec {
            shapes: Default::default(),
            derivative_source: pse_kernels::DerivativeSource::Implicit,
            id: id(20),
            revision: hash(20),
            data: hash(21),
            inputs: vec![port(id(11))],
            outputs: vec![port(id(12))],
            derivatives: DerivativeOrder::First,
            smoothness: DerivativeOrder::First,
        };
        let unknowns = vec![Unknown {
            id: id(12),
            lower: 0.01,
            upper: 3.5,
        }];
        let residual = Factory {
            selection: Selection::default(),
            requirements: pse_kernels::DerivativeRequirements::new(
                DerivativeOrder::First,
                DerivativeOrder::First,
                DerivativeOrder::First,
                DerivativeOrder::First,
                DerivativeOrder::First,
            )
            .unwrap(),
            spec: spec.clone(),
            body,
            unknowns: unknowns.clone(),
            rows: vec![id(21)],
            configuration: Configuration::Fixed(
                unknowns,
                Options {
                    start: vec![1.0],
                    variable_nominals: vec![1.0],
                    variable_tolerance: vec![1e-9],
                    residual_tolerance: vec![1e-9],
                    iterations: 100,
                    time_limit: Duration::from_secs(5),
                    derivative_tolerance: 1e-10,
                },
            ),
            hints: None,
            terms: None,
            solver: Arc::new(crate::implicit::Kinsol),
            cancel: cancel.clone(),
            max_entries: 100,
            providers: BTreeMap::new(),
        };
        let factory = RegimeFactory {
            spec,
            alternatives: vec![RegimeFactoryBranch {
                residual,
                eligibility,
                criterion,
                isolation: Some(Arc::new(projection)),
            }],
            maximum_regimes: 1,
            time_limit: Duration::from_secs(5),
            cancel: cancel.clone(),
            verifier: Some(Arc::new(crate::root_isolation::Ibex)),
        };
        let normalization = Normalization::identity(2, 2);
        let original = Arc::new(
            OriginalContract::new(
                hash(23),
                normalization.key(),
                vec![
                    Coordinate {
                        id: id(11),
                        lower: 0.1,
                        upper: 9.0,
                    },
                    Coordinate {
                        id: id(12),
                        lower: 0.01,
                        upper: 3.5,
                    },
                ],
                vec![
                    Constraint {
                        id: id(21),
                        lower: 1.0,
                        upper: 1.0,
                    },
                    Constraint {
                        id: id(22),
                        lower: -1.0,
                        upper: 10.0,
                    },
                ],
                vec![
                    Entry::new(GlobalRow::new(0), GlobalCol::new(0)),
                    Entry::new(GlobalRow::new(0), GlobalCol::new(1)),
                    Entry::new(GlobalRow::new(1), GlobalCol::new(0)),
                    Entry::new(GlobalRow::new(1), GlobalCol::new(1)),
                ],
                DerivativeSupport {
                    order: DerivativeOrder::First,
                    jacobian_product: true,
                    source: hash(24),
                },
                OriginalObligations {
                    guards: hash(25),
                    selection: hash(26),
                    objective: Some(hash(27)),
                },
            )
            .unwrap(),
        );
        #[derive(Debug)]
        struct Coupled {
            contract: Arc<OriginalContract>,
        }
        impl Oracle for Coupled {
            type Error = ProblemError;
            fn contract(&self) -> &OriginalContract {
                &self.contract
            }
            fn values(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
                out[0] = x[1] * x[1] - x[0];
                out[1] = x[0] + x[1];
                Ok(())
            }
            fn jacobian_product(
                &mut self,
                x: &[f64],
                v: &[f64],
                out: &mut [f64],
            ) -> Result<(), ProblemError> {
                out[0] = 2.0 * x[1] * v[1] - v[0];
                out[1] = v[0] + v[1];
                Ok(())
            }
        }
        impl ObjectiveOracle for Coupled {
            fn objective_support(&self) -> DerivativeSupport {
                self.contract.support()
            }
            fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
                Ok(x[0] * x[1])
            }
            fn objective_product(&mut self, x: &[f64], v: &[f64]) -> Result<f64, ProblemError> {
                Ok(x[1] * v[0] + x[0] * v[1])
            }
        }
        assert!(
            SelectedImplicitReconstruction::<ProblemError>::prepare_contract(
                &factory,
                original.clone(),
                vec![GlobalCol::new(0)],
                vec![GlobalRow::new(0)],
                hash(28),
                &cancel
            )
            .is_err()
        );
        let binding = pse_math::implicit::reconstruction::SelectedResidualBinding::for_original(
            &factory,
            &original,
            &[GlobalRow::new(0)],
        )
        .unwrap();
        assert_eq!(binding.rows(), [(GlobalRow::new(0), id(21), 1.0)]);
        assert!(
            pse_math::implicit::reconstruction::SelectedResidualBinding::for_original(
                &factory,
                &original,
                &[GlobalRow::new(1)]
            )
            .is_err()
        );
        let map = SelectedImplicitReconstruction::<ProblemError>::prepare_contract_with_binding(
            &factory,
            original.clone(),
            vec![GlobalCol::new(0)],
            vec![GlobalRow::new(0)],
            hash(28),
            &cancel,
            &binding,
        )
        .unwrap();
        assert_eq!(
            map.incidence(),
            [
                Entry::new(GlobalCol::new(0), GlobalCol::new(0)),
                Entry::new(GlobalCol::new(1), GlobalCol::new(0))
            ]
        );
        let reconstruction = SelectedImplicitReconstruction::<ProblemError>::new_with_binding(
            &factory,
            map.clone(),
            normalization,
            pse_kernels::ExecutionScope::new(cancel.clone(), None),
            &binding,
        )
        .unwrap();
        let family = Arc::new(math::DerivedFamily::reduced_space(original.clone(), map).unwrap());
        let bound = family
            .bind_reduced(Coupled { contract: original }, reconstruction, hash(29))
            .unwrap();
        let mut native = ReducedOracle::new(
            bound,
            &[4.0],
            ReconstructionAccuracy {
                point: 1e-6,
                action: 1e-6,
                class: AccuracyClass::Certified,
                refinement: math::RefinementLimits {
                    rounds: 8,
                    proof_cells: 64,
                },
            },
            DerivativeOrder::First,
            100,
        )
        .unwrap();
        assert_eq!(
            native.row_map(),
            [
                ReducedRow::Constraint(GlobalRow::new(1)),
                ReducedRow::CoordinateBound(GlobalCol::new(1))
            ]
        );
        assert_eq!(native.constraint_bounds(), [(-1.0, 10.0), (0.01, 3.5)]);
        let y = 5.0_f64.sqrt();
        let action = 0.5 / y;
        let proposal = native.original_proposal(&[4.0]).unwrap();
        assert!((proposal.values[1] - y).abs() < 1e-7);
        assert!((native.objective(&[4.0]).unwrap() - 4.0 * y).abs() < 1e-7);
        let mut gradient = [0.0];
        native.gradient(&[4.0], &mut gradient).unwrap();
        assert!((gradient[0] - (y + 4.0 * action)).abs() < 1e-7);
        let mut values = [0.0; 2];
        native.constraints(&[4.0], &mut values).unwrap();
        assert!((values[0] - (4.0 + y)).abs() < 1e-7);
        assert!((values[1] - y).abs() < 1e-7);
        let mut jacobian = [0.0; 2];
        native.jacobian(&[4.0], &mut jacobian).unwrap();
        assert!((jacobian[0] - (1.0 + action)).abs() < 1e-7);
        assert!((jacobian[1] - action).abs() < 1e-7);
        assert!(native.hessian(&[4.0], 1.0, &[0.0; 2], &mut []).is_err());
        let mut bound = native.into_bound();
        let point = bound.point_product(&[4.0]).unwrap();
        let impossible = AccuracyDemand {
            product: point.key().unwrap(),
            normalization: point.normalization,
            allowance: 1e-20,
            class: AccuracyClass::Certified,
        };
        assert!(
            bound
                .reconstruct(
                    &[4.0],
                    &impossible,
                    math::RefinementLimits {
                        rounds: 8,
                        proof_cells: 64
                    }
                )
                .is_err()
        );
        let mut full = [0.0; 2];
        let demand = AccuracyDemand {
            allowance: 1e-6,
            ..impossible
        };
        bound
            .original_values(
                &[4.0],
                &demand,
                math::RefinementLimits {
                    rounds: 8,
                    proof_cells: 64,
                },
                &mut full,
            )
            .unwrap();
        assert!((full[0] - 1.0).abs() < 1e-8);
        assert!((full[1] - (4.0 + y)).abs() < 1e-7);
        let (_, reconstruction) = bound.into_suppliers();
        assert!(reconstruction.selection().observed_action_cells() > 0);
    }
}
