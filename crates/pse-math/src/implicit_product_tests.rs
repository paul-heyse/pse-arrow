// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent coupled-state products, ordering and finite-demand refusal controls.
use super::*;
use pse_kernels::{ProviderBuffer, ProviderDemand, ProviderRuntime};

#[derive(Debug)]
struct CountedAffine { actual: Affine, calls: Arc<AtomicUsize> }
impl InnerSolver for CountedAffine {
    fn minimum_order(&self) -> DerivativeOrder { self.actual.minimum_order() }
    fn identity(&self) -> ContentHash { self.actual.identity() }
    fn worker(&self) -> Box<dyn InnerWorker> { Box::new(CountedAffineWorker { actual: self.actual.worker(), calls: self.calls.clone() }) }
}
#[derive(Debug)]
struct CountedAffineWorker { actual: Box<dyn InnerWorker>, calls: Arc<AtomicUsize> }
impl InnerWorker for CountedAffineWorker {
    fn solve(&mut self, problem: Arc<Problem>, parameters: &[f64], options: &Options, cancel: &pse_diagnostics::Cancellation) -> Result<Vec<f64>, MathError> {
        self.calls.fetch_add(1, Ordering::Relaxed); self.actual.solve(problem, parameters, options, cancel)
    }
}
fn coupled(parameters: usize, max_entries: usize) -> (Factory, Vec<f64>, Arc<AtomicUsize>) {
    crate::initialize().unwrap();
    let id = pse_ids::named_id(SemanticId::NIL, "coupled implicit product oracle");
    let control = pse_diagnostics::Cancellation::default();
    let formals = (0..parameters+2).map(|i| crate::library::formal(i).unwrap()).collect::<Vec<_>>();
    let a = &formals[2]; let b = &formals[3];
    let tail = (2..parameters).fold(Atom::num(0), |sum,j| sum + &formals[j+2]*Atom::num((j+1) as i64));
    let u = a.clone().pow(2)+b+tail;
    // A=[[1,1],[2,-1]] is deliberately nonsymmetric; z0=(u+ab)/3,z1=(2u-ab)/3.
    let body = PreparedBody::new(parameters+2, parameters+4, vec![parameters+2,parameters+3], vec![Stage::Block {
        expressions: vec![&formals[0]+&formals[1]-u, Atom::num(2)*&formals[0]-&formals[1]-a*b],
        outputs: vec![parameters+2,parameters+3], source: id,
    }], DerivativeOrder::Second).unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let solver = Arc::new(CountedAffine { actual: Affine::new(&body,2).unwrap(), calls: calls.clone() });
    let compiled = Arc::new(body.compile(&[0,1], &(0..parameters+2).collect::<Vec<_>>(), DerivativeOrder::Second,
        Optimization::default(), EvaluationLimits::default(), &ExecutionScope::new(control.clone(),None)).unwrap());
    let registry = pse_quantity::standard::standard_registry().unwrap(); let q = registry.neutral_dimensionless().unwrap();
    let unit = registry.quantity_type(q).unwrap().canonical_unit;
    let port = |name: &str| pse_kernels::Port { id: pse_ids::named_id(id,name), quantity:q, unit };
    let outputs = vec![port("z0"),port("z1")];
    let unknowns = outputs.iter().map(|port| Unknown { id:port.id, lower:-1e6, upper:1e6 }).collect::<Vec<_>>();
    let factory = Factory {
        selection: Selection::default(), requirements: pse_kernels::DerivativeRequirements::new(
            DerivativeOrder::Second,DerivativeOrder::Second,DerivativeOrder::Second,DerivativeOrder::First,DerivativeOrder::Second).unwrap(),
        spec: ProviderSpec { id,revision:ContentHash::from_bytes([46;32]),data:ContentHash::from_bytes([47;32]),
            inputs:(0..parameters).map(|i| port(&format!("p{i}"))).collect(),outputs,shapes:Default::default(),
            derivative_source:pse_kernels::DerivativeSource::Analytic,derivatives:DerivativeOrder::Second,smoothness:DerivativeOrder::Second },
        bodies:residual_bodies_for_test(compiled),unknowns:unknowns.clone(),rows:vec![port("f0").id,port("f1").id],
        configuration:Configuration::Fixed(unknowns,Options { start:vec![0.;2],variable_nominals:vec![1.;2],variable_tolerance:vec![1e-10;2],
            residual_tolerance:vec![1e-10;2],iterations:10,time_limit:Duration::from_secs(10),derivative_tolerance:1e-11 }),
        hints:None,terms:None,solver,cancel:control,max_entries,providers:BTreeMap::new(),
    };
    let mut values = vec![0.001;parameters]; values[0]=1.5; values[1]=0.4;
    (factory,values,calls)
}
fn analytic(parameters:&[f64]) -> ProviderValues {
    let p=parameters.len(); let a=parameters[0]; let b=parameters[1];
    let u=a*a+b+(2..p).map(|j| (j+1) as f64*parameters[j]).sum::<f64>();
    let mut jacobian=vec![0.;2*p];
    jacobian[0]=(2.*a+b)/3.;jacobian[1]=(1.+a)/3.;jacobian[p]=(4.*a-b)/3.;jacobian[p+1]=(2.-a)/3.;
    for j in 2..p { jacobian[j]=(j+1) as f64/3.;jacobian[p+j]=2.*(j+1) as f64/3.; }
    let mut hessians=vec![0.;2*p*p];hessians[0]=2./3.;hessians[1]=1./3.;hessians[p]=1./3.;
    hessians[p*p]=4./3.;hessians[p*p+1]=-1./3.;hessians[p*p+p]=-1./3.;
    ProviderValues { values:vec![(u+a*b)/3.,(2.*u-a*b)/3.],jacobian,hessians }
}

