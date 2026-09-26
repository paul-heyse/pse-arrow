"""Bounded source-extract and authored-formulation tests, NOT full library tests.

Run: python scripts/run_numerical_probes.py
Fixtures and tolerances are read before calculations. All failures are retained.
The ideal VLE and reaction fixtures are invented test materials, not real-fluid
predictions. The LLE solver and numerical inverse machinery are authored here.
"""
from __future__ import annotations
import copy, hashlib, itertools, json, math, platform, sys, traceback
from pathlib import Path
import numpy as np
import scipy
from scipy.optimize import brentq, root
from extracted_chedl_kernels import NRTL_gammas, Rachford_Rice_flash_error
ROOT=Path(__file__).resolve().parents[1]
FX=json.loads((ROOT/'fixtures.json').read_text()); TOL=FX['tolerances']
RECORDS=[]

def run(test_id, name, evidence, fun):
    try:
        detail=fun()
        RECORDS.append(dict(id=test_id,name=name,evidence_kind=evidence,status='PASS',detail=detail))
    except Exception as exc:
        RECORDS.append(dict(id=test_id,name=name,evidence_kind=evidence,status='FAIL',error=repr(exc),traceback=traceback.format_exc()))

def close(a,b,atol=1e-10,rtol=0):
    a,b=np.asarray(a,dtype=float),np.asarray(b,dtype=float)
    if not np.all(np.isfinite(a)) or not np.allclose(a,b,atol=atol,rtol=rtol):
        raise AssertionError(f'{a} != {b}; abs={np.max(np.abs(a-b))}; atol={atol}, rtol={rtol}')

def composition(z):
    a=np.asarray(z,dtype=float)
    if a.ndim!=1 or len(a)==0 or not np.all(np.isfinite(a)) or np.any(a<0) or abs(float(a.sum())-1)>TOL['composition_abs']:
        raise ValueError('INVALID_COMPOSITION: no implicit normalization')
    return a

def split(z,K):
    z=composition(z);K=np.asarray(K,dtype=float)
    if K.shape!=z.shape or np.any(K<=0) or not np.all(np.isfinite(K)):raise ValueError('INVALID_K_VALUES')
    active=z>0
    if np.all(K[active]==1):raise ValueError('UNDER_SPECIFIED_ALLOCATION: active K_i all equal one')
    f=lambda b:Rachford_Rice_flash_error(b,z.tolist(),K.tolist())
    if f(0)<=0:
        return dict(beta=0.,x=z.copy(),y=None,phase='liquid',residual=None)
    if f(1)>=0:
        return dict(beta=1.,x=None,y=z.copy(),phase='vapor',residual=None)
    beta=brentq(f,0,1,xtol=1e-14,rtol=1e-14)
    x=z/(1+beta*(K-1));y=K*x
    close((1-beta)*x+beta*y,z,atol=TOL['composition_abs'])
    close([sum(x),sum(y)],[1,1],atol=TOL['composition_abs'])
    return dict(beta=float(beta),x=x,y=y,phase='VL',residual=float(f(beta)))

MODEL=FX['synthetic_ideal_VLE']; R=MODEL['R_J_per_mol_K']; TR=MODEL['Tref_K'];P0=MODEL['P0_Pa']
CP=np.array(MODEL['cpL_equals_cpV_J_per_mol_K'],float);LH=np.array(MODEL['latent_J_per_mol'],float);DS=LH/np.array(MODEL['Tb_at_P0_K'],float)
def mix_entropy(x):
    return -R*sum(float(t)*math.log(float(t)) for t in x if t>0)
def phase_hs(T,P,x,phase):
    h=float(np.dot(x,CP))*(T-TR)
    s=float(np.dot(x,CP))*math.log(T/TR)+mix_entropy(x)
    if phase=='vapor':h+=float(np.dot(x,LH));s+=float(np.dot(x,DS))-R*math.log(P/P0)
    return h,s

def state(T,P,z=None):
    if T<=0 or P<=0:raise ValueError('INVALID_STATE')
    if z is None:z=MODEL['zs']
    z=composition(z);K=np.exp(DS/R-LH/(R*T))*P0/P
    out=split(z,K);b=out['beta'];H=S=0.
    if b<1:
        h,s=phase_hs(T,P,out['x'],'liquid');H+=(1-b)*h;S+=(1-b)*s
    if b>0:
        h,s=phase_hs(T,P,out['y'],'vapor');H+=b*h;S+=b*s
    out.update(T=float(T),P=float(P),z=z,K=K,H=H,S=S)
    return out

