import sys, json, urllib.request, base64
def run(sql, db="probe"):
    req = urllib.request.Request("http://127.0.0.1:39539/sql", data=sql.encode(), method="POST",
        headers={"Authorization": "Basic " + base64.b64encode(b"root:root").decode(),
                 "surreal-ns": "probe", "surreal-db": db, "Accept": "application/json"})
    try:
        return json.loads(urllib.request.urlopen(req).read())
    except urllib.error.HTTPError as e:
        return {"http_error": e.code, "body": e.read().decode()}
if __name__ == "__main__":
    sql = open(sys.argv[1]).read()
    db = sys.argv[2] if len(sys.argv) > 2 else "probe"
    out = run(sql, db)
    if isinstance(out, dict):
        print(json.dumps(out)); sys.exit()
    for i, r in enumerate(out):
        r.pop("time", None); r.pop("type", None)
        print(f"[{i}] " + json.dumps(r))