#[test]
fn selected_implicit_products_match_coupled_analytic_oracle_without_complete_parameter_jets() {
    for p in [2,12,64] {
        let hub=pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation=hub.start("coupled selective implicit products");let observation=operation.context();let _entered=observation.enter();
        let (factory,parameters,solves)=coupled(p,(2*p+16).max(32));
        let expected=analytic(&parameters);let mut provider=factory.create().unwrap();let flag=factory.cancel.flag();
        let context=EvaluationContext { cancelled:flag.as_ref(),max_result_bytes:1<<20 };
        let outputs=vec![1,0];
        let columns=vec![1,0];
        let selected=provider.prepare(ProviderRequest { outputs:outputs.clone(),order:DerivativeOrder::Second,demand:ProviderDemand::SelectedColumns { columns:columns.clone() }}).unwrap();
        let mut buffer=ProviderBuffer::new(&selected);
        let actual=buffer.fill(&mut provider,&parameters,&selected,Default::default(),&context).unwrap();
        close(actual.values,&[expected.values[1],expected.values[0]]);
        for (row,&i) in outputs.iter().enumerate() { for (axis,&j) in columns.iter().enumerate() {
            close(&actual.jacobian[row*2+axis..row*2+axis+1],&[expected.jacobian[i*p+j]]);
            for (other,&k) in columns.iter().enumerate() { close(&actual.hessians[(row*2+axis)*2+other..(row*2+axis)*2+other+1],&[expected.hessians[(i*p+j)*p+k]]); }
        }}
        assert_eq!(solves.load(Ordering::Relaxed),1);
        assert_eq!(count(&observation,"implicit.derivatives.parameter_columns"),2);
        assert_eq!(count(&observation,"implicit.factor.right_hand_sides"),5,"two selected first and three selected second RHS, independent of total parameters");
        let direction=(0..2*p).map(|j| (j+1) as f64/11.).collect::<Vec<_>>();
        let forward=provider.prepare(ProviderRequest { outputs:outputs.clone(),order:DerivativeOrder::First,demand:ProviderDemand::JacobianAction { directions:2 }}).unwrap();
        let mut buffer=ProviderBuffer::new(&forward);
        let actual=buffer.fill(&mut provider,&parameters,&forward,ProviderRuntime { directions:&direction,..Default::default() },&context).unwrap();
        for (row,&i) in outputs.iter().enumerate() { for d in 0..2 {
            close(&actual.jacobian[row*2+d..row*2+d+1],&[(0..p).map(|j| expected.jacobian[i*p+j]*direction[d*p+j]).sum()]);
        }}
        let cotangents=[2.,-3.,-1.,4.];
        let transpose=provider.prepare(ProviderRequest { outputs:outputs.clone(),order:DerivativeOrder::First,demand:ProviderDemand::TransposeAction { cotangents:2 }}).unwrap();
        let mut buffer=ProviderBuffer::new(&transpose);
        let actual=buffer.fill(&mut provider,&parameters,&transpose,ProviderRuntime { cotangents:&cotangents,..Default::default() },&context).unwrap();
        for c in 0..2 { for j in 0..p { close(&actual.jacobian[c*p+j..c*p+j+1],&[(0..2).map(|row| cotangents[c*2+row]*expected.jacobian[outputs[row]*p+j]).sum()]); }}
        let weights=[2.,-3.];
        let second=provider.prepare(ProviderRequest { outputs:outputs.clone(),order:DerivativeOrder::Second,demand:ProviderDemand::WeightedSecondAction { directions:2 }}).unwrap();
        let mut buffer=ProviderBuffer::new(&second);
        let actual=buffer.fill(&mut provider,&parameters,&second,ProviderRuntime { weights:&weights,directions:&direction,..Default::default() },&context).unwrap();
        for j in 0..p {
            close(&actual.jacobian[j..j+1],&[(0..2).map(|row| weights[row]*expected.jacobian[outputs[row]*p+j]).sum()]);
            for d in 0..2 { close(&actual.hessians[j*2+d..j*2+d+1],&[(0..2).flat_map(|row| (0..p).map(move |k| (row,k))).map(|(row,k)| weights[row]*expected.hessians[(outputs[row]*p+j)*p+k]*direction[d*p+k]).sum()]); }
        }
        assert_eq!(solves.load(Ordering::Relaxed),1,"different numerical products preserve the qualified complete state");
        if p>2 {
            let before=solves.load(Ordering::Relaxed);
            assert!(provider.prepare(ProviderRequest::all(&factory.spec,DerivativeOrder::Second)).is_err());
            assert_eq!(solves.load(Ordering::Relaxed),before,"complete extent refusal precedes native effects");
        } else {
            let full=provider.prepare(ProviderRequest::all(&factory.spec,DerivativeOrder::Second)).unwrap();let mut buffer=ProviderBuffer::new(&full);
            let actual=buffer.fill(&mut provider,&parameters,&full,Default::default(),&context).unwrap();
            close(actual.values,&expected.values);close(actual.jacobian,&expected.jacobian);close(actual.hessians,&expected.hessians);
        }
    }
}

