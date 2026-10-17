// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Library-owned requested products. Runtime weights and seeds are ordinary parameters.
use super::*;

#[derive(Clone)]
pub(super) struct DirectProduct {
    evaluator: BlockEvaluator,
    pairs: Arc<Vec<(usize, usize)>>,
    kind: ProductKind,
    inputs: usize,
    outputs: usize,
    coordinates: usize,
    scratch_bytes: usize,
}
pub(super) struct DirectWorker {
    evaluator: BlockEvaluator,
    arguments: Vec<f64>,
    values: Vec<f64>,
}
impl DirectWorker {
    pub(super) fn reset(mut self) -> Result<Self, MathError> {
        self.evaluator = self.evaluator.reset()?;
        self.arguments.fill(f64::NAN);
        self.values.fill(f64::NAN);
        Ok(self)
    }
}
// Selection intentionally drops whole-body expression views. Reconstruct a bounded
// temporary library expression only for this concrete product, never as another model
// authority; original stage execution remains the source/guard authority.
fn flat_expressions(body:&PreparedBody,limits:EvaluationLimits,cancel:&Arc<AtomicBool>) -> Result<Option<Vec<Atom>>,MathError> {
    if !body.is_flat_arithmetic() { return Ok(None); }
    let parameters=body.parameters()?;
    let symbols=symbol_map(&parameters);
    let mut expressions=BTreeMap::<usize,Atom>::new();
    let mut work=0usize;
    for stage in &body.stages {
        if cancel.load(Ordering::Acquire) { return Err(MathError::Cancelled); }
        let Stage::Block{expressions:values,outputs,..}=stage else { continue; };
        for (value,&output) in values.iter().zip(outputs) {
            let mut value=value.clone();
            for input in reads(std::slice::from_ref(&value),&symbols)? {
                if let Some(replacement)=expressions.get(&input) {
                    let bound=value.as_view().get_byte_size().checked_mul(replacement.as_view().get_byte_size().saturating_add(1));
                    let Some(bound)=bound.filter(|n|*n<=limits.scratch_bytes.min(1024*1024)) else { return Ok(None); };
                    crate::construction::additional(bound)?;
                    value=value.replace(parameters[input].clone()).with(replacement.clone());
                    work=work.checked_add(operation_count(value.count_operations()))
                        .ok_or(MathError::Limit("weighted flattening operations"))?;
                    if work>limits.operations { return Ok(None); }
                }
            }
            expressions.insert(output,value);
        }
    }
    Ok(Some(body.outputs.iter().map(|slot|expressions.get(slot).cloned().unwrap_or_else(||parameters[*slot].clone())).collect()))
}

