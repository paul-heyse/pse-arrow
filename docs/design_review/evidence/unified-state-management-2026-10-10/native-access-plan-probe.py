from pathlib import Path
import base64, json, os, re, sys, time, urllib.request, uuid
sys.path.insert(0, str(Path.cwd()))
from scripts import surreal_server as server, test_resources as resources
state=Path(os.environ['PSE_SURREAL_STATE']).resolve()
namespace=server.config_for(state)['namespace']
database=os.environ.get('SM34_PROBE_DATABASE') or 'canonical_test_'+uuid.uuid4().hex
resource=os.environ.get('SM34_PROBE_RESOURCE') or resources.register({'state':str(state),'database':database,'kind':'database','units':[]})
borrow=resources.borrow(resource)
borrow.__enter__()
results={'database':database,'resource':resource,'cases':[]}
def sql(query):
    c=server.config_for(state); credentials=server.read_json(state/'credentials.json')
    token=base64.b64encode(f"{credentials['username']}:{credentials['password']}".encode()).decode()
    request=urllib.request.Request(f"http://127.0.0.1:{c['port']}/sql",data=query.encode(),headers=server.administrative_headers(c,token,namespace,database))
    with urllib.request.urlopen(request,timeout=60) as response: rows=json.loads(response.read(4*1024*1024))
    errors=[(i,row.get('result')) for i,row in enumerate(rows) if row.get('status')!='OK']
    if errors: raise RuntimeError(str(errors[:3]))
    return rows
def q(s): return json.dumps(s)
def batch(table, records, relation=False):
    for start in range(0,len(records),128):
        sql('INSERT '+('RELATION ' if relation else '')+'INTO '+table+' ['+','.join(records[start:start+128])+'] RETURN NONE;')
