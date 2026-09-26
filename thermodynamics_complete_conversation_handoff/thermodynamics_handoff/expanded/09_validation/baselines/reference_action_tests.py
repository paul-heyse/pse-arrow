"""Executable sketches of selected THERMO-ACTIONS-007 guards and arithmetic.
Standard library only. Not a process simulator, provider adapter, native-failure
containment test, concurrent transaction test, or thermodynamic validation suite.
The deliberately conservative reference policy requires an exact model revision
and one coherent view token per publication group. Other verified dependency
policies can satisfy the specification without using these exact data structures.
"""
from __future__ import annotations
import copy
import hashlib
import io
import json
import sys
import unittest
from dataclasses import dataclass
from fractions import Fraction as F
from typing import Any
from pathlib import Path

@dataclass(frozen=True)
class Candidate:
    revision: int
    run: str
    view: str
    scope: str
    accepted: bool
    values: tuple[float, ...] = ()

class ReferenceAuthority:
    def __init__(self) -> None:
        self.revision = 1
        self.runs: dict[str, dict[str, Any]] = {}
        self.slots: dict[str, Candidate] = {}
        self.generations: dict[str, int] = {}
        self.receipts: dict[str, tuple[str, dict[str, Any]]] = {}
        self.amounts = {'bulk': F(2), 'surface': F(1)}
        self.transfers: set[str] = set()

    def start(self, name: str, parent: str | None = None) -> None:
        if name in self.runs: raise ValueError('run identity already used')
        if parent is not None and not self.active(parent): raise ValueError('parent revoked')
        self.runs[name] = {'revision': self.revision, 'status': 'active', 'parent': parent}

    def active(self, name: str) -> bool:
        seen: set[str] = set()
        while name is not None:
            if name in seen: return False
            seen.add(name)
            r = self.runs.get(name)
            if not r or r['status'] != 'active': return False
            name = r['parent']
        return True

    def cancel(self, name: str) -> None:
        if name not in self.runs: raise ValueError('unknown run')
        if self.runs[name]['status'] == 'active': self.runs[name]['status'] = 'cancelled'

    def _replay(self, command: str, payload: Any) -> tuple[str, dict[str, Any] | None]:
        fp = hashlib.sha256(json.dumps(payload, sort_keys=True, default=str).encode()).hexdigest()
        if command in self.receipts:
            prev, receipt = self.receipts[command]
            if prev != fp: return fp, {'status': 'COMMAND_PAYLOAD_CONFLICT'}
            return fp, copy.deepcopy(receipt)
        return fp, None

    def _finish(self, command: str, fp: str, receipt: dict[str, Any]) -> dict[str, Any]:
        self.receipts[command] = (fp, copy.deepcopy(receipt))
        return receipt

    def edit(self, command: str, expected: int, detail: str) -> dict[str, Any]:
        fp, receipt = self._replay(command, ('edit', expected, detail))
        if receipt is not None: return receipt
        if expected != self.revision:
            return self._finish(command, fp, {'status': 'STALE_CHANGE'})
        self.revision += 1
        # Historical immutable slot records retained; current reads compare revision.
        return self._finish(command, fp, {'status': 'COMMITTED', 'revision': self.revision})

    def publish(self, command: str, group: dict[str, Candidate], required: set[str],
                expected_slots: dict[str, int], scope: str) -> dict[str, Any]:
        payload = ('publish', {k: vars(v) for k,v in group.items()}, sorted(required), expected_slots, scope)
        fp, receipt = self._replay(command, payload)
        if receipt is not None: return receipt
        status = 'COMMITTED'
        if not group or set(group) != required or set(expected_slots) != required:
            status = 'GROUP_INCOMPLETE'
        elif any(not x.accepted for x in group.values()): status = 'UNQUALIFIED'
        elif any(x.scope != scope for x in group.values()): status = 'SCOPE_MISMATCH'
        elif len({(x.revision, x.run, x.view) for x in group.values()}) != 1: status = 'INCOHERENT_GROUP'
        elif any(x.revision != self.revision for x in group.values()): status = 'OBSOLETE_CONTEXT'
        elif any(not self.active(x.run) for x in group.values()): status = 'RUN_REVOKED'
        elif any(self.generations.get(k,0) != expected_slots[k] for k in required): status = 'SLOT_CONFLICT'
        if status == 'COMMITTED':
            # This is one logical operation in a sequential reference model.
            # It does not demonstrate atomicity in a real concurrent implementation.
            self.slots.update(group)
            for k in required: self.generations[k] = expected_slots[k]+1
        return self._finish(command, fp, {'status': status, 'revision': self.revision})

    def current(self, slot: str) -> Candidate | None:
        c = self.slots.get(slot)
        # A later cancellation does not invalidate an earlier committed result.
        return c if c is not None and c.revision == self.revision else None

    def transfer(self, command: str, expected: int, intent: str, delta: F) -> dict[str, Any]:
        fp, receipt = self._replay(command, ('transfer', expected, intent, str(delta)))
        if receipt is not None: return receipt
        status = 'COMMITTED'
        if expected != self.revision: status = 'STALE_CHANGE'
        elif intent in self.transfers: status = 'INTENT_ALREADY_APPLIED'
        elif delta < 0 or delta > self.amounts['bulk']: status = 'INFEASIBLE'
        if status == 'COMMITTED':
            self.amounts['bulk'] -= delta; self.amounts['surface'] += delta
            self.transfers.add(intent); self.revision += 1
        return self._finish(command, fp, {'status':status, 'revision':self.revision})