def inverse(P,target,property_name='H',z=None):
    lo,hi=MODEL['inverse_bounds_K']
    f=lambda T:state(T,P,z)[property_name]-target
    if f(lo)*f(hi)>0:raise ValueError('NO_BRACKET_IN_DECLARED_BOUNDS; not proof of global infeasibility')
    T=brentq(f,lo,hi,xtol=1e-10,rtol=1e-13)
    out=state(T,P,z)
    tol=TOL['energy_abs_J_per_mol'] if property_name=='H' else TOL['entropy_abs_J_per_mol_K']
    close(out[property_name],target,atol=tol)
    return out

def total_h_derivative(s):
    T,P,z,K,b=s['T'],s['P'],s['z'],s['K'],s['beta']
    if s['phase']!='VL':return float(np.dot(z,CP))
    d=1+b*(K-1);kp=K*LH/(R*T*T)
    f_beta=-float(np.sum(z*(K-1)**2/d**2));f_T=float(np.sum(z*kp/d**2))
    bp=-f_T/f_beta;xp=-z*(bp*(K-1)+b*kp)/d**2;yp=kp*s['x']+K*xp
    return float(np.dot(z,CP)+bp*np.dot(s['y'],LH)+b*np.dot(yp,LH))

def simplified(s):
    return {k:(v.tolist() if isinstance(v,np.ndarray) else v) for k,v in s.items()}

KIND='source_extracted_function_not_installed_package'
AUTH='authored_formulation_using_extracted_kernel_and_SciPy_not_full_provider'

def t01():
    a=FX['source_examples']['nrtl'];g=NRTL_gammas(a['xs'],a['taus'],a['alphas']);close(g,a['gammas'],atol=TOL['source_regression_abs']);return {'obtained':g,'reference':a['gammas'],'reference_kind':'upstream worked example, not independent experiment'}
run('NP-01','NRTL published numerical example',KIND,t01)
def t02():
    a=FX['source_examples']['rr'];v=Rachford_Rice_flash_error(.5,a['zs'],a['Ks']);close(v,a['residual_at_half'],atol=TOL['source_regression_abs']);return {'obtained':v,'reference':a['residual_at_half']}
run('NP-02','Rachford-Rice residual example',KIND,t02)
def t03():
    a=FX['source_examples']['rr'];s=split(a['zs'],a['Ks']);close(s['beta'],a['beta'],atol=TOL['source_regression_abs']);return simplified(s)
run('NP-03','Host Brent root of extracted residual',AUTH,t03)
def t04():
    z=[.2,.3,.5];tau=np.array([[0,1,-.5],[2,0,.4],[.7,-.2,0]]);alpha=np.full((3,3),.3);np.fill_diagonal(alpha,0)
    g=np.array(NRTL_gammas(z,tau,alpha));errs=[]
    for p in itertools.permutations(range(3)):
        p=list(p);v=NRTL_gammas(np.array(z)[p],tau[np.ix_(p,p)],alpha[np.ix_(p,p)])
        close(v,g[p],atol=1e-12);errs.append(float(max(abs(np.array(v)-g[p]))))
    return {'permutations':6,'max_error':max(errs)}
run('NP-04','NRTL all-axis constituent permutation',KIND,t04)
def t05():
    a=FX['source_examples']['rr'];ref=split(a['zs'],a['Ks']);errors=[]
    for p in itertools.permutations(range(3)):
        p=list(p);s=split(np.array(a['zs'])[p],np.array(a['Ks'])[p]);close(s['beta'],ref['beta']);close(s['x'],ref['x'][p]);errors.append(abs(s['beta']-ref['beta']))
    return {'permutations':6,'max_beta_error':max(errors)}
run('NP-05','Flash component permutation invariance',AUTH,t05)
def t06():
    g=NRTL_gammas([.2,.8],[[0,0],[0,0]],[[0,.7],[.1,0]]);close(g,[1,1]);a=FX['source_examples']['nrtl'];pure=NRTL_gammas([1.,0.],a['taus'],a['alphas']);close(pure[0],1);return {'ideal':g,'pure_present_component_gamma':pure[0]}