try:
    server.administrative_query(state,'USE NS '+namespace+';DEFINE DATABASE IF NOT EXISTS '+database+';',time.monotonic()+60,select_context=False)
    tables={'canonical_memberships','canonical_version_manifests','canonical_products','canonical_roots','canonical_edges','canonical_problems'}
    schema=Path('crates/pse-operations/src/generated/surreal.surql').read_text()
    selected=[]
    for line in schema.splitlines():
        m=re.match(r'DEFINE TABLE (\w+)|DEFINE FIELD(?: OVERWRITE)? \S+ ON (\w+)|DEFINE INDEX \S+ ON (\w+)',line)
        if m and next(x for x in m.groups() if x) in tables: selected.append(line)
    sql('\n'.join(re.sub(r'^DEFINE (TABLE|FIELD|INDEX) (?!OVERWRITE)',r'DEFINE \1 OVERWRITE ',line) for line in selected))
    for n in (256,4096):
        problem='p'+str(n)
        manifests=[]; memberships=[]; products=[]; roots=[]; edges=[]
        for i in range(n):
            key=f'{problem}-{i:06d}'; version='v-'+key; rare=i==n-1
            manifests.append('{id:type::record("canonical_version_manifests",'+q(version)+'),key:'+q(version)+',logical:'+q(key)+',kind:'+q('rare' if rare else 'common')+',interpretation:"fixture",payload_digest:"fixture",payload_len:0dec,block_count:0dec,reference_count:0dec,creator_stage:"fixture",closed:true}')
            memberships.append('{id:type::record("canonical_memberships",'+q(key)+'),key:'+q(key)+',in:type::record("canonical_problems",'+q(problem)+'),problem:'+q(problem)+',scope:'+q('rare' if rare else 'common')+',name:'+q(key)+',logical:'+q(key)+',out:type::record("canonical_version_manifests",'+q(version)+'),version:'+q(version)+',from_sequence:1dec,to_sequence:NONE}')
            products.append('{id:type::record("canonical_products",'+q(key)+'),key:'+q(key)+',problem:'+q(problem)+',revision:"r",request:encoding::base64::decode('+q('dGFyZ2V0' if rare else 'b3RoZXI=')+'),payload:encoding::base64::decode(""),dependencies:encoding::base64::decode(""),producer:"producer",interpretation:"fixture"}')
            roots.append('{id:type::record("canonical_roots",'+q(key)+'),key:'+q(key)+',problem:'+q(problem)+',revision:"r",sequence:1dec,owner_kind:"product",owner:'+q(key)+'}')
            edges.append('{id:type::record("canonical_edges",'+q(key)+'),key:'+q(key)+',source_version:'+q(version)+',ordinal:0dec,target_scope:"target",target_name:'+q('rare' if rare else 'common')+'}')
        batch('canonical_version_manifests',manifests)
        batch('canonical_memberships',memberships,True)
        batch('canonical_products',products)
        batch('canonical_roots',roots)
        batch('canonical_edges',edges)
        common='LET $problem = '+q(problem)+'; LET $sequence = 1dec; LET $after = ""; '
        membership='SELECT * FROM canonical_memberships WHERE problem = $problem AND ($scope = NONE OR scope = $scope) AND key > $after AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence)'
        full=membership+' AND ($source_kind = NONE OR (out.kind = $source_kind AND out.closed = true)) AND ($target_scope = NONE OR (SELECT VALUE key FROM canonical_edges WHERE source_version = $parent.version AND target_scope = $target_scope AND target_name = $target_name LIMIT 1) != []) ORDER BY key LIMIT 64'
        prod='SELECT * FROM canonical_products WHERE problem = $problem AND producer = $producer AND request = $request AND key > $after AND interpretation = $interpretation AND type::record("canonical_roots", key).owner_kind = "product" AND type::record("canonical_roots", key).owner = key AND type::record("canonical_roots", key).problem = problem AND type::record("canonical_roots", key).revision = revision ORDER BY key LIMIT 1'
        cases=[('membership', 'LET $scope=NONE;',membership+' ORDER BY key LIMIT 64'),('scoped_skew','LET $scope="rare";',membership+' ORDER BY key LIMIT 64'),('kind_skew','LET $scope=NONE;LET $source_kind="rare";LET $target_scope=NONE;LET $target_name=NONE;',full),('inverse_skew','LET $scope=NONE;LET $source_kind=NONE;LET $target_scope="target";LET $target_name="rare";',full),('inverse_negative','LET $scope=NONE;LET $source_kind=NONE;LET $target_scope="target";LET $target_name="absent";',full),('negative_name','', 'SELECT * FROM canonical_memberships WHERE problem=$problem AND scope="rare" AND name="absent" AND from_sequence <= $sequence AND (to_sequence=NONE OR to_sequence>$sequence) ORDER BY from_sequence DESC LIMIT 1')]
        for name, request in [('product_present','dGFyZ2V0'),('product_absent','YWJzZW50')]:
            cases.append((name,'LET $producer="producer";LET $interpretation="fixture";LET $request=encoding::base64::decode('+q(request)+');',prod))
        for name,params,query in cases:
            prefix=common+params
            actual=sql(prefix+query+';')[-1]
            plan=sql(prefix+query+' EXPLAIN FULL;')[-1]
            try: analyze=sql(prefix+'EXPLAIN ANALYZE '+query+';')[-1]
            except Exception as error: analyze={'error':type(error).__name__}
            record={'n':n,'case':name,'returned':len(actual['result']),'time':actual['time'],'full':plan['result'],'analyze':analyze}
            results['cases'].append(record)
            Path('/tmp/sm34-explain-results.json').write_text(json.dumps(results,indent=2))
            print(n,name,len(actual['result']),actual['time'],flush=True)
    # Compare an exact request/key access path without changing production declarations.
    sql('DEFINE INDEX product_discovery_exact ON canonical_products FIELDS problem,producer,request,key;')
    for n in (256,4096):
        for name, request in [('product_present','dGFyZ2V0'),('product_absent','YWJzZW50')]:
            prefix='LET $problem='+q('p'+str(n))+';LET $producer="producer";LET $interpretation="fixture";LET $after="";LET $request=encoding::base64::decode('+q(request)+');'
            actual=sql(prefix+prod+';')[-1]
            plan=sql(prefix+prod+' EXPLAIN FULL;')[-1]
            try: analyze=sql(prefix+'EXPLAIN ANALYZE '+prod+';')[-1]
            except Exception as error: analyze={'error':type(error).__name__}
            results['cases'].append({'n':n,'case':name+'_index_request_key','returned':len(actual['result']),'time':actual['time'],'full':plan['result'],'analyze':analyze})
            print(n,name+'_index_request_key',len(actual['result']),actual['time'],flush=True)
    sql('DEFINE INDEX membership_scope_pages ON canonical_memberships FIELDS problem,scope,key;')
    for n in (256,4096):
        prefix='LET $problem='+q('p'+str(n))+';LET $sequence=1dec;LET $after="";LET $scope="rare";'
        for name,query in [('scope_optional_newindex',membership+' ORDER BY key LIMIT 64'),('scope_exact_newindex',membership.replace('($scope = NONE OR scope = $scope)','scope = $scope')+' ORDER BY key LIMIT 64')]:
            actual=sql(prefix+query+';')[-1]
            plan=sql(prefix+query+' EXPLAIN FULL;')[-1]
            results['cases'].append({'n':n,'case':name,'returned':len(actual['result']),'time':actual['time'],'full':plan['result']})
            print(n,name,len(actual['result']),actual['time'],flush=True)
    results['complete']=True
finally:
    borrow.__exit__(None,None,None)
    resources.record_drain(resource)
    if results.get('complete'):
        resources.pin(resource,False)
        results['cleanup']=resources.reclaim(resource)
    Path('/tmp/sm34-explain-results.json').write_text(json.dumps(results,indent=2))
    print('fixture',database,'cleanup',results.get('cleanup','retained'),flush=True)
