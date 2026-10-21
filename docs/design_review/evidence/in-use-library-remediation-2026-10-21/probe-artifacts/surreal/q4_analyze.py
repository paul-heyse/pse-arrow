import sys, json; sys.path.insert(0,'.')
from q import run
def q(sql):
    out = run(sql, "q4")
    if isinstance(out, dict): return out
    r = out[-1]
    if r['status'] != 'OK': return r
    return r['result']
def num(rid): return rid.split(':',1)[1]
# edges out of hub, in edge-id order
edges = q("SELECT id, out, seq FROM deps WHERE in = points:hub ORDER BY id;")
edge_order_targets = [e['out'] for e in edges]
print("hub out-edges:", len(edges), "first 3 edge ids:", [e['id'] for e in edges[:3]])
# 1) traversal as projection with outer LIMIT/START
r = q("SELECT ->deps->points AS o FROM points:hub LIMIT 64 START 0;")
print("1a projection LIMIT64 START0: rows", len(r), "len(o)", len(r[0]['o']))
r = q("SELECT ->deps->points AS o FROM points:hub LIMIT 64 START 64;")
print("1b projection LIMIT64 START64:", r)
proj = q("SELECT VALUE ->deps->points FROM ONLY points:hub;")
print("1c projection order == edge-id order:", proj == edge_order_targets,
      "| == target-id order:", proj == sorted(proj, key=lambda x: int(num(x))))
# 2) FROM traversal paging
pages = []
for start in range(0, 256, 64):
    r = q(f"SELECT VALUE id FROM points:hub->deps->points LIMIT 64 START {start};")
    if not isinstance(r, list): print("2 err", r); break
    pages.append(r)
flat = [x for p in pages for x in p]
print("2 FROM-traversal pages sizes", [len(p) for p in pages], "total", len(flat), "unique", len(set(flat)),
      "| concat == edge-id order:", flat == edge_order_targets,
      "| == unpaged projection:", flat == proj)
full = q("SELECT VALUE id FROM points:hub->deps->points;")
print("2b unpaged FROM-traversal == edge-id order:", full == edge_order_targets, "len", len(full))
# repeatability
reps = [q("SELECT VALUE id FROM points:hub->deps->points LIMIT 64 START 64;") for _ in range(5)]
print("2c 5 repeats identical:", all(x == reps[0] for x in reps))
# 3) edges themselves via traversal
e = q("SELECT VALUE id FROM points:hub->deps LIMIT 64 START 0;")
print("3 hub->deps edge ids LIMIT 64 sorted by id:", e == sorted(e), "== first 64 edge-id order:", e == [x['id'] for x in edges[:64]])
# 4) inverse direction: edges deps:801..1000 created i=0..199 with id 1000-i
inv = q("SELECT VALUE <-deps<-points FROM ONLY points:hub;")
print("4a <-deps<-points first 5:", inv[:5], "last 2:", inv[-2:])
inedges = q("SELECT VALUE id FROM deps WHERE out = points:hub ORDER BY id;")
print("4b inbound edge ids first/last:", inedges[:2], inedges[-2:])
ip = []
for start in range(0, 256, 64):
    ip += q(f"SELECT VALUE id FROM points:hub<-deps<-points LIMIT 64 START {start};")
print("4c paged inverse concat == unpaged:", ip == inv, "len", len(ip), "unique", len(set(ip)))
# 5) ORDER BY with traversal source; WHERE + paging
o = q("SELECT VALUE id FROM points:hub->deps->points ORDER BY id LIMIT 5 START 0;")
print("5 ORDER BY id (record id ordering) first 5:", o)
o2 = q("SELECT VALUE n FROM points:hub->deps->points ORDER BY n LIMIT 5 START 190;")
print("5b ORDER BY n START 190:", o2)
# 6) explain
print("6a EXPLAIN FROM-traversal:", json.dumps(q("SELECT id FROM points:hub->deps->points LIMIT 64 START 64 EXPLAIN;"))[:700])
print("6b EXPLAIN WHERE in=:", json.dumps(q("SELECT id FROM deps WHERE in = points:hub LIMIT 64 EXPLAIN;"))[:500])
# 7) edge-id range paging over relation table with array ids? (keyset) - use edge id cursor
k = q(f"SELECT VALUE out FROM points:hub->deps WHERE id > {edges[63]['id']} LIMIT 64;")
print("7 keyset WHERE id > 64th edge id, LIMIT 64 == edge-id order 64..127:", k == edge_order_targets[64:128])
