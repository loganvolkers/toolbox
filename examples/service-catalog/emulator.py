"""Local Cloudflare Workers Deployments and GitHub repository HTTP emulation."""

import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlsplit

ACCOUNT = "a" * 32
WORKER = f"/client/v4/accounts/{ACCOUNT}/workers/scripts/seikika/deployments"
REPOSITORY = {"id": 42, "name": "seikika", "full_name": "seikika/seikika",
              "default_branch": "main", "archived": False}
HISTORY = [
    {
        "id": "22222222-2222-4222-8222-222222222222",
        "created_on": "2026-09-10T12:00:00.125Z",
        "versions": [
            {"version_id": "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", "percentage": 80},
            {"version_id": "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", "percentage": 20},
        ],
    },
    {
        "id": "11111111-1111-4111-8111-111111111111",
        "created_on": "2026-09-10T12:00:00Z",
        "versions": [{"version_id": "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", "percentage": 100}],
    },
]


class Emulator(BaseHTTPRequestHandler):
    def reply(self, status, body):
        data = json.dumps(body).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        url = urlsplit(self.path)
        query = parse_qs(url.query)
        if url.path.startswith("/client/v4/"):
            if self.headers.get("Authorization") != "Bearer local-cloudflare":
                self.reply(403, {"success": False})
            elif url.path == WORKER and query == {"page": ["1"], "per_page": ["5"]}:
                self.reply(200, {"success": True, "result": {"deployments": HISTORY}})
            elif url.path == WORKER.replace("/seikika/", "/unavailable/"):
                self.reply(503, {"success": False})
            else:
                self.reply(404, {"success": False})
        elif url.path == "/orgs/seikika/repos":
            self.reply(200, [] if query.get("page", ["1"]) != ["1"] else [REPOSITORY])
        elif url.path == "/repositories/42":
            self.reply(200, REPOSITORY)
        elif url.path == "/repos/seikika/seikika/branches/main":
            self.reply(200, {"protected": True})
        elif url.path == "/repos/seikika/seikika/collaborators":
            self.reply(200, [])
        else:
            self.reply(404, {"message": "Not found"})


if __name__ == "__main__":
    with ThreadingHTTPServer(("127.0.0.1", 0), Emulator) as server:
        print(server.server_port, flush=True)
        server.serve_forever()
