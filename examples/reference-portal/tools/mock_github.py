#!/usr/bin/env python3
"""A mock of the GitHub REST endpoints the github and git-files connectors call, serving the
reference portal's fixtures in GitHub's response shapes. If the connectors are right, they turn
these responses back into exactly the records in fixtures/github.ndjson.

Usage: python tools/mock_github.py PORT    (org: acme)
Endpoints: /orgs/acme/repos, /repositories/{id}, /repos/acme/{repo}/branches/{b},
/repos/acme/{repo}/collaborators, /repos/acme/{repo}/contents/{path}, /orgs/acme/installations,
/user/installations/{id}/repositories, /apps/{slug}. Pages beyond 1 are empty.
State can be changed with POST /_mock/repos/{name} {"admins": [...], ...} (used by refresh tests).
"""
import base64, json, os, sys, threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlparse, parse_qs

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
records = [json.loads(l) for l in open(os.path.join(ROOT, "fixtures", "github.ndjson")) if l.strip()]
catalog = [json.loads(l) for l in open(os.path.join(ROOT, "fixtures", "git-files.ndjson")) if l.strip()]
LOCK = threading.Lock()
repos = {r["data"]["name"]: dict(r["data"]) for r in records if r["kind"] == "repository"}
codeowners = {r["data"]["repo"]: r["data"]["rules"] for r in records if r["kind"] == "codeowners"}
apps = {r["data"]["slug"]: r["data"] for r in records if r["kind"] == "app"}
installs = [r["data"] for r in records if r["kind"] == "installation"]
files = {c["data"]["repo"]: c["data"]["content"] for c in catalog}

def repo_json(r):
    return {"id": r["id"], "name": r["name"], "full_name": f"acme/{r['name']}", "archived": r["archived"], "default_branch": "main"}

def b64(text):
    return base64.b64encode(text.encode()).decode()

class H(BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def send(self, code, body):
        data = json.dumps(body).encode()
        self.send_response(code)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_POST(self):
        parts = urlparse(self.path).path.strip("/").split("/")
        body = json.loads(self.rfile.read(int(self.headers.get("content-length", 0))) or b"{}")
        if parts[:2] == ["_mock", "repos"] and parts[2] in repos:
            with LOCK:
                repos[parts[2]].update(body)
            return self.send(200, repos[parts[2]])
        self.send(404, {"message": "Not Found"})

    def do_GET(self):
        u = urlparse(self.path)
        q = parse_qs(u.query)
        page = int(q.get("page", ["1"])[0])
        p = u.path.strip("/").split("/")
        with LOCK:
            if p == ["orgs", "acme", "repos"]:
                return self.send(200, [repo_json(r) for r in repos.values()] if page == 1 else [])
            if p[0] == "repositories" and len(p) == 2:
                r = next((r for r in repos.values() if str(r["id"]) == p[1]), None)
                return self.send(200, repo_json(r)) if r else self.send(404, {"message": "Not Found"})
            if p[:2] == ["repos", "acme"] and len(p) >= 4 and p[2] in repos:
                r = repos[p[2]]
                if p[3] == "branches":
                    return self.send(200, {"name": p[4], "protected": r["default_branch_protected"]})
                if p[3] == "collaborators":
                    return self.send(200, [{"login": a, "permissions": {"admin": True}} for a in r["admins"]] if page == 1 else [])
                if p[3] == "contents":
                    path = "/".join(p[4:])
                    if path == ".github/CODEOWNERS" and p[2] in codeowners:
                        text = "".join(f"{x['pattern']} {' '.join(x['owners'])}\n" for x in codeowners[p[2]])
                        return self.send(200, {"type": "file", "path": path, "content": b64(text), "encoding": "base64"})
                    if path == "catalog-info.yaml" and p[2] in files:
                        return self.send(200, {"type": "file", "path": path, "content": b64(json.dumps(files[p[2]])), "encoding": "base64"})
                    return self.send(404, {"message": "Not Found"})
            if p == ["orgs", "acme", "installations"]:
                items = [{"id": i["id"], "app_slug": i["app_slug"], "repository_selection": i["repository_selection"]} for i in installs]
                return self.send(200, {"total_count": len(items), "installations": items if page == 1 else []})
            if p[:2] == ["user", "installations"] and len(p) == 4 and p[3] == "repositories":
                i = next((i for i in installs if str(i["id"]) == p[2]), None)
                if not i:
                    return self.send(404, {"message": "Not Found"})
                rs = [repo_json(repos[n]) for n in i["repositories"] if n in repos]
                return self.send(200, {"total_count": len(rs), "repositories": rs if page == 1 else []})
            if p[0] == "apps" and len(p) == 2 and p[1] in apps:
                a = apps[p[1]]
                return self.send(200, {"slug": a["slug"], "owner": {"login": a["owner"]}, "permissions": a["permissions"]})
        self.send(404, {"message": "Not Found"})

if __name__ == "__main__":
    ThreadingHTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