#[test]
fn selected_implicit_products_keep_off_column_values_cancellation_and_refusal_recovery() {
    let (factory,mut parameters,solves)=coupled(12,64);let mut provider=factory.create().unwrap();
    let request=provider.prepare(ProviderRequest { outputs:vec![0],order:DerivativeOrder::Second,demand:ProviderDemand::SelectedColumns { columns:vec![0] }}).unwrap();
    let mut buffer=ProviderBuffer::new(&request);let flag=factory.cancel.flag();let context=EvaluationContext { cancelled:flag.as_ref(),max_result_bytes:1<<20 };
    let original=buffer.fill(&mut provider,&parameters,&request,Default::default(),&context).unwrap().values[0];
    parameters[11]+=1.;let expected=analytic(&parameters);
    let actual=buffer.fill(&mut provider,&parameters,&request,Default::default(),&context).unwrap();
    close(actual.values,&expected.values[..1]);assert_ne!(actual.values[0],original);assert_eq!(solves.load(Ordering::Relaxed),2);
    factory.cancel.cancel();assert!(matches!(buffer.fill(&mut provider,&parameters,&request,Default::default(),&context),Err(pse_kernels::ProviderError::Cancelled)));
    flag.store(false,Ordering::Release);
    close(buffer.fill(&mut provider,&parameters,&request,Default::default(),&context).unwrap().values,&expected.values[..1]);
}

