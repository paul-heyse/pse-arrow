"""Optional full-package probes; dependency absence is BLOCKED, never PASS.

No installation is attempted. Numerical workers run in separate subprocesses.
The workers were not exercised in this session because both packages were absent.
Version mismatches remain blocked unless the fixture target is deliberately changed.
This script is a small starting probe set, not all 38 inherited native probes.
"""
from __future__ import annotations
import argparse,hashlib,importlib.metadata,importlib.util,json,math,platform,subprocess,sys,traceback
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
F=json.loads((ROOT/'fixtures.json').read_text())
SPECS=[('NPkg-01','thermo','nrtl'),('NPkg-02','thermo','explicit_flash'),
       ('NPkg-03','CoolProp','water_inverse'),('NPkg-04','CoolProp','saturation'),
       ('NPkg-05','CoolProp','mixture_inverse'),('NPkg-06','CoolProp','single_phase_derivative')]

def assert_close(a,b,atol,rtol=0):
    if not math.isfinite(float(a)) or abs(a-b)>atol+rtol*abs(b):raise AssertionError((a,b,atol,rtol))

def worker(package,kind):
    manifest={'package':package,'version':importlib.metadata.version(package),'python':sys.version}
    if package=='thermo':
        import thermo
        manifest['core_file_sha256']=hashlib.sha256(Path(thermo.__file__).read_bytes()).hexdigest()
        if kind=='nrtl':
            from thermo.nrtl import NRTL_gammas
            d=F['source_examples']['nrtl'];g=NRTL_gammas(d['xs'],d['taus'],d['alphas'])
            for v,e in zip(g,d['gammas']):assert_close(v,e,1e-12)
            return {'manifest':manifest,'gammas':g,'scope':'installed function, documentation regression only'}
        from thermo import ChemicalConstantsPackage, PropertyCorrelationsPackage, HeatCapacityGas, CEOSGas, CEOSLiquid, PRMIX, FlashVL
        # Explicit fixture, no database selection. Constant ideal Cp values are synthetic,
        # so this cannot be interpreted as validated real-hydrocarbon calorics.
        data=dict(Tcs=[190.564,305.322],Pcs=[4599200.,4872200.],omegas=[.01142,.0995],MWs=[16.04246,30.06904])
        constants=ChemicalConstantsPackage(**data)
        hcs=[HeatCapacityGas(poly_fit=(100.,1000.,[v])) for v in [40.,55.]]
        corr=PropertyCorrelationsPackage(constants=constants,HeatCapacityGases=hcs,skip_missing=True)
        eos=dict(Tcs=data['Tcs'],Pcs=data['Pcs'],omegas=data['omegas'],kijs=[[0.,0.],[0.,0.]])
        gas=CEOSGas(PRMIX,eos_kwargs=eos,HeatCapacityGases=hcs)
        liquid=CEOSLiquid(PRMIX,eos_kwargs=eos,HeatCapacityGases=hcs)
        flash=FlashVL(constants,corr,gas=gas,liquid=liquid)
        s=flash.flash(T=220.,P=2e6,zs=[.5,.5]);h=s.H();ent=s.S()
        a=flash.flash(P=2e6,H=h,zs=[.5,.5]);b=flash.flash(P=2e6,S=ent,zs=[.5,.5])
        assert_close(a.H(),h,1e-5,1e-8);assert_close(b.S(),ent,1e-7,1e-8)
        return {'manifest':manifest,'resolved_constants':data,'ideal_Cp_J_per_mol_K':[40,55],'kijs':eos['kijs'],'TP':[220,2e6],'H':h,'S':ent,'PH_T':a.T,'PS_T':b.T,'scope':'installed explicit cubic assembly; synthetic Cp and internal inverse consistency, not physical validation'}
    import CoolProp
    import CoolProp.CoolProp as CP
    manifest.update(gitrevision=CP.get_global_param_string('gitrevision'),config=CP.get_config_as_json_string(),core_file_sha256=hashlib.sha256(Path(CP.__file__).read_bytes()).hexdigest())
    if kind in ['water_inverse','mixture_inverse']:
        cases=[('Water',300.,3e6),('Water',500.,1e5),('Water',700.,3e6)] if kind=='water_inverse' else [('HEOS::Methane[0.5]&Ethane[0.5]',220.,2e6)]
        records=[]
        for fluid,T,P in cases:
            h=CP.PropsSI('Hmolar','T',T,'P',P,fluid);s=CP.PropsSI('Smolar','T',T,'P',P,fluid)
            th=CP.PropsSI('T','P',P,'Hmolar',h,fluid);ts=CP.PropsSI('T','P',P,'Smolar',s,fluid)
            assert_close(th,T,1e-5);assert_close(ts,T,1e-5)
            hh=CP.PropsSI('Hmolar','T',th,'P',P,fluid);ss=CP.PropsSI('Smolar','T',ts,'P',P,fluid)
            assert_close(hh,h,1e-4,1e-8);assert_close(ss,s,1e-7,1e-8)
            records.append({'fluid':fluid,'T':T,'P':P,'H':h,'S':s,'PH_T':th,'PS_T':ts})
        return {'manifest':manifest,'cases':records,'scope':'installed selected-backend roundtrip, no external physical reference'}
    if kind=='saturation':
        p=101325.;l=CP.PropsSI('Hmass','P',p,'Q',0,'Water');v=CP.PropsSI('Hmass','P',p,'Q',1,'Water');h=l+.4*(v-l);q=CP.PropsSI('Q','P',p,'Hmass',h,'Water');assert_close(q,.4,1e-7)
        return {'manifest':manifest,'HL':l,'HV':v,'target_H':h,'quality_recovered':q,'scope':'specified pure-fluid allocation, not TP uniqueness'}
    T=300.;P=3e6;d=CP.PropsSI('d(Hmass)/d(T)|P','T',T,'P',P,'Water');dt=.001
    fd=(CP.PropsSI('Hmass','T',T+dt,'P',P,'Water')-CP.PropsSI('Hmass','T',T-dt,'P',P,'Water'))/(2*dt)
    assert_close(d,fd,1e-3,1e-5);return {'manifest':manifest,'derivative':d,'finite_difference':fd,'scope':'selected homogeneous derivative only'}

