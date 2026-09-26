"""Structural check of THERMO-INTEGRATION-008. No native/physical tests are run.
Run after rendering: python validate_integration_blueprint.py
"""
from __future__ import annotations
import hashlib,json,re
from collections import Counter
from pathlib import Path
from urllib.parse import unquote,urlsplit
ROOT=Path(__file__).resolve().parent
S='thermodynamics_cross_library_solver_integration_v0_1'
r=json.loads((ROOT/f'{S}_register.json').read_text())
b7=json.loads((ROOT/'thermodynamics_action_workflow_lifecycle_blueprint_v0_1_register.json').read_text())
b6=json.loads((ROOT/'thermodynamics_semantic_information_dictionary_v0_1_register.json').read_text())
auditfile=ROOT/f'{S}_audit.json'
# The audit is an expected companion target of the rendered document.
if not auditfile.exists(): auditfile.write_text('{}\n')
checks=[]
def ck(name: str,passed:bool,detail:object='') -> None:
 checks.append({'name':name,'passed':bool(passed),'detail':detail})
def unique(rows,key='id'):return len(rows)==len({x[key] for x in rows})
R={x['id'] for x in r['integration_rules']}; A={x['id'] for x in b7['actions']}; I={x['id'] for x in b6['concepts']}
P={x['id'] for x in b7['ownership_packages']}; M={x['id'] for x in r['solver_modes']}; IP={x['id'] for x in r['candidate_profile_templates']}
E={x['id'] for x in r['qualification_experiments']}; SC={x['scenario'] for x in b7['scenario_action_allocation']}
reqbase={x['requirement_id']:x for x in b7['requirement_action_allocation']}
scbase={x['scenario']:x for x in b7['scenario_action_allocation']}
ck('24 unique integration rules',len(R)==24 and unique(r['integration_rules']))
ck('Rule content complete',all(all(x.get(k) for k in ['title','accountable_package','decision','applicability','qualification_evidence','rejection_or_limit','positive_witness','negative_witness','actions','information_concepts','source_basis']) for x in r['integration_rules']))
ck('Rule owners valid',all(x['accountable_package'] in P for x in r['integration_rules']))
ck('Rule actions and concepts valid',all(set(x['actions'])<=A and set(x['information_concepts'])<=I for x in r['integration_rules']))
ck('Five unique reuse boundaries',len(r['reuse_boundaries'])==5 and unique(r['reuse_boundaries']))
ck('Five unique solver modes',len(M)==5 and unique(r['solver_modes']))
ck('Solver-mode contracts and experiment links complete',all(all(x.get(k) for k in ['unknown_owner','contract','derivatives','phase_policy','best_use','candidates','limit','rules','qualification_experiments']) and set(x['rules'])<=R and set(x['qualification_experiments'])<=E for x in r['solver_modes']))
ck('Eighteen unique composition assessments',len(r['composition_assessments'])==18 and unique(r['composition_assessments']))
ck('Every composition has explicit admission rejection and validation route',all(x.get('admission_contract') and x.get('forbidden_shortcut') and set(x['solver_modes'])<=M and bool(x.get('qualification_experiments')) and set(x['qualification_experiments'])<=E for x in r['composition_assessments']))
ck('Fourteen unique profile templates',len(IP)==14 and unique(r['candidate_profile_templates']))
ck('Profiles have actual-data initialization derivative and first-witness requirements',all(all(x.get(k) for k in ['physical_definition','qualification_candidates','reuse_boundary','solver_modes','required_resolved_data','initialization_contract','derivative_contract','hard_limits_and_open_work','first_witness','rules','qualification_experiments']) for x in r['candidate_profile_templates']))
ck('Profile scenario mode and rule references valid',all(set(x['scenarios'])<=SC and set(x['solver_modes'])<=M and set(x['rules'])<=R and set(x['qualification_experiments'])<=E for x in r['candidate_profile_templates']))
ck('Templates do not claim numerical or production qualification',all(x['production_realization']=='unselected' and x['numerical_execution']=='not_performed' and x['independent_validation']=='not_performed' for x in r['candidate_profile_templates']))
ck('Ten qualification gates with valid owners',len(r['qualification_gates'])==10 and unique(r['qualification_gates']) and all(x['owner'] in P and x.get('blocker') for x in r['qualification_gates']))
ck('Eighteen unexecuted qualification experiments',len(E)==18 and unique(r['qualification_experiments']) and all(x['status']=='planned_native_or_formulation_probe_not_executed' for x in r['qualification_experiments']))
ck('Experiments link valid profiles and integration rules',all(set(x['profiles'])<=IP and set(x['basis'])<=R for x in r['qualification_experiments']))
ck('All 94 functional identifiers preserved',len(r['requirement_integration_allocation'])==94 and {x['requirement_id'] for x in r['requirement_integration_allocation']}==set(reqbase))
fields=['primary_package','obligation_class','primary_action','primary_information_concept','acceptance_test_ids','source_anchor']
ck('Original primary owners actions information and witness links unchanged',all(all(x[k]==reqbase[x['requirement_id']][k] for k in fields) for x in r['requirement_integration_allocation']))
ck('Original H/C/R3 classes preserved',Counter(x['obligation_class'] for x in r['requirement_integration_allocation'])==Counter({'H':65,'C':25,'R3':4}))
ck('Every requirement has governing integration rules',all(x['integration_rules'] and set(x['integration_rules'])<=R for x in r['requirement_integration_allocation']))
ck('All 56 actions retain owner and have rule coverage',len(r['action_integration_allocation'])==56 and {x['action'] for x in r['action_integration_allocation']}==A and all(x['integration_rules'] and set(x['integration_rules'])<=R and x['accountable_package']==next(b['accountable_package'] for b in b7['actions'] if b['id']==x['action']) for x in r['action_integration_allocation']))
ck('All 72 concept action mappings inherited unchanged',r['inherited_information_action_coverage']==b7['information_action_coverage'] and len(r['inherited_information_action_coverage'])==72)
ck('Eighteen information products unchanged',r['inherited_information_products']==b7['inherited_information_products'] and len(r['inherited_information_products'])==18)
ck('108 semantic relationships unchanged',r['inherited_relationship_contracts']==b7['inherited_relationship_contracts'] and len(r['inherited_relationship_contracts'])==108)
ck('28 semantic rule routes unchanged',r['inherited_semantic_rule_action_coverage']==b7['semantic_rule_action_coverage'] and len(r['inherited_semantic_rule_action_coverage'])==28)
ck('12 physical operation variants unchanged',r['inherited_operation_variants']==b7['operation_variants'] and len(r['inherited_operation_variants'])==12)
ck('All 34 scenarios and profiles preserved',len(r['scenario_profile_allocation'])==34 and {x['scenario'] for x in r['scenario_profile_allocation']}==SC and all(x['profile']==scbase[x['scenario']]['profile'] for x in r['scenario_profile_allocation']))
ck('Original 14/16/4 scenario profile counts',Counter(x['profile'] for x in r['scenario_profile_allocation'])==Counter({'P1':14,'P2':16,'P3':4}))
ck('Every scenario has a valid provisional route and no added execution claim',all(x['candidate_profiles'] and set(x['candidate_profiles'])<=IP and x['execution_evidence']=='not_added' and x['validation_evidence']=='not_added' for x in r['scenario_profile_allocation']))
ck('All P3 routes preserve deferred numerical scope',all(x['candidate_profiles']==['IP-14'] for x in r['scenario_profile_allocation'] if x['profile']=='P3'))
ck('Original integrated and synthetic fixture records unchanged',r['inherited_integrated_journeys']==b7['inherited_integrated_journeys'] and r['inherited_synthetic_contract_fixtures']==b7['inherited_synthetic_contract_fixtures'])
expected={f'B2:P{n:02d}' for n in range(1,19)}|{f'B3:PV-{n:02d}' for n in range(1,21)}
ck('All 38 prior native probes mapped without promotion',{x['probe'] for x in r['inherited_probe_crosswalk']}==expected and all(x['integration_experiments'] and set(x['integration_experiments'])<=E and x['status']=='not_executed' for x in r['inherited_probe_crosswalk']))
oldopen={x['inherited_id']:x for x in b7['inherited_open_item_disposition']}
ck('All ten open items preserve owners',len(r['open_item_disposition'])==10 and all(x['primary_package']==oldopen[x['inherited_id']]['primary_package'] for x in r['open_item_disposition']))
ck('Open items distinguish policy from remaining runtime evidence',all(x.get('step8_design_resolution') and x.get('remaining_empirical_or_implementation_gate') for x in r['open_item_disposition']))
ck('Decision rules reference valid integration rules',unique(r['decisions']) and all(set(x['rules'])<=R for x in r['decisions']))
sourcecheck=[]
for x in r['sources']:
 p=ROOT/x['filename']; found=p.is_file(); digest=hashlib.sha256(p.read_bytes()).hexdigest() if found else None
 sourcecheck.append({'key':x['key'],'file':x['filename'],'expected':x['sha256'],'actual':digest,'matched':found and digest==x['sha256']})