#[test]
fn selected_implicit_nonlinear_second_actions_use_the_full_adjoint_equation() {
    let control=pse_diagnostics::Cancellation::default(); let (factory,work)=factory(&control);
    let mut provider=factory.create().unwrap(); let flag=control.flag();
    let context=EvaluationContext { cancelled:flag.as_ref(),max_result_bytes:1<<20 };
    let forward=provider.prepare(ProviderRequest { outputs:vec![0],order:DerivativeOrder::First,demand:ProviderDemand::JacobianAction { directions:2 }}).unwrap();
    let transpose=provider.prepare(ProviderRequest { outputs:vec![0],order:DerivativeOrder::First,demand:ProviderDemand::TransposeAction { cotangents:1 }}).unwrap();
    let second=provider.prepare(ProviderRequest { outputs:vec![0],order:DerivativeOrder::Second,demand:ProviderDemand::WeightedSecondAction { directions:2 }}).unwrap();
    let parameters=[0.1,0.01]; let directions=[1.3,-0.4,-0.2,0.9];let weights=[-2.4];
    let mut buffer=ProviderBuffer::new(&forward);
    let actual=buffer.fill(&mut provider,&parameters,&forward,ProviderRuntime { directions:&directions,..Default::default() },&context).unwrap();
    let expected=oracle(parameters[0],parameters[1],actual.values[0]);let token=actual.linearization.expect("First qualifies state/A");
    for d in 0..2 { close(&actual.jacobian[d..d+1],&[(0..2).map(|j| expected.jacobian[j]*directions[d*2+j]).sum()]); }
    let mut buffer=ProviderBuffer::new(&transpose);
    let actual=buffer.fill(&mut provider,&parameters,&transpose,ProviderRuntime { cotangents:&weights,expected_linearization:Some(token),..Default::default() },&context).unwrap();
    assert_eq!(actual.linearization,Some(token));
    close(actual.jacobian,&expected.jacobian.iter().map(|j| weights[0]*j).collect::<Vec<_>>());
    let mut buffer=ProviderBuffer::new(&second);
    let actual=buffer.fill(&mut provider,&parameters,&second,ProviderRuntime { weights:&weights,directions:&directions,expected_linearization:Some(token),..Default::default() },&context).unwrap();
    assert_eq!(actual.linearization,Some(token),"product directions and weights do not enter the linearization");
    close(actual.jacobian,&expected.jacobian.iter().map(|j| weights[0]*j).collect::<Vec<_>>());
    for j in 0..2 { for d in 0..2 { close(&actual.hessians[j*2+d..j*2+d+1],&[weights[0]*(0..2).map(|k| expected.hessians[j*2+k]*directions[d*2+k]).sum::<f64>()]); }}
    assert_eq!(work.solves.load(Ordering::Relaxed),1);
    // A new physical input invalidates the old owner-issued context after effects;
    // unqualified use of the new point still recovers normally.
    let changed=[0.11,0.01];
    assert!(matches!(buffer.fill(&mut provider,&changed,&second,ProviderRuntime { weights:&weights,directions:&directions,expected_linearization:Some(token),..Default::default() },&context),Err(pse_kernels::ProviderError::LinearizationChanged)));
    assert!(buffer.fill(&mut provider,&changed,&second,ProviderRuntime { weights:&weights,directions:&directions,..Default::default() },&context).is_ok());
}

#[test]
fn implicit_factor_first_upgrade_affine_use_and_failure_preserve_exact_associations() {
    let (factory,parameters,_)=coupled(2,128);let scope=ExecutionScope::new(factory.cancel.clone(),None);
    let problem=factory.problem(scope).unwrap();let expected=analytic(&parameters);let options=match &factory.configuration { Configuration::Fixed(_,options)=>options.clone(),_=>unreachable!() };
    let cancel=factory.cancel.flag();
    let hub=pse_diagnostics::observation::ObservationHub::new(Default::default());let operation=hub.start("IFT association controls");let observation=operation.context();let _entered=observation.enter();
    problem.derivatives(&parameters,&expected.values,DerivativeOrder::First,&options,&cancel).unwrap();
    let worker_before={ let worker=problem.worker.lock().unwrap();(worker.evaluation.values.as_ptr(),worker.evaluation.first.as_ptr()) };
    problem.derivatives(&parameters,&expected.values,DerivativeOrder::First,&options,&cancel).unwrap();
    assert_eq!(count(&observation,"implicit.factor.numeric_reuse"),1);assert_eq!(count(&observation,"implicit.derivatives.first_reuse"),1);
    assert_eq!({ let worker=problem.worker.lock().unwrap();(worker.evaluation.values.as_ptr(),worker.evaluation.first.as_ptr()) },worker_before,"completed residual buffers remain borrowed and retained");
    problem.derivatives(&parameters,&expected.values,DerivativeOrder::Second,&options,&cancel).unwrap();
    assert_eq!(count(&observation,"implicit.factor.numeric_reuse"),2);assert_eq!(count(&observation,"implicit.derivatives.first_reuse"),2,"Second upgrade consumes the identical cached first solve");
    let mut solver=factory.solver.worker();solver.solve(problem.clone(),&parameters,&options,&factory.cancel).unwrap();
    assert!(problem.factor.lock().unwrap().association.is_none(),"affine workspace use invalidates IFT identity");
    problem.derivatives(&parameters,&expected.values,DerivativeOrder::First,&options,&cancel).unwrap();
    assert_eq!(count(&observation,"implicit.factor.numeric_reuse"),2,"same coefficients alone cannot reuse the affine numeric workspace");
    let mut changed_options=options.clone();changed_options.derivative_tolerance*=0.5;
    problem.derivatives(&parameters,&expected.values,DerivativeOrder::First,&changed_options,&cancel).unwrap();
    assert_eq!(count(&observation,"implicit.factor.numeric_reuse"),2,"actual resolved configuration participates in the association");
    factory.cancel.cancel();assert!(problem.derivatives(&parameters,&expected.values,DerivativeOrder::First,&options,&cancel).is_err());
    assert!(problem.factor.lock().unwrap().association.is_none());cancel.store(false,Ordering::Release);
    close(&problem.derivatives(&parameters,&expected.values,DerivativeOrder::Second,&options,&cancel).unwrap().hessians,&expected.hessians);
}