impl PreparedSupport {
    /// Compile the mathematical product consumed by the caller. Numeric bindings do not
    /// participate in immutable program identity.
    pub fn compile_product(
        &self, kind: ProductKind, options: Optimization, limits: EvaluationLimits,
        cancel: &Arc<AtomicBool>,
    ) -> Result<CompiledBody, MathError> {
        self.compile_product_inner(kind, options, limits, cancel, None)
    }
    /// Reuse exact scalar preparations while constructing a distinct requested product.
    pub fn compile_product_reusing(
        &self, kind: ProductKind, options: Optimization, limits: EvaluationLimits,
        cancel: &Arc<AtomicBool>, scalars: &library::ScalarProducts,
    ) -> Result<CompiledBody, MathError> {
        self.compile_product_inner(kind, options, limits, cancel, Some(scalars))
    }
    fn compile_product_inner(
        &self, kind: ProductKind, options: Optimization, limits: EvaluationLimits,
        cancel: &Arc<AtomicBool>, retained: Option<&library::ScalarProducts>,
    ) -> Result<CompiledBody, MathError> {
        let required = match kind {
            ProductKind::SelectedPartials => self.order(),
            ProductKind::JacobianAction => DerivativeOrder::First,
            ProductKind::WeightedHessian | ProductKind::WeightedHessianAction => DerivativeOrder::Second,
        };
        if self.order() != required {
            return Err(MathError::Contract("product support order mismatch".into()));
        }
        let outputs = (0..self.outputs().len()).collect::<Vec<_>>();
        let coordinates = self.dense_coordinates()?;
        if matches!(kind, ProductKind::WeightedHessian | ProductKind::WeightedHessianAction)
            && let Some(expressions) = flat_expressions(self.body(), limits, cancel)?
        {
            return self.compile_direct_product(kind, expressions, options, limits, cancel, retained);
        }
        self.body().compile_scope(&outputs, &coordinates, required, options, limits,
            cancel, false, kind, self, retained)
    }
    fn compile_direct_product(
        &self, kind: ProductKind, flattened:Vec<Atom>, options: Optimization, limits: EvaluationLimits,
        cancel: &Arc<AtomicBool>, retained: Option<&library::ScalarProducts>,
    ) -> Result<CompiledBody, MathError> {
        let body = self.body();
        let selected = (0..self.outputs().len()).collect::<Vec<_>>();
        let coordinates = self.dense_coordinates()?;
        let available = body.selected_order(&selected, &coordinates, false)?;
        if available < DerivativeOrder::Second {
            return Err(MathError::DerivativeDemand { source_id:None, body:None,
                outputs:self.outputs().to_vec(),coordinates:self.coordinates().to_vec(),
                requested:DerivativeOrder::Second, available });
        }
        let mut parameters = body.parameters()?;
        parameters.truncate(body.inputs);
        let next_formal = body.formal_slots.iter().copied().max().unwrap_or(body.slots)
            .checked_add(1).ok_or(MathError::Limit("product formal identity"))?;
        let weighted_extent = flattened.iter().try_fold(0usize, |n, expression| {
            n.checked_add(expression.as_view().get_byte_size())
        }).ok_or(MathError::Limit("weighted symbolic input extent"))?;
        crate::construction::additional(weighted_extent.checked_mul(4)
            .ok_or(MathError::Limit("weighted symbolic input extent"))?)?;
        let mut weighted = Atom::num(0);
        for expression in &flattened {
            let weight = library::formal(next_formal + parameters.len() - body.inputs)?;
            parameters.push(weight.clone());
            weighted += weight * expression;
        }
        let n = coordinates.len();
        let directions = if kind == ProductKind::WeightedHessianAction {
            let directions = (0..n).map(|i| library::formal(next_formal + body.outputs.len() + i))
                .collect::<Result<Vec<_>, _>>()?;
            parameters.extend(directions.iter().cloned());
            directions
        } else { Vec::new() };
        let axes = self.coordinates().iter().enumerate().map(|(axis,&formal)| (formal,axis))
            .collect::<BTreeMap<_,_>>();
        let pairs = self.support().second.iter().flatten()
            .filter_map(|&(i,j)| Some((*axes.get(&i)?,*axes.get(&j)?)))
            .map(|(i,j)| (i.min(j),i.max(j))).collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
        if pairs.len().checked_mul(2).is_none_or(|calls|calls>limits.operations) {
            return Err(MathError::Limit("weighted symbolic derivative calls"));
        }
        let mut expressions = if kind == ProductKind::WeightedHessianAction {
            vec![Atom::num(0); n]
        } else { Vec::with_capacity(pairs.len()) };
        for &(i,j) in &pairs {
            if cancel.load(Ordering::Acquire) { return Err(MathError::Cancelled); }
            let bytes = weighted.as_view().get_byte_size().checked_mul(8)
                .ok_or(MathError::Limit("weighted derivative construction"))?;
            crate::construction::additional(bytes)?;
            let partial = weighted.derivative(Indeterminate::try_from(parameters[coordinates[i]].clone())
                .map_err(MathError::Library)?)
                .derivative(Indeterminate::try_from(parameters[coordinates[j]].clone())
                    .map_err(MathError::Library)?);
            if kind == ProductKind::WeightedHessianAction {
                expressions[i] += &partial * &directions[j];
                if i != j { expressions[j] += &partial * &directions[i]; }
            } else { expressions.push(partial); }
        }
        // Empty sparse products still use a finite constant library output internally;
        // the public product has zero entries and never invents a nonzero support cell.
        let product_len = if kind == ProductKind::WeightedHessianAction { n } else { pairs.len() };
        if expressions.is_empty() { expressions.push(Atom::num(0)); }
        let mut scalars = retained.map(library::ScalarCache::from_products).unwrap_or_default();
        let evaluator = library::bounded_scalar_evaluator(&expressions, &parameters,
            options, cancel, limits, &mut scalars)?;
        let storage = library::storage(&evaluator)?;
        let operations = operation_count(evaluator.count_operations());
        let export = evaluator.export_instructions();
        let modules = options.evaluator.module_bytes();
        let evaluator = match options.evaluator {
            library::Evaluator::Interpreted => BlockEvaluator::Interpreted(evaluator),
            library::Evaluator::CompiledScalar { module_bytes } => {
                #[cfg(feature="compiled-evaluator")]
                { crate::construction::additional(module_bytes)?;
                    BlockEvaluator::Native(crate::native_compiled::compile(&evaluator,&storage,&export,module_bytes,cancel)?) }
                #[cfg(not(feature="compiled-evaluator"))]
                { let _ = (module_bytes,export); return Err(MathError::Contract("compiled product requires compiled-evaluator feature".into())); }
            }
        };
        let numeric = storage.numeric_entries.checked_add(parameters.len())
            .and_then(|n| n.checked_add(expressions.len())).and_then(|n| n.checked_add(product_len))
            .ok_or(MathError::Limit("weighted product scratch"))?;
        limits.allocation(numeric)?;
        let scratch_bytes = numeric.checked_mul(size_of::<f64>())
            .and_then(|n| n.checked_add(storage.instruction_bytes))
            .ok_or(MathError::Limit("weighted product scratch"))?;
        let direct = DirectProduct { evaluator,pairs:Arc::new(pairs),kind,inputs:body.inputs,
            outputs:body.outputs.len(),coordinates:n,scratch_bytes };
        let direct_scalars = scalars.freeze();
        let mut compiled = body.compile_scope(&selected,&coordinates,DerivativeOrder::Value,
            options,limits,cancel,false,ProductKind::SelectedPartials,self,Some(&direct_scalars))?;
        // Derivative-order obligations retain their original execution position and are
        // checked even though this semantic route carries only values through the stages.
        for stage in Arc::make_mut(&mut compiled.programs).values_mut().flatten().flatten() {
            if let CompiledStage::Require { order, .. } = stage { *order = DerivativeOrder::Value; }
        }
        compiled.census[DerivativeOrder::Second] = Some(CompiledOrderCensus {
            order:DerivativeOrder::Second,directional:kind==ProductKind::WeightedHessianAction,
            product:kind,direct_weighted:true,
            stages:1,arithmetic_blocks:1,native_arithmetic_blocks:usize::from(modules>0),
            provider_stages:0,control_stages:0,library_operations:operations,
            library_instructions:storage.instruction_count,instruction_storage_bytes:storage.instruction_bytes,
            frame_entries:0,derivative_components:1,second_pairs:direct.pairs.len(),
            result_entries:product_len,worker_bytes:scratch_bytes,
        });
        compiled.evaluation_cache_bytes=compiled.evaluation_cache_bytes.checked_add(
            (product_len+body.outputs.len()+if kind==ProductKind::WeightedHessianAction {n}else{0})
                .checked_mul(2*size_of::<f64>()).ok_or(MathError::Limit("weighted point cache"))?
        ).ok_or(MathError::Limit("weighted point cache"))?;
        compiled.product = kind;
        compiled.direct_product = Some(Arc::new(direct));
        compiled.worker_bytes = compiled.worker_bytes.checked_add(scratch_bytes)
            .ok_or(MathError::Limit("weighted product worker storage"))?;
        compiled.scratch_bytes = compiled.scratch_bytes.checked_add(numeric * size_of::<f64>())
            .ok_or(MathError::Limit("weighted product numeric storage"))?;
        if compiled.scratch_bytes > limits.scratch_bytes { return Err(MathError::Limit("weighted product numeric storage")); }
        compiled.retained_bytes = compiled.retained_bytes.checked_add(scratch_bytes)
            .and_then(|n| n.checked_add(modules)).ok_or(MathError::Limit("weighted product retained storage"))?;
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("math.weighted_direct_constructions", "count", 1);
            context.integer("math.weighted_direct_outputs", "components",product_len as i64);
            context.integer("math.weighted_direct_operations", "operations",operations as i64);
        }
        Ok(compiled)
    }
}
impl CompiledBody {
    /// Immutable mathematical product, independent of runtime coefficient values.
    pub fn product_kind(&self) -> ProductKind { self.product }
    /// Number of entries consumed by this product's actual caller.
    pub fn product_len(&self) -> usize {
        match self.product {
            ProductKind::SelectedPartials => 0,
            ProductKind::JacobianAction => self.outputs.len(),
            ProductKind::WeightedHessian => self.product_pairs().len(),
            ProductKind::WeightedHessianAction => self.coordinate_offsets.len(),
        }
    }
    /// Ordered selected-axis symmetric pairs of a weighted matrix product.
    pub fn product_pairs(&self) -> &[(usize,usize)] {
        self.direct_product.as_ref().map(|d| d.pairs.as_slice()).unwrap_or_else(||
            self.layouts[DerivativeOrder::Second].as_ref().map_or(&[],|l| l.pairs.as_slice()))
    }
}
impl Worker {
    /// Evaluate into private storage, then publish a complete finite requested product.
    pub fn evaluate_product_into(
        &mut self, inputs:&[f64], bindings:ProductValues<'_>,
        providers:&mut BTreeMap<ProviderKey,Box<dyn Provider>>, cancel:&Arc<AtomicBool>,
        output:&mut[f64],
    ) -> Result<(),MathError> {
        let mut trial=TrialEvaluation::default();
        self.evaluate_product_trial_into(inputs,bindings,providers,cancel,output,&mut trial)
    }
    pub(crate) fn evaluate_product_trial_into(
        &mut self,inputs:&[f64],bindings:ProductValues<'_>,providers:&mut BTreeMap<ProviderKey,Box<dyn Provider>>,
        cancel:&Arc<AtomicBool>,output:&mut[f64],trial:&mut TrialEvaluation,
    ) -> Result<(),MathError> {
        trial.clear();
        self.check_scope()?;
        let (weights,direction) = match bindings {
            ProductValues::SelectedPartials => return Err(MathError::Contract("selected partials use ordered evaluation".into())),
            ProductValues::JacobianAction{direction} => (None,Some(direction)),
            ProductValues::WeightedHessian{weights} => (Some(weights),None),
            ProductValues::WeightedHessianAction{weights,direction} => (Some(weights),Some(direction)),
        };
        if bindings.kind() != self.body.product || output.len() != self.body.product_len()
            || weights.is_some_and(|w|w.len()!=self.body.outputs.len() || w.iter().any(|v|!v.is_finite()))
            || direction.is_some_and(|d|d.len()!=self.body.coordinate_offsets.len() || d.iter().any(|v|!v.is_finite()))
        { return Err(MathError::Contract("finite exact product binding extent/kind".into())); }
        let mut scratch = std::mem::take(&mut self.product_scratch);
        scratch.resize(output.len(),f64::NAN);
        scratch.fill(f64::NAN);
        let evaluated = (|| {
            if let Some(template) = self.body.direct_product.clone() {
                self.evaluate_seeded(inputs,DerivativeOrder::Value,None,providers,cancel,trial)?;
                if self.direct_product.is_none() {
                    let used = self.used_bytes.checked_add(template.scratch_bytes)
                        .ok_or(MathError::Limit("weighted product worker charge"))?;
                    if used > self.prepaid_bytes { return Err(MathError::Limit("weighted product worker charge")); }
                    let mut evaluator = template.evaluator.clone();
                    evaluator.instantiate()?;
                    self.direct_product = Some(DirectWorker { evaluator,
                        arguments:vec![f64::NAN;template.inputs+template.outputs+if template.kind==ProductKind::WeightedHessianAction{template.coordinates}else{0}],
                        values:vec![f64::NAN;scratch.len().max(1)] });
                    self.used_bytes = used;
                }
                let worker = self.direct_product.as_mut().ok_or_else(||MathError::Contract("weighted product worker missing".into()))?;
                worker.arguments[..template.inputs].copy_from_slice(inputs);
                worker.arguments[template.inputs..template.inputs+template.outputs].copy_from_slice(weights.ok_or_else(||MathError::Contract("weighted coefficients missing".into()))?);
                if let Some(direction)=direction { worker.arguments[template.inputs+template.outputs..].copy_from_slice(direction); }
                worker.values.fill(f64::NAN);
                worker.evaluator.evaluate(&worker.arguments,&mut worker.values)
                    .map_err(|detail|MathError::Evaluation{source_id:SemanticId::NIL,order:DerivativeOrder::Second,detail})?;
                let count = scratch.len();
                scratch.copy_from_slice(&worker.values[..count]);
            } else {
                if self.body.product == ProductKind::JacobianAction {
                    self.evaluate_directional_trial_into(inputs,direction.ok_or_else(||MathError::Contract("direction missing".into()))?,providers,cancel,trial)?;
                    scratch.copy_from_slice(&trial.jacobian);
                } else {
                    self.evaluate_seeded(inputs,DerivativeOrder::Second,direction,providers,cancel,trial)?;
                    scratch.fill(0.0);
                    let weights=weights.ok_or_else(||MathError::Contract("weights missing".into()))?;
                    for (row,&weight) in weights.iter().enumerate() {
                        if self.body.product==ProductKind::WeightedHessianAction {
                            let axis=self.body.coordinate_offsets.len();
                            for (i,out) in scratch.iter_mut().enumerate() { *out+=weight*trial.second(row,i,axis); }
                        } else {
                            for (out,&(i,j)) in scratch.iter_mut().zip(self.body.product_pairs()) { *out+=weight*trial.second(row,i,j); }
                        }
                    }
                }
            }
            if scratch.iter().any(|v|!v.is_finite()) { return Err(MathError::Domain{source_id:SemanticId::NIL,requirement:"finite requested product"}); }
            if cancel.load(Ordering::Acquire) { return Err(MathError::Cancelled); }
            self.check_scope()?;
            output.copy_from_slice(&scratch);
            Ok(())
        })();
        if evaluated.is_err() { scratch.fill(f64::NAN); trial.clear(); }
        self.product_scratch=scratch;
        evaluated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn weighted_body() -> PreparedBody {
        crate::initialize().unwrap();
        let x=library::formal(0).unwrap();
        let y=library::formal(1).unwrap();
        PreparedBody::new(2,4,vec![2,3],vec![Stage::Block {
            expressions:vec![x.pow(2)+y.pow(2),&x*&y],outputs:vec![2,3],source:SemanticId::NIL,
        }],DerivativeOrder::Second).unwrap()
    }
    #[test]
    fn weighted_library_products_bind_runtime_coefficients_without_per_output_hessians() {
        let body=weighted_body();
        let cancel=Arc::new(AtomicBool::new(false));
        let support=body.prepare_support(&[0,1],&[0,1],DerivativeOrder::Second,&cancel).unwrap();
        let matrix=support.compile_product(ProductKind::WeightedHessian,Optimization::default(),EvaluationLimits::default(),&cancel).unwrap();
        assert_eq!(matrix.product_pairs(),&[(0,0),(0,1),(1,1)]);
        assert_eq!(matrix.product_len(),3);
        assert_eq!(matrix.census[DerivativeOrder::Second].unwrap().result_entries,3);
        assert_eq!(matrix.census[DerivativeOrder::Second].unwrap().derivative_components,1);
        assert!(matrix.programs[DerivativeOrder::Second].is_none(),"no per-output second-jet program");
        let mut worker=matrix.worker();
        let mut providers=BTreeMap::new();
        let mut output=[f64::NAN;3];
        for (weights,expected) in [([2.,3.],[4.,3.,4.]),([-1.,0.5],[-2.,0.5,-2.]),([2.,3.],[4.,3.,4.])] {
            worker.evaluate_product_into(&[2.,3.],ProductValues::WeightedHessian{weights:&weights},&mut providers,&cancel,&mut output).unwrap();
            assert_eq!(output,expected);
        }
        output=[99.;3];
        assert!(worker.evaluate_product_into(&[1e200,3.],ProductValues::WeightedHessian{weights:&[0.,0.]},&mut providers,&cancel,&mut output).is_err());
        assert_eq!(output,[99.;3],"invalid original values cannot publish a valid weighted cancellation");
        let action=support.compile_product(ProductKind::WeightedHessianAction,Optimization::default(),EvaluationLimits::default(),&cancel).unwrap();
        assert_eq!(action.product_len(),2);
        assert_eq!(action.census[DerivativeOrder::Second].unwrap().result_entries,2);
        let mut worker=action.worker();
        let mut action_output=[f64::NAN;2];
        worker.evaluate_product_into(&[2.,3.],ProductValues::WeightedHessianAction{weights:&[2.,3.],direction:&[5.,7.]},&mut providers,&cancel,&mut action_output).unwrap();
        assert_eq!(action_output,[41.,43.]);
        worker.evaluate_product_into(&[2.,3.],ProductValues::WeightedHessianAction{weights:&[-1.,0.5],direction:&[7.,5.]},&mut providers,&cancel,&mut action_output).unwrap();
        assert_eq!(action_output,[-11.5,-6.5]);
        assert!(worker.evaluate_product_into(&[2.,3.],ProductValues::WeightedHessian{weights:&[2.,3.]},&mut providers,&cancel,&mut action_output).is_err());
    }
    #[test]
    fn weighted_products_retain_derivative_only_original_guard_and_cancel() {
        crate::initialize().unwrap();
        let x=library::formal(0).unwrap();
        let source=pse_ids::named_id(SemanticId::NIL,"weighted original guard");
        let body=PreparedBody::new(1,2,vec![1],vec![
            Stage::Require{argument:0,condition:Condition::Nonzero,order:DerivativeOrder::Second,source,lineage:None},
            Stage::Block{expressions:vec![x.pow(2)],outputs:vec![1],source},
        ],DerivativeOrder::Second).unwrap();
        let cancel=Arc::new(AtomicBool::new(false));
        let support=body.prepare_support(&[0],&[0],DerivativeOrder::Second,&cancel).unwrap();
        let compiled=support.compile_product(ProductKind::WeightedHessian,Optimization::default(),EvaluationLimits::default(),&cancel).unwrap();
        let mut worker=compiled.worker();
        let mut out=[93.];
        assert!(matches!(worker.evaluate_product_into(&[0.],ProductValues::WeightedHessian{weights:&[1.]},&mut BTreeMap::new(),&cancel,&mut out),Err(MathError::Domain{source_id,..}) if source_id==source));
        assert_eq!(out,[93.]);
        cancel.store(true,Ordering::Release);
        assert!(matches!(worker.evaluate_product_into(&[2.],ProductValues::WeightedHessian{weights:&[1.]},&mut BTreeMap::new(),&cancel,&mut out),Err(MathError::Cancelled)));
        assert_eq!(out,[93.]);
    }
}
