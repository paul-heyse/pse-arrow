// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fixed derivative contribution maps; faer owns storage and Gram multiplication.
use super::*;
use faer::reborrow::{Reborrow, ReborrowMut};
use faer::{
    dyn_stack::{MemBuffer, MemStack},
    sparse::{SparseColMat, SymbolicSparseColMat, linalg::matmul},
};
use native::ProblemError;
use pse_math::sparse::AssemblyMatrix;

fn error(message: impl Into<String>) -> ProblemError {
    ProblemError::Contract(message.into())
}
#[derive(Clone, Debug)]
pub(super) struct ResponseTerm {
    pub observation: usize,
    pub local: usize,
    pub contribution: usize,
}
#[derive(Clone, Debug, Default)]
pub(super) struct Mapping {
    pub responses: Vec<ResponseTerm>,
    pub constraints: Vec<(usize, usize)>,
    pub hessian: Vec<(usize, usize)>,
}
#[derive(Clone, Debug)]
pub(super) struct Layout {
    pub responses: AssemblyMatrix,
    pub constraints: AssemblyMatrix,
    pub hessian: Option<AssemblyMatrix>,
    pub mappings: Vec<Mapping>,
    pub gram: Option<Arc<GramPlan>>,
    pub cells: usize,
}
pub(super) struct GramPlan {
    pattern: SymbolicSparseColMat<usize>,
    info: matmul::SparseMatMulInfo,
    /// Canonical Gram value index to original lower-Hessian contribution index.
    contributions: Vec<(usize, usize)>,
}
impl std::fmt::Debug for GramPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GramPlan")
            .field("nonzeros", &self.pattern.row_idx().len())
            .finish_non_exhaustive()
    }
}
fn push(
    pairs: &mut Vec<(usize, usize)>,
    pair: (usize, usize),
    limit: usize,
) -> Result<usize, ProblemError> {
    if pairs.len() >= limit {
        return Err(error("fit sparse contribution allowance"));
    }
    let index = pairs.len();
    pairs.push(pair);
    Ok(index)
}
impl Layout {
    pub(super) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.responses.retained_bytes()
            + self.constraints.retained_bytes()
            + self
                .hessian
                .as_ref()
                .map_or(0, AssemblyMatrix::retained_bytes)
            + self.mappings.capacity() * size_of::<Mapping>()
            + self
                .mappings
                .iter()
                .map(|m| {
                    m.responses.capacity() * size_of::<ResponseTerm>()
                        + (m.constraints.capacity() + m.hessian.capacity())
                            * size_of::<(usize, usize)>()
                })
                .sum::<usize>()
            + self.gram.as_ref().map_or(0, |g| {
                size_of::<GramPlan>()
                    + size_of_val(g.pattern.col_ptr())
                    + size_of_val(g.pattern.row_idx())
                    + g.contributions.capacity() * size_of::<(usize, usize)>()
            })
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "Fixed experiment, observation and global coordinate contracts"
    )]
    pub(super) fn new(
        experiments: &[Experiment],
        measurements: &[Measurement],
        parameters: &[SemanticId],
        parameter_columns: &[Option<usize>],
        rows: usize,
        columns: usize,
        order: DerivativeOrder,
        limit: usize,
    ) -> Result<Self, ProblemError> {
        let mut response_pairs = Vec::new();
        let mut constraint_pairs = Vec::new();
        let mut hessian_pairs = Vec::new();
        let mut mappings = Vec::new();
        for (ei, experiment) in experiments.iter().enumerate() {
            let mut mapping = Mapping::default();
            match experiment {
                Experiment::Steady(s) => {
                    let j = s.case.assembly.jacobian_pattern();
                    let constraints = s.constraints.iter().copied().collect::<BTreeMap<_, _>>();
                    let mut observations = BTreeMap::<usize, Vec<usize>>::new();
                    for (oi, obs) in measurements
                        .iter()
                        .enumerate()
                        .filter(|(_, o)| o.experiment == ei && o.included)
                    {
                        observations.entry(obs.row).or_default().push(oi);
                    }
                    for (local_col, (_, global_col)) in s.coordinates.iter().enumerate() {
                        for k in j.col_range(local_col) {
                            let row = j.row_idx()[k];
                            if let Some(global_row) = constraints.get(&row) {
                                let c =
                                    push(&mut constraint_pairs, (*global_row, *global_col), limit)?;
                                mapping.constraints.push((k, c));
                            }
                            for &observation in observations.get(&row).into_iter().flatten() {
                                let c =
                                    push(&mut response_pairs, (observation, *global_col), limit)?;
                                mapping.responses.push(ResponseTerm {
                                    observation,
                                    local: k,
                                    contribution: c,
                                });
                            }
                        }
                    }
                    if order >= DerivativeOrder::Second {
                        let h = s.case.assembly.hessian_pattern();
                        for (local_col, (_, gc)) in s.coordinates.iter().enumerate() {
                            for k in h.col_range(local_col) {
                                let gr = s.coordinates[h.row_idx()[k]].1;
                                let c =
                                    push(&mut hessian_pairs, (gr.max(*gc), gr.min(*gc)), limit)?;
                                mapping.hessian.push((k, c));
                            }
                        }
                    }
                }
                Experiment::Transient(s) => {
                    for (j, id) in s.declaration.parameters.iter().enumerate() {
                        if let Some(k) = parameters.iter().position(|p| *p == *id)
                            && let Some(column) = parameter_columns[k]
                        {
                            for (observation, _) in measurements
                                .iter()
                                .enumerate()
                                .filter(|(_, o)| o.experiment == ei && o.included)
                            {
                                let c = push(&mut response_pairs, (observation, column), limit)?;
                                mapping.responses.push(ResponseTerm {
                                    observation,
                                    local: j,
                                    contribution: c,
                                });
                            }
                        }
                    }
                }
            }
            mappings.push(mapping);
        }
        let responses = AssemblyMatrix::new(measurements.len(), columns, &response_pairs, limit)?;
        let constraints = AssemblyMatrix::new(rows, columns, &constraint_pairs, limit)?;
        let gram = if order >= DerivativeOrder::Second {
            let transposed = responses
                .matrix()
                .symbolic()
                .transpose()
                .to_col_major()
                .map_err(|e| error(e.to_string()))?;
            // Bound product support before asking faer to allocate it. This is a
            // count of declared row-support pairs, never a dense n-by-n estimate.
            let products = (0..transposed.ncols())
                .try_fold(0usize, |n, c| {
                    let width = transposed.col_range(c).len();
                    n.checked_add(width.checked_mul(width)?)
                })
                .ok_or_else(|| error("fit Gram support overflow"))?;
            if products > limit {
                return Err(error("fit Gram contribution allowance"));
            }
            let (pattern, info) = matmul::sparse_sparse_matmul_symbolic(
                transposed.as_ref(),
                responses.matrix().symbolic(),
            )
            .map_err(|e| error(e.to_string()))?;
            let mut contributions = Vec::new();
            for col in 0..columns {
                for k in pattern.col_range(col) {
                    let row = pattern.row_idx()[k];
                    if row >= col {
                        let c = push(&mut hessian_pairs, (row, col), limit)?;
                        contributions.push((k, c));
                    }
                }
            }
            Some(Arc::new(GramPlan {
                pattern,
                info,
                contributions,
            }))
        } else {
            None
        };
        let hessian = if order >= DerivativeOrder::Second {
            Some(AssemblyMatrix::new(
                columns,
                columns,
                &hessian_pairs,
                limit,
            )?)
        } else {
            None
        };
        let cells = response_pairs
            .len()
            .checked_add(constraint_pairs.len())
            .and_then(|v| v.checked_add(hessian_pairs.len()))
            .and_then(|v| v.checked_add(gram.as_ref().map_or(0, |g| g.pattern.row_idx().len())))
            .ok_or_else(|| error("fit derivative extent"))?;
        if cells > limit {
            return Err(error("fit derivative cell allowance"));
        }
        Ok(Self {
            responses,
            constraints,
            hessian,
            mappings,
            gram,
            cells,
        })
    }
}
/// Attempt-owned fixed numeric buffers and the library's reusable work memory.
pub(super) struct GramWorker {
    plan: Arc<GramPlan>,
    weighted: SparseColMat<usize, f64>,
    result: SparseColMat<usize, f64>,
    transpose_values: Vec<f64>,
    transpose_columns: Vec<usize>,
    transpose_rows: Vec<usize>,
    memory: MemBuffer,
}
impl GramWorker {
    pub(super) fn new(
        plan: Arc<GramPlan>,
        response: &AssemblyMatrix,
        bytes: usize,
    ) -> Result<Self, ProblemError> {
        let weighted = response.matrix().clone();
        let result = SparseColMat::new(
            plan.pattern.clone(),
            vec![0.0; plan.pattern.row_idx().len()],
        );
        let request = matmul::sparse_sparse_matmul_numeric_scratch::<usize, f64>(
            plan.pattern.as_ref(),
            faer::Par::Seq,
        )
        .or(faer::sparse::utils::transpose_scratch::<usize>(
            weighted.nrows(),
            weighted.ncols(),
        ));
        if request.size_bytes() > bytes {
            return Err(error("fit Gram scratch allowance"));
        }
        let memory = MemBuffer::try_new(request).map_err(|e| error(e.to_string()))?;
        Ok(Self {
            transpose_values: vec![0.0; weighted.val().len()],
            transpose_columns: vec![0; weighted.nrows() + 1],
            transpose_rows: vec![0; weighted.val().len()],
            plan,
            weighted,
            result,
            memory,
        })
    }
    pub(super) fn refill(
        &mut self,
        response: &AssemblyMatrix,
        weights: &[f64],
        scale: f64,
        hessian: &mut AssemblyMatrix,
    ) -> Result<(), ProblemError> {
        for col in 0..self.weighted.ncols() {
            for k in self.weighted.symbolic().col_range(col) {
                let row = self.weighted.row_idx()[k];
                let v = response.matrix().val()[k] * weights[row];
                if !v.is_finite() {
                    return Err(error("nonfinite weighted response"));
                }
                self.weighted.val_mut()[k] = v;
            }
        }
        let transposed = faer::sparse::utils::transpose(
            &mut self.transpose_values,
            &mut self.transpose_columns,
            &mut self.transpose_rows,
            self.weighted.as_ref(),
            MemStack::new(&mut self.memory),
        );
        matmul::sparse_sparse_matmul_numeric(
            self.result.rb_mut(),
            faer::Accum::Replace,
            transposed.rb(),
            self.weighted.as_ref(),
            scale,
            &self.plan.info,
            faer::Par::Seq,
            MemStack::new(&mut self.memory),
        );
        // The full product is required by faer's scratch-clearing contract. Project
        // its lower triangle only after multiplication, then add local Hessians.
        for &(source, target) in &self.plan.contributions {
            hessian.add(target, self.result.val()[source])?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for GramWorker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GramWorker")
            .field("plan", &self.plan)
            .finish_non_exhaustive()
    }
}
