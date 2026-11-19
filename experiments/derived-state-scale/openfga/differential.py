"""Differential test: OpenFGA Check (sidecar, reading the derived-tuple view) vs the SQL permission
view mv_perm. Prints agreement and Check latency. Usage: python3 differential.py STORE_ID PGHOST [N]"""
import http.client, json, random, subprocess, sys, time
store, pghost = sys.argv[1], sys.argv[2]
n = int(sys.argv[3]) if len(sys.argv) > 3 else 2000
def sql(q):
    out = subprocess.run(["psql", "-At", "-X", "-h", pghost, "-p", "54329", "-U", "postgres", "-c", q],
                         check=True, capture_output=True, text=True).stdout
    return [l.split("|") for l in out.splitlines() if l]
random.seed(7)
pos = [(int(u), int(s)) for u, s in sql(f"SELECT user_id, service FROM mv_perm TABLESAMPLE SYSTEM (1) LIMIT {n//2}")]
rnd = [(random.randrange(10000), random.randrange(100000)) for _ in range(n - len(pos))]
pairs = pos + rnd
vals = ",".join(f"({u},{s})" for u, s in pairs)
truth = {(int(u), int(s)): b == "t" for u, s, b in sql(
    f"SELECT p.u, p.s, EXISTS (SELECT 1 FROM mv_perm m WHERE m.user_id = p.u AND m.service = p.s) FROM (VALUES {vals}) p(u, s)")}
c = http.client.HTTPConnection("127.0.0.1", 8080)
lat, agree = [], 0
for u, s in pairs:
    body = json.dumps({"tuple_key": {"user": f"user:{u}", "relation": "can_view_details", "object": f"service:{s}"}})
    t = time.perf_counter()
    c.request("POST", f"/stores/{store}/check", body, {"Content-Type": "application/json"})
    r = json.loads(c.getresponse().read())
    lat.append((time.perf_counter() - t) * 1e3)
    agree += (r["allowed"] == truth[(u, s)])
lat.sort()
print(f"checks={len(pairs)} positives_in_truth={sum(truth.values())} agree={agree} "
      f"p50={lat[len(lat)//2]:.2f}ms p95={lat[int(len(lat)*.95)]:.2f}ms p99={lat[int(len(lat)*.99)]:.2f}ms")
# ListObjects for a user whose visible set is below OpenFGA's default 1000-result cap
u, cnt = sql("SELECT user_id, count(*) FROM mv_perm GROUP BY 1 HAVING count(*) BETWEEN 200 AND 900 ORDER BY 1 LIMIT 1")[0]
t = time.perf_counter()
c.request("POST", f"/stores/{store}/list-objects", json.dumps({"type": "service", "relation": "can_view_details", "user": f"user:{u}"}),
          {"Content-Type": "application/json"})
objs = json.loads(c.getresponse().read())["objects"]
print(f"list_objects user={u} openfga={len(objs)} sql={cnt} time={(time.perf_counter()-t)*1e3:.1f}ms")
