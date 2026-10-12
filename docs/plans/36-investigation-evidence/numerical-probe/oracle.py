from fractions import Fraction as F
from math import sqrt
import json
# Independent exact test construction: distinct rational roots and multiplicity are authored.
fixtures=[('three',[-2,0,3],[1,1,1]),('repeated',[1],[3]),('quadratic',[-1,1],[1,1]),('linear',[2],[1]),('near-critical',[-F(1,2**30),0,F(1,2**30)],[1,1,1])]
out=[]
for name,roots,mults in fixtures:
    c=[F(1)]
    for root,m in zip(roots,mults):
        for _ in range(m):
            n=[F(0)]*(len(c)+1)
            for i,a in enumerate(c): n[i]-=F(root)*a;n[i+1]+=a
            c=n
    for root in roots:
        val=sum(a*F(root)**i for i,a in enumerate(c));assert val==0
    out.append({'name':name,'coefficients':[str(x) for x in c],'distinct_roots':len(roots),'multiplicities':mults})
near=fixtures[-1];assert sum(-1<=r<=1 for r in near[1])==3
assert sum(F(-1,2**31)<=r<=F(1,2**31) for r in near[1])==1
# Numerical rank-method counterexample with stable closed-form singular values.
e=2.0**-30; smax=sqrt(1+e*e/2+sqrt(1+e**4/4));smin=e/smax;tau=.75*e
qrmax=sqrt(1+e*e);qrmin=e/qrmax
svd_rank=int(smax>tau*smax)+int(smin>tau*smax)
qr_rank=int(qrmax>tau*qrmax)+int(qrmin>tau*qrmax)
assert (svd_rank,qr_rank)==(1,2)
limit=64<<20; maxn=max(n for n in range(2000) if 64*n*n+512*n+4096<=limit)
print(json.dumps({'baseline':'zero assertion failures','cubic_authored_oracles':out,'nearcritical_candidate_zero_residual_is_not_completeness':True,'nearcritical_wide_branch_roots':3,'narrow_branch_roots':1,'rank':{'matrix':[[1,1],[0,e]],'relative_cutoff':tau,'singular_values':[smax,smin],'qr_diagonal':[qrmax,qrmin],'svd_rank':svd_rank,'qr_diagonal_rank':qr_rank},'current_64MiB_factor_allowance_max_dimension':maxn},indent=2))
