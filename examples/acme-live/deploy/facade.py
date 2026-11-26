#!/usr/bin/env python3
"""GitHub REST façade in front of Forgejo.

The Seikika GitHub and git-files connectors speak GitHub's REST API. Forgejo's API is a
different API with a partial compatibility shim, so this process reads Forgejo and answers
the six calls those connectors make, in GitHub's shape. A push to Forgejo is visible on the
next GET. It does not serve fixtures.
"""

import base64
import json
import os
import urllib.error
import urllib.parse
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

FORGEJO = os.environ["FORGEJO_URL"].rstrip("/")
FORGEJO_TOKEN = os.environ["FORGEJO_TOKEN"]
ORG = os.environ.get("FORGEJO_ORG", "acme")
READER = os.environ["FACADE_TOKEN"]
LISTEN = os.environ.get("FACADE_LISTEN", "0.0.0.0:8091")


def forgejo(path):
    req = urllib.request.Request(
        FORGEJO + path,
        headers={"authorization": f"token {FORGEJO_TOKEN}", "accept": "application/json"},
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            body = resp.read()
            return resp.status, json.loads(body) if body else None
    except urllib.error.HTTPError as err:
        return err.code, None


def all_repos():
    out = []
    page = 1
    while page < 100:
        status, body = forgejo(f"/api/v1/orgs/{ORG}/repos?limit=50&page={page}")
        if status == 404 or not body:
            break
        if status != 200:
            raise RuntimeError(f"forgejo repos HTTP {status}")
        out.extend(body)
        if len(body) < 50:
            break
        page += 1
    return out


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, fmt, *args):
        return

    def send_json(self, status, body):
        raw = json.dumps(body).encode()
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def denied(self):
        accept = self.headers.get("accept", "")
        auth = self.headers.get("authorization", "")
        if self.path == "/healthz":
            return False
        if "application/vnd.github+json" not in accept:
            self.send_json(415, {"message": "Accept must be application/vnd.github+json"})
            return True
        if auth != f"Bearer {READER}":
            self.send_json(401, {"message": "bad credentials"})
            return True
        return False

    def do_GET(self):
        if self.path == "/healthz":
            self.send_json(200, {"ok": True})
            return
        if self.denied():
            return
        path, _, query = self.path.partition("?")
        qs = urllib.parse.parse_qs(query)
        parts = [urllib.parse.unquote(p) for p in path.split("/") if p]
        try:
            self.route(parts, qs)
        except Exception as exc:  # a Forgejo miss is an error, not an empty catalog
            self.send_json(502, {"message": str(exc)})

    def route(self, parts, qs):
        if len(parts) == 3 and parts[0] == "orgs" and parts[2] == "repos":
            self.org_repos(parts[1], qs)
            return
        if len(parts) >= 5 and parts[0] == "repos" and parts[3] == "branches":
            self.branch(parts[1], parts[2], "/".join(parts[4:]))
            return
        if len(parts) == 4 and parts[0] == "repos" and parts[3] == "collaborators":
            self.collaborators(parts[1], parts[2], qs)
            return
        if len(parts) >= 5 and parts[0] == "repos" and parts[3] == "contents":
            self.contents(parts[1], parts[2], "/".join(parts[4:]))
            return
        if parts[:3] == ["orgs", ORG, "installations"] or (
            len(parts) > 2 and parts[0] == "user" and parts[1] == "installations"
        ):
            key = "installations" if parts[0] == "orgs" else "repositories"
            self.send_json(200, {key: []})
            return
        self.send_json(404, {"message": "not found"})

    def page(self, items, qs):
        per = int(qs.get("per_page", ["100"])[0])
        page = int(qs.get("page", ["1"])[0])
        start = max(page - 1, 0) * per
        self.send_json(200, items[start : start + per])

    def org_repos(self, org, qs):
        if org != ORG:
            self.send_json(404, {"message": "not found"})
            return
        mapped = []
        for repo in all_repos():
            mapped.append(
                {
                    "id": repo["id"],
                    "name": repo["name"],
                    "archived": bool(repo.get("archived")),
                    "default_branch": repo.get("default_branch") or "main",
                    "private": bool(repo.get("private")),
                }
            )
        self.page(mapped, qs)

    def branch(self, org, repo, branch):
        status, body = forgejo(f"/api/v1/repos/{org}/{repo}/branches/{urllib.parse.quote(branch)}")
        if status == 404 or body is None:
            self.send_json(404, {"message": "branch not found"})
            return
        if status != 200:
            raise RuntimeError(f"branch HTTP {status}")
        self.send_json(200, {"name": branch, "protected": bool(body.get("protected", False))})

    def collaborators(self, org, repo, qs):
        status, body = forgejo(f"/api/v1/repos/{org}/{repo}/collaborators")
        people = body if status == 200 and isinstance(body, list) else []
        mapped = [{"login": p.get("login", "")} for p in people]
        self.page(mapped, qs)

    def contents(self, org, repo, path):
        quoted = urllib.parse.quote(path)
        status, body = forgejo(f"/api/v1/repos/{org}/{repo}/contents/{quoted}")
        if status == 404 or body is None:
            self.send_json(404, {"message": "not found"})
            return
        if status != 200:
            raise RuntimeError(f"contents HTTP {status}")
        if isinstance(body, list):
            self.send_json(
                200,
                [
                    {"type": item.get("type", "file"), "name": item.get("name"), "path": item.get("path")}
                    for item in body
                ],
            )
            return
        content = body.get("content") or ""
        if body.get("encoding") != "base64":
            content = base64.b64encode(content.encode()).decode()
        self.send_json(
            200,
            {
                "type": "file",
                "encoding": "base64",
                "size": body.get("size") or 0,
                "name": body.get("name") or path.rsplit("/", 1)[-1],
                "path": body.get("path") or path,
                "sha": body.get("sha") or "",
                "content": content,
            },
        )


def main():
    host, port = LISTEN.rsplit(":", 1)
    ThreadingHTTPServer((host, int(port)), Handler).serve_forever()


if __name__ == "__main__":
    main()