if __name__=='__main__':
    ap=argparse.ArgumentParser();ap.add_argument('--worker',nargs=2);args=ap.parse_args()
    if args.worker:
        try:print(json.dumps({'status':'PASS','detail':worker(*args.worker)},allow_nan=False))
        except Exception as e:print(json.dumps({'status':'FAIL','error':repr(e),'traceback':traceback.format_exc()}));sys.exit(1)
    else:
        results=[]
        for tid,pkg,kind in SPECS:
            r={'id':tid,'package':pkg,'probe':kind,'target_version':F['native_targets'][pkg]}
            try:version=importlib.metadata.version(pkg)
            except importlib.metadata.PackageNotFoundError:
                r.update(status='BLOCKED_DEPENDENCY',reason='package not installed in current execution environment',executed=False);results.append(r);continue
            if version!=F['native_targets'][pkg]:
                r.update(status='BLOCKED_VERSION',found_version=version,executed=False);results.append(r);continue
            try:
                p=subprocess.run([sys.executable,str(Path(__file__).resolve()),'--worker',pkg,kind],capture_output=True,text=True,timeout=45)
                lines=p.stdout.strip().splitlines();res=json.loads(lines[-1]) if lines else {'status':'PROVIDER_PROCESS_FAILURE'}
                r.update(res,executed=True,returncode=p.returncode,stdout=p.stdout,stderr=p.stderr)
            except subprocess.TimeoutExpired as e:r.update(status='TIMEOUT',executed=True,reason='probe subprocess exceeded its declared 45-second execution budget')
            except Exception as e:r.update(status='HARNESS_ERROR',executed=False,error=repr(e))
            results.append(r)
        report={'kind':'optional_full_package_probe_dispatch','tests':results,'environment':{'python':sys.version,'platform':platform.platform()},'executed':sum(r['executed'] for r in results),'passed':sum(r['status']=='PASS' for r in results),'blocked':sum(r['status'].startswith('BLOCKED') for r in results),'limitations':['Worker branches not verified when dependency absent','Does not constitute all native probe coverage','No auto-installation or backend substitution','No empirical validation']}
        (ROOT/'results'/'native_probe_dispatch.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(report,indent=2));sys.exit(0 if report['executed']==len(results) and report['passed']==len(results) else 2)
