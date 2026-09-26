"""Structural audit of THERMO-ACTIONS-007 and its unchanged baseline register.
This checks artifact and allocation integrity only, not runtime conformance.
Run from anywhere with this script and the extracted bundle in one directory.
"""
from __future__ import annotations
import collections
import hashlib
import json
import re
import sys
from pathlib import Path
from urllib.parse import unquote, urlsplit

NAME='thermodynamics_action_workflow_lifecycle_blueprint_v0_1'
def audit(root: Path) -> dict:
    reg=json.loads((root/(NAME+'_register.json')).read_text())
    b4=json.loads((root/'thermodynamics_functional_requirements_v0_1_register.json').read_text())
    b5=json.loads((root/'thermodynamics_conceptual_packaging_v0_1_register.json').read_text())
    b6=json.loads((root/'thermodynamics_semantic_information_dictionary_v0_1_register.json').read_text())
    a=reg['actions']; aa={x['id']:x for x in a}; rr={x['id']:x for x in b4['requirements']}
    cc={x['id']:x for x in b6['concepts']}; pp={x['id'] for x in b5['packages']}
    alloc={x['requirement_id']:x for x in b5['requirement_allocation']}
    primary=collections.Counter(x for y in a for x in y['primary_requirement_ids'])
    checks=[]
    def ck(name,condition,detail=''):
        checks.append({'check':name,'status':'PASS' if condition else 'FAIL','detail':detail})
    ck('56 unique action IDs',len(a)==len(aa)==56)
    mandatory=['id','title','accountable_package','kind','initiator_and_trigger','preconditions','steps','successful_postconditions','failure_and_recovery','repetition_and_replay','permitted_effects','adoption_boundary','witnesses','source_basis']
    ck('action contracts populated',all(all(x.get(k) for k in mandatory) for x in a))
    ck('valid action owners',all(x['accountable_package'] in pp for x in a))
    ck('94 requirements assigned exactly once',set(primary)==set(rr) and all(x==1 for x in primary.values()))
    ck('primary ownership unchanged',all(alloc[r]['primary_package']==x['accountable_package'] for x in a for r in x['primary_requirement_ids']))
    ck('supporting requirement references valid',all(r in rr for x in a for r in x['supporting_requirement_ids']))
    ck('input and output concept references valid',all(c in cc for x in a for c in x['input_concepts']+x['output_concepts']))
    ck('all 72 concepts have a producer or proposal',set(cc)=={c for x in a for c in x['output_concepts']} )
    ck('all 72 concepts have consumers',set(cc)=={c for x in a for c in x['input_concepts']})
    ck('18 information products unchanged',reg['inherited_information_products']==b6['information_products'])
    ck('108 relationship contracts unchanged',reg['inherited_relationship_contracts']==b6['relationship_contracts'])
    original_test_ids={t['id'] for x in b4['requirements'] for t in x['acceptance_tests']}
    ck('original witness IDs preserved',set(t for x in reg['requirement_action_allocation'] for t in x['acceptance_test_ids'])==original_test_ids)
    ck('H C R3 class allocation unchanged',all(x['obligation_class']==alloc[x['requirement_id']]['obligation_class'] for x in reg['requirement_action_allocation']))
    ck('112 action witness descriptions',sum(len(x['witnesses']) for x in a)==112 and all(len(x['witnesses'])==2 for x in a))
    ck('no claimed action witness execution',all(t['status']=='specified_not_executed_against_simulator' for x in a for t in x['witnesses']))
    ss={x['scenario']:x['profile'] for x in b6['scenario_information_allocation']}
    ck('34 scenario profiles unchanged',ss=={x['scenario']:x['profile'] for x in reg['scenario_action_allocation']})
    ww={w['id']:w for w in reg['workflows']}
    ck('18 workflows uniquely identified',len(ww)==len(reg['workflows'])==18)
    ck('every scenario has a real workflow route',all(x['step7_workflows'] and all(w in ww and x['scenario'] in ww[w]['scenarios'] for w in x['step7_workflows']) for x in reg['scenario_action_allocation']))
    ck('workflow actions valid',all(x in aa for w in ww.values() for x in w['action_route']))
    ck('all actions used in workflows',set(aa)=={x for w in ww.values() for x in w['action_route']})
    ck('Step 5 walkthroughs mapped',set(reg['step5_walkthrough_crosswalk'])=={x['id'] for x in b5['walkthroughs']} and all(x in ww for x in reg['step5_walkthrough_crosswalk'].values()))
    ck('lifecycle endpoints actions and owners valid',all(l['owner'] in pp and set(l['initial_states'])<=set(l['states']) and all(t['from'] in l['states'] and t['to'] in l['states'] and t['guard'] and t['effect'] and all(x in aa for x in t['actions']) for t in l['transitions']) for l in reg['lifecycle_views']))
    ck('all lifecycle states reachable in declared table',all(_reachable(l)==set(l['states']) for l in reg['lifecycle_views']))
    ck('handoff facts references valid',all(all(x in aa for x in e['producing_actions']+e['consuming_actions']) and all(c in cc for c in e['information_concepts']) for e in reg['handoff_facts']))
    ck('action rule references valid',all(x in aa for r in reg['action_rules'] for x in r['actions']))
    ck('all semantic rules linked',set(x['semantic_rule'] for x in reg['semantic_rule_action_coverage'])=={x['id'] for x in b6['semantic_rules']} and all(x['actions'] for x in reg['semantic_rule_action_coverage']))
    ck('operation variants references valid',all(v['action'] in aa and all(r in rr for r in v['requirements']) and all(s in ss for s in v['scenarios']) for v in reg['operation_variants']))
    ck('open item ownership unchanged',{x['inherited_id']:x['primary_package'] for x in reg['inherited_open_item_disposition']}=={x['inherited_id']:x['primary_package'] for x in b6['inherited_open_item_disposition']})
    ck('integrated journeys and synthetic fixtures unchanged',reg['inherited_integrated_journeys']==b4['integrated_journeys'] and reg['inherited_synthetic_contract_fixtures']==b4['synthetic_contract_fixtures'])
    mismatches=[]
    for s in reg['sources']:
        p=root/s['filename']; actual=hashlib.sha256(p.read_bytes()).hexdigest() if p.exists() else None
        if actual!=s['sha256']:mismatches.append({'file':str(p),'expected':s['sha256'],'actual':actual})
    ck('predecessor files and hashes match',not mismatches,str(mismatches))
    ck('no provider or inherited acceptance execution claimed',reg['evidence_boundary']['original_acceptance_tests_executed']==0 and reg['evidence_boundary']['native_provider_tests_executed']==0 and reg['evidence_boundary']['independent_REV_promotions']==0)
    ref=json.loads((root/(NAME+'_reference_checks.json')).read_text())
    ck('26 selected reference checks executed successfully',ref['tests_run']==26 and ref['passed'] and ref['failures']==ref['errors']==0)
    ck('reference results in register agree',ref==reg['authored_reference_checks'])
    md=(root/(NAME+'.md')).read_text()
    anchors=re.findall(r'<a id="([^"]+)"',md)
    ck('unique explicit document anchors',len(anchors)==len(set(anchors)))
    unresolved=[]; link_count=0
    for dest in re.findall(r'\]\(([^)]+)\)',md):
        u=urlsplit(dest)
        if u.scheme or u.netloc: continue
        p=(root/unquote(u.path)) if u.path else root/(NAME+'.md')
        link_count+=1
        if not p.exists():
            # The audit itself is created immediately after this check; exclude this one known output.
            if p.name==NAME+'_audit.json': continue
            unresolved.append(dest); continue
        if u.fragment and f'id="{unquote(u.fragment)}"' not in p.read_text(errors='replace'):
            unresolved.append(dest)
    ck('local blueprint links and exact anchors resolve',not unresolved,str(unresolved[:20]))
    counts={'actions':len(a),'action_witnesses':sum(len(x['witnesses']) for x in a),'requirements':len(primary),
            'concepts':len(cc),'scenarios':len(ss),'workflows':len(ww),'lifecycle_views':len(reg['lifecycle_views']),
            'lifecycle_transitions':sum(len(l['transitions']) for l in reg['lifecycle_views']),
            'handoff_facts':len(reg['handoff_facts']),'action_rules':len(reg['action_rules']),
            'operation_variants':len(reg['operation_variants']),'structural_checks':len(checks),
            'local_links_checked':link_count,'reference_model_tests':ref['tests_run']}
    return {'document':'THERMO-ACTIONS-007','audit_kind':'catalog_and_artifact_integrity_not_runtime_conformance',
            'passed':all(c['status']=='PASS' for c in checks),'counts':counts,'checks':checks,
            'limitations':['No numeric thermodynamic provider run','No concurrent implementation tested','No independent scenario R/E/V pass','Reference tests cover selected sequential guards and authored arithmetic only']}

def _reachable(l: dict) -> set[str]:
    reached=set(l['initial_states'])
    while True:
        new=reached|{t['to'] for t in l['transitions'] if t['from'] in reached}
        if new==reached:return new
        reached=new

if __name__=='__main__':
    root=Path(__file__).resolve().parent
    result=audit(root)
    (root/(NAME+'_audit.json')).write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({'passed':result['passed'],'counts':result['counts'],
                      'failures':[c for c in result['checks'] if c['status']!='PASS']},indent=2))
    sys.exit(0 if result['passed'] else 1)