run('NP-06','NRTL ideal and pure-component limits',KIND,t06)
def t07():
    a=FX['source_examples']['nrtl'];x=.252;dt=1e-5
    gm=np.log(NRTL_gammas([x-dt,1-x+dt],a['taus'],a['alphas']));gp=np.log(NRTL_gammas([x+dt,1-x-dt],a['taus'],a['alphas']))
    res=float(np.dot([x,1-x],(gp-gm)/(2*dt)));close(res,0,atol=TOL['gibbs_duhem_abs']);return {'Gibbs_Duhem_tangent_residual':res,'step':dt}
run('NP-07','NRTL Gibbs-Duhem composition tangent',KIND,t07)
def t08():
    a=FX['source_examples']['nrtl_caloric'];x=a['xs'];B=np.array(a['b_cal_per_mol'])*4.184/R;alpha=[[0,a['alpha']],[a['alpha'],0]]
    def GE(T):return R*T*float(np.dot(x,np.log(NRTL_gammas(x,B/T,alpha))))
    T=a['T_K'];dt=.001;g=GE(T);slope=(GE(T+dt)-GE(T-dt))/(2*dt);he=g-T*slope
    close(he,a['HE_J_per_mol'],atol=TOL['doc_excess_enthalpy_abs_J_per_mol']);return {'GE_J_per_mol':g,'HE_J_per_mol':he,'documented_HE':a['HE_J_per_mol'],'route':'authored GE=RT sum x ln gamma and centered temperature differentiation; not upstream HE method'}
run('NP-08','Temperature-dependent NRTL caloric identity',AUTH,t08)
def t09():
    bad=[.6,.6];a=FX['source_examples']['nrtl'];raw=NRTL_gammas(bad,a['taus'],a['alphas']);norm=NRTL_gammas([.5,.5],a['taus'],a['alphas']);close(raw,norm)
    try:composition(bad)
    except ValueError as e:return {'raw_extracted_function_returns':raw,'same_as_normalized':True,'host_rejects':str(e),'finding':'kernel is not the input-contract boundary'}
    raise AssertionError('host accepted non-normalized composition')
run('NP-09','Invalid composition is rejected at host boundary',AUTH,t09)
def t10():
    a=FX['source_examples']['nrtl'];wrong=NRTL_gammas(a['xs'],np.array(a['taus']).T,a['alphas']);delta=float(np.max(abs(np.array(wrong)-a['gammas'])));assert delta>1e-3;return {'wrong_order_gammas':wrong,'maximum_reference_error':delta,'expected_rejection':'ordered parameter roles differ'}
run('NP-10','Directed-parameter transpose negative control',KIND,t10)
def t11():
    outputs=[split([.5,.5],[.2,.4]),split([.5,.5],[2,3])];assert outputs[0]['y'] is None and outputs[1]['x'] is None;return [simplified(s) for s in outputs]
run('NP-11','Single-phase states do not fabricate absent composition',AUTH,t11)
def t12():
    try:split([.5,.5],[1,1])
    except ValueError as e:return {'outcome':str(e),'rr_residual_at_beta_0_and_1':[Rachford_Rice_flash_error(b,[.5,.5],[1,1]) for b in [0,1]]}
    raise AssertionError('all-K-one allocation falsely unique')
run('NP-12','Identical K values expose allocation degeneracy',AUTH,t12)
def t13():
    K=np.array([1+1e-8,1-1e-8]);b=.5;z=np.array([.5,.5]);f=Rachford_Rice_flash_error(b,z.tolist(),K.tolist());slope=-float(np.sum(z*(K-1)**2/(1+b*(K-1))**2));assert abs(f)<1e-8 and abs(slope)<1e-14
    return {'residual':f,'d_residual_d_beta':slope,'acceptance_lesson':'small residual alone does not establish accurate phase fraction or a well-conditioned solved sensitivity'}
run('NP-13','Near-degenerate allocation conditioning diagnostic',AUTH,t13)
def t14():
    a=FX['source_examples']['rr'];s=split(a['zs']+[0.],a['Ks']+[1000.]);close(s['beta'],a['beta']);close([s['x'][-1],s['y'][-1]],[0,0]);return simplified(s)
