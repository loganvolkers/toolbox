#!/usr/bin/env python3
"""Start one arm, time it to ready and to a fully readable catalog, sample memory, then load its
read endpoints with oha. Prints one JSON object. Usage: measure.py <arm> <catalog-dir> <out.json>.

Both arms read the same catalog (gen.py), from the same Postgres 16, on the same machine, through
the same client. Each arm is started from scratch (an empty database) for every run."""
import json
import os
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
PG = os.environ.get("BENCH_PG", "postgres://postgres@127.0.0.1:54350")
OHA = os.environ.get("OHA", "oha")
BS_APP = Path(os.environ.get("BS_APP", "/tmp/bs/prod"))
NODE_BIN = os.environ.get("NODE_BIN", "/tmp/bs/node24/bin")
TOOLBOX = os.environ.get("TOOLBOX", str(ROOT / "target/release/toolbox"))
TOKEN = "bench-static-token-0123456789abcdef"
DURATION = os.environ.get("BENCH_DURATION", "10s")
CONCURRENCY = os.environ.get("BENCH_CONCURRENCY", "16")


def get(url, headers, timeout=5):
    req = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return r.status, r.read()


def wait(pred, limit=600.0):
    start = time.monotonic()
    while time.monotonic() - start < limit:
        try:
            if pred():
                return True
        except Exception:
            pass
        time.sleep(0.05)
    return False


def tree_rss_mb(pid):
    """Resident memory of a process and all its descendants (the arm's whole footprint)."""
    kids = {}
    for p in Path("/proc").iterdir():
        if p.name.isdigit():
            try:
                ppid = int((p / "stat").read_text().rsplit(")", 1)[1].split()[1])
                kids.setdefault(ppid, []).append(int(p.name))
            except (OSError, IndexError, ValueError):
                pass
    total, todo = 0, [pid]
    while todo:
        q = todo.pop()
        todo.extend(kids.get(q, []))
        try:
            for line in Path(f"/proc/{q}/status").read_text().splitlines():
                if line.startswith("VmRSS:"):
                    total += int(line.split()[1])
        except OSError:
            pass
    return round(total / 1024, 1)


def kill_tree(pid):
    """SIGKILL a process and its descendants (OpenFGA, Node workers): a crash, not a shutdown."""
    kids = subprocess.run(["pgrep", "-P", str(pid)], capture_output=True, text=True).stdout.split()
    for k in kids:
        kill_tree(int(k))
    try:
        os.kill(pid, 9)
    except ProcessLookupError:
        pass


def psql(sql):
    subprocess.run(["psql", PG + "/postgres", "-qAt", "-c", sql], check=True,
                   capture_output=True)


def oha(urls, headers):
    """Each URL for DURATION at CONCURRENCY connections."""
    for url in urls:
        cmd = [OHA, "-z", DURATION, "-c", CONCURRENCY, "--no-tui", "--output-format", "json"]
        for k, v in headers.items():
            cmd += ["-H", f"{k}: {v}"]
        out = json.loads(subprocess.run(cmd + [url], check=True, capture_output=True,
                                        text=True).stdout)
        s = out["summary"]
        pct = out["latencyPercentiles"]
        codes = out.get("statusCodeDistribution", {})
        ms = lambda q: None if pct.get(q) is None else round(pct[q] * 1000, 2)
        # Throughput counts only 200s: a fast error is not a read.
        ok = codes.get("200", 0) / max(sum(codes.values()), 1)
        yield {"url": url, "rps": round(s["requestsPerSec"] * ok, 1), "p50ms": ms("p50"),
               "p95ms": ms("p95"), "p99ms": ms("p99"), "ok": ok, "codes": codes,
               "errors": out.get("errorDistribution", {})}


class Toolbox:
    """As deployed: token auth (a PAT), and every read authorized by the OpenFGA co-process."""
    name = "toolbox"
    port = 18090
    headers = {"authorization": f"Bearer tbx_{TOKEN}"}

    def start(self, catalog, fresh=True):
        if fresh:
            psql("DROP DATABASE IF EXISTS bench_toolbox")
            psql("CREATE DATABASE bench_toolbox")
        env = dict(os.environ, TOOLBOX_CONNECTORS="off", RUST_LOG="warn",
                   TOOLBOX_AUTH_MODE="token", TOOLBOX_PATS=f"bench=tbx_{TOKEN}",
                   TOOLBOX_CONNECTOR_TOKENS=f"catalog=conn_{TOKEN}",
                   TOOLBOX_OPENFGA_BIN=os.environ.get("TOOLBOX_OPENFGA_BIN",
                                                      "/tmp/tools/openfga"))
        return subprocess.Popen(
            [TOOLBOX, "serve", str(HERE / "portal"), "--listen", f"127.0.0.1:{self.port}",
             "--database-url", PG + "/bench_toolbox"],
            env=env, stdout=self.log, stderr=subprocess.STDOUT)

    def ready(self):
        return get(f"{self.base}/readyz", {})[0] == 200

    def load(self, catalog):
        req = urllib.request.Request(
            f"{self.base}/v1/ingest", data=(catalog / "catalog.ndjson").read_bytes(),
            headers={"authorization": f"Bearer conn_{TOKEN}"}, method="POST")
        with urllib.request.urlopen(req, timeout=600) as r:
            assert r.status in (200, 202), r.status

    def loaded(self, n):
        return get(f"{self.base}/v1/entities/component:c{n - 1}", self.headers)[0] == 200

    def reads(self, n):
        mid = n // 2
        return [f"{self.base}/v1/entities/component:c{mid}",
                f"{self.base}/v1/entities?kind=Component&limit=100"
                f"&filter=entity.owner%3D%3D%27group%3Ag3%27"]

    @property
    def base(self):
        return f"http://127.0.0.1:{self.port}"


