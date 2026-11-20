#!/usr/bin/env python3
"""Provision the reference portal's IdP fixtures (fixtures/idp.ndjson) into a SCIM endpoint, the
way an identity provider would: users first, then groups with the Toolbox extension.
Usage: python tools/scim_feed.py http://127.0.0.1:9100/scim/v2 [TOKEN]"""
import json, os, sys, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EXT = "urn:ietf:params:scim:schemas:extension:toolbox:2.0:Group"
base, token = sys.argv[1], (sys.argv[2] if len(sys.argv) > 2 else None)
recs = [json.loads(l) for l in open(os.path.join(ROOT, "fixtures", "idp.ndjson")) if l.strip()]

def put(path, body):
    req = urllib.request.Request(f"{base}/{path}", data=json.dumps(body).encode(), method="PUT",
                                 headers={"content-type": "application/scim+json", **({"authorization": f"Bearer {token}"} if token else {})})
    urllib.request.urlopen(req).read()

for r in recs:
    if r["kind"] == "user":
        d = r["data"]
        put(f"Users/u-{d['login']}", {"schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"], "userName": d["login"],
                                     "name": {"formatted": d["name"]}, "emails": [{"value": d["email"], "primary": True}], "active": True})
for r in recs:
    if r["kind"] == "group":
        d = r["data"]
        ext = {"slug": d["slug"], "githubTeam": d["githubTeam"], **({"parent": d["parent"]} if "parent" in d else {})}
        put(f"Groups/g-{d['slug']}", {"schemas": ["urn:ietf:params:scim:schemas:core:2.0:Group", EXT], "displayName": d["displayName"],
                                     "members": [{"value": f"u-{m}"} for m in d["members"]], EXT: ext})
print(f"provisioned {sum(r['kind'] == 'user' for r in recs)} users and {sum(r['kind'] == 'group' for r in recs)} groups")
