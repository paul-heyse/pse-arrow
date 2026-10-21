import sys, json, time; sys.path.insert(0,'.')
from q import run
def q(sql):
    out = run(sql, "q4"); return out[-1]
print(json.dumps(q("SELECT VALUE out FROM points:hub->deps WHERE id > deps:5 LIMIT 64 EXPLAIN;")['result'])[:600])
t=time.time()

for start in (0, 10000, 19936):
    ts=[]
    for _ in range(5):
        r = run(f"SELECT VALUE id FROM points:hub2->deps LIMIT 64 START {start};", "q4")[-1]
        ts.append(float(r['time'].rstrip('µms')) * (1e-3 if r['time'].endswith('µs') else (1 if r['time'].endswith('ms') else 1e3)))
    print(f"START {start}: n={len(r['result'])} server time ms min={min(ts):.2f} med={sorted(ts)[2]:.2f}")