#[test]
fn implicit_preparation_retains_finite_demands_and_current_admission_owner() {
    #[derive(Debug)] struct Admission(AtomicUsize);
    impl crate::construction::ConstructionAdmission for Admission { fn try_grow(&self,bytes:usize)->Result<(),MathError>{ self.0.fetch_add(bytes,Ordering::Relaxed);Ok(()) } }
    let (factory,parameters,_)=coupled(12,64);let admission=Arc::new(Admission(AtomicUsize::new(0)));
    let mut provider=crate::construction::scoped(admission.clone(),||factory.create().unwrap());
    let first=ProviderRequest { outputs:vec![0],order:DerivativeOrder::First,demand:ProviderDemand::SelectedColumns { columns:vec![0] }};
    let second=ProviderRequest { outputs:vec![1],order:DerivativeOrder::First,demand:ProviderDemand::SelectedColumns { columns:vec![1] }};
    let prepared=provider.prepare(first.clone()).unwrap();let one=admission.0.load(Ordering::Relaxed);assert!(one>0,"preparation reinstalls the current attempt's owner");
    provider.prepare(second).unwrap();let two=admission.0.load(Ordering::Relaxed);assert!(two>one);
    provider.prepare(first).unwrap();assert_eq!(admission.0.load(Ordering::Relaxed),two,"alternating demands reuse retained programs");
    let flag=factory.cancel.flag();let context=EvaluationContext { cancelled:flag.as_ref(),max_result_bytes:1<<20 };
    let mut buffer=ProviderBuffer::new(&prepared);buffer.fill(&mut provider,&parameters,&prepared,Default::default(),&context).unwrap();
    let after=admission.0.load(Ordering::Relaxed);
    buffer.fill(&mut provider,&parameters,&prepared,Default::default(),&context).unwrap();assert_eq!(admission.0.load(Ordering::Relaxed),after);
}