run('NP-14','Zero-feed component is preserved in fixed-species split',AUTH,t14)
def t15():
    arr=[]
    for T in [320.,340.,350.,360.,370.,380.,420.]:
        s=state(T,1e5);p=inverse(1e5,s['H']);q=inverse(1e5,s['S'],'S');close([p['T'],q['T']],[T,T],atol=TOL['temperature_abs_K']);arr.append({'T':T,'phase':s['phase'],'beta':s['beta'],'H':s['H'],'S':s['S'],'PH_T':p['T'],'PS_T':q['T']})
    return arr
run('NP-15','Synthetic coherent caloric TP to PH/PS roundtrips',AUTH,t15)
def t16():
    s=state(360,1e5);d=total_h_derivative(s);dt=.001;fd=(state(360+dt,1e5)['H']-state(360-dt,1e5)['H'])/(2*dt);frozen=float(np.dot(s['z'],CP));close(d,fd,rtol=TOL['derivative_relative']);assert abs(d-frozen)>100
    return {'total_equilibrium_dH_dT':d,'full_resolve_finite_difference':fd,'frozen_allocation_Cp':frozen,'step_K':dt}
run('NP-16','Condensed VLE derivative includes changing allocation',AUTH,t16)
def t17():
    T=360.;dt=.001;dS=(state(T+dt,1e5)['S']-state(T-dt,1e5)['S'])/(2*dt);dH=total_h_derivative(state(T,1e5));close(T*dS,dH,rtol=TOL['derivative_relative']);return {'dH_dT':dH,'T_dS_dT':T*dS,'reason':'internal consistency of authored ideal formulation, not experiment'}
run('NP-17','Coherent energy/entropy temperature response',AUTH,t17)
def t18():
    inlet=state(380,2e5);out=inverse(1e5,inlet['H']);close(out['H'],inlet['H'],atol=TOL['energy_abs_J_per_mol']);return {'inlet':simplified(inlet),'outlet':simplified(out),'H_residual':out['H']-inlet['H']}
run('NP-18','Synthetic valve process balance and outlet PH',AUTH,t18)
def t19():
    inp=state(420,1e5);ideal=inverse(3e5,inp['S'],'S');eta=.8;target=inp['H']+(ideal['H']-inp['H'])/eta;actual=inverse(3e5,target);assert actual['T']>ideal['T']>inp['T'];close((ideal['H']-inp['H'])/(actual['H']-inp['H']),eta)
    return {'inlet_T':inp['T'],'isentropic_reference_T':ideal['T'],'actual_T':actual['T'],'eta':eta,'shaft_J_per_mol':actual['H']-inp['H']}
run('NP-19','Compressor PS reference is distinct from actual PH outlet',AUTH,t19)
def t20():
    s=state(360,1e5);F=10.;nv=F*s['beta'];nl=F-nv;components=nv*s['y']+nl*s['x'];close(components,F*s['z']);hv,_=phase_hs(s['T'],s['P'],s['y'],'vapor');hl,_=phase_hs(s['T'],s['P'],s['x'],'liquid');err=nv*hv+nl*hl-F*s['H'];close(err,0,atol=1e-7);return {'liquid_mol_s':nl,'vapor_mol_s':nv,'component_sum_mol_s':components.tolist(),'energy_residual_W':err}
run('NP-20','Separator routing after shared equilibrium',AUTH,t20)
def t21():
    x=FX['synthetic_LLE'];tau=x['taus'];al=x['alphas']
    def mu_diff(a):
        g=NRTL_gammas([a,1-a],tau,al);return math.log(a*g[0]/((1-a)*g[1]))
    xa=brentq(mu_diff,1e-8,.49,xtol=1e-14);xb=1-xa;f=(xb-x['zA'])/(xb-xa);na=x['total_mol']*f;nb=x['total_mol']-na
    a=np.array([xa,1-xa]);b=np.array([xb,1-xb]);la=np.log(a*NRTL_gammas(a,tau,al));lb=np.log(b*NRTL_gammas(b,tau,al));close(la,lb,atol=TOL['equilibrium_residual_abs']);close(na*a+nb*b,x['total_mol']*np.array([x['zA'],1-x['zA']]))
    def gmix(v):return float(np.dot([v,1-v],np.log(np.array([v,1-v])*NRTL_gammas([v,1-v],tau,al))))
    splitg=f*gmix(xa)+(1-f)*gmix(xb);hom=gmix(x['zA']);assert splitg<hom
    sampled_margin=min(gmix(v)-float(np.dot([v,1-v],la)) for v in np.linspace(1e-7,1-1e-7,1001));assert sampled_margin>-1e-8
    return {'xA_liquid_1':xa,'xA_liquid_2':xb,'phase_moles':[na,nb],'log_activity_residual':float(max(abs(la-lb))),'split_G_over_RT':splitg,'homogeneous_G_over_RT':hom,'sampled_common_tangent_margin':sampled_margin,'stability_claim':'sampled binary search only; no global certificate and no real chemical data'}
