// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded LP/MILP analyses of a supplied Jacobian. These say nothing about nonlinear feasibility.
use crate::{CoefficientProblem,OracleContract,ProblemError,Variable,highs,quality::Tolerances,solve::*};
use faer::sparse::{SparseColMat,SparseColMatRef,Triplet};
use pse_ids::{FramedHasher,SemanticId};
use pse_kernels::DerivativeOrder;
use pse_math::binding::{ObjectiveSense,VariableDomain};
use std::collections::BTreeSet;

/// Finite search over anchored left-null vectors.
#[derive(Clone, Copy, Debug)]
pub struct Policy {
    pub maximum_rows: usize,
    pub maximum_entries: usize,
    pub maximum_attempts: usize,
    pub multiplier_bound: f64,
    pub tolerance: f64,
    pub rank_relative: f64,
}
/// A verified approximate left-null vector; residuals use the supplied matrix coordinates.
#[derive(Clone, Debug)]
pub struct Certificate {
    pub weights: Vec<(SemanticId, f64)>,
    pub residual_maximum: f64,
    pub pivot: SemanticId,
}
/// A minimum-support candidate qualified by removing each member and checking numerical rank.
#[derive(Clone, Debug)]
pub struct DegenerateSet {
    pub rows: Vec<SemanticId>,
    pub certificate: Certificate,
    pub irreducible_at_tolerance: bool,
}
#[derive(Debug)]
pub struct Report {
    pub conditioning: Vec<Certificate>,
    pub degenerate: Vec<DegenerateSet>,
    pub attempts: Vec<SolveReport>,
    pub complete: bool,
    pub unavailable: Vec<String>,
}
struct Builder {
    id: SemanticId,
    variables: Vec<Variable>,
    domains: Vec<VariableDomain>,
    objective: Vec<f64>,
    rows: Vec<SemanticId>,
    bounds: Vec<(f64, f64)>,
    entries: Vec<Triplet<usize, usize, f64>>,
}
impl Builder {
    fn variable(
        &mut self,
        name: &str,
        lower: f64,
        upper: f64,
        domain: VariableDomain,
        cost: f64,
    ) -> usize {
        let i = self.variables.len();
        self.variables.push(Variable {
            id: pse_ids::named_id(self.id, name),
            lower,
            upper,
        });
        self.domains.push(domain);
        self.objective.push(cost);
        i
    }
    fn row(&mut self, terms: impl IntoIterator<Item = (usize, f64)>, lower: f64, upper: f64) {
        let row = self.rows.len();
        self.rows
            .push(pse_ids::named_id(self.id, &format!("row-{row}")));
        self.bounds.push((lower, upper));
        self.entries.extend(
            terms
                .into_iter()
                .filter(|(_, v)| *v != 0.)
                .map(|(col, value)| Triplet::new(row, col, value)),
        );
    }
    fn finish(self) -> Result<CoefficientProblem, ProblemError> {
        let constraints = SparseColMat::try_new_from_triplets(
            self.rows.len(),
            self.variables.len(),
            &self.entries,
        )
        .map_err(|e| ProblemError::Internal(format!("diagnostic matrix: {e}")))?;
        let mut h = FramedHasher::new("pse.jacobian-diagnostic.problem.v1");
        h.id(&self.id);
        h.u64(self.variables.len() as u64)
            .u64(self.rows.len() as u64);
        for (v, domain) in self.variables.iter().zip(&self.domains) {
            h.id(&v.id)
                .u64(v.lower.to_bits())
                .u64(v.upper.to_bits())
                .u64(*domain as u64);
        }
        for v in &self.objective {
            h.u64(v.to_bits());
        }
        for (lo, hi) in &self.bounds {
            h.u64(lo.to_bits()).u64(hi.to_bits());
        }
        for v in constraints.col_ptr() {
            h.u64(*v as u64);
        }
        for v in constraints.row_idx() {
            h.u64(*v as u64);
        }
        for v in constraints.val() {
            h.u64(v.to_bits());
        }
        let identity = h.finish_hash();
        let problem = CoefficientProblem {
            contract: OracleContract {
                identity,
                variables: self.variables,
                rows: self.rows,
                derivatives: DerivativeOrder::Value,
                smoothness: DerivativeOrder::Second,
            },
            objective: self.objective,
            objective_constant: 0.,
            sense: ObjectiveSense::Minimize,
            domains: self.domains,
            assumptions: identity,
            constraints,
            hessian: None,
            bounds: self.bounds,
        };
        problem.validate()?;
        Ok(problem)
    }
}
fn problem(
    matrix: SparseColMatRef<'_, usize, f64>,
    rows: &[SemanticId],
    pivot: usize,
    milp: bool,
    policy: Policy,
) -> Result<CoefficientProblem, ProblemError> {
    let id = pse_ids::named_id(
        rows[pivot],
        if milp { "degeneracy" } else { "conditioning" },
    );
    let mut b = Builder {
        id,
        variables: vec![],
        domains: vec![],
        objective: vec![],
        rows: vec![],
        bounds: vec![],
        entries: vec![],
    };
    let bound = if milp { policy.multiplier_bound } else { 1. };
    for (i, id) in rows.iter().enumerate() {
        b.variable(
            &format!("multiplier-{id}"),
            if i == pivot { 1. } else { -bound },
            if i == pivot { 1. } else { bound },
            VariableDomain::Continuous,
            0.,
        );
    }
    if milp {
        for id in rows {
            b.variable(
                &format!("selected-{id}"),
                0.,
                1.,
                VariableDomain::Binary,
                1.,
            );
        }
        for i in 0..rows.len() {
            b.row([(i, 1.), (rows.len() + i, -bound)], f64::NEG_INFINITY, 0.);
            b.row([(i, -1.), (rows.len() + i, -bound)], f64::NEG_INFINITY, 0.);
        }
        for col in 0..matrix.ncols() {
            b.row(
                matrix
                    .row_idx_of_col(col)
                    .zip(matrix.val_of_col(col))
                    .map(|(r, v)| (r, *v)),
                0.,
                0.,
            );
        }
    } else {
        let residual = b.variable(
            "residual-infinity-norm",
            0.,
            f64::INFINITY,
            VariableDomain::Continuous,
            1.,
        );
        for col in 0..matrix.ncols() {
            for sign in [-1., 1.] {
                b.row(
                    matrix
                        .row_idx_of_col(col)
                        .zip(matrix.val_of_col(col))
                        .map(|(r, v)| (r, sign * v))
                        .chain([(residual, -1.)]),
                    f64::NEG_INFINITY,
                    0.,
                );
            }
        }
    }
    b.finish()
}
fn verify(
    matrix: SparseColMatRef<'_, usize, f64>,
    rows: &[SemanticId],
    pivot: usize,
    point: &[f64],
    policy: Policy,
) -> Option<Certificate> {
    let weights = point.get(..rows.len())?;
    if weights.iter().any(|v| !v.is_finite()) || (weights[pivot] - 1.).abs() > policy.tolerance {
        return None;
    }
    let residual = (0..matrix.ncols())
        .map(|j| {
            matrix
                .row_idx_of_col(j)
                .zip(matrix.val_of_col(j))
                .map(|(i, v)| v * weights[i])
                .sum::<f64>()
                .abs()
        })
        .fold(0., f64::max);
    if !residual.is_finite() {
        return None;
    }
    Some(Certificate {
        weights: rows.iter().copied().zip(weights.iter().copied()).collect(),
        residual_maximum: residual,
        pivot: rows[pivot],
    })
}
fn rank(
    matrix: SparseColMatRef<'_, usize, f64>,
    selected: &[usize],
    policy: Policy,
    execution: &Execution,
) -> Result<usize, ProblemError> {
    if selected.is_empty() {
        return Ok(0);
    }
    let mut entries = vec![];
    for j in 0..matrix.ncols() {
        for (i, v) in matrix.row_idx_of_col(j).zip(matrix.val_of_col(j)) {
            if let Some(k) = selected.iter().position(|s| *s == i) {
                entries.push(Triplet::new(k, j, *v));
            }
        }
    }
    let sub = SparseColMat::try_new_from_triplets(selected.len(), matrix.ncols(), &entries)
        .map_err(|e| ProblemError::Internal(e.to_string()))?;
    Ok(pse_math::diagnostics::analyze_matrix(
        sub.as_ref(),
        &vec![1.; selected.len()],
        &vec![1.; matrix.ncols()],
        pse_math::diagnostics::MatrixPolicy {
            dense_entries: policy.maximum_entries,
            findings: policy.maximum_entries,
            parallel_tolerance: 0.,
            rank_absolute: policy.tolerance,
            rank_relative: policy.rank_relative,
        },
        &execution.cancel,
    )?
    .rank)
}
/// Run native LP conditioning certificates and anchored minimum-support MILPs under
/// one outer execution deadline. Native limits and rank-budget refusals remain inconclusive.
pub fn analyze(matrix:SparseColMatRef<'_,usize,f64>,rows:&[SemanticId],policy:Policy,controls:&Controls,execution:Execution)->Result<Report,ProblemError>{
    if rows.len()!=matrix.nrows()||rows.is_empty()||rows.iter().collect::<BTreeSet<_>>().len()!=rows.len()
        ||rows.len()>policy.maximum_rows||matrix.val().len().checked_mul(2).and_then(|n|rows.len().checked_mul(4).and_then(|m|n.checked_add(m))).is_none_or(|n|n>policy.maximum_entries)||policy.maximum_attempts==0
        ||!policy.multiplier_bound.is_finite()||policy.multiplier_bound<1.||!policy.tolerance.is_finite()||policy.tolerance<=0.
        ||!policy.rank_relative.is_finite()||policy.rank_relative<0.||policy.rank_relative>=1.||matrix.val().iter().any(|v|!v.is_finite()){
        return Err(ProblemError::Contract("invalid bounded Jacobian diagnostic request".into()));
    }
    let mut report=Report{conditioning:vec![],degenerate:vec![],attempts:vec![],complete:true,unavailable:vec![]};
    // The diagnostic LP/MILPs are this analysis's own problems: their budgets derive from
    // its tolerance, not from any model's numerical policy.
    let accuracy=ResolvedAccuracy::from_policy(&Default::default(),policy.tolerance)?;
    for pivot in 0..rows.len(){
        for milp in [false,true]{
            if report.attempts.len()>=policy.maximum_attempts||execution.stopped().is_some(){report.complete=false;report.unavailable.push("diagnostic attempt budget or deadline exhausted".into());return Ok(report);}
            let p=problem(matrix,rows,pivot,milp,policy)?;
            let stamp=Compatibility{layout:p.contract.identity,profile:p.contract.identity,data:p.assumptions,backend:Backend::Highs};
            let mut session=highs::Session::new(&p,None,stamp)?;
            let t=Tolerances{variables:vec![policy.tolerance;p.contract.variables.len()],rows:vec![policy.tolerance;p.contract.rows.len()],integrality:policy.tolerance};
            let outcome=session.solve(&p,controls,&accuracy,highs::Method::Choose,execution.clone(),&t,None)?;
            drop(session);
            let optimal=outcome.termination.category==Termination::Success && outcome.quality.as_ref().is_some_and(crate::quality::Quality::feasible);
            if optimal && let Some(point)=outcome.candidate.as_ref().map(|c|&c.primal) {
                if let Some(certificate)=verify(matrix,rows,pivot,point,policy){
                    if !milp{report.conditioning.push(certificate);}
                    else if certificate.residual_maximum<=policy.tolerance {
                        let selected=point[..rows.len()].iter().enumerate().filter_map(|(i,v)|(v.abs()>policy.tolerance).then_some(i)).collect::<Vec<_>>();
                        let sources=selected.iter().map(|i|rows[*i]).collect::<Vec<_>>();
                        if !report.degenerate.iter().any(|s|s.rows==sources){
                            let irreducible=(||->Result<bool,ProblemError>{
                                if rank(matrix,&selected,policy,&execution)?+1!=selected.len(){return Ok(false);}
                                for omit in &selected{
                                    let subset=selected.iter().copied().filter(|i|i!=omit).collect::<Vec<_>>();
                                    if rank(matrix,&subset,policy,&execution)?!=subset.len(){return Ok(false);}
                                }
                                Ok(true)
                            })();
                            let irreducible_at_tolerance=match irreducible{Ok(v)=>v,Err(error)=>{report.complete=false;report.unavailable.push(error.to_string());false}};
                            report.degenerate.push(DegenerateSet{rows:sources,certificate,irreducible_at_tolerance});
                        }
                    }
                }else{report.complete=false;}
            }else if outcome.termination.category!=Termination::Infeasible {report.complete=false;}
            report.attempts.push(outcome);
        }
    }
    Ok(report)
}
#[cfg(test)]
mod tests{
    use super::*;
    use std::sync::{Arc,atomic::AtomicBool};
    #[test]
    fn diagnostic_highs_lp_milp_find_and_verify_a_small_degenerate_set(){
        let matrix=SparseColMat::try_new_from_triplets(2,2,&[Triplet::new(0,0,1.),Triplet::new(0,1,1.),Triplet::new(1,0,2.),Triplet::new(1,1,2.)]).unwrap();
        let rows=[SemanticId::from_bytes([1;16]),SemanticId::from_bytes([2;16])];
        let controls=Controls::default();
        let policy=Policy{maximum_rows:10,maximum_entries:1000,maximum_attempts:4,multiplier_bound:10.,tolerance:1e-7,rank_relative:1e-8};
        let r=analyze(matrix.as_ref(),&rows,policy,&controls,Execution::new(Arc::new(AtomicBool::new(false)),&controls)).unwrap();
        assert!(r.complete);
        assert_eq!(r.attempts.len(),4);
        assert_eq!(r.degenerate.len(),1);
        assert_eq!(r.degenerate[0].rows,rows);
        assert!(r.degenerate[0].irreducible_at_tolerance);
        assert_eq!(r.conditioning.len(),2);
        assert!(r.conditioning.iter().any(|c|c.residual_maximum<=policy.tolerance));
        assert!(r.conditioning.iter().any(|c|c.residual_maximum>policy.tolerance));
    }
}
