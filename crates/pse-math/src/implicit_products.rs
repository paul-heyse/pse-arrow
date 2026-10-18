// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Demanded IFT products over the complete verified state and physical inputs.
use super::*;
use crate::jets::{ProductKind, ProductValues};
use pse_kernels::{PreparedProviderRequest, ProviderDemand, ProviderRuntime};

#[derive(Debug)]
pub(super) struct ResidualProduct {
    kind: ProductKind,
    columns: Vec<usize>,
    order: DerivativeOrder,
    worker: Mutex<ResidualWorker>,
    values: Mutex<Vec<f64>>,
}
#[derive(Debug)]
pub(super) struct ProductScratch {
    tangent: Mat<f64>, lambda: Mat<f64>, mu: Mat<f64>, original: Mat<f64>,
    seed: Vec<f64>, weights: Vec<f64>, dual: Vec<f64>, contracted: Vec<f64>,
    children: Vec<crate::execution::ProviderVisit>,
}
impl ProductScratch {
    pub(super) fn new(n: usize) -> Self {
        Self { tangent:Mat::zeros(n,0),lambda:Mat::zeros(n,0),mu:Mat::zeros(n,0),original:Mat::zeros(n,0),
            seed:Vec::new(),weights:Vec::new(),dual:Vec::new(),contracted:Vec::new(),children:Vec::new() }
    }
}
fn copy_matrix(target: &mut Mat<f64>, source: &Mat<f64>) {
    target.resize_with(source.nrows(),source.ncols(),|_,_|0.);
    for j in 0..source.ncols() { for i in 0..source.nrows() { target[(i,j)]=source[(i,j)]; }}
}
impl Problem {
    fn prepare_program(
        &self, kind: ProductKind, columns: &[usize], order: DerivativeOrder,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<Arc<ResidualProduct>, MathError> {
        scope.check().map_err(crate::error::scope_error)?;
        let mut programs = self.product_workers.lock().map_err(|_| MathError::Contract("implicit product lock poisoned".into()))?;
        if let Some(program) = programs.iter().find(|p| p.kind == kind && p.order == order && p.columns == columns) {
            return Ok(program.clone());
        }
        let n = self.unknowns.len();
        let coordinates = if kind == ProductKind::SelectedPartials {
            (0..n).chain(columns.iter().map(|i| n+i)).collect::<Vec<_>>()
        } else { (0..n+self.inputs).collect() };
        let support = self.parameter_source.prepared_support().incidence(&coordinates, scope.cancellation())?
            .upgrade(order, scope.cancellation())?.at_order(order)?;
        let body = support.compile_product_reusing(kind, crate::library::Optimization::default(),
            self.parameter_source.evaluation_limits(), scope, self.parameter_source.scalar_products())?;
        crate::construction::additional(body.worker_bytes().checked_add(body.evaluation_cache_bytes())
            .and_then(|v| v.checked_add((body.product_len()+body.input_formals().len())*size_of::<f64>()+size_of::<ResidualProduct>()))
            .ok_or(MathError::Limit("implicit residual product storage"))?)?;
        let program = Arc::new(ResidualProduct {
            kind, columns: columns.to_vec(), order,
            values: Mutex::new(vec![f64::NAN; body.product_len()]),
            worker: Mutex::new(ResidualWorker {
                worker: body.worker_scoped(scope.clone()), inputs: vec![f64::NAN; body.input_formals().len()],
                evaluation: crate::execution::TrialEvaluation::default(),
            }),
        });
        program.worker.lock().map_err(|_| MathError::Contract("implicit product lock poisoned".into()))?.worker
            .prepare_providers(&mut *self.providers.lock().map_err(|_| MathError::Contract("implicit provider lock poisoned".into()))?)?;
        programs.push(program.clone());
        Ok(program)
    }
    fn product_program(&self, kind: ProductKind, columns: &[usize], order: DerivativeOrder,
        scope: &pse_kernels::ExecutionScope) -> Result<Arc<ResidualProduct>, MathError> {
        scope.check().map_err(crate::error::scope_error)?;
        self.product_workers.lock().map_err(|_| MathError::Contract("implicit product lock poisoned".into()))?
            .iter().find(|p| p.kind == kind && p.order == order && p.columns == columns).cloned()
            .ok_or_else(|| MathError::Contract("implicit product was not prepared before numerical execution".into()))
    }
    pub(super) fn prepare_product(&self, request: &PreparedProviderRequest, scope: &pse_kernels::ExecutionScope) -> Result<(), MathError> {
        scope.check().map_err(crate::error::scope_error)?;
        let prepare = || {
            let mut demands=self.prepared_demands.lock().map_err(|_| MathError::Contract("implicit preparation lock poisoned".into()))?;
            if demands.contains(request.request()) { return Ok(()); }
            self.prepare_product_inner(request, scope)?;
            crate::construction::additional(size_of::<pse_kernels::ProviderRequest>()+(request.outputs.len()+match &request.demand { ProviderDemand::SelectedColumns { columns }=>columns.len(),_=>0 })*size_of::<usize>())?;
            demands.push(request.request().clone());
            Ok(())
        };
        match &self.construction_owner { Some(owner) => crate::construction::scoped(owner.clone(), prepare), None => prepare() }
    }
    fn prepare_product_inner(&self, request: &PreparedProviderRequest, scope: &pse_kernels::ExecutionScope) -> Result<(), MathError> {
        let n = self.unknowns.len();
        if request.order == DerivativeOrder::Value { return Ok(()); }
        let partial_extent = |k: usize| {
            let width = n.checked_add(k)?;
            let base = n.checked_mul(width)?;
            if request.order >= DerivativeOrder::Second { base.checked_mul(width) } else { Some(base) }
        };
        let extent = match &request.demand {
            ProviderDemand::CompleteJets => if request.order == DerivativeOrder::Value { Some(n) } else { partial_extent(self.inputs) },
            ProviderDemand::SelectedColumns { columns } => partial_extent(columns.len()),
            ProviderDemand::JacobianAction { directions } => n.checked_mul(*directions),
            ProviderDemand::TransposeAction { cotangents } => n.checked_mul(*cotangents).map(|v| v.max(n+self.inputs)),
            ProviderDemand::WeightedSecondAction { directions } => n.checked_mul(*directions).map(|v| v.max(n+self.inputs)),
        };
        let extent = extent.and_then(|v| if request.order >= DerivativeOrder::First { n.checked_mul(n).map(|state| v.max(state)) } else { Some(v) });
        if request.extents().jacobian > self.max_entries || request.extents().hessians > self.max_entries || extent.is_none_or(|v| v > self.max_entries) { return Err(MathError::Limit("implicit demanded derivative extent")); }
        let batch = match &request.demand {
            ProviderDemand::JacobianAction { directions } | ProviderDemand::WeightedSecondAction { directions } => *directions,
            ProviderDemand::TransposeAction { cotangents } => *cotangents,
            ProviderDemand::SelectedColumns { columns } => if request.order == DerivativeOrder::Second { columns.len().checked_mul(columns.len()+1).and_then(|v| v.checked_div(2)).ok_or(MathError::Limit("implicit RHS extent"))? } else { columns.len() },
            ProviderDemand::CompleteJets => if request.order == DerivativeOrder::Second { self.inputs.checked_mul(self.inputs+1).and_then(|v| v.checked_div(2)).ok_or(MathError::Limit("implicit RHS extent"))? } else { self.inputs },
        }.max(1);
        let output_width = match &request.demand {
            ProviderDemand::SelectedColumns { columns } => columns.len(),
            ProviderDemand::JacobianAction { directions } => *directions,
            ProviderDemand::TransposeAction { cotangents } => self.inputs.checked_mul(*cotangents).ok_or(MathError::Limit("implicit result extent"))?,
            _ => self.inputs,
        };
        let numerical = match &request.demand {
            ProviderDemand::CompleteJets | ProviderDemand::SelectedColumns { .. } => n.checked_mul(1+output_width+if request.order == DerivativeOrder::Second { output_width.checked_mul(output_width).ok_or(MathError::Limit("implicit result extent"))? } else {0}),
            ProviderDemand::JacobianAction { .. } => n.checked_mul(1+output_width),
            ProviderDemand::TransposeAction { .. } => n.checked_add(output_width),
            ProviderDemand::WeightedSecondAction { directions } => self.inputs.checked_mul(1+directions).and_then(|v| v.checked_add(n)),
        }.ok_or(MathError::Limit("implicit numerical product extent"))?;
        let parallel=self.factor.lock().map_err(|_| MathError::Contract("implicit factor lock poisoned".into()))?.parallel;
        let memory = self.symbolic.solve_in_place_scratch::<f64>(batch, parallel)
            .or(self.symbolic.solve_transpose_in_place_scratch::<f64>(batch, parallel)).size_bytes();
        let bindings = match &request.demand {
            ProviderDemand::JacobianAction { directions } => self.inputs.checked_mul(*directions),
            ProviderDemand::WeightedSecondAction { directions } => self.inputs.checked_mul(*directions).and_then(|v| v.checked_add(request.outputs.len())),
            ProviderDemand::TransposeAction { cotangents } => request.outputs.len().checked_mul(*cotangents),
            _ => Some(0),
        }.ok_or(MathError::Limit("implicit numeric bindings"))?;
        let bytes = n.checked_mul(batch).and_then(|v| v.checked_mul(8))
            .and_then(|v| v.checked_add(8*(n+self.inputs)))
            .and_then(|v| v.checked_add(2*numerical+bindings))
            .and_then(|v| v.checked_mul(size_of::<f64>()))
            .and_then(|v| v.checked_add(memory))
            .and_then(|v| v.checked_add(if request.demand == ProviderDemand::CompleteJets { self.parameter_source.worker_bytes()+self.parameter_source.evaluation_cache_bytes() } else {0}))
            .ok_or(MathError::Limit("implicit prepared scratch extent"))?;
        let mut admitted = self.prepared_bytes.lock().map_err(|_| MathError::Contract("implicit preparation lock poisoned".into()))?;
        if bytes > *admitted { crate::construction::additional(bytes-*admitted)?; *admitted=bytes; }
        drop(admitted);
        if request.demand == ProviderDemand::CompleteJets {
            self.worker.lock().map_err(|_| MathError::Contract("implicit worker lock poisoned".into()))?.worker.prepare_providers(
                &mut *self.providers.lock().map_err(|_| MathError::Contract("implicit provider lock poisoned".into()))?)?;
        }
        if matches!(request.demand, ProviderDemand::JacobianAction { .. } | ProviderDemand::TransposeAction { .. } | ProviderDemand::WeightedSecondAction { .. }) {
            self.prepare_program(ProductKind::JacobianAction, &[], DerivativeOrder::First, scope)?;
        }
        match &request.demand {
            ProviderDemand::SelectedColumns { columns } => { self.prepare_program(ProductKind::SelectedPartials, columns, request.order, scope)?; }
            ProviderDemand::JacobianAction { .. } => { self.prepare_program(ProductKind::JacobianAction, &[], DerivativeOrder::First, scope)?; }
            ProviderDemand::TransposeAction { .. } => { self.prepare_program(ProductKind::TransposeJacobianAction, &[], DerivativeOrder::First, scope)?; }
            ProviderDemand::WeightedSecondAction { .. } => {
                self.prepare_program(ProductKind::JacobianAction, &[], DerivativeOrder::First, scope)?;
                self.prepare_program(ProductKind::TransposeJacobianAction, &[], DerivativeOrder::First, scope)?;
                self.prepare_program(ProductKind::WeightedHessianAction, &[], DerivativeOrder::Second, scope)?;
            }
            ProviderDemand::CompleteJets => {}
        }
        Ok(())
    }
    fn with_product<R>(&self, program: &ResidualProduct, parameters: &[f64], point: &[f64], product: ProductValues<'_>,
        cancel: &Arc<AtomicBool>, expected: Option<&[crate::execution::ProviderVisit]>, consume: impl FnOnce(&[f64], &crate::execution::TrialEvaluation) -> Result<R, MathError>) -> Result<R, MathError> {
        let mut held = program.worker.lock().map_err(|_| MathError::Contract("implicit residual product lock poisoned".into()))?;
        let ResidualWorker { worker, inputs, evaluation } = &mut *held;
        for (&formal, input) in worker.input_formals().iter().zip(inputs.iter_mut()) {
            *input = if formal < point.len() { point[formal] } else { parameters[formal-point.len()] };
        }
        let mut values = program.values.lock().map_err(|_| MathError::Contract("implicit residual product result lock poisoned".into()))?;
        worker.evaluate_product_trial_into(inputs, product, &mut *self.providers.lock()
            .map_err(|_| MathError::Contract("implicit provider lock poisoned".into()))?, cancel, &mut values, evaluation)?;
        if expected.is_some_and(|expected| evaluation.provider_visits() != expected) || evaluation.provider_visits().iter().any(|visit| visit.linearization.is_none()) {
            return Err(MathError::Scope(pse_kernels::ProviderError::LinearizationChanged));
        }
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("implicit.derivatives.product_components", "components", values.len() as u64);
        }
        consume(&values, evaluation)
    }
    fn solve_transpose(&self, factor: &mut FactorWorkspace, rhs: faer::MatMut<'_, f64>, cancel: &Arc<AtomicBool>) -> Result<(), MathError> {
        observed_ift!("implicit.factor.transpose", cancel, {
            self.ensure_solve_scratch(factor, rhs.ncols())?;
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("implicit.factor.transpose_right_hand_sides", "columns", rhs.ncols() as u64);
            }
            LuRef::new_unchecked(&self.symbolic, &factor.numeric).solve_transpose_in_place_with_conj(
                Conj::No, rhs, factor.parallel, MemStack::new(&mut factor.memory));
            Ok(())
        })
    }
    fn checked_solve(&self, matrix: &SparseColMat<usize, f64>, factor: &mut FactorWorkspace,
        rhs: &mut Mat<f64>, original: &Mat<f64>, transpose: bool, options: &Options, cancel: &Arc<AtomicBool>) -> Result<(), MathError> {
        if transpose { self.solve_transpose(factor, rhs.as_mut(), cancel)?; }
        else { self.solve_factor(factor, rhs.as_mut(), cancel)?; }
        check_solve_into(matrix, rhs, original, options.derivative_tolerance, self.id, transpose, &mut factor.sums, &mut factor.scales)
    }
    pub(super) fn demanded_derivatives(&self, parameters: &[f64], point: &[f64], request: &PreparedProviderRequest,
        runtime: ProviderRuntime<'_>, options: &Options, premise: ContentHash, scope: &pse_kernels::ExecutionScope) -> Result<ProviderValues, MathError> {
        let cancel = scope.cancellation();
        if request.order == DerivativeOrder::Value { return Ok(ProviderValues { values:point.to_vec(),jacobian:Vec::new(),hessians:Vec::new() }); }
        if request.demand == ProviderDemand::CompleteJets {
            return self.derivatives_at_verified_point(parameters, point, request.order, options, cancel, premise);
        }
        if let ProviderDemand::SelectedColumns { columns } = &request.demand {
            let program = self.product_program(ProductKind::SelectedPartials, columns, request.order, scope)?;
            return self.derivatives_from_worker(parameters, point, request.order, options, cancel, premise, &program.worker, columns);
        }
        if point.iter().zip(&self.unknowns).zip(&options.variable_tolerance).any(|((x,b),t)| x-b.lower <= *t || b.upper-x <= *t) {
            return Err(MathError::Domain { source_id: self.id, requirement: "implicit derivative neighborhood meets a declared unknown bound" });
        }
        let n = point.len(); let p = self.inputs; let width = n+p;
        let state = self.product_program(ProductKind::JacobianAction, &[], DerivativeOrder::First, scope)?;
        let mut scratch = self.product_scratch.lock().map_err(|_| MathError::Contract("implicit product scratch lock poisoned".into()))?;
        let ProductScratch { tangent,seed,children,.. }=&mut *scratch;
        tangent.resize_with(n,n,|_,_|0.);seed.resize(width,0.);seed.fill(0.);children.clear();
        // All physical coordinates remain selected, even for a zero parameter
        // seed. Each active differentiable child therefore qualifies its actual
        // First context rather than returning a Value-only observation.
        for j in 0..n {
            if j>0 { seed[j-1]=0.; } seed[j]=1.;
            self.with_product(&state,parameters,point,ProductValues::JacobianAction { direction:seed },cancel,None,|values,trial| {
                self.verify_values(&trial.values,options)?;
                if j==0 { children.extend_from_slice(trial.provider_visits()); }
                else if children.as_slice()!=trial.provider_visits() { return Err(MathError::Scope(pse_kernels::ProviderError::LinearizationChanged)); }
                for i in 0..n { tangent[(i,j)]=values[i]; } Ok(())
            })?;
        }
        seed.fill(0.);
        let coefficients=(0..n).flat_map(|j| self.pattern.as_ref().row_idx_of_col(j).map(move |i| (i,j)))
            .map(|(i,j)| tangent[(i,j)]).collect();
        let matrix=SparseColMat::new(self.pattern.clone(),coefficients);
        let mut factor=self.factorize_ift(&matrix,premise,point,children,cancel)?;
        let ProductScratch { tangent,lambda,mu,original,seed,weights,dual,contracted,children }=&mut *scratch;
        let result = (|| {
            match request.demand {
                ProviderDemand::JacobianAction { directions } => {
                    let forward = self.product_program(ProductKind::JacobianAction, &[], DerivativeOrder::First, scope)?;
                    seed.resize(width,0.);seed.fill(0.);
                    tangent.resize_with(n,directions,|_,_|0.);
                    for d in 0..directions {
                        seed[n..].copy_from_slice(&runtime.directions[d*p..(d+1)*p]);
                        self.with_product(&forward, parameters, point, ProductValues::JacobianAction { direction: &seed }, cancel, Some(children), |values,_| {
                            for i in 0..n { tangent[(i,d)] = -values[i]; } Ok(())
                        })?;
                    }
                    copy_matrix(original,tangent);
                    self.checked_solve(&matrix, &mut factor, tangent, original, false, options, cancel)?;
                    Ok(ProviderValues { values: point.to_vec(), jacobian: (0..n).flat_map(|i| (0..directions).map(move |d| (i,d))).map(|(i,d)| tangent[(i,d)]).collect(), hessians: Vec::new() })
                }
                ProviderDemand::TransposeAction { cotangents } => {
                    let transpose = self.product_program(ProductKind::TransposeJacobianAction, &[], DerivativeOrder::First, scope)?;
                    lambda.resize_with(n,cotangents,|_,_|0.);for c in 0..cotangents { for i in 0..n { lambda[(i,c)]=0.; }}
                    for c in 0..cotangents { for (i,&row) in request.outputs.iter().enumerate() { lambda[(row,c)] = runtime.cotangents[c*request.outputs.len()+i]; }}
                    copy_matrix(original,lambda);
                    self.checked_solve(&matrix, &mut factor, lambda, original, true, options, cancel)?;
                    let mut jacobian = vec![0.; cotangents*p]; weights.resize(n,0.);
                    for c in 0..cotangents {
                        for i in 0..n { weights[i] = lambda[(i,c)]; }
                        self.with_product(&transpose, parameters, point, ProductValues::TransposeJacobianAction { weights: &weights }, cancel, Some(children), |values,_| {
                            for j in 0..p { jacobian[c*p+j] = -values[n+j]; } Ok(())
                        })?;
                    }
                    Ok(ProviderValues { values: point.to_vec(), jacobian, hessians: Vec::new() })
                }
                ProviderDemand::WeightedSecondAction { directions } => {
                    let forward = self.product_program(ProductKind::JacobianAction, &[], DerivativeOrder::First, scope)?;
                    let transpose = self.product_program(ProductKind::TransposeJacobianAction, &[], DerivativeOrder::First, scope)?;
                    let second = self.product_program(ProductKind::WeightedHessianAction, &[], DerivativeOrder::Second, scope)?;
                    tangent.resize_with(n,directions,|_,_|0.);seed.resize(width,0.);seed.fill(0.);
                    for d in 0..directions {
                        seed[n..].copy_from_slice(&runtime.directions[d*p..(d+1)*p]);
                        self.with_product(&forward, parameters, point, ProductValues::JacobianAction { direction: &seed }, cancel, Some(children), |values,_| {
                            for i in 0..n { tangent[(i,d)] = -values[i]; } Ok(())
                        })?;
                    }
                    copy_matrix(original,tangent);
                    self.checked_solve(&matrix, &mut factor, tangent, original, false, options, cancel)?;
                    lambda.resize_with(n,1,|_,_|0.);for i in 0..n { lambda[(i,0)]=0.; }
                    for (i,&row) in request.outputs.iter().enumerate() { lambda[(row,0)] = runtime.weights[i]; }
                    copy_matrix(original,lambda);
                    self.checked_solve(&matrix, &mut factor, lambda, original, true, options, cancel)?;
                    weights.resize(n,0.);for i in 0..n { weights[i]=lambda[(i,0)]; }
                    let mut jacobian = vec![0.; p];
                    self.with_product(&transpose, parameters, point, ProductValues::TransposeJacobianAction { weights: &weights }, cancel, Some(children), |values,_| {
                        for j in 0..p { jacobian[j] = -values[n+j]; } Ok(())
                    })?;
                    let mut hessians = vec![0.; p*directions]; contracted.resize(width,0.);
                    for d in 0..directions {
                        for i in 0..n { seed[i] = tangent[(i,d)]; }
                        seed[n..].copy_from_slice(&runtime.directions[d*p..(d+1)*p]);
                        self.with_product(&second, parameters, point, ProductValues::WeightedHessianAction { weights: &weights, direction: &seed }, cancel, Some(children), |values,_| { contracted.copy_from_slice(values); Ok(()) })?;
                        mu.resize_with(n,1,|_,_|0.);for i in 0..n { mu[(i,0)]=contracted[i]; }copy_matrix(original,mu);
                        self.checked_solve(&matrix, &mut factor, mu, original, true, options, cancel)?;
                        dual.resize(n,0.);for i in 0..n { dual[i]=mu[(i,0)]; }
                        self.with_product(&transpose, parameters, point, ProductValues::TransposeJacobianAction { weights: &dual }, cancel, Some(children), |values,_| {
                            for j in 0..p { hessians[j*directions+d] = values[n+j]-contracted[n+j]; } Ok(())
                        })?;
                    }
                    Ok(ProviderValues { values: point.to_vec(), jacobian, hessians })
                }
                _ => Err(MathError::Contract("implicit product dispatch".into())),
            }
        })();
        if result.is_err() { factor.association = None; }
        result
    }
}