run('NP-21','Synthetic two-liquid equilibrium using extracted NRTL',AUTH,t21)
def t22():
    inlet=state(420,1e5);Q=5000.;hotF=1.;hot=inverse(1e5,inlet['H']-Q/hotF);coldm=2.;coldcp=2000.;coldT=300+Q/(coldm*coldcp)
    err=hotF*(hot['H']-inlet['H'])+coldm*coldcp*(coldT-300);close(err,0,atol=1e-5)
    shift=12345.;shifted=hotF*((hot['H']+shift)-(inlet['H']+shift))+coldm*coldcp*(coldT-300);close(shifted,err,atol=1e-8)
    return {'hot_outlet_K':hot['T'],'cold_outlet_K':coldT,'Q_W':Q,'energy_residual_W':err,'reference_shifted_residual_W':shifted,'cold_material':'mass-only empirical fixture; no molecular weight'}
run('NP-22','Heat-only coupling to a mass-only empirical material',AUTH,t22)
def t23():
    a=FX['synthetic_reaction'];cp=a['cp_J_per_mol_K'];dh=a['delta_h0_J_per_mol'];ds=a['delta_s0_J_per_mol_K'];T0=a['feed_T_K']
    def xi(T):return 1/(1+math.exp((dh-T*ds)/(R*T)))
    f=lambda T:cp*(T-T0)+dh*xi(T)
    T=brentq(f,350,550,xtol=1e-11);extent=xi(T)
    def residual(v):
        temp,e=v;return [math.log(e/(1-e))+(dh-temp*ds)/(R*temp),(cp*(temp-T0)+dh*e)/20000]
    sim=root(residual,a['exposed_initial_guess']);assert sim.success;close(sim.x,[T,extent],atol=1e-7);close(residual(sim.x),[0,0],atol=1e-9)
    # Change the bookkeeping of enthalpy by c_i; subtract resulting c.delta_n explicitly.
    reference_c=np.array([10000,40000]);delta_n=np.array([-extent,extent]);offset=float(reference_c@delta_n)
    nout=np.array([1-extent,extent]); hout=np.array([cp*(T-TR),cp*(T-TR)+dh]); hin=cp*(T0-TR)
    shifted_raw=float(nout@(hout+reference_c))-(hin+reference_c[0]);shifted_balance=shifted_raw-offset
    close(shifted_balance,0,atol=1e-5);assert abs(shifted_raw)>1000
    return {'adiabatic_T_K':T,'extent_mol':extent,'exposed_initial_guess':a['exposed_initial_guess'],'exposed_root_solution':sim.x.tolist(),'uncorrected_shifted_energy_residual_J':shifted_raw,'corrected_shifted_energy_residual_J':shifted_balance,'residuals':residual(sim.x),'bookkeeping_correction_J':offset,'scope':'authored balanced isomerization, not an installed chemistry engine'}
run('NP-23','Condensed versus exposed reactive energy solve',AUTH,t23)
def t24():
    available=1.;extent=.4;amounts=[available-extent,extent];close(amounts,[.6,.4]);bad=1.2;assert bad>available
    return {'feasible_amounts':amounts,'strict_1_2_mol_outcome':'INFEASIBLE','clipped_alternative':'different problem, not counted as success'}
run('NP-24','Strict extent rejects a hidden limiting alternative',AUTH,t24)
def t25():
    Twrong=325.;partial=2*(Twrong-300);complete=5*(Twrong-300);right=brentq(lambda T:5*(T-300)-50,300,400);close(partial,50);close(complete,125);close(right,310)
    return {'partial_PH_temperature':Twrong,'full_H_at_wrong_T':complete,'correct_total_PH_temperature':right,'negative_control_rejected':abs(complete-50)>TOL['energy_abs_J_per_mol']}
