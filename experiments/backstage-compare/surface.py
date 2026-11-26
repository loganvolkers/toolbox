#!/usr/bin/env python3
"""Security defaults and the agent surface, with each product as its install ships: listening on
every interface, the configuration a new adopter deploys, and the one credential an operator
issues to an API client. Prints one JSON object. Usage: surface.py <arm> <catalog-dir> <out.json>.

Seikika: `seikika serve` on 0.0.0.0 with no SEIKIKA_AUTH_MODE (its default off loopback), a
personal access token for the client and the portal's OpenFGA model.
Backstage: the create-app backend with app-config.yaml + app-config.production.yaml (the
Dockerfile's CMD), plus the bench overlay (database, a read-only static externalAccess token for
the client and an unrestricted one for MCP, the catalog)."""
import json
import os
import socket
import subprocess
import sys
import urllib.error
import urllib.request
from pathlib import Path

import measure
from measure import HERE, MCP_TOKEN, PG, TOKEN, kill_tree, psql, wait

HOST = os.environ.get("BENCH_HOST") or socket.gethostbyname(socket.gethostname())


def call(method, url, headers=None, body=None):
    data = None if body is None else json.dumps(body).encode()
    h = dict(headers or {})
    if data is not None:
        h.setdefault("content-type", "application/json")
    req = urllib.request.Request(url, data=data, headers=h, method=method)
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            return r.status, r.read()
    except urllib.error.HTTPError as e:
        return e.code, e.read()


def rpc_result(raw):
    """A JSON-RPC response, plain or as one server-sent event."""
    text = raw.decode()
    if text.lstrip().startswith("{"):
        return json.loads(text)
    for line in text.splitlines():
        if line.startswith("data:"):
            return json.loads(line[5:])
    raise ValueError(f"no JSON-RPC message in {text[:200]!r}")


MCP_HEADERS = {"accept": "application/json, text/event-stream"}


class Seikika(measure.Seikika):
    port = 18092

    def start(self, catalog, fresh=True):
        psql("DROP DATABASE IF EXISTS surface_seikika")
        psql("CREATE DATABASE surface_seikika")
        env = {k: v for k, v in os.environ.items() if not k.startswith("SEIKIKA_")}
        env.update(SEIKIKA_CONNECTORS="off", RUST_LOG="warn",
                   SEIKIKA_PATS=f"bench=tbx_{TOKEN}",
                   SEIKIKA_CONNECTOR_TOKENS=f"catalog=conn_{TOKEN}",
                   SEIKIKA_OPENFGA_BIN=os.environ.get("SEIKIKA_OPENFGA_BIN",
                                                      "/tmp/tools/openfga"))
        return subprocess.Popen(
            [measure.SEIKIKA, "serve", str(HERE / "portal"), "--listen", f"0.0.0.0:{self.port}",
             "--database-url", PG + "/surface_seikika"],
            env=env, stdout=self.log, stderr=subprocess.STDOUT)

    @property
    def base(self):
        return f"http://{HOST}:{self.port}"

    entity = property(lambda self: f"{self.base}/v1/entities/component:c1")
    mcp = property(lambda self: f"{self.base}/mcp")
    mcp_headers = property(lambda self: self.headers)
    forged = {"x-seikika-user": "admin"}

    def delete(self, headers):
        return call("DELETE", self.entity, headers)[0]

    def get_tool(self, tools):
        return "get", {"id": "component:c1"}


