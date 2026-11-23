#!/usr/bin/env python3
"""Score a running portal against the live Git server.

Shared probes S1–S3 and Toolbox probes T1–T4, as pre-registered in
docs/design/local-simulation.md §7 and experiments/deploy-ab/README.md.
The truth is the catalog-info.yaml files on the Git server at score time.
A static catalog, a fixture load, and a one-shot import fail S3.

  python3 experiments/deploy-ab/harness/probes.py              # L2: S1–S3 and T1–T4
  python3 experiments/deploy-ab/harness/probes.py --suite l4   # S2, T3, S3
  python3 experiments/deploy-ab/harness/probes.py --arm backstage
"""

import json
import os
import re
import secrets
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request

try:
    import yaml
except ImportError:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "--user", "pyyaml"], stdout=subprocess.DEVNULL)
    import yaml

KINDS = ("Component", "API", "System", "Domain", "Resource")
PORTAL = os.environ.get("PORTAL_URL", "http://127.0.0.1:8080").rstrip("/")
FORGEJO = os.environ.get("FORGEJO_URL", "http://127.0.0.1:3030").rstrip("/")
FORGEJO_TOKEN = os.environ.get("FORGEJO_TOKEN", "")
ORG = os.environ.get("FORGEJO_ORG", "acme")
ADMIN = os.environ.get("ADMIN_USER", "bob")
READER = os.environ.get("READER_USER", "alice")
OUTSIDER = os.environ.get("OUTSIDER_USER", "carol")
GIT_USER = os.environ.get("GIT_USER", "harness")
GIT_PASSWORD = os.environ.get("GIT_PASSWORD", "")
ARM = "toolbox"
SUITE = "l2"


def load_env():
    path = os.environ.get("DEPLOY_AB_ENV", "/tmp/deploy-ab/env")
    if os.path.isfile(path):
        for line in open(path):
            line = line.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue
            key, value = line.split("=", 1)
            os.environ.setdefault(key, value)
    global PORTAL, FORGEJO, FORGEJO_TOKEN, ORG, ADMIN, READER, OUTSIDER, GIT_USER, GIT_PASSWORD
    PORTAL = os.environ.get("PORTAL_URL", PORTAL).rstrip("/")
    FORGEJO = os.environ.get("FORGEJO_URL", FORGEJO).rstrip("/")
    FORGEJO_TOKEN = os.environ.get("FORGEJO_TOKEN", FORGEJO_TOKEN)
    ORG = os.environ.get("FORGEJO_ORG", ORG)
    ADMIN = os.environ.get("ADMIN_USER", ADMIN)
    READER = os.environ.get("READER_USER", READER)
    OUTSIDER = os.environ.get("OUTSIDER_USER", OUTSIDER)
    GIT_USER = os.environ.get("GIT_USER", GIT_USER)
    GIT_PASSWORD = os.environ.get("GIT_PASSWORD", GIT_PASSWORD)


def http(method, url, body=None, headers=None, timeout=30):
    req = urllib.request.Request(url, data=body, method=method, headers=headers or {})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.status, resp.read()
    except urllib.error.HTTPError as err:
        return err.code, err.read()


def forgejo(path):
    status, raw = http(
        "GET",
        FORGEJO + path,
        headers={"authorization": f"token {FORGEJO_TOKEN}", "accept": "application/json"},
    )
    if status != 200:
        return status, None
    return status, json.loads(raw) if raw else None


def git_repos():
    names = []
    page = 1
    while page < 100:
        status, body = forgejo(f"/api/v1/orgs/{ORG}/repos?limit=50&page={page}")
        if status != 200 or not body:
            break
        names.extend(repo["name"] for repo in body)
        if len(body) < 50:
            break
        page += 1
    return sorted(names)


def git_file(repo, path):
    quoted = urllib.parse.quote(path)
    status, body = forgejo(f"/api/v1/repos/{ORG}/{repo}/contents/{quoted}")
    if status != 200 or not isinstance(body, dict) or "content" not in body:
        return None
    import base64

    return base64.b64decode(body["content"]).decode("utf-8", "replace")


def documents(text):
    docs = []
    for part in text.split("\n---"):
        doc = part.strip()
        if doc.startswith("---"):
            doc = doc[3:].strip()
        if not doc:
            continue
        loaded = yaml.safe_load(doc)
        if isinstance(loaded, dict):
            docs.append(loaded)
    return docs


def truth():
    """(kind, name) -> {owner, system} from every catalog-info.yaml on the Git server."""
    found = {}
    for repo in git_repos():
        text = git_file(repo, "catalog-info.yaml")
        if text is None:
            continue
        for doc in documents(text):
            kind = str(doc.get("kind") or "")
            if kind not in KINDS:
                continue
            name = (doc.get("metadata") or {}).get("name")
            spec = doc.get("spec") or {}
            if not name:
                continue
            found[(kind, name)] = {
                "owner": spec.get("owner") or "",
                "system": spec.get("system") or "",
            }
    return found