ck('All exact predecessor hashes match',all(x['matched'] for x in sourcecheck),sourcecheck)
ck('Source keys unique',len(r['sources'])==len({x['key'] for x in r['sources']}))
ck('14 official-documentation sources are separately tracked',len(r['official_documentation_sources'])==14 and unique(r['official_documentation_sources']) and all(x.get('document_track') and x['evidence']=='official_documentation_observed_not_runtime_tested' for x in r['official_documentation_sources']))
ref=json.loads((ROOT/f'{S}_reference_checks.json').read_text())
ck('Authored reference checks passed with evidence limitation',ref['failed']==0 and ref['passed']==ref['count'] and bool(ref['limits']),{'count':ref['count'],'scope':ref['kind']})
# Validate generated Markdown and all local Markdown/companion links it contains.
text=(ROOT/f'{S}.md').read_text()
ck('No unexpanded rendering placeholders',not re.search(r'{{[A-Z_]+}}',text))
ck('No production-certification assertion in template statuses',all('not_' in x['status'] for x in r['candidate_profile_templates']))
def anchors(p):
 s=p.read_text(); vals=set(re.findall(r'<a\s+id=["\']([^"\']+)["\']',s))
 for line in s.splitlines():
  if re.match(r'^#{1,6} ',line):
   t=re.sub(r'^#{1,6} ','',line).strip().lower(); t=re.sub(r'[`*_]','',t)
   t=re.sub(r'[^\w\-\s]','',t); vals.add(re.sub(r'\s+','-',t))
 return vals