run('NP-25','Caloric supplementation must enter inverse equations',AUTH,t25)
def t26():
    candidates=[0.,.25,.5,.75,1.];values=[Rachford_Rice_flash_error(b,[.5,.5],[1.,1.]) for b in candidates];close(values,[0]*5)
    jump=lambda T:0. if T<350 else 100.
    fake=brentq(lambda T:jump(T)-50,300,400,xtol=1e-10);res=jump(fake)-50;assert abs(res)==50
    return {'all_K1_residuals':values,'jump_root_solver_return_K':fake,'original_target_residual':res,'target_check_rejects':True}
run('NP-26','Saturation-like discontinuity defeats naive sign-bracket acceptance',AUTH,t26)
def t27():
    s=state(360,1e5);original=copy.deepcopy(s);h,sval=phase_hs(355,1e5,[.2,.8],'liquid');assert s['T']==original['T'];close(s['x'],original['x']);return {'supplied_separate_phase_H':h,'supplied_separate_phase_S':sval,'original_split_unchanged':True}
run('NP-27','Phase-property evaluation preserves separate bulk authority',AUTH,t27)
def t28():
    rng=np.random.default_rng(FX['seed']);maxerr=0.;n=0
    for _ in range(200):
        nn=5;x=rng.dirichlet(np.ones(nn));y=rng.dirichlet(np.ones(nn));beta=float(rng.uniform(.05,.95));z=(1-beta)*x+beta*y;K=y/x;out=split(z,K);close(out['beta'],beta,atol=1e-10);maxerr=max(maxerr,abs(out['beta']-beta));n+=1
    return {'generated_feasible_cases':n,'seed':FX['seed'],'maximum_beta_error':maxerr,'limitations':'constructed K values guarantee an interior algebraic root; not real-mixture validation'}
run('NP-28','Generated positive five-component allocation cases',AUTH,t28)
def t29():
    inlet1=state(340,1e5);inlet2=state(420,1e5);f1,f2=1.,2.;target=(f1*inlet1['H']+f2*inlet2['H'])/(f1+f2);mixed=inverse(1e5,target)
    fractions=[.3,.7];hflows=[a*(f1+f2)*mixed['H'] for a in fractions];close(sum(hflows),(f1+f2)*target,atol=1e-6)
    return {'mixed_T':mixed['T'],'incorrect_arithmetic_T':(f1*340+f2*420)/(f1+f2),'mixed_H':target,'split_enthalpy_flow_sum':sum(hflows)}
run('NP-29','Mixer energy balance and mechanical split',AUTH,t29)
def t30():
    # A simple material-recycle fixed point exercises parent convergence rather than thermodynamic scope.
    recycle=0.;fresh=10.;r=.8
    for it in range(1,301):
        new=r*(fresh+recycle)
        if abs(new-recycle)<1e-10:recycle=new;break
        recycle=new
    residual=recycle-r*(fresh+recycle);close(recycle,40,atol=1e-8);close(residual,0,atol=1e-9)
    # One local iteration is numerically valid but is not a parent fixed-point solution.
    first=r*fresh;assert abs(first-r*(fresh+first))>1
    return {'recycle_flow':recycle,'iterations':it,'parent_residual':residual,'first_local_result':first,'first_is_not_parent_converged':True}
run('NP-30','Parent recycle convergence differs from local calculation',AUTH,t30)

report={'document':'THERMO-VALIDATION-009','kind':'bounded_extracted_source_and_authored_numerical_probes','fixture_sha256':hashlib.sha256((ROOT/'fixtures.json').read_bytes()).hexdigest(),
        'runtime':{'python':sys.version,'platform':platform.platform(),'numpy':np.__version__,'scipy':scipy.__version__},
        'tests':RECORDS,'counts':{'tests':len(RECORDS),'passed':sum(r['status']=='PASS' for r in RECORDS),'failed':sum(r['status']=='FAIL' for r in RECORDS)},
        'limitations':['No complete thermo/chemicals import','No CoolProp/ThermoPack/Clapeyron/IDAES/FeOS/Reaktoro/DWSIM execution','Source function extraction is not a binary or complete-library test','Source example agreement is not independent experimental validation','Authored VLE, LLE, and reaction materials are synthetic','No global stability certification','No production approval or whole-scenario REV promotion']}
(ROOT/'results'/'numerical_probes.json').write_text(json.dumps(report,indent=2,allow_nan=False)+'\n')
print(json.dumps(report['counts']));
for r in RECORDS:print(r['id'],r['status'],r['name'],r.get('error',''))
sys.exit(1 if report['counts']['failed'] else 0)