#[derive(Debug,Clone)]
struct MovingResidual {
    spec:ProviderSpec, coefficient:Arc<AtomicU64>, calls:Arc<AtomicUsize>, qualified:bool,
}
impl ProviderFactory for MovingResidual {
    fn spec(&self)->&ProviderSpec { &self.spec }
    fn create(&self)->Result<pse_kernels::ProviderBinding,pse_kernels::ProviderError> {
        Ok(pse_kernels::ProviderBinding::new(self.spec.clone(),Box::new(self.clone())))
    }
}
impl pse_kernels::Provider for MovingResidual {
    fn supports(&self,_:&ProviderDemand)->bool { true }
    fn supports_linearization(&self)->bool { self.qualified }
    fn completed_linearization(&self)->Option<ContentHash> {
        let mut h=pse_ids::FramedHasher::new(pse_ids::Frame::DerivedBindingV1);h.str("moving residual coefficient").u64(self.coefficient.load(Ordering::Relaxed));Some(h.finish_hash())
    }
    fn fill(&mut self,inputs:&[f64],request:&pse_kernels::PreparedProviderRequest,runtime:ProviderRuntime<'_>,output:pse_kernels::ProviderOutput<'_>,_:&EvaluationContext<'_>)->Result<(),pse_kernels::ProviderError> {
        self.calls.fetch_add(1,Ordering::Relaxed);let c=f64::from_bits(self.coefficient.load(Ordering::Relaxed));
        output.values.fill(c*inputs[0]);output.hessians.fill(0.);
        match &request.demand {
            ProviderDemand::CompleteJets | ProviderDemand::SelectedColumns { .. } => output.jacobian.fill(c),
            ProviderDemand::JacobianAction { directions } => for d in 0..*directions { output.jacobian[d]=c*runtime.directions[d]; },
            ProviderDemand::TransposeAction { cotangents } => for d in 0..*cotangents { output.jacobian[d]=c*runtime.cotangents[d]; },
            ProviderDemand::WeightedSecondAction { .. } => output.jacobian.fill(c*runtime.weights[0]),
        }
        Ok(())
    }
}
fn moving_residual(qualified:bool)->(Factory,MovingResidual) {
    let (mut factory,_,_)=coupled(2,128);factory.spec.inputs.truncate(1);factory.spec.outputs.truncate(1);factory.unknowns.truncate(1);factory.rows.truncate(1);
    factory.configuration=Configuration::Fixed(factory.unknowns.clone(),Options { start:vec![0.],variable_nominals:vec![1.],variable_tolerance:vec![1e-10],residual_tolerance:vec![1e-10],iterations:10,time_limit:Duration::from_secs(10),derivative_tolerance:1e-11 });
    let mut spec=factory.spec.clone();spec.id=pse_ids::named_id(spec.id,"moving child");
    let child=MovingResidual { spec:spec.clone(),coefficient:Arc::new(AtomicU64::new(1_f64.to_bits())),calls:Arc::new(AtomicUsize::new(0)),qualified };
    factory.providers.insert(spec.key(),pse_kernels::Registration::new(Arc::new(child.clone()),&pse_quantity::standard::standard_registry().unwrap()).unwrap());
    let body=PreparedBody::new(2,4,vec![3],vec![Stage::Provider { spec,partial:vec![],inputs:vec![1],outputs:vec![2],source:factory.spec.id },Stage::Block { expressions:vec![crate::library::formal(0).unwrap()+crate::library::formal(2).unwrap()],outputs:vec![3],source:factory.spec.id }],DerivativeOrder::Second).unwrap();
    factory.solver=Arc::new(Affine::new(&body,1).unwrap());
    factory.bodies=residual_bodies_for_test(Arc::new(body.compile(&[0],&[0,1],DerivativeOrder::Second,Optimization::default(),EvaluationLimits::default(),&ExecutionScope::new(factory.cancel.clone(),None)).unwrap()));
    (factory,child)
}
#[test]
fn implicit_child_premises_invalidate_same_point_matrix_and_fixed_numerical_products() {
    let (factory,child)=moving_residual(true);let mut provider=factory.create().unwrap();let flag=factory.cancel.flag();let context=EvaluationContext { cancelled:flag.as_ref(),max_result_bytes:1<<20 };
    let request=provider.prepare(ProviderRequest::all(&factory.spec,DerivativeOrder::First)).unwrap();let mut buffer=ProviderBuffer::new(&request);
    let actual=buffer.fill(&mut provider,&[0.],&request,Default::default(),&context).unwrap();close(actual.values,&[0.]);close(actual.jacobian,&[-1.]);let token=actual.linearization.unwrap();let before=child.calls.load(Ordering::Relaxed);
    buffer.fill(&mut provider,&[0.],&request,Default::default(),&context).unwrap();assert!(child.calls.load(Ordering::Relaxed)>before,"fixed point/full numerical cache must revisit the actual residual child");
    child.coefficient.store(3_f64.to_bits(),Ordering::Relaxed);
    assert!(matches!(buffer.fill(&mut provider,&[0.],&request,ProviderRuntime { expected_linearization:Some(token),..Default::default() },&context),Err(pse_kernels::ProviderError::LinearizationChanged)));
    let actual=buffer.fill(&mut provider,&[0.],&request,Default::default(),&context).unwrap();close(actual.values,&[0.]);close(actual.jacobian,&[-3.]);assert_ne!(actual.linearization,Some(token),"B changed while root, inputs, config and A were identical");
}
#[test]
fn implicit_unknown_child_actions_refuse_before_any_provider_effect() {
    let (factory,child)=moving_residual(false);let mut provider=factory.create().unwrap();let before=child.calls.load(Ordering::Relaxed);
    for demand in [ProviderDemand::JacobianAction { directions:1 },ProviderDemand::TransposeAction { cotangents:1 },ProviderDemand::WeightedSecondAction { directions:1 }] {
        let order=if matches!(demand,ProviderDemand::WeightedSecondAction { .. }) { DerivativeOrder::Second } else { DerivativeOrder::First };
        assert!(provider.prepare(ProviderRequest { outputs:vec![0],order,demand }).is_err());
    }
    assert_eq!(child.calls.load(Ordering::Relaxed),before);
}