def toolbox_get(user, path):
    status, raw = http("GET", PORTAL + path, headers={"x-toolbox-user": user, "accept": "application/json"})
    body = json.loads(raw) if raw else {}
    return status, body


def toolbox_entities(user):
    out = []
    for kind in KINDS:
        status, body = toolbox_get(user, f"/v1/entities?kind={kind}&limit=500")
        if status != 200:
            raise RuntimeError(f"list {kind} HTTP {status}: {body}")
        out.extend(body.get("items") or [])
    detailed = []
    for item in out:
        status, body = toolbox_get(user, "/v1/entities/" + urllib.parse.quote(item["id"], safe=""))
        if status != 200:
            raise RuntimeError(f"get {item['id']} HTTP {status}")
        detailed.append(body)
    return detailed


def backstage_entities():
    status, raw = http("GET", PORTAL + "/api/catalog/entities?limit=500")
    if status != 200:
        raise RuntimeError(f"backstage catalog HTTP {status}: {raw[:200]!r}")
    body = json.loads(raw)
    items = body if isinstance(body, list) else body.get("items") or []
    return [item for item in items if item.get("kind") in KINDS]


def s1(failures):
    expected = truth()
    if not expected:
        failures.append("S1: the Git server has no catalog entities")
        return
    if ARM == "toolbox":
        actual = {}
        for entity in toolbox_entities(ADMIN):
            kind = entity.get("kind")
            fields = entity.get("fields") or {}
            name = fields.get("title") or entity["id"].split(":", 1)[-1]
            outbound = entity.get("outbound") or {}
            system = fields.get("system") or ""
            if not system:
                targets = outbound.get("system") or []
                if targets:
                    system = targets[0].split(":", 1)[-1]
            actual[(kind, name)] = {"owner": fields.get("owner") or "", "system": system}
    else:
        actual = {}
        for entity in backstage_entities():
            meta = entity.get("metadata") or {}
            spec = entity.get("spec") or {}
            actual[(entity.get("kind"), meta.get("name"))] = {
                "owner": spec.get("owner") or "",
                "system": spec.get("system") or "",
            }
    missing = sorted(set(expected) - set(actual))
    extra = sorted(set(actual) - set(expected))
    wrong = sorted(
        key
        for key in set(expected) & set(actual)
        if actual[key]["owner"] != expected[key]["owner"] or actual[key]["system"] != expected[key]["system"]
    )
    if missing or extra or wrong:
        failures.append(f"S1: missing={missing} extra={extra} wrong={wrong}")
    else:
        print(f"S1 pass ({len(expected)} entities)")


def page_text(url, headers):
    status, raw = http("GET", url, headers=headers)
    text = raw.decode("utf-8", "replace")
    if status == 200 and "payments-api" in text and "group:payments" in text:
        return text
    rendered = render(url)
    if rendered is None:
        return text if status == 200 else ""
    return rendered


def render(url):
    """Open the page. An empty shell that never shows the entity stays empty."""
    try:
        from playwright.sync_api import sync_playwright
    except ImportError:
        return None
    with sync_playwright() as play:
        browser = play.chromium.launch(headless=True)
        page = browser.new_page()
        page.goto(url, wait_until="networkidle", timeout=30000)
        if ARM == "backstage":
            guest = page.get_by_text(re.compile(r"guest", re.I))
            if guest.count():
                guest.first.click()
                page.wait_for_timeout(1500)
        text = page.inner_text("body")
        browser.close()
        return text


def s2(failures):
    if ARM == "toolbox":
        url = PORTAL + "/e/" + urllib.parse.quote("component:payments-api", safe="") + "?as=" + READER
        headers = {"x-toolbox-user": READER}
    else:
        url = PORTAL + "/catalog/default/component/payments-api"
        headers = {}
    text = page_text(url, headers)
    if "payments-api" in text and "group:payments" in text:
        print("S2 pass")
    else:
        failures.append("S2: the payments-api page does not contain its name and its owner")


def push_catalog(name):
    if not GIT_PASSWORD:
        raise RuntimeError("S3 needs the harness git credential")
    work = tempfile.mkdtemp()
    path = os.path.join(work, "catalog-info.yaml")
    with open(path, "w") as fh:
        fh.write(
            "apiVersion: backstage.io/v1alpha1\n"
            "kind: Component\n"
            "metadata:\n"
            f"  name: {name}\n"
            "spec:\n"
            "  type: service\n"
            "  lifecycle: experimental\n"
            "  owner: group:platform\n"
            "  system: checkout\n"
        )
    subprocess.run(["git", "init", "-q", "-b", "main"], cwd=work, check=True)
    subprocess.run(["git", "config", "user.email", "harness@acme.test"], cwd=work, check=True)
    subprocess.run(["git", "config", "user.name", "Harness"], cwd=work, check=True)
    subprocess.run(["git", "add", "catalog-info.yaml"], cwd=work, check=True)
    subprocess.run(["git", "commit", "-q", "-m", "probe"], cwd=work, check=True)
    status, _ = http(
        "POST",
        f"{FORGEJO}/api/v1/orgs/{ORG}/repos",
        body=json.dumps({"name": name, "auto_init": False, "default_branch": "main"}).encode(),
        headers={"authorization": f"token {FORGEJO_TOKEN}", "content-type": "application/json"},
    )
    if status not in (201, 409):
        raise RuntimeError(f"create repo HTTP {status}")
    remote = f"http://{GIT_USER}:{urllib.parse.quote(GIT_PASSWORD)}@127.0.0.1:{FORGEJO.rsplit(':', 1)[-1]}/{ORG}/{name}.git"
    subprocess.run(["git", "remote", "add", "origin", remote], cwd=work, check=True)
    subprocess.run(["git", "push", "-q", "--force", "-u", "origin", "main"], cwd=work, check=True)


