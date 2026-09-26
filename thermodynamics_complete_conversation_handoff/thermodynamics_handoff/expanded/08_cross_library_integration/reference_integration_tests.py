"""Authored integration-reference checks, NOT a simulator or thermodynamics adapter.

Run: python reference_integration_tests.py
Uses only the standard library. Numerical examples are synthetic. Sequential guards
are sketches of contract logic, not proofs of native safety or concurrent atomicity.
"""
from __future__ import annotations
from dataclasses import dataclass
from fractions import Fraction as Q
from pathlib import Path
import json
import math

checks: list[dict[str, object]]=[]
def check(name: str, actual: object, expected: object, explanation: str, *, atol: float=0.0) -> None:
    ok = abs(float(actual)-float(expected)) <= atol if atol else actual == expected
    checks.append({'id':f'RC8-{len(checks)+1:02d}','name':name,'actual':str(actual),
                   'expected':str(expected),'tolerance':atol,'passed':ok,'scope':explanation})

# Condensed implicit block: g(u,y)=y*y-u=0, y>0; F(u)=y**3+u.
u=Q(4); y=Q(2); dy=Q(1)/(2*y)
check('Implicit state response',dy,Q(1,4),'Local exact algebra on the specified positive branch.')
check('Reduced derivative includes inner response',1+3*y*y*dy,Q(4),'Frozen-y partial derivative is only 1.')
check('Reduced second derivative',Q(3)/(4*y),Q(3,8),'For F(u)=u**1.5+u on u>0.')
check('Frozen-y derivative rejected',(Q(1)==Q(4)),False,'A partial derivative is not the solved response.')
f=lambda v: v**1.5+v
h=1e-4
check('Reduced derivative matches full re-solve central perturbation',(f(4+h)-f(4-h))/(2*h),4.0,
      'Synthetic smooth expression; this does not test a library.',atol=1e-8)
# Local first-order error amplification; sign depends on whether residual is correction/error.
check('Linearized outer sensitivity to inner residual',3*y*y/(2*y),Q(3),'Magnitude only, a local first-order estimate, not a universal nonlinear bound.')
check('Inner error magnification example',Q(3)*Q(1,10**6),Q(3,10**6),'Local linearized illustration with authored residual scale.')
# Central derivative error from two bounded value errors: <= epsilon/h.
check('Noise amplification for tiny perturbation',Q(1,10**8)/Q(1,10**10),Q(100),'Worst-case central difference contribution if each value error is bounded by epsilon.')
# Nonlinear coordinate Hessian: v=u², f(v)=v², F(u)=u⁴ at u=2.
u=Q(2); v=u*u; j=2*u
first_term=j*j*2; extra=2*v*2
check('Hessian chart first term alone',first_term,Q(32),'J transpose H J is incomplete for nonlinear charts.')
check('Hessian chart curvature contribution',extra,Q(16),'Gradient times second derivative of coordinate transform.')
check('Full nonlinear chart Hessian',first_term+extra,12*u*u,'Exact second chain rule, 48 not 32.')
# Residual r=3*x, x=100*xhat, rhat=r/10.
check('Scaled Jacobian',Q(1,10)*3*100,Q(30),'Physical and scaled residuals remain distinct.')
check('Zero at a point is not structural independence',((0,0)==(0,0) and (2,3)!=(0,0)),True,
      'For r=x*y gradient=(y,x), the gradient at zero does not establish zero structure.')