class ActionReferenceTests(unittest.TestCase):
    def state(self) -> ReferenceAuthority:
        s=ReferenceAuthority(); s.start('r'); return s
    def cand(self, rev=1, run='r', view='v1', scope='unit', accepted=True):
        return Candidate(rev,run,view,scope,accepted,(50.0,))
    def pub(self,s,cmd='p',candidate=None,expected=0):
        return s.publish(cmd,{'out':candidate or self.cand()},{'out'},{'out':expected},'unit')

    def test_01_ph_target_not_solver_flag(self):
        target=F(50); returned=F(20); residual=abs(returned-target)
        self.assertEqual(residual,30); self.assertGreater(residual,F(1,1000))
    def test_02_optional_status_does_not_waive_primary(self):
        checks={'h':True,'mass':True}; optional={'viscosity':'unavailable'}
        self.assertTrue(all(checks.values())); self.assertEqual(optional['viscosity'],'unavailable')
        checks['h']=False; self.assertFalse(all(checks.values()))
    def test_03_strict_conversion(self):
        initial=F(1); feasible=F(2,5); impossible=F(6,5)
        self.assertEqual(initial-feasible,F(3,5)); self.assertLess(initial-impossible,0)
    def test_04_phase_permutation_and_aggregate(self):
        p=[(F(6),(F(7,10),F(3,10))),(F(4),(F(1,5),F(4,5)))]
        def total(ps): return [sum(n*x[i] for n,x in ps) for i in (0,1)]
        self.assertEqual(total(p),[F(5),F(5)]); self.assertEqual(total(p[::-1]),total(p))
        self.assertEqual(sum(n for n,_ in p),10) # aggregate view excluded
    def test_05_reference_transform_not_model_discrepancy(self):
        T=F(350); ha=T-300; hb=F(6,5)*(T-300)+100
        self.assertEqual((hb-100)-ha,10)
        self.assertEqual(F(300)+ha/F(6,5),F(1025,3))
    def test_06_empirical_mass_heating(self):
        self.assertEqual(F(2)*F(2)*(325-300),100)
    def test_07_distribution_and_surface_basis(self):
        self.assertEqual((F(1)*F(4,5)+F(3)*F(1,5))/4,F(7,20))
        self.assertEqual(F(1,5)*5,1)
    def test_08_derivative_chart(self):
        self.assertEqual((1-4,2-4),(-3,-2))
        frozen=F(1); response=frozen+100*F(1,100); self.assertEqual(response,2)
    def test_09_publish_success(self):
        s=self.state(); self.assertEqual(self.pub(s)['status'],'COMMITTED'); self.assertIsNotNone(s.current('out'))
    def test_10_edit_before_publish(self):
        s=self.state(); s.edit('e',1,'kij'); self.assertEqual(self.pub(s)['status'],'OBSOLETE_CONTEXT'); self.assertFalse(s.slots)
    def test_11_publish_before_edit(self):
        s=self.state(); self.pub(s); old=s.slots['out']; s.edit('e',1,'kij')
        self.assertIsNone(s.current('out')); self.assertEqual(s.slots['out'],old)
    def test_12_cancel_same_revision(self):
        s=self.state(); s.cancel('r'); self.assertEqual(self.pub(s)['status'],'RUN_REVOKED')
    def test_13_parent_cancel(self):
        s=self.state(); s.start('child',parent='r'); s.cancel('r')
        self.assertEqual(self.pub(s,candidate=self.cand(run='child'))['status'],'RUN_REVOKED')
    def test_14_cancel_after_committed_publish(self):
        s=self.state(); self.pub(s); s.cancel('r'); self.assertIsNotNone(s.current('out'))
        self.assertEqual(self.pub(s,'new',expected=1)['status'],'RUN_REVOKED')
    def test_15_group_failure_changes_neither_slot(self):
        s=self.state(); prior={'hot':self.cand(),'cold':self.cand()}
        self.assertEqual(s.publish('old',prior,set(prior),{'hot':0,'cold':0},'unit')['status'],'COMMITTED')
        before=copy.deepcopy(s.slots)
        group={'hot':self.cand(view='v2'),'cold':self.cand(view='v2',accepted=False)}
        result=s.publish('p',group,set(group),{'hot':1,'cold':1},'unit')
        self.assertEqual(result['status'],'UNQUALIFIED'); self.assertEqual(s.slots,before)
    def test_16_same_revision_incoherent_view(self):
        s=self.state(); group={'hot':self.cand(),'cold':self.cand(view='v0')}
        self.assertEqual(s.publish('p',group,set(group),{'hot':0,'cold':0},'unit')['status'],'INCOHERENT_GROUP')
    def test_17_local_scope_not_flowsheet(self):
        s=self.state(); self.assertEqual(self.pub(s,candidate=self.cand(scope='local'))['status'],'SCOPE_MISMATCH')
    def test_18_duplicate_publish_does_not_reapply_old_result(self):
        s=self.state(); first=self.pub(s); self.pub(s,'new',self.cand(view='v2'),1)
        old_replay=self.pub(s); self.assertEqual(first,old_replay); self.assertEqual(s.slots['out'].view,'v2')
    def test_19_command_payload_conflict(self):
        s=self.state(); self.pub(s); self.assertEqual(self.pub(s,candidate=self.cand(view='different'))['status'],'COMMAND_PAYLOAD_CONFLICT')
    def test_20_duplicate_edit_and_stale_edit(self):
        s=self.state(); r=s.edit('e',1,'x'); self.assertEqual(s.edit('e',1,'x'),r); self.assertEqual(s.revision,2)
        self.assertEqual(s.edit('other',1,'y')['status'],'STALE_CHANGE')
    def test_21_transfer_applied_once(self):
        s=self.state(); r=s.transfer('t',1,'bulk-to-surface-1',F(1,10)); self.assertEqual(r['status'],'COMMITTED')
        self.assertEqual(s.transfer('t',1,'bulk-to-surface-1',F(1,10)),r)
        self.assertEqual(s.amounts['surface'],F(11,10)); self.assertEqual(sum(s.amounts.values()),3)
        self.assertEqual(s.transfer('t2',2,'bulk-to-surface-1',F(1,10))['status'],'INTENT_ALREADY_APPLIED')
    def test_22_failed_workspace_does_not_change_snapshot(self):
        original={'amounts':[1,2],'h':50}; work=copy.deepcopy(original); work['amounts'][0]=-9; work['h']=20
        self.assertEqual(original,{'amounts':[1,2],'h':50})
    def test_23_initializer_restoration_is_required(self):
        original={'T':'unknown','P':'fixed'}; work=copy.deepcopy(original); work['T']='fixed_for_initialization'
        self.assertNotEqual(work,original); work=copy.deepcopy(original); self.assertEqual(work,original)
    def test_24_native_cancel_does_not_free_busy_session(self):
        session={'health':'busy','native_active':True,'run_active':True}; session['run_active']=False
        reusable=(not session['native_active']) and session['health']=='ready'
        self.assertFalse(reusable); self.assertEqual(session['health'],'busy')
    def test_25_archive_is_not_authority_or_mixed_capture(self):
        snapshot={'revision':1,'package':'A','params':'A'}; captured=copy.deepcopy(snapshot)
        snapshot.update(revision=2,package='B',params='B'); self.assertEqual(captured['package'],captured['params'])
        restored=ReferenceAuthority(); self.assertFalse(restored.active('saved-cancelled-run'))
    def test_26_slot_generation_blocks_stale_same_run_proposal(self):
        s=self.state(); self.pub(s); self.assertEqual(self.pub(s,'stale',self.cand(view='v2'),0)['status'],'SLOT_CONFLICT')

if __name__=='__main__':
    stream=io.StringIO()
    suite=unittest.defaultTestLoader.loadTestsFromTestCase(ActionReferenceTests)
    names=[t.id().split('.')[-1] for t in suite]
    result=unittest.TextTestRunner(stream=stream,verbosity=2).run(suite)
    print(stream.getvalue())
    report={'kind':'sequential_authored_reference_model_checks_not_simulator_or_provider_conformance',
            'tests_run':result.testsRun,'passed':result.wasSuccessful(),'failures':len(result.failures),
            'errors':len(result.errors),'tests':names,'log':stream.getvalue(),
            'limitations':['No provider numerical calls','No concurrent transaction execution','No native crash/interruption experiment',
                           'No original AT/PV/P/SC evidence promotion','Only selected arithmetic and sequential guard/replay traces']}
    if len(sys.argv)>1: Path(sys.argv[1]).write_text(json.dumps(report,indent=2)+'\n')
    sys.exit(0 if result.wasSuccessful() else 1)