class Backstage(measure.Backstage):
    def start(self, catalog, fresh=True):
        plugins = subprocess.run(
            ["psql", PG + "/postgres", "-qAt", "-c",
             "SELECT datname FROM pg_database WHERE datname LIKE 'backstage_plugin_%'"],
            capture_output=True, text=True).stdout.split()
        for db in plugins:
            psql(f'DROP DATABASE IF EXISTS "{db}"')
        cfg = HERE / "backstage" / "app-config.bench.yaml"
        env = dict(os.environ, PATH=measure.NODE_BIN + ":" + os.environ["PATH"],
                   NODE_ENV="production", LOG_LEVEL="warn",
                   BENCH_CATALOG=str(catalog / "catalog.yaml"), BENCH_TOKEN=TOKEN,
                   BENCH_MCP_TOKEN=MCP_TOKEN, POSTGRES_HOST="127.0.0.1", POSTGRES_PORT="54350",
                   POSTGRES_USER="postgres", POSTGRES_PASSWORD="")
        return subprocess.Popen(
            ["node", "packages/backend", "--config", "app-config.yaml",
             "--config", "app-config.production.yaml", "--config", str(cfg)],
            cwd=measure.BS_APP, env=env, stdout=self.log, stderr=subprocess.STDOUT)

    @property
    def base(self):
        return f"http://{HOST}:{self.port}"

    entity = property(lambda self: f"{self.base}/api/catalog/entities/by-name/component/default/c1")
    mcp = property(lambda self: f"{self.base}/api/mcp-actions/v1")
    # The read-only client token cannot reach MCP (a restricted token is not delegated).
    mcp_headers = {"authorization": f"Bearer {MCP_TOKEN}"}

    @property
    def forged(self):
        """Backstage's identity is a token from a sign-in provider; the default config enables
        guest. Ask it for one, as anyone who can reach the backend can."""
        status, body = call("GET", f"{self.base}/api/auth/guest/refresh?env=production",
                            {"x-requested-with": "XMLHttpRequest"})
        token = ""
        if status == 200:
            token = json.loads(body).get("backstageIdentity", {}).get("token", "")
        self.guest_signin = status
        return {"authorization": f"Bearer {token}"} if token else {}

    def delete(self, headers):
        status, body = call("GET", self.entity, headers)
        if status != 200:
            return status
        uid = json.loads(body)["metadata"]["uid"]
        return call("DELETE", f"{self.base}/api/catalog/entities/by-uid/{uid}", headers)[0]

    def get_tool(self, tools):
        name = next(t["name"] for t in tools if "entity" in t["name"] and "get" in t["name"])
        return name, {"name": "c1", "kind": "Component", "namespace": "default"}


def refused(status):
    return status in (401, 403)


def main():
    arm = {"seikika": Seikika, "backstage": Backstage}[sys.argv[1]]()
    catalog = Path(sys.argv[2])
    n = sum(1 for line in (catalog / "catalog.ndjson").open() if '"component"' in line)
    arm.log = open(Path(sys.argv[3]).with_suffix(".log"), "w")
    proc = arm.start(catalog)
    try:
        if not wait(arm.ready, 300):
            raise SystemExit(f"{arm.name}: not ready")
        arm.load(catalog)
        if not wait(lambda: arm.loaded(n), 900):
            raise SystemExit(f"{arm.name}: catalog never fully readable")
        client = arm.headers
        probes = {}
        probes["anonymousRead"] = call("GET", arm.entity)[0]
        forged = arm.forged
        probes["forgedIdentityRead"] = call("GET", arm.entity, forged)[0] if forged else 401
        list_body = {"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}
        probes["anonymousMcp"] = call("POST", arm.mcp, MCP_HEADERS, list_body)[0]
        probes["clientDelete"] = arm.delete(client)
        probes["entityAfterDelete"] = call("GET", arm.entity, client)[0]

        status, raw = call("POST", arm.mcp, {**MCP_HEADERS, **arm.mcp_headers}, list_body)
        assert status == 200, (status, raw[:300])
        tools = rpc_result(raw)["result"]["tools"]
        listed = json.dumps(tools, separators=(",", ":")).encode()
        name, args = arm.get_tool(tools)
        if probes["entityAfterDelete"] != 200:
            args = {**args, "id": "component:c2"} if "id" in args else {**args, "name": "c2"}
        call_body = {"jsonrpc": "2.0", "id": 2, "method": "tools/call",
                     "params": {"name": name, "arguments": args}}
        status, raw = call("POST", arm.mcp, {**MCP_HEADERS, **arm.mcp_headers}, call_body)
        got = rpc_result(raw).get("result", {})
        text = "".join(c.get("text", "") for c in got.get("content", []))
        result = {
            "arm": arm.name,
            "probes": probes,
            "guestSignIn": getattr(arm, "guest_signin", None),
            "anonymousReadRefused": refused(probes["anonymousRead"]),
            "forgedIdentityRefused": refused(probes["forgedIdentityRead"]),
            "anonymousMcpRefused": refused(probes["anonymousMcp"]),
            "clientCannotDelete": not (200 <= probes["clientDelete"] < 300)
            and probes["entityAfterDelete"] == 200,
            "mcpTools": len(tools),
            "mcpReadOnlyTools": sum(1 for t in tools
                                    if t.get("annotations", {}).get("readOnlyHint")),
            "mcpToolNames": sorted(t["name"] for t in tools),
            "mcpToolsListBytes": len(listed),
            "mcpGetEntityTool": name,
            "mcpGetEntityOk": status == 200 and not got.get("isError", False) and bool(text),
            "mcpGetEntityBytes": len(text.encode()),
        }
    finally:
        kill_tree(proc.pid)
        proc.wait()
    Path(sys.argv[3]).write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result))


if __name__ == "__main__":
    main()
