import base64, concurrent.futures, json, os, pathlib, socket, subprocess, tempfile, time, urllib.request
# Installation location is configurable; the executed receipt records the exact binary.
B=os.environ.get('PSE_PLAN36_SURREAL_BINARY',str(pathlib.Path.home()/'.local/share/pse-arrow/tools/surreal/v3.3.0/linux-amd64/surreal'))
D=pathlib.Path(tempfile.mkdtemp(prefix='plan36-coordination-'))
s=socket.socket(); s.bind(('127.0.0.1',0)); port=s.getsockname()[1]; s.close()
log=(D/'server.log').open('w')
p=subprocess.Popen([B,'start','--bind',f'127.0.0.1:{port}','--user','probe','--pass','probe','--log','warn',f'rocksdb://{D}/store'],stdout=log,stderr=log)
H={'Accept':'application/json','Authorization':'Basic '+base64.b64encode(b'probe:probe').decode(),'Surreal-NS':'plan36','Surreal-DB':'probe'}
def sql(q,check=True):
    req=urllib.request.Request(f'http://127.0.0.1:{port}/sql',q.encode(),H,method='POST')
    with urllib.request.urlopen(req,timeout=30) as r: data=json.load(r)
    if check and any(x.get('status')!='OK' for x in data): raise RuntimeError(data)
    return data
