from pathlib import Path
import base64,json,os,re,sys,time,urllib.request,uuid
sys.path.insert(0,str(Path.cwd()))
from scripts import surreal_server as server,test_resources as resources
controls_only="--controls-only" in sys.argv
output=Path(os.environ.get("PSE_PROBE_OUTPUT", "/tmp/sm34-inverse-results.json"))
state=Path(os.environ['PSE_SURREAL_STATE']).resolve();config=server.config_for(state);namespace=config['namespace'];database='canonical_test_'+uuid.uuid4().hex
resource=resources.register({'state':str(state),'database':database,'kind':'database','units':[]});borrow=resources.borrow(resource);borrow.__enter__()
results={'database':database,'resource':resource,'cases':[],'conditions':'SurrealDB3.3; generated exact schema/indexes; full protection lock/expiry guards; 30s original RPC clock; sequence5; inactive/future memberships and crossproblem edges'}
def sql(query):
 c=server.config_for(state);credentials=server.read_json(state/'credentials.json');token=base64.b64encode(f"{credentials['username']}:{credentials['password']}".encode()).decode()
 request=urllib.request.Request(f"http://127.0.0.1:{c['port']}/sql",data=query.encode(),headers=server.administrative_headers(c,token,namespace,database))
 with urllib.request.urlopen(request,timeout=35) as response:rows=json.loads(response.read(4*1024*1024))
 errors=[(i,row.get('result')) for i,row in enumerate(rows) if row.get('status')!='OK']
 if errors:raise RuntimeError(str([item for item in errors if 'not executed due to a failed transaction' not in str(item[1])] or errors[:3]))
 return rows
def q(s):return json.dumps(s)
def batch(table,records,relation=False):
 for start in range(0,len(records),128):sql('INSERT '+('RELATION ' if relation else '')+'INTO '+table+' ['+','.join(records[start:start+128])+'] RETURN NONE;')