class Backstage:
    name = "backstage"
    port = 7007
    headers = {"authorization": f"Bearer {TOKEN}"}

    def start(self, catalog, fresh=True):
        if fresh:
            plugins = subprocess.run(
                ["psql", PG + "/postgres", "-qAt", "-c",
                 "SELECT datname FROM pg_database WHERE datname LIKE 'backstage_plugin_%'"],
                capture_output=True, text=True).stdout.split()
            for db in ["bench_backstage"] + plugins:
                psql(f'DROP DATABASE IF EXISTS "{db}"')
        cfg = HERE / "backstage" / "app-config.bench.yaml"
        env = dict(os.environ, PATH=NODE_BIN + ":" + os.environ["PATH"],
                   NODE_ENV="production", BENCH_CATALOG=str(catalog / "catalog.yaml"),
                   BENCH_TOKEN=TOKEN)
        return subprocess.Popen(
            ["node", "packages/backend", "--config", "app-config.yaml", "--config", str(cfg)],
            cwd=BS_APP, env=env, stdout=self.log, stderr=subprocess.STDOUT)

    def ready(self):
        return get(f"{self.base}/.backstage/health/v1/readiness", {})[0] == 200

    def load(self, catalog):
        pass  # the catalog location is in the config; the processing loop reads it

    def loaded(self, n):
        _, body = get(f"{self.base}/api/catalog/entities/by-query?filter=kind=component&limit=1",
                      self.headers)
        return json.loads(body)["totalItems"] >= n

    def reads(self, n):
        mid = n // 2
        return [f"{self.base}/api/catalog/entities/by-name/component/default/c{mid}",
                f"{self.base}/api/catalog/entities/by-query?limit=100"
                f"&filter=kind=component,spec.owner=group:default/g3"]

    @property
    def base(self):
        return f"http://127.0.0.1:{self.port}"


def main():
    arm = {"toolbox": Toolbox, "backstage": Backstage}[sys.argv[1]]()
    catalog = Path(sys.argv[2])
    n = sum(1 for line in (catalog / "catalog.ndjson").open() if '"component"' in line)
    arm.log = open(Path(sys.argv[3]).with_suffix(".log"), "w")
    t0 = time.monotonic()
    proc = arm.start(catalog)
    try:
        if not wait(arm.ready, 300):
            raise SystemExit(f"{arm.name}: not ready")
        ready_s = time.monotonic() - t0
        rss_idle = tree_rss_mb(proc.pid)
        arm.load(catalog)
        if not wait(lambda: arm.loaded(n), 900):
            raise SystemExit(f"{arm.name}: catalog never fully readable")
        loaded_s = time.monotonic() - t0
        time.sleep(2)
        rss_loaded = tree_rss_mb(proc.pid)
        reads = list(oha(arm.reads(n), arm.headers))
        rss_peak = tree_rss_mb(proc.pid)
        # Recovery: kill -9, restart on the same database, time until the whole catalog reads.
        kill_tree(proc.pid)
        proc.wait()
        t1 = time.monotonic()
        proc = arm.start(catalog, fresh=False)
        if not wait(arm.ready, 300) or not wait(lambda: arm.loaded(n), 900):
            raise SystemExit(f"{arm.name}: did not recover after kill -9")
        recovered_s = time.monotonic() - t1
        result = {"arm": arm.name, "components": n, "readyS": round(ready_s, 2),
                  "catalogReadableS": round(loaded_s, 2), "rssIdleMb": rss_idle,
                  "rssLoadedMb": rss_loaded, "rssAfterLoadMb": rss_peak,
                  "recoveredS": round(recovered_s, 2), "reads": reads}
    finally:
        kill_tree(proc.pid)
        proc.wait()
    Path(sys.argv[3]).write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result))


if __name__ == "__main__":
    main()