cache={}; broken=[]; local_count=0
for target in re.findall(r'(?<!!)\[[^\]]*\]\(([^)]+)\)',text):
 target=target.strip().split(' "')[0]
 u=urlsplit(target)
 if u.scheme or u.netloc: continue
 local_count+=1
 p=(ROOT/unquote(u.path)) if u.path else ROOT/f'{S}.md'
 if not p.is_file(): broken.append({'target':target,'reason':'missing_file'}); continue
 if u.fragment and p.suffix=='.md':
  if p not in cache:cache[p]=anchors(p)
  if unquote(u.fragment) not in cache[p]:broken.append({'target':target,'reason':'missing_anchor'})
ck('All generated local artifact links resolve',not broken,{'links_checked':local_count,'broken':broken[:50]})
report={'document':'THERMO-INTEGRATION-008','count':len(checks),'passed':sum(x['passed'] for x in checks),'failed':sum(not x['passed'] for x in checks),
 'checks':checks,'counts':{k:len(r[k]) for k in ['integration_rules','reuse_boundaries','solver_modes','composition_assessments','candidate_profile_templates','qualification_gates','qualification_experiments','requirement_integration_allocation','action_integration_allocation','scenario_profile_allocation']},
 'native_tests_executed':0,'REV_promotions':0,
 'evidence_limits':'Catalog/reference integrity and authored algebra only; not simulator/provider conformance, concurrency correctness, native crash containment or empirical validation.'}
auditfile.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in {'checks'}},indent=2))
if report['failed']:
 for x in checks:
  if not x['passed']: print('FAILED',x)
 raise SystemExit(1)
