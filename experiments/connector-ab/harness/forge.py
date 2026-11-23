#!/usr/bin/env python3
"""Local Forge mock: Jira Cloud-shaped REST for the connector A/B.

    python3 -I experiments/connector-ab/harness/forge.py --tenant a --port 9743
    python3 -I experiments/connector-ab/harness/forge.py --tenant b --port 9744

Auth: Authorization: Bearer <token> where token is tenant-a-token or tenant-b-token.
Pagination: startAt + maxResults on list endpoints (default maxResults=50).
Phase 3+: GET /rest/api/3/project/{key}/components
"""

from __future__ import annotations

import argparse
import json
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse

HERE = Path(__file__).resolve().parent
FIX = HERE.parent / "fixtures"
TOKENS = {"a": "tenant-a-token", "b": "tenant-b-token"}


class Forge(BaseHTTPRequestHandler):
    tenant: str = "a"
    page_size_default: int = 50

    def log_message(self, fmt, *args):  # quieter
        sys.stderr.write("%s - %s\n" % (self.address_string(), fmt % args))

    def _auth(self) -> bool:
        want = TOKENS[self.tenant]
        got = self.headers.get("Authorization", "")
        return got == f"Bearer {want}"

    def _json(self, code: int, body):
        data = json.dumps(body).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def _load(self, name: str):
        return json.loads((FIX / f"tenant-{self.tenant}" / name).read_text())

    def do_GET(self):  # noqa: N802
        if not self._auth():
            return self._json(401, {"errorMessages": ["Unauthorized"], "errors": {}})
        u = urlparse(self.path)
        q = parse_qs(u.query)
        start = int(q.get("startAt", ["0"])[0])
        max_r = int(q.get("maxResults", [str(self.page_size_default)])[0])
        path = u.path.rstrip("/")

        if path == "/rest/api/3/project":
            vals = self._load("projects.json")
            page = vals[start : start + max_r]
            return self._json(
                200,
                {
                    "self": f"http://127.0.0.1{path}",
                    "maxResults": max_r,
                    "startAt": start,
                    "total": len(vals),
                    "isLast": start + len(page) >= len(vals),
                    "values": page,
                },
            )

        if path == "/rest/api/3/search":
            issues = self._load("issues.json")
            page = issues[start : start + max_r]
            return self._json(
                200,
                {
                    "expand": "names,schema",
                    "startAt": start,
                    "maxResults": max_r,
                    "total": len(issues),
                    "issues": page,
                },
            )

        # /rest/api/3/project/{key}/components  → ['', 'rest', 'api', '3', 'project', key, 'components']
        parts = path.split("/")
        if (
            len(parts) == 7
            and parts[1:5] == ["rest", "api", "3", "project"]
            and parts[6] == "components"
        ):
            key = parts[5]
            comps = self._load("components.json")
            if key not in comps:
                return self._json(
                    404,
                    {"errorMessages": [f"Project {key} not found"], "errors": {}},
                )
            return self._json(200, comps[key])

        return self._json(404, {"errorMessages": [f"Not found: {path}"], "errors": {}})


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--tenant", choices=["a", "b"], required=True)
    p.add_argument("--port", type=int, required=True)
    args = p.parse_args()
    Forge.tenant = args.tenant
    httpd = ThreadingHTTPServer(("127.0.0.1", args.port), Forge)
    print(f"forge tenant={args.tenant} http://127.0.0.1:{args.port} token={TOKENS[args.tenant]}", flush=True)
    httpd.serve_forever()


if __name__ == "__main__":
    main()