def seen(name):
    if ARM == "toolbox":
        status, body = toolbox_get(ADMIN, "/v1/entities?kind=Component&limit=500&q=" + urllib.parse.quote(name))
        if status != 200:
            return False
        return any((item.get("fields") or {}).get("title") == name or item.get("id", "").endswith(":" + name) for item in body.get("items") or [])
    return any((item.get("metadata") or {}).get("name") == name for item in backstage_entities())


def s3(failures):
    name = "p" + secrets.token_hex(8)
    push_catalog(name)
    deadline = time.time() + 90
    while time.time() < deadline:
        if seen(name):
            print(f"S3 pass ({name})")
            return
        time.sleep(3)
    failures.append(f"S3: {name} never appeared after the push")


def t1(failures):
    status, body = toolbox_get(ADMIN, "/v1/entities/" + urllib.parse.quote("component:payments-api", safe=""))
    provenance = (body.get("provenance") or {}) if status == 200 else {}
    source = str(provenance.get("title") or "")
    if status == 200 and ("git-files" in source or "github" in source) and "native" not in source and "fixture" not in source:
        print(f"T1 pass ({source})")
    else:
        failures.append(f"T1: title provenance is {source!r}")


def t2(failures):
    allowed, _ = toolbox_get(READER, "/v1/entities/" + urllib.parse.quote("component:payments-api", safe=""))
    denied, _ = toolbox_get(OUTSIDER, "/v1/entities/" + urllib.parse.quote("component:payments-api", safe=""))
    if allowed == 200 and denied in (401, 403, 404):
        print("T2 pass")
    else:
        failures.append(f"T2: payments member HTTP {allowed}, outsider HTTP {denied}")


def t3(failures):
    missing = sorted(repo for repo in git_repos() if git_file(repo, "README.md") is None)
    status, body = toolbox_get(ADMIN, "/v1/scorecards/readme?rule=has-readme&outcome=fail&limit=500")
    if status != 200:
        failures.append(f"T3: scorecard HTTP {status}")
        return
    failing = sorted(
        item["id"].split(":", 1)[-1]
        for item in body.get("entities") or []
        if "has-readme" in (item.get("failing") or {})
    )
    if failing == missing:
        print(f"T3 pass ({failing})")
    else:
        failures.append(f"T3: scorecard {failing} != repos without README {missing}")


def t4(failures):
    headers = {"content-type": "application/json", "x-toolbox-user": ADMIN}
    status, raw = http(
        "POST",
        PORTAL + "/mcp",
        body=json.dumps(
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {"protocolVersion": "2025-03-26", "capabilities": {}, "clientInfo": {"name": "probe", "version": "0"}},
            }
        ).encode(),
        headers=headers,
    )
    init = json.loads(raw) if raw else {}
    status2, raw2 = http(
        "POST",
        PORTAL + "/mcp",
        body=json.dumps(
            {"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "get", "arguments": {"id": "component:payments-api"}}}
        ).encode(),
        headers=headers,
    )
    got = json.loads(raw2) if raw2 else {}
    blob = json.dumps(got)
    if status == 200 and "protocolVersion" in json.dumps(init) and status2 == 200 and "payments-api" in blob and "component:payments-api" in blob:
        print("T4 pass")
    else:
        failures.append(f"T4: initialize HTTP {status}, get HTTP {status2}")


SUITES = {
    "l2": ["S1", "S2", "T1", "T2", "T3", "T4", "S3"],
    "l4": ["S2", "T3", "S3"],
    "benchmark": ["S1", "S2", "S3", "T1", "T2", "T3", "T4"],
}
PROBES = {"S1": s1, "S2": s2, "S3": s3, "T1": t1, "T2": t2, "T3": t3, "T4": t4}


def main():
    global ARM, SUITE
    args = sys.argv[1:]
    if "--arm" in args:
        ARM = args[args.index("--arm") + 1]
    if "--suite" in args:
        SUITE = args[args.index("--suite") + 1]
    load_env()
    names = SUITES[SUITE]
    if ARM == "backstage":
        names = [name for name in names if name.startswith("S")]
    failures = []
    for name in names:
        try:
            PROBES[name](failures)
        except Exception as exc:
            failures.append(f"{name}: {exc}")
    if failures:
        print("FAIL")
        for item in failures:
            print(" -", item)
        return 1
    print("PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