out={'binary':B,'version':subprocess.check_output([B,'version'],text=True).strip(),'directory':str(D),'command':p.args,'plans':{},'controls':{}}
try:
    for _ in range(80):
        if p.poll() is not None: raise RuntimeError((D/'server.log').read_text())
        try: sql('RETURN 1;'); break
        except OSError: time.sleep(.1)
    else: raise RuntimeError('startup timeout')
    sql('DEFINE NAMESPACE plan36; USE NS plan36; DEFINE DATABASE probe; USE DB probe;')
    sql('DEFINE TABLE canonical_memberships SCHEMALESS; DEFINE INDEX membership_pages ON canonical_memberships FIELDS problem,key; DEFINE INDEX membership_scope_pages ON canonical_memberships FIELDS problem,scope,key; DEFINE INDEX membership_version ON canonical_memberships FIELDS version; DEFINE INDEX membership_logical ON canonical_memberships FIELDS problem,logical,from_sequence; DEFINE TABLE canonical_runs SCHEMALESS; DEFINE INDEX run_order ON canonical_runs FIELDS problem,sequence UNIQUE; DEFINE INDEX run_terminal_selection ON canonical_runs FIELDS problem,terminal_class,sequence; DEFINE TABLE canonical_result_seeds SCHEMALESS; DEFINE INDEX result_seed_eligibility ON canonical_result_seeds FIELDS layout,preparation,backend,run_sequence,attempt_generation,step; DEFINE TABLE canonical_result_retirements SCHEMALESS; DEFINE TABLE canonical_attempts SCHEMALESS; DEFINE TABLE ack SCHEMALESS; DEFINE INDEX ack_lookup ON ack FIELDS problem,descriptor,store UNIQUE;')
    members=[]
    for gen in range(11):
        for i in range(128):
            row={'id':f'canonical_memberships:m{gen:02}-{i:03}','problem':'p','scope':'s','key':f'{i:03}-{gen:02}','logical':f'v{i:03}','version':f'v{i:03}-{gen:02}','from_sequence':gen}
            if gen<10: row['to_sequence']=gen+1
            members.append(row)
    for i in range(0,len(members),128): sql('INSERT INTO canonical_memberships '+json.dumps(members[i:i+128])+' RETURN NONE;')
    runs=[{'id':f'canonical_runs:r{i:04}','problem':'p','key':f'r{i:04}','sequence':i,**({'terminal_attempt':f'a{i:04}','terminal_class':'completed'} if i%10==0 else {})} for i in range(1000)]
    for i in range(0,1000,100): sql('INSERT INTO canonical_runs '+json.dumps(runs[i:i+100])+' RETURN NONE;')
    sql('CREATE canonical_attempts:a0 SET terminal=true, terminal_eligible=true; CREATE canonical_problems:p SET sequence=10; DEFINE INDEX membership_head_pages ON canonical_memberships FIELDS problem,to_sequence,key; DEFINE INDEX membership_scope_head_pages ON canonical_memberships FIELDS problem,scope,to_sequence,key;')
    seeds=[{'id':f'canonical_result_seeds:s{i:03}','key':f's{i:03}','layout':'l','preparation':'p','backend':'b','attempt':f'a{i}','run':f'r{i}','run_sequence':i,'attempt_generation':0,'step':0,'terminal_eligible':i==0} for i in range(65)]
    sql('INSERT INTO canonical_result_seeds '+json.dumps(seeds)+' RETURN NONE;')
    acks=[{'id':f'ack:k{i:04}','problem':'p','descriptor':f'd{i:04}','store':'store'} for i in range(1000)]
    for i in range(0,1000,100): sql('INSERT INTO ack '+json.dumps(acks[i:i+100])+' RETURN NONE;')
    queries={
     'membership-head':"SELECT * FROM canonical_memberships WHERE problem='p' AND to_sequence=NONE AND key>'' AND scope='s' ORDER BY key LIMIT 64",
     'membership-head-scoped':"SELECT * FROM canonical_memberships WITH INDEX membership_scope_head_pages WHERE problem='p' AND scope='s' AND to_sequence=NONE AND key>'' ORDER BY key LIMIT 64",
     'membership-current':"SELECT * FROM canonical_memberships WHERE problem='p' AND scope='s' AND key>'' AND from_sequence<=10 AND (to_sequence=NONE OR to_sequence>10) ORDER BY key LIMIT 64",
     'membership-historical':"SELECT * FROM canonical_memberships WHERE problem='p' AND scope='s' AND key>'' AND from_sequence<=5 AND (to_sequence=NONE OR to_sequence>5) ORDER BY key LIMIT 64",
     'membership-exact-version':"SELECT * FROM canonical_memberships WHERE problem='p' AND version='v064-05' AND from_sequence<=5 AND (to_sequence=NONE OR to_sequence>5) LIMIT 1",
     'latest-results':"SELECT VALUE {run:key,attempt:terminal_attempt} FROM canonical_runs WHERE problem='p' AND terminal_attempt!=NONE AND terminal_class!=NONE ORDER BY sequence DESC LIMIT 1",
     'seed-original':"SELECT * FROM canonical_result_seeds WHERE layout='l' AND preparation='p' AND backend='b' AND (SELECT VALUE key FROM canonical_result_retirements WHERE run=$parent.run LIMIT 1)=[] ORDER BY run_sequence DESC,attempt_generation DESC,step DESC LIMIT 64",
     'seed-eligible':"SELECT * FROM canonical_result_seeds WHERE layout='l' AND preparation='p' AND backend='b' AND type::record('canonical_attempts',attempt).terminal_eligible=true AND (SELECT VALUE key FROM canonical_result_retirements WHERE run=$parent.run LIMIT 1)=[] ORDER BY run_sequence DESC,attempt_generation DESC,step DESC,key DESC LIMIT 64",
     'ack-exact-record':"SELECT * FROM ONLY ack:k0999",
     'ack-exact-index':"SELECT * FROM ack WHERE problem='p' AND descriptor='d0999' AND store='store' LIMIT 1",
    }
    for name,q in queries.items():
        data=sql(q+';')[0]['result']; out['plans'][name]={'query':q,'rows':len(data) if isinstance(data,list) else 1,'result_keys':[r.get('key',r.get('run',r.get('version'))) for r in data] if isinstance(data,list) else None,'explain':sql(q+' EXPLAIN FULL;')[0]['result']}
    out['controls']['seed-original-masks-eligible']=all(r!='s000' for r in out['plans']['seed-original']['result_keys'])
    out['controls']['seed-eligible-selects-older']=out['plans']['seed-eligible']['result_keys']==['s000']
    for name in ['membership-current','membership-historical','membership-head']:
        q=queries[name]; noindex=q.replace('FROM canonical_memberships','FROM canonical_memberships WITH NOINDEX')
        out['controls'][name+'-noindex-equality']=sql(q+';')[0]['result']==sql(noindex+';')[0]['result']
    for name in ['membership-head','membership-head-scoped']:
        out['controls'][name+'-interval-equality']=sql(queries[name]+';')[0]['result']==sql(queries['membership-current']+';')[0]['result']
    out['controls']['guarded-head']=sql("BEGIN; LET $head=SELECT * FROM ONLY canonical_problems:p FOR UPDATE; IF $head.sequence!=10 { THROW 'not selected head'; }; SELECT * FROM canonical_memberships WITH INDEX membership_scope_head_pages WHERE problem='p' AND scope='s' AND to_sequence=NONE AND key>'' ORDER BY key LIMIT 64; COMMIT;")
    # Exact absence registration: delayed read-then-write must fail after another creator commits.
    sql('DEFINE TABLE race SCHEMALESS; CREATE race:fence SET value=0;')
    with concurrent.futures.ThreadPoolExecutor(2) as ex:
        fut=ex.submit(sql,"BEGIN; SELECT * FROM ONLY race:absent FOR UPDATE; SLEEP 800ms; CREATE race:other SET value=1; COMMIT;",False)
        time.sleep(.2); creator=sql('CREATE race:absent SET value=1;'); delayed=fut.result()
    out['controls']['absent-registration-response']=delayed
    out['controls']['absent-registration-refuses']=any(x.get('status')=='ERR' for x in delayed)
    # Shared unchanged read authority plus disjoint writes both commit.
    with concurrent.futures.ThreadPoolExecutor(2) as ex:
        fs=[ex.submit(sql,f'BEGIN; SELECT * FROM ONLY race:fence FOR UPDATE; SLEEP 300ms; CREATE race:disjoint{i} SET value=1; COMMIT;',False) for i in range(2)]
        out['controls']['disjoint-registered-readers']=[f.result() for f in fs]
    # Abstract proposed protocols, isolated generic authority rows; not emitted app queries.
    def pair(label,first,second):
        with concurrent.futures.ThreadPoolExecutor(2) as ex:
            a=ex.submit(sql,first,False); b=ex.submit(sql,second,False)
            out['controls'][label]={'a':a.result(),'b':b.result()}
    for order,ca,co in [('creator-first',100,700),('collector-first',700,100)]:
        sql('DELETE race; CREATE race:creator SET v=0; CREATE race:reclaim SET v=0;')
        pair(order,f"BEGIN; SELECT * FROM ONLY race:reclaim FOR UPDATE; SELECT * FROM ONLY race:root FOR UPDATE; SLEEP {ca}ms; CREATE race:root SET v=1; UPDATE race:creator SET v=1; COMMIT;",f"BEGIN; SELECT * FROM ONLY race:creator FOR UPDATE; SELECT * FROM race WHERE id=race:root; SLEEP {co}ms; UPDATE race:reclaim SET v=1; COMMIT;")
    for order,ap,cn in [('append-first',100,700),('cancel-first',700,100)]:
        sql('DELETE race; CREATE race:cancel SET cancelled=false;')
        pair(order,f"BEGIN; SELECT * FROM ONLY race:cancel FOR UPDATE; SLEEP {ap}ms; CREATE race:append SET v=1; COMMIT;",f"BEGIN; SELECT * FROM ONLY race:cancel FOR UPDATE; SLEEP {cn}ms; UPDATE race:cancel SET cancelled=true; COMMIT;")
        out['controls'][order]['retained_append']=sql('SELECT * FROM race:append;')[0]['result']
    sql('DELETE race; CREATE race:floor SET closed_through=0;')
    pair('grant-versus-settlement',"BEGIN; SELECT * FROM ONLY race:floor FOR UPDATE; SELECT * FROM ONLY race:grant FOR UPDATE; SLEEP 700ms; CREATE race:grant SET nonce=1; COMMIT;","BEGIN; SELECT * FROM ONLY race:grant FOR UPDATE; SLEEP 100ms; UPDATE race:floor SET closed_through=1; DELETE race:grant; COMMIT;")
    out['controls']['late-closed-grant']=sql("BEGIN; LET $f=SELECT * FROM ONLY race:floor FOR UPDATE; IF $f.closed_through>=1 { THROW 'closed grant'; }; CREATE race:grant SET nonce=1; COMMIT;",False)
    out['fixture']={'memberships':1408,'current':128,'superseded':1280,'runs':1000,'seeds':65,'acks':1000,'scope':'schema-compatible query shapes, not generated canonical functions'}
finally:
    p.terminate()
    try: out['server_exit']=p.wait(timeout=10)
    except subprocess.TimeoutExpired: p.kill(); out['server_exit']=p.wait(timeout=10)
    log.close(); (D/'receipt.json').write_text(json.dumps(out,indent=2)); print(json.dumps({'receipt':str(D/'receipt.json'),'controls':out.get('controls'),'server_exit':out.get('server_exit')}))
