// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Primal-only library affine reduction. Optimization dual recovery stays separate.
use super::{Pass, Policy};
use crate::{NleOracle, OracleContract, ProblemError, execution::{self, Retained, Roots, Step}, quality, solve::{Metric, WarmPayload}};
use pounce_presolve::linear_eq_plan::{EliminationPlan, PlanConfig, PlanInput, VarRecovery, build_plan};
use pse_math::presolve::Facts;
use std::{cell::RefCell, rc::Rc};

/// Exact current source facts and original authored row bounds for root reduction.
#[derive(Debug)]
pub struct Primal<'a> {
    /// Source/provider-qualified proof projection, not a warm fingerprint.
    pub facts: &'a Facts,
    /// Original finite equality bounds, before residual subtraction.
    pub bounds: &'a [(f64, f64)],
    /// Only affine elimination is consumed; no NLP pass is implicitly enabled.
    pub policy: &'a Policy,
}
pub(super) fn plan(contract: &OracleContract, input: &Primal<'_>) -> Result<Option<EliminationPlan>, ProblemError> {
    let requested = match input.policy {
        Policy::Off => false,
        Policy::Auto => true,
        Policy::Explicit { options, required } => {
            if required.iter().any(|pass| *pass != Pass::AffineElimination) {
                return Err(ProblemError::unsupported("root primal reduction supports only affine elimination; NLP passes retain separate admission"));
            }
            options.enabled && options.linear_eq_reduction
        }
    };
    if !requested { return Ok(None); }
    let n = contract.variables.len();
    let m = contract.rows.len();
    if input.facts.structure != contract.identity || input.bounds.len() != m || input.facts.affine.len() != m || input.bounds.iter().any(|(l,u)| !l.is_finite() || l != u) {
        return Err(ProblemError::Contract("root affine original row inventory".into()));
    }
    if input.facts.affine.iter().flatten().any(|row| !row.constant.is_finite() || row.entries.iter().any(|(j,value)| *j>=n || !value.is_finite())) {
        return Err(ProblemError::Contract("root affine coefficient inventory or values".into()));
    }
    let rows = input.facts.affine.iter().map(|r| r.as_ref().map_or_else(Vec::new, |r| r.entries.iter().map(|(j,a)| (*j,*a)).collect())).collect::<Vec<_>>();
    let constants = input.facts.affine.iter().enumerate().map(|(r, row)| row.as_ref().map_or(0., |row| row.constant - input.bounds[r].0)).collect::<Vec<_>>();
    let eligible = input.facts.affine.iter().map(Option::is_some).collect::<Vec<_>>();
    let lower = contract.variables.iter().map(|v| v.lower).collect::<Vec<_>>();
    let upper = contract.variables.iter().map(|v| v.upper).collect::<Vec<_>>();
    let zeros = vec![0.; m];
    let plan = build_plan(&PlanInput { n_vars:n, n_rows:m, rows:&rows, row_const:&constants, g_l:&zeros, g_u:&zeros, eligible:&eligible, x_l:&lower, x_u:&upper }, &PlanConfig { eq_tol:0., coeff_tol:0., feas_tol:0., max_passes:50 });
    let valid = !plan.report.infeasible && !plan.report.pass_cap_hit
        && plan.n_reduced_vars() == plan.n_reduced_rows()
        && plan.recovery.iter().all(|recovery| match recovery {
            VarRecovery::Kept(_) => true,
            VarRecovery::Constant(c) => c.is_finite(),
            VarRecovery::Affine { coeff,offset,.. } => coeff.is_finite() && *coeff != 0. && offset.is_finite(),
        });
    if !valid || plan.is_identity() {
        if matches!(input.policy, Policy::Explicit { required,.. } if required.contains(&Pass::AffineElimination)) {
            return Err(ProblemError::unsupported("required root affine reduction has no qualified square primal plan"));
        }
        return Ok(None);
    }
    Ok(Some(plan))
}
#[derive(Debug)]
struct Reduced {
    original: Rc<RefCell<Box<dyn NleOracle>>>,
    plan: Rc<EliminationPlan>,
    contract: OracleContract,
    pattern: faer::sparse::SymbolicSparseColMat<usize>,
    gather: Vec<(usize, usize, f64)>,
    point: Vec<f64>,
    residual: Vec<f64>,
    jacobian: Vec<f64>,
    direction: Vec<f64>,
}
impl Reduced {
    fn contract(source: &OracleContract, plan: &EliminationPlan, identity: pse_ids::ContentHash) -> OracleContract {
        let mut contract = source.clone();
        contract.identity = identity;
        contract.variables = plan.vars_kept.iter().enumerate().map(|(i,j)| { let mut v = source.variables[*j].clone(); v.lower=plan.x_l_red[i]; v.upper=plan.x_u_red[i]; v }).collect();
        contract.rows = plan.rows_kept.iter().map(|r| source.rows[*r]).collect();
        contract
    }
    fn new(original: Rc<RefCell<Box<dyn NleOracle>>>, plan: Rc<EliminationPlan>, identity: pse_ids::ContentHash) -> Result<Self, ProblemError> {
        let source = original.borrow();
        let contract = Self::contract(source.contract(), &plan, identity);
        let mut coordinates = vec![None; plan.n_full];
        for (j,recovery) in plan.recovery.iter().enumerate() {
            coordinates[j] = match recovery { VarRecovery::Kept(k)=>Some((*k,1.)), VarRecovery::Constant(_)=>None, VarRecovery::Affine {rep,coeff,..}=>match plan.recovery[*rep] { VarRecovery::Kept(k)=>Some((k,*coeff)), _=>return Err(ProblemError::internal("root affine representative is not retained")) } };
        }
        let mut row_slots=vec![None; plan.m_full];
        for (r,full) in plan.rows_kept.iter().enumerate() { row_slots[*full]=Some(r); }
        let source_pattern=source.jacobian_pattern();
        let mut entries=Vec::new();
        for (j,coordinate) in coordinates.iter().enumerate() { if let Some((column,scale))=coordinate { for index in source_pattern.col_range(j) { if let Some(row)=row_slots[source_pattern.row_idx()[index]] { entries.push((row,*column,index,*scale)); } } } }
        let pairs=entries.iter().map(|(r,c,_,_)| faer::sparse::Pair::new(*r,*c)).collect::<Vec<_>>();
        let (pattern,_)=faer::sparse::SymbolicSparseColMat::try_new_from_indices(contract.rows.len(),contract.variables.len(),&pairs).map_err(|e| ProblemError::numerical(format!("root affine sparse layout: {e:?}")))?;
        let gather=entries.into_iter().map(|(row,column,source,scale)| { let target=pattern.as_ref().col_range(column).find(|slot|pattern.as_ref().row_idx()[*slot]==row).ok_or_else(||ProblemError::internal("root affine gather"))?; Ok((target,source,scale)) }).collect::<Result<Vec<_>,ProblemError>>()?;
        let nnz=source_pattern.compute_nnz();
        drop(source);
        Ok(Self { original, contract, pattern, gather, point:vec![0.;plan.n_full], residual:vec![0.;plan.m_full], jacobian:vec![0.;nnz], direction:vec![0.;plan.n_full], plan })
    }
    fn lift(&mut self, point:&[f64]) -> Result<(),ProblemError> {
        if point.len()!=self.plan.n_reduced_vars() { return Err(ProblemError::Contract("root affine primal extent".into())); }
        self.plan.lift_x(point,&mut self.point);
        if self.point.iter().any(|v|!v.is_finite()) { return Err(ProblemError::numerical("nonfinite reconstructed original root")); }
        Ok(())
    }
}
impl NleOracle for Reduced {
    fn contract(&self)->&OracleContract { &self.contract }
    fn operations(&self)->crate::RootOperations { self.original.borrow().operations() }
    fn jacobian_pattern(&self)->faer::sparse::SymbolicSparseColMatRef<'_,usize> { self.pattern.as_ref() }
    fn residual(&mut self,x:&[f64],out:&mut[f64])->Result<(),ProblemError> {
        self.lift(x)?;
        self.original.borrow_mut().residual(&self.point,&mut self.residual)?;
        for (value,row) in out.iter_mut().zip(&self.plan.rows_kept) { *value=self.residual[*row]; }
        Ok(())
    }
    fn jacobian(&mut self,x:&[f64],out:&mut[f64])->Result<(),ProblemError> {
        self.lift(x)?;
        self.original.borrow_mut().jacobian(&self.point,&mut self.jacobian)?;
        out.fill(0.);
        for (target,source,scale) in &self.gather { out[*target]+=scale*self.jacobian[*source]; }
        Ok(())
    }
    fn jacobian_product(&mut self,x:&[f64],direction:&[f64],out:&mut[f64])->Result<(),ProblemError> {
        self.lift(x)?;
        for (j,recovery) in self.plan.recovery.iter().enumerate() { self.direction[j]=match recovery { VarRecovery::Kept(k)=>direction[*k],VarRecovery::Constant(_)=>0.,VarRecovery::Affine {rep,coeff,..}=>match self.plan.recovery[*rep] {VarRecovery::Kept(k)=>coeff*direction[k],_=>return Err(ProblemError::internal("root affine directional representative"))} }; }
        self.original.borrow_mut().jacobian_product(&self.point,&self.direction,&mut self.residual)?;
        for (value,row) in out.iter_mut().zip(&self.plan.rows_kept) { *value=self.residual[*row]; }
        Ok(())
    }
}
/// Conservative reservation for the retained primal map, support gather and work buffers.
/// This is separate from native factor/solver storage.
pub fn allowance(n:usize,nnz:usize)->Option<usize> {
    n.checked_mul(n)?.checked_add(nnz)?.checked_add(n)?.checked_mul(512)
}
/// Run a qualified primal reduction and independently re-evaluate every original row/bound.
/// # Errors
/// Source admission, native execution, reconstruction or original evaluation failure.
pub fn execute(step:Step<'_>,retained:&mut Retained,run:Roots<'_>,input:Primal<'_>)->Result<crate::solve::SolveReport,ProblemError> {
    step.execution.check()?;
    if matches!(input.policy,Policy::Off) || matches!(input.policy,Policy::Explicit {options,required} if required.is_empty() && (!options.enabled || !options.linear_eq_reduction)) {
        return execution::roots(step,retained,run);
    }
    step.preflight()?;
    let n = run.oracle.contract().variables.len();
    if run.initial.len()!=n || run.initial.iter().any(|value|!value.is_finite()) {return Err(ProblemError::Contract("root affine original start inventory".into()));}
    run.oracle.operations().admit(run.oracle.contract(),!run.oracle.operations().jacobian)?;
    let extent = allowance(n,run.oracle.jacobian_pattern().compute_nnz()).ok_or_else(||ProblemError::memory("root affine projection extent"))?;
    if step.execution.memory.is_some_and(|bytes|extent>bytes) { return Err(ProblemError::memory("root affine projection allowance")); }
    crate::structural::check(run.oracle.contract(),run.oracle.jacobian_pattern(),&vec![(0.,0.);n],crate::structural::Mode::Roots,step.structure.map(|structure|&*structure.witness))?;
    step.adapter.admit_contract(run.oracle.contract(), &std::collections::BTreeMap::new(), step.settings, execution::Budgets {accuracy:step.accuracy,tolerances:step.tolerances,normalization:step.normalization})?;
    let Some(plan)=plan(run.oracle.contract(),&input)? else { return execution::roots(step,retained,run); };
    // Root warm payloads are primal-only and are projected through the same library map.
    let original_contract=run.oracle.contract().clone();
    let mut h=pse_ids::FramedHasher::new(pse_ids::Frame::ImplicitNumericalProductV1);
    h.str("primal-root-affine-native-layout").hash(&original_contract.identity).hash(&step.compatibility.layout).hash(&step.compatibility.profile).hash(&step.normalization.key()).hash(&pse_math::sparse::pattern_key(run.oracle.jacobian_pattern()));
    for value in &plan.recovery { match value { VarRecovery::Kept(k)=>{h.u64(0).u64(*k as u64);},VarRecovery::Constant(c)=>{h.u64(1).f64(*c);},VarRecovery::Affine {rep,coeff,offset}=>{h.u64(2).u64(*rep as u64).f64(*coeff).f64(*offset);} } }
    for row in &plan.rows_kept {h.u64(*row as u64);}
    let layout_identity=h.finish_hash();
    let mut context=pse_ids::FramedHasher::new(pse_ids::Frame::ImplicitNumericalProductV1);
    context.str("primal-root-affine-current-source").hash(&layout_identity).hash(&input.facts.key).hash(&step.compatibility.data);
    let identity=context.finish_hash();
    let plan=Rc::new(plan);
    let mut initial=vec![0.;plan.n_reduced_vars()];
    plan.project_x(run.initial,&mut initial);
    let normalization=pse_math::normalization::Normalization {variables:plan.vars_kept.iter().map(|j|step.normalization.variables[*j]).collect(),rows:plan.rows_kept.iter().map(|r|step.normalization.rows[*r]).collect(),objective:step.normalization.objective};
    let tolerances=quality::Tolerances {variables:plan.vars_kept.iter().map(|j|step.tolerances.variables[*j]).collect(),rows:plan.rows_kept.iter().map(|r|step.tolerances.rows[*r]).collect(),integrality:step.tolerances.integrality};
    // An algebraically valid map may transfer one-sided native bounds into a box.
    // Keep the original admitted route unless this selected adapter admits the actual
    // reduced contract and its physical budgets. A required pass reports the refusal.
    if !initial.is_empty() {
        if let Err(cause) = step.adapter.admit_contract(&Reduced::contract(&original_contract, &plan, layout_identity), &std::collections::BTreeMap::new(), step.settings, execution::Budgets {accuracy:step.accuracy,tolerances:&tolerances,normalization:&normalization}) {
            if matches!(input.policy,Policy::Explicit {required,..} if required.contains(&Pass::AffineElimination)) {
                return Err(ProblemError::unsupported(format!("required root affine reduced native contract: {cause}")));
            }
            return execution::roots(step,retained,run);
        }
    }
    let original=Rc::new(RefCell::new(run.oracle));
    let reduced=Reduced::new(original.clone(),plan.clone(),layout_identity)?;
    let mut compatibility=step.compatibility.clone(); compatibility.layout=layout_identity;compatibility.data=identity;
    let mut warm=step.warm.cloned();
    if let Some(w)=warm.as_mut() { w.validate(&step.compatibility)?; if let WarmPayload::Root(primal)=&mut w.payload {let mut projected=vec![0.;plan.n_reduced_vars()];plan.project_x(primal,&mut projected);*primal=projected;} else {return Err(ProblemError::unsupported("root affine transform requires a root primal warm payload"));} w.compatibility=compatibility.clone(); }
    // A zero-dimensional reduction is a library primal result, never a native convergence.
    let mut report=if initial.is_empty() {
        let mut report=crate::solve::SolveReport::new(step.adapter.backend(),&reduced.contract,crate::solve::NativeTermination {code:0,name:"RootAffineZeroDimensional".into(),message:None,category:crate::solve::Termination::FeasibleOnly,assurance:crate::solve::Assurance::None},&step.execution);
        report.metrics.insert("native.attempted".into(),Metric::Bool(false));
        report.evidence.work=crate::solve::WorkEvidence {evaluations:Some(0),iterations:Some(0),factorizations:Some(0),proof_steps:Some(0)};
        report.candidate=Some(crate::solve::Candidate {kind:crate::solve::CandidateKind::FinalIterate,primal:vec![],objective:None,row_dual:None,bound_dual:None,reduced_costs:None,slacks:None,commitment:None});
        report
    } else {
        execution::roots(Step {adapter:step.adapter,settings:step.settings,snapshot:step.snapshot,structure:None,controls:step.controls,accuracy:step.accuracy,execution:step.execution.clone(),tolerances:&tolerances,normalization:&normalization,compatibility,warm:warm.as_ref()},retained,Roots {oracle:Box::new(reduced),initial:&initial,owner:run.owner})?
    };
    report.variables=original_contract.variables.iter().map(|v|v.id).collect(); report.rows=original_contract.rows.clone();
    let mut original_rechecked=0;
    if let Some(candidate)=report.candidate.as_mut() {
        let mut full=vec![0.;plan.n_full]; plan.lift_x(&candidate.primal,&mut full); candidate.primal=full;
        step.execution.check()?;
        let mut values=vec![0.;plan.m_full];
        let observation=step.execution.counted(crate::solve::WorkEvidence {evaluations:Some(1),iterations:Some(0),factorizations:Some(0),proof_steps:Some(0)},|| {
            original.borrow_mut().residual(&candidate.primal,&mut values)?;
            original.borrow().observe(values)
        })?;
        report.evidence.work.evaluations=report.evidence.work.evaluations.map(|count|count.checked_add(1).ok_or_else(||ProblemError::internal("root original evaluation count overflow"))).transpose()?;
        report.quality=Some(quality::observed(&original_contract,&observation.bounds,&candidate.primal,&observation.values,step.tolerances)?);
        report.observation=Some(observation);
        original_rechecked=plan.m_full;
    }
    // The full-space seed remains authoritative at the boundary.
    if let Some(w)=report.warm_start.as_mut() { if let WarmPayload::Root(primal)=&mut w.payload {let mut full=vec![0.;plan.n_full];plan.lift_x(primal,&mut full);*primal=full;} w.compatibility=step.compatibility.clone(); }
    let mut effective=pounce_presolve::PresolveOptions::defaults();
    effective.enabled=true;effective.linear_eq_reduction=true;
    effective.bound_tightening=false;effective.redundant_constraint_removal=false;effective.fbbt=false;effective.licq_check=false;effective.auxiliary=false;effective.warm_z_bounds=false;
    let requested=input.policy.options();
    let passes=[(Pass::AffineElimination,requested.linear_eq_reduction),(Pass::LinearBounds,requested.bound_tightening),(Pass::RedundantRows,requested.redundant_constraint_removal),(Pass::Fbbt,requested.fbbt),(Pass::RankDiagnostics,requested.licq_check),(Pass::Auxiliary,requested.auxiliary)].into_iter().map(|(pass,wanted)| {
        let affine=pass==Pass::AffineElimination;
        (pass,super::PassReport {requested:requested.enabled&&wanted,eligible:affine,applied:affine,reason:(!affine).then(||"root primal consumer qualifies only the independent affine reconstruction pass".into())})
    }).collect();
    report.preprocessing=Some(super::Report {requested:input.policy.clone(),effective,passes,facts:Some(input.facts.key),transformation:identity,dimensions:super::Dimensions {original_columns:plan.n_full,original_rows:plan.m_full,presolved_columns:plan.n_reduced_vars(),presolved_rows:plan.n_reduced_rows()},diagnostics:std::collections::BTreeMap::from([("native.qualification".into(),"pounce-presolve 0.12.0 exact represented affine primal map; zero coefficient/equality/clamp thresholds; every original row/bound rechecked; no optimizer KKT recovery claim".into())]),columns:plan.vars_kept.iter().map(|j|pse_math::index::OriginalCol::new(*j)).collect(),rows:plan.rows_kept.iter().map(|r|pse_math::index::OriginalRow::new(*r)).collect(),proof:None,working_set:None,resolution:None});
    report.metrics.insert("root.affine.original_columns".into(),Metric::Integer(plan.n_full as i64));
    report.metrics.insert("root.affine.native_columns".into(),Metric::Integer(plan.n_reduced_vars() as i64));
    report.metrics.insert("root.affine.original_rows_rechecked".into(),Metric::Integer(original_rechecked as i64));
    if report.quality.as_ref().is_none_or(|quality|!quality.feasible()) { report.termination.assurance=crate::solve::Assurance::None; }
    quality::qualify(&mut report,step.accuracy);
    Ok(report)
}
