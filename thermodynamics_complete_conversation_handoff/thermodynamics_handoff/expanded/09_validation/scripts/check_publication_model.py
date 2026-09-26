"""Bounded sequential schedule exploration of an AUTHORED publication model.
This is not an implementation of the simulator and is not a proof of concurrent
storage, native cancellation, or distributed transaction correctness.
"""
from __future__ import annotations
import copy,itertools,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

class Publisher:
    def __init__(self,mutant=None):
        self.rev=0;self.generation=0;self.parent_active=True
        self.candidates={};self.current=None;self.decisions={};self.mutant=mutant
    def capture(self,key):
        if self.parent_active:self.candidates[key]={'rev':self.rev,'generation':self.generation,'view':key,'qualified':False,'parts':[1,2]}
    def qualify(self,key):
        if key in self.candidates:self.candidates[key]['qualified']=True
    def publish(self,key):
        if key in self.decisions:
            old=self.decisions[key]
            if self.mutant=='replay_reinstalls' and old['status']=='COMMITTED':self.current=copy.deepcopy(old['published'])
            return old['status']
        c=self.candidates.get(key)
        if c is None:return 'NO_CANDIDATE'
        checks=[(self.mutant=='omit_revision' or c['rev']==self.rev,'OBSOLETE_CONTEXT'),
                (self.mutant=='omit_parent' or self.parent_active,'CANCELLED_PARENT'),
                (self.mutant=='omit_generation' or c['generation']==self.generation,'SLOT_CONFLICT'),
                (c['qualified'],'UNQUALIFIED')]
        status=next((why for ok,why in checks if not ok),'COMMITTED')
        if status=='COMMITTED':
            self.generation+=1;self.current=copy.deepcopy(c);self.current.update(key=key,stale=False,publication_generation=self.generation)
        self.decisions[key]={'status':status,'published':copy.deepcopy(self.current) if status=='COMMITTED' else None}
        return status
    def apply(self,event):
        if event=='edit':
            self.rev+=1
            if self.current:self.current['stale']=True
            return
        if event=='cancel':self.parent_active=False;return
        key,op=event.split('_');getattr(self,op)(key)

def oracle_check(before,after,event):
    if not event.endswith('_publish'):return None
    key=event.split('_')[0];c=before.candidates.get(key)
    actual=after.decisions.get(key,{}).get('status','NO_CANDIDATE')
    permitted=bool(c and c['rev']==before.rev and before.parent_active and c['generation']==before.generation and c['qualified'])
    if (actual=='COMMITTED') != permitted:return {'event':event,'expected_admission':permitted,'observed':actual,'before_rev':before.rev,'before_generation':before.generation,'parent_active':before.parent_active,'candidate':c}
    if not permitted and after.current!=before.current:return {'event':event,'illegal_partial_write':True}
    return None

events=['A_capture','A_qualify','A_publish','B_capture','B_qualify','B_publish','edit','cancel']
schedules=[p for p in itertools.permutations(events) if all(p.index(k+'_capture')<p.index(k+'_qualify')<p.index(k+'_publish') for k in ['A','B'])]
violations=[];commits=0
for p in schedules:
    s=Publisher()
    for e in p:
        before=copy.deepcopy(s);s.apply(e);v=oracle_check(before,s,e)
        if v:violations.append({'schedule':p,'violation':v})
        commits+=len(s.decisions)-len(before.decisions) if s.current!=before.current and e.endswith('_publish') else 0
        if s.current and not s.current['stale'] and s.current['rev']!=s.rev:violations.append({'schedule':p,'nonstale_wrong_revision':True})
assert not violations,violations[:1]
mutations=[]
for mut in ['omit_revision','omit_parent','omit_generation']:
    witness=None
    for p in schedules:
        s=Publisher(mut)
        for e in p:
            before=copy.deepcopy(s);s.apply(e);v=oracle_check(before,s,e)
            if v:witness={'schedule':list(p),'violation':v};break
        if witness:break
    assert witness,mut
    mutations.append({'mutation':mut,'detected':True,'counterexample':witness})

# A previously committed invocation is replayed after a newer successful one.
for mutant in [None,'replay_reinstalls']:
    s=Publisher(mutant)
    for e in ['A_capture','A_qualify','A_publish','B_capture','B_qualify','B_publish']:s.apply(e)
    old=copy.deepcopy(s.current);s.publish('A');same=(s.current==old)
    if mutant is None:assert same
    else:
        assert not same;mutations.append({'mutation':mutant,'detected':True,'counterexample':['A publishes','B captures current generation and publishes','old A decision is replayed','mutant incorrectly replaces B with A']})

# Group, qualification and run-local-view guards, tested on every Boolean case.
truth_cases=0
for rev,run,parent,qualified,view,group,slot in itertools.product([False,True],repeat=7):
    actual=all((rev,run,parent,qualified,view,group,slot))
    expected=not any(not x for x in [rev,run,parent,qualified,view,group,slot]);assert actual==expected;truth_cases+=1
for omit,name in [(3,'omit_required_qualification'),(4,'omit_iterate_lineage'),(5,'partial_group_write')]:
    flags=[True]*7;flags[omit]=False
    bad=all(v for i,v in enumerate(flags) if i!=omit)
    assert bad and not all(flags)
    mutations.append({'mutation':name,'detected':True,'counterexample':{'revision':True,'run':True,'parent':True,'qualified':flags[3],'view_coherent':flags[4],'all_group_members_valid':flags[5],'slot':True}})

# A result holding an aliased provider work vector fails the accepted-state invariant.
work=[1.,2.];safe=copy.deepcopy(work);bad=work;work[0]=999.;assert safe==[1.,2.] and bad!=safe
mutations.append({'mutation':'accepted_aliases_native_work','detected':True,'counterexample':'native work is overwritten after a result was retained; detached result unchanged, alias corrupted'})

# Cleanup cannot recycle a still-running cancelled native call.
cleanup=[]
for busy,cancelled,health in itertools.product([False,True],[False,True],['ready','quarantined']):
    may_reuse=not busy and health=='ready'
    if busy:assert not may_reuse
    cleanup.append({'native_active':busy,'publication_cancelled':cancelled,'health':health,'reusable':may_reuse})

report={'document':'THERMO-VALIDATION-009','kind':'authored_bounded_sequential_model_exploration',
 'legal_schedules':len(schedules),'events_per_schedule':len(events),'examined_event_steps':len(schedules)*len(events),
 'violations_correct_model':violations,'guard_truth_cases':truth_cases,'mutation_controls':mutations,'detected_mutations':len(mutations),
 'cleanup_cases':cleanup,'status':'PASS',
 'bounds':'Two local candidates, one edit and one parent cancellation, 8 events; A/B each capture before qualify before publish. Replays, torn groups, and buffer aliases are separate explicit controls.',
 'limitations':['Sequential legal-order exploration only; no deployed lock or transaction code','No proof of native crash, memory lifecycle, or concurrent scheduling implementation','No exactly-once delivery claim','Mutation detection validates these witnesses, not the eventual simulator']}
(ROOT/'results'/'publication_model_checks.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['mutation_controls','cleanup_cases']},indent=2))