def save():output.write_text(json.dumps(results,indent=2))
try:
 server.administrative_query(state,'USE NS '+namespace+';DEFINE DATABASE '+database+';',time.monotonic()+30,select_context=False)
 schema=Path('crates/pse-operations/src/generated/surreal.surql').read_text();tables={'canonical_memberships','canonical_version_manifests','canonical_edges','canonical_problems','canonical_guards','canonical_protections'};selected=[]
 for line in schema.splitlines():
  m=re.match(r'DEFINE TABLE (\w+)|DEFINE FIELD(?: OVERWRITE)? \S+ ON (\w+)|DEFINE INDEX \S+ ON (\w+)',line)
  if m and next(x for x in m.groups() if x) in tables:selected.append(line)
 sql('\n'.join(selected))
 deadline=re.search(r'DEFINE FUNCTION fn::pse_execution_v1::deadline\(.*?\n};',schema,re.S).group();sql(deadline)
 protected=re.search(r'const PROTECTED_BEGIN: &str = r#"(.*?)"#;',Path('crates/pse-operations/src/canonical.rs').read_text(),re.S).group(1)
 for n in ((256,) if controls_only else (256,4096)):
  p='p'+str(n);members=[];manifests=[];edges=[]
  sql('CREATE type::record("canonical_guards",'+q('retention:'+p)+') SET key='+q('retention:'+p)+',generation=1dec;CREATE type::record("canonical_protections",'+q('pin-'+p)+') SET key='+q('pin-'+p)+',problem='+q(p)+',revision="r",sequence=5dec,expires_at=time::micros()+600000000,released=false;')
  for i in range(n):
   key=f'{p}-{i:06d}';v='v-'+key;rare=i==n-1;kind='dataset' if rare or i%3 else 'definition';scope='rare' if rare else 'common';start=7 if i%7==0 and not rare else 1;end='3dec' if i%5==0 and not rare else 'NONE'
   manifests.append('{id:type::record("canonical_version_manifests",'+q(v)+'),key:'+q(v)+',logical:'+q(key)+',kind:'+q(kind)+',interpretation:"fixture",payload_digest:"fixture",payload_len:0dec,block_count:0dec,reference_count:0dec,creator_stage:"fixture",closed:true}')
   members.append('{id:type::record("canonical_memberships",'+q(key)+'),key:'+q(key)+',in:type::record("canonical_problems",'+q(p)+'),problem:'+q(p)+',scope:'+q(scope)+',name:'+q(key)+',logical:'+q(key)+',out:type::record("canonical_version_manifests",'+q(v)+'),version:'+q(v)+',from_sequence:'+str(start)+'dec,to_sequence:'+end+'}')
   for target in (['rare','common'] if rare or i%5==0 else ['common']):
    ek=key+'-'+target
    edges.append('{id:type::record("canonical_edges",'+q(ek)+'),key:'+q(ek)+',source_version:'+q(v)+',ordinal:' + ('1dec' if target=='rare' else '0dec') + ',target_scope:"target",target_name:'+q(target)+'}')
  batch('canonical_version_manifests',manifests);batch('canonical_memberships',members,True);batch('canonical_edges',edges)
  cases=[('rare',None,None,'rare',''),('absent',None,None,'absent',''),('rare_scoped','rare',None,'rare',''),('rare_kind',None,'dataset','rare',''),('rare_wrong_kind',None,'absent','rare',''),('common_tail',None,'dataset','common',f'{p}-{n-65:06d}'),('rare_after',None,None,'rare',f'{p}-{n-1:06d}'),('unfiltered',None,None,None,'')]
  for name,scope,kind,target,after in ([] if controls_only else cases):
   actuals={}
   for variant in ('correlated','inline','let'):
    prefix='LET $problem='+q(p)+';LET $revision="r";LET $protection='+q('pin-'+p)+';LET $sequence=5dec;LET $after='+q(after)+';LET $scope='+('NONE' if scope is None else q(scope))+';LET $source_kind='+('NONE' if kind is None else q(kind))+';LET $target_scope='+('NONE' if target is None else '"target"')+';LET $target_name='+('NONE' if target is None else q(target))+';LET $pse_rpc_timeout=30s;LET $pse_rpc_expires_at='+str(time.time_ns()//1000+30000000)+';'
    select='SELECT * FROM canonical_memberships WHERE problem=$problem'+(' AND scope=$scope' if scope is not None else '')+' AND key>$after AND from_sequence<=$sequence AND (to_sequence=NONE OR to_sequence>$sequence) AND ($source_kind=NONE OR (out.kind=$source_kind AND out.closed=true))'
    let=''
    if variant=='correlated':select+=' AND ($target_scope=NONE OR (SELECT VALUE key FROM canonical_edges WHERE source_version=$parent.version AND target_scope=$target_scope AND target_name=$target_name LIMIT 1)!=[])'
    elif target is not None:
     if variant=='inline':select+=' AND version IN (SELECT VALUE source_version FROM canonical_edges WHERE target_scope=$target_scope AND target_name=$target_name)'
     else:let='LET $suppliers = SELECT VALUE source_version FROM canonical_edges WHERE target_scope=$target_scope AND target_name=$target_name TIMEOUT $pse_rpc_timeout;';select+=' AND version IN $suppliers'
    select+=' ORDER BY key LIMIT 64 TIMEOUT $pse_rpc_timeout'
    suffix=';fn::pse_execution_v1::deadline($pse_rpc_expires_at);IF $pin.expires_at<=time::micros(){THROW "immutable selection protection expired";};RETURN $rpc_result;COMMIT;'
    query=prefix+protected+'\n'+let+'LET $rpc_result='+select+suffix
    rows=sql(query);actual=rows[-2]['result'];actuals[variant]=actual
    plans=sql(prefix+protected+'\n'+let+'LET $rpc_result='+select+' EXPLAIN FULL'+suffix)
    results['cases'].append({'n':n,'case':name,'variant':variant,'returned':len(actual),'keys':[r['key'] for r in actual],'statement_times':[r['time'] for r in rows],'plan':plans[-2]['result'],'query':query})
    save();print(n,name,variant,len(actual),flush=True)
   assert actuals['correlated']==actuals['inline']==actuals['let'],(n,name,'operation equality differs')
 if controls_only:
  # One additional witness per source must retain IN/existence semantics.
  duplicate=[]
  for i in range(256):
   key=f'p256-{i:06d}';v='v-'+key
   duplicate.append('{id:type::record("canonical_edges",'+q(key+'-duplicate')+'),key:'+q(key+'-duplicate')+',source_version:'+q(v)+',ordinal:2dec,target_scope:"target",target_name:"common"}')
  batch('canonical_edges',duplicate)
  results['controls']=[]
  active=[f'p256-{i:06d}' for i in range(256) if (i%7!=0 and i%5!=0 and i%3!=0) or i==255]
  for scope,kind,target,expected in [(None,'dataset','common',active),('rare','dataset','common',['p256-000255']),(None,'absent','common',[]),(None,'dataset','absent',[])]:
   after='';all_rows=[]
   while True:
    actuals={}
    for variant in ('correlated','let'):
     sql('UPDATE canonical_protections SET expires_at=time::micros()+60000000,released=false WHERE key="pin-p256";')
     prefix='LET $problem="p256";LET $revision="r";LET $protection="pin-p256";LET $sequence=5dec;LET $after='+q(after)+';LET $scope='+('NONE' if scope is None else q(scope))+';LET $source_kind='+q(kind)+';LET $target_scope="target";LET $target_name='+q(target)+';LET $pse_rpc_timeout=30s;LET $pse_rpc_expires_at='+str(time.time_ns()//1000+30000000)+';'
     supplier='SELECT VALUE source_version FROM canonical_edges WHERE target_scope=$target_scope AND target_name=$target_name TIMEOUT $pse_rpc_timeout'
     prelude='' if variant=='correlated' else 'LET $suppliers='+supplier+';fn::pse_execution_v1::deadline($pse_rpc_expires_at);'
     select='SELECT * FROM canonical_memberships WHERE problem=$problem'+(' AND scope=$scope' if scope is not None else '')+' AND key>$after AND from_sequence<=$sequence AND (to_sequence=NONE OR to_sequence>$sequence) AND ($source_kind=NONE OR (out.kind=$source_kind AND out.closed=true))'
     select+=(' AND ($target_scope=NONE OR (SELECT VALUE key FROM canonical_edges WHERE source_version=$parent.version AND target_scope=$target_scope AND target_name=$target_name LIMIT 1)!=[])' if variant=='correlated' else ' AND version IN $suppliers')
     select+=' ORDER BY key LIMIT 64 TIMEOUT $pse_rpc_timeout'
     suffix=';fn::pse_execution_v1::deadline($pse_rpc_expires_at);IF $pin.expires_at<=time::micros(){THROW "immutable selection protection expired";};RETURN $rpc_result;COMMIT;'
     query=prefix+protected+'\n'+prelude+'LET $rpc_result='+select+suffix
     rows=sql(query);actual=rows[-2]['result'];actuals[variant]=actual
     plans=sql(prefix+protected+'\n'+prelude+'LET $rpc_result='+select+' EXPLAIN FULL'+suffix)
     supplier_plan=sql(prefix+protected+'\nLET $rpc_result='+supplier+' EXPLAIN FULL'+suffix)[-2]['result'] if variant=='let' else None
     results['controls'].append({'scope':scope,'kind':kind,'target':target,'after':after,'variant':variant,'returned':len(actual),'keys':[r['key'] for r in actual],'statement_times':[r['time'] for r in rows],'plan':plans[-2]['result'],'supplier_plan':supplier_plan,'query':query})
     save();print('control',scope,kind,target,variant,len(actual),flush=True)
    assert actuals['correlated']==actuals['let'],'complete paged row objects differ'
    page=actuals['let'];all_rows.extend(r['key'] for r in page)
    if not page:break
    after=page[-1]['key']
   assert all_rows==expected,(scope,kind,target,len(all_rows),len(expected))
   assert len(all_rows)==len(set(all_rows)),'duplicate source edge widened membership'
  # The same proposed statement must reject a mismatched, expired or released pin.
  for guard,mutation in [('revision','UPDATE canonical_protections SET revision="wrong" WHERE key="pin-p256";'),('expired','UPDATE canonical_protections SET revision="r",expires_at=0 WHERE key="pin-p256";'),('released','UPDATE canonical_protections SET expires_at=time::micros()+60000000,released=true WHERE key="pin-p256";')]:
   sql(mutation)
   try:sql(query)
   except RuntimeError as error:results['controls'].append({'guard':guard,'refused':True,'error':str(error)})
   else:raise AssertionError('invalid selection guard admitted '+guard)
 results['complete']=True
finally:
 borrow.__exit__(None,None,None);resources.record_drain(resource)
 if results.get('complete'):resources.pin(resource,False);results['cleanup']=resources.reclaim(resource)
 save();print('fixture',database,'cleanup',results.get('cleanup','retained'),flush=True)