# Complete caloric inverse: h_partial=2(T-300), missing=3(T-300).
t_partial=Q(300)+Q(50,2)
t_full=Q(300)+Q(50,5)
check('Partial caloric solve temperature',t_partial,Q(325),'Authored model, no real-fluid data.')
check('Completed enthalpy at wrong partial solution',5*(t_partial-300),Q(125),'Post hoc completion does not meet total target 50.')
check('Total caloric inverse temperature',t_full,Q(310),'Inverse evaluated with complete function.')
check('Original complete target satisfied',5*(t_full-300),Q(50),'Complete function used in original-target check.')
# A sign-changing discontinuity has no zero.
h_step=lambda t: 0 if t<300 else 100
check('Discontinuous bracket has opposite residual signs',(h_step(299)-50)*(h_step(301)-50)<0,True,'A bracket sign test alone is insufficient without continuity.')
check('Discontinuous endpoint has nonzero target residual',abs(h_step(300)-50),50,'No enthalpy 50 exists in this artificial TP step function.')
check('Explicit phase fraction resolves synthetic endpoint mixture',Q(1,2)*100+Q(1,2)*0,Q(50),'Linear phase mixing with declared endpoints, not a TP-only unique allocation.')
# Same parameter value, different temperature derivative.
T=Q(300)
check('Parameter match at one temperature',Q(2)-300/T,Q(1),'tau_a=1 and tau_b=2-300/T agree at this point.')
check('Parameter slope differs',300/(T*T),Q(1,300),'tau_a derivative is zero, so equality at a point does not establish equivalence.')
# Reference shifts and reaction energy.
check('Equimolar reference shift',Q(1,2)*10+Q(1,2)*40,Q(25),'Declared species molar reference offsets.')
check('Unequal-composition reference shift',Q(3,4)*10+Q(1,4)*40,Q(35,2),'Reference shift is composition dependent.')
delta_ref=(40-10)*Q(2,5)
check('Reaction reference contribution',delta_ref,Q(12),'Synthetic balanced A to B with extent 0.4.')
check('Compensating reaction correction preserves accounting',Q(100)+delta_ref-Q(12),Q(100),'If energy reference adds +12, corresponding separate correction changes by -12.')
check('Conserved-basis chemical gauge leaves reaction affinity unchanged',-10+10,0,'Equal shifts for A and B with one conserved moiety.')
check('Arbitrary species offset changes reaction affinity',-10+40,30,'An arbitrary c is not necessarily a gauge transformation.')
check('Model discrepancy after known offset',Q(6,5)*50-50,Q(10),'Package B slope differs after removing its stated 100 offset.')
check('Temperature for corrected enthalpy match',300+Q(50)/Q(6,5),Q(1025,3),'Choosing PH instead of TP preservation changes target temperature.')
# Coordinate permutation must transform both axes.
J=[[1,2],[3,4]]; order=[1,0]
Jmapped=[[J[i][j] for j in order] for i in order]
check('Both Jacobian axes permuted',Jmapped,[[4,3],[2,1]],'A permutation must map inputs and outputs according to their coordinate definitions.')
# Duplicate account example: 2 disjoint phases and one aggregate must not sum all 3.
check('Disjoint phase material count',6+4,10,'Aggregate is a view, not a third additive phase.')
check('Lossy map lacks unique inverse',sum([1,3])==sum([2,2]),True,'Same lump can correspond to distinct detailed compositions.')

@dataclass(frozen=True)
class Profile:
    operation: str
    phases: frozenset[str]
    caloric_complete: bool
    derivative_kind: str

def eligible(p: Profile, operation: str, phases: frozenset[str], derivative_kind: str='none') -> bool:
    return (p.operation==operation and phases<=p.phases and
            (operation not in {'PH','PS','UV'} or p.caloric_complete) and
            (derivative_kind=='none' or p.derivative_kind==derivative_kind))
p=Profile('PH',frozenset({'V','L1'}),True,'local')
check('Required second liquid blocked',eligible(p,'PH',frozenset({'V','L1','L2'})),False,'Reference predicate, not a library capability test.')
check('Local derivative does not qualify total response',eligible(p,'PH',frozenset({'V','L1'}),'total'),False,'Semantic derivative demand is exact, not any derivative flag.')
check('Value request can use same eligible profile',eligible(p,'PH',frozenset({'V','L1'})),True,'Derivative-free use is distinct from differentiated use.')
check('Partial caloric PH is blocked',eligible(Profile('PH',p.phases,False,'total'),'PH',p.phases),False,'Caloric prerequisite checked before dispatch.')

def reuse_session(active: bool, health: str, context_match: bool) -> bool:
    return not active and health=='qualified_usable' and context_match
check('Running session not reused after logical cancellation',reuse_session(True,'qualified_usable',True),False,'Sequential lifetime guard only; not native interruption test.')
check('Quarantined session not reused',reuse_session(False,'quarantined',True),False,'Sequential lifecycle guard only.')

def publish(*,context:bool,run:bool,parent:bool,qualified:bool,coherent:bool) -> bool:
    return all((context,run,parent,qualified,coherent))
check('Same revision with revoked parent blocks publication',publish(context=True,run=True,parent=False,qualified=True,coherent=True),False,'Sequential Boolean guard, not concurrent implementation proof.')
check('Same revision with stale iterative input blocks publication',publish(context=True,run=True,parent=True,qualified=True,coherent=False),False,'Input-lineage coherence is distinct from base revision match.')
check('Fully matching synthetic publication eligibility',publish(context=True,run=True,parent=True,qualified=True,coherent=True),True,'Reference eligibility only; no persistence system executed.')
check('Initializer not restored cannot claim original model',(True and False),False,'Successful temporary solve plus failed restoration does not imply original-model acceptance.')

report={'document':'THERMO-INTEGRATION-008','kind':'authored_algebra_and_sequential_reference_contract_checks',
        'count':len(checks),'passed':sum(bool(c['passed']) for c in checks),'failed':sum(not c['passed'] for c in checks),
        'checks':checks,'limits':'No native thermodynamics, real-fluid validation, deployed solver, adapter, concurrency or crash containment is exercised.'}
path=Path(__file__).resolve().parent/'thermodynamics_cross_library_solver_integration_v0_1_reference_checks.json'
path.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k!='checks'},indent=2))
if report['failed']:
    for c in checks:
        if not c['passed']: print('FAILED',c)
    raise SystemExit(1)
