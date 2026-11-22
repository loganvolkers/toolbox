#!/usr/bin/env python3
"""Repository security scorecard.

Usage: run <repo-path>   (prints JSON to stdout, progress/errors to stderr)

Every check returns a score 0-10, or null when it does not apply or a tool is
missing. The overall score is the weighted mean of the non-null scores.
The scorecard never executes code from the repository: it only reads files and
runs static tools (zizmor, actionlint, semgrep, ruff) over them.
"""
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent

# Directories never scanned: third-party, generated or tool caches.
SKIP_DIRS = {".git", "node_modules", "vendor", "third_party", "third-party", "target",
             ".venv", "venv", "__pycache__", ".tox", ".mypy_cache", ".ruff_cache",
             "dist", "bazel-out", ".pnpm-store", ".next", "site-packages"}
TEST_PARTS = {"test", "tests", "__tests__", "testdata", "test-data", "fixtures",
              "fixture", "testing", "e2e", "spec", "snapshots", "mocks", "examples"}
TEST_FILE_RE = re.compile(r"(^test_.*\.py$|_test\.(py|go|rs)$|_tests?\.rs$|\.(test|spec)\.[jt]sx?$|^conftest\.py$)")
BINARY_EXT = {".png", ".jpg", ".jpeg", ".gif", ".ico", ".pdf", ".zip", ".gz", ".tgz",
              ".zst", ".woff", ".woff2", ".ttf", ".wasm", ".so", ".dylib", ".dll",
              ".exe", ".jar", ".bin", ".snap", ".lock"}


def log(msg):
    print(f"[scorecard] {msg}", file=sys.stderr)


def finding(message, path=None, line=None):
    return {"message": message, "path": path, "line": line}


def run_tool(cmd, cwd, timeout=900):
    """Run a static tool; returns (rc, stdout, stderr) or None if missing."""
    if shutil.which(cmd[0]) is None:
        return None
    try:
        p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout)
        return p.returncode, p.stdout, p.stderr
    except subprocess.TimeoutExpired:
        return -1, "", "timeout"


def capped_score(deductions):
    """deductions: list of (group, points, cap). Each group loses at most `cap` points."""
    per_group = {}
    caps = {}
    for group, pts, cap in deductions:
        per_group[group] = per_group.get(group, 0.0) + pts
        caps[group] = cap
    total = sum(min(v, caps[g]) for g, v in per_group.items())
    return round(max(0.0, 10.0 - total), 1)


class Repo:
    def __init__(self, root):
        self.root = Path(root).resolve()
        self.files = []
        for dirpath, dirnames, filenames in os.walk(self.root):
            dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
            for f in filenames:
                p = Path(dirpath) / f
                if p.is_symlink():
                    continue
                self.files.append(p.relative_to(self.root).as_posix())
        self.files.sort()
        self._text = {}

    def text(self, rel, limit=2_000_000):
        if rel not in self._text:
            p = self.root / rel
            try:
                if p.stat().st_size > limit:
                    self._text[rel] = None
                else:
                    self._text[rel] = p.read_text(encoding="utf-8", errors="replace")
            except OSError:
                self._text[rel] = None
        return self._text[rel]

    def exists(self, rel):
        return (self.root / rel).exists()

    def glob(self, pattern):
        rx = re.compile(pattern)
        return [f for f in self.files if rx.search(f)]

    @staticmethod
    def is_test(rel):
        parts = rel.split("/")
        if any(p.lower() in TEST_PARTS for p in parts[:-1]):
            return True
        return bool(TEST_FILE_RE.search(parts[-1]))

    def test_start_line(self, rel):
        """First line of an inline test module (Rust `#[cfg(test)] mod`), or None."""
        if not rel.endswith(".rs"):
            return None
        m = re.search(r"^\s*#\[cfg\(test\)\]\s*\n(\s*#\[[^\n]*\]\s*\n)*\s*(pub(\([^)]*\))?\s+)?mod\s", self.text(rel) or "", re.M)
        return (self.text(rel) or "").count("\n", 0, m.start()) + 1 if m else None

    def is_test_line(self, rel, line):
        if self.is_test(rel):
            return True
        start = self.test_start_line(rel)
        return start is not None and line >= start

    def workflows(self):
        # Only root .github/workflows run on GitHub; nested ones are dead config.
        return [f for f in self.files if re.match(r"^\.github/workflows/[^/]+\.ya?ml$", f)]

    def local_actions(self):
        return [f for f in self.files if re.search(r"(^|/)action\.ya?ml$", f)
                and not f.startswith(".github/workflows/")]


def load_yaml(text):
    try:
        import yaml  # optional; regex fallbacks are used without it
    except ImportError:
        return None
    try:
        return yaml.safe_load(text)
    except Exception:
        return None


# --------------------------------------------------------------------------
# Checks. Each returns dict(score, findings) and is wrapped with metadata.
# --------------------------------------------------------------------------

def check_ci_workflow_audit(repo):
    targets = repo.workflows() + [a for a in repo.local_actions() if a.startswith(".github/")]
    if not targets:
        return None, [finding("No GitHub Actions workflows found; check not applicable.")]
    res = run_tool(["zizmor", "--offline", "--no-progress", "--format", "json-v1", *targets], repo.root)
    if res is None:
        return None, [finding("zizmor is not installed.")]
    rc, out, err = res
    if rc not in (0, 10, 11, 12, 13, 14):  # 10-14 = findings at increasing severity
        return None, [finding(f"zizmor failed (exit {rc}): {err.strip()[-300:]}")]
    try:
        data = json.loads(out or "[]")
    except json.JSONDecodeError:
        return None, [finding(f"zizmor failed: {err.strip()[-300:]}")]
    sev_pts = {"High": 1.5, "Medium": 0.5, "Low": 0.1, "Informational": 0.0, "Unknown": 0.1}
    conf_mul = {"High": 1.0, "Medium": 0.7, "Low": 0.4, "Unknown": 0.5}
    deductions, findings = [], []
    for item in data:
        if item.get("ignored"):
            continue
        det = item.get("determinations", {})
        sev, conf = det.get("severity", "Unknown"), det.get("confidence", "Unknown")
        loc = next((l for l in item.get("locations", []) if l.get("symbolic", {}).get("kind") == "Primary"),
                   (item.get("locations") or [{}])[0])
        sym = loc.get("symbolic", {})
        path = sym.get("key", {}).get("Local", {}).get("given_path") or \
            sym.get("key", {}).get("Local", {}).get("verbatim_path")
        row = loc.get("concrete", {}).get("location", {}).get("start_point", {}).get("row")
        ident = item.get("ident")
        deductions.append((ident, sev_pts.get(sev, 0.1) * conf_mul.get(conf, 0.5), 3.0))
        findings.append(finding(f"[{sev}/{conf}] {ident}: {item.get('desc')}"
                                + (f" ({sym.get('annotation')})" if sym.get("annotation") else ""),
                                path, row + 1 if isinstance(row, int) else None))
    order = {"[High": 0, "[Medium": 1, "[Low": 2, "[Info": 3}
    findings.sort(key=lambda f: (order.get(f["message"].split("/")[0], 4), f["path"] or "", f["line"] or 0))
    return capped_score(deductions), findings


ACTIONLINT_SECURITY_KINDS = {"expression": 0.5, "credentials": 2.0, "permissions": 1.0,
                             "events": 0.5, "syntax-check": 0.5, "shellcheck": 0.3,
                             "pyflakes": 0.1, "action": 0.3, "deprecated-commands": 1.0,
                             "workflow-call": 0.5, "job-needs": 0.3, "matrix": 0.2,
                             "env-var": 0.2, "id": 0.2, "glob": 0.2}


def check_workflow_lint(repo):
    wfs = repo.workflows()
    if not wfs:
        return None, [finding("No GitHub Actions workflows found; check not applicable.")]
    res = run_tool(["actionlint", "-no-color", "-format", "{{json .}}", *wfs], repo.root)
    if res is None:
        return None, [finding("actionlint is not installed.")]
    rc, out, err = res
    try:
        data = json.loads(out or "[]")
    except json.JSONDecodeError:
        return None, [finding(f"actionlint failed: {err.strip()[-300:]}")]
    deductions, findings = [], []
    for e in data:
        kind = e.get("kind", "?")
        # runner-label is usually a custom self-hosted label: reported, not scored.
        pts = 0.0 if kind == "runner-label" else ACTIONLINT_SECURITY_KINDS.get(kind, 0.2)
        deductions.append((kind, pts, 3.0))
        findings.append(finding(f"[{kind}] {e.get('message')}", e.get("filepath"), e.get("line")))
    return capped_score(deductions), findings


USES_RE = re.compile(r"^\s*(?:-\s*)?uses:\s*['\"]?([^\s'\"#]+)", re.M)
SHA_RE = re.compile(r"@[0-9a-f]{40}$")


def check_action_pinning(repo):
    files = repo.workflows() + [a for a in repo.local_actions() if a.startswith(".github/")]
    total, pinned, findings = 0, 0, []
    for f in files:
        text = repo.text(f) or ""
        for m in USES_RE.finditer(text):
            ref = m.group(1)
            if ref.startswith("./") or ref.startswith("../"):
                continue  # local action: same commit
            line = text.count("\n", 0, m.start()) + 1
            total += 1
            if ref.startswith("docker://"):
                ok = "@sha256:" in ref
            else:
                ok = bool(SHA_RE.search(ref))
            if ok:
                pinned += 1
            else:
                findings.append(finding(f"'{ref}' is not pinned to an immutable commit SHA/digest; "
                                        "a moved tag runs new code with this workflow's secrets.", f, line))
    if total == 0:
        return None, [finding("No external actions referenced; check not applicable.")]
    findings.insert(0, finding(f"{pinned}/{total} external action references are SHA-pinned."))
    return round(10.0 * pinned / total, 1), findings


def check_workflow_permissions(repo):
    wfs = repo.workflows()
    if not wfs:
        return None, [finding("No GitHub Actions workflows found; check not applicable.")]
    ok, findings = 0, []
    dangerous_triggers = ("pull_request_target", "workflow_run")
    for f in wfs:
        text = repo.text(f) or ""
        doc = load_yaml(text)
        top = re.search(r"^permissions:\s*(.*)$", text, re.M)
        all_jobs_scoped = False
        if isinstance(doc, dict) and isinstance(doc.get("jobs"), dict) and doc["jobs"]:
            all_jobs_scoped = all(isinstance(j, dict) and ("permissions" in j or "uses" in j and "permissions" in j)
                                  for j in doc["jobs"].values())
        if top or all_jobs_scoped:
            ok += 1
        else:
            findings.append(finding("No top-level `permissions:` and not every job sets one: the GITHUB_TOKEN gets the "
                                    "repository default, which may be read-write.", f, 1))
        for m in re.finditer(r"^\s*permissions:\s*write-all\b", text, re.M):
            findings.append(finding("`permissions: write-all` grants every scope.", f, text.count("\n", 0, m.start()) + 1))
            ok -= 0.5
        for trig in dangerous_triggers:
            m = re.search(rf"^\s*-?\s*{trig}\s*:?\s*$|^on:\s*\[?.*\b{trig}\b", text, re.M)
            if m:
                findings.append(finding(f"Uses `{trig}`, which runs with secrets in the base repository context; "
                                        "never check out or run PR code in it.", f, text.count("\n", 0, m.start()) + 1))
    findings.insert(0, finding(f"{max(ok, 0):g}/{len(wfs)} workflows restrict GITHUB_TOKEN permissions explicitly."))
    return round(max(0.0, 10.0 * ok / len(wfs)), 1), findings


SECRET_PATTERNS = [
    ("AWS access key id", re.compile(r"\b(AKIA|ASIA)[0-9A-Z]{16}\b"), 5),
    ("GitHub token", re.compile(r"\b(ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{36,}\b|\bgithub_pat_[A-Za-z0-9_]{60,}\b"), 5),
    ("OpenAI/Anthropic API key", re.compile(r"\bsk-(proj-|ant-[a-z0-9]+-)?[A-Za-z0-9_\-]{32,}\b"), 5),
    ("Slack token", re.compile(r"\bxox[baprs]-[0-9A-Za-z\-]{10,}\b"), 5),
    ("Google API key", re.compile(r"\bAIza[0-9A-Za-z_\-]{35}\b"), 4),
    ("Stripe live key", re.compile(r"\b(sk|rk)_live_[0-9A-Za-z]{20,}\b"), 5),
    ("npm token", re.compile(r"\bnpm_[A-Za-z0-9]{36}\b"), 5),
    ("Private key block", re.compile(r"-----BEGIN (RSA |EC |DSA |OPENSSH |ENCRYPTED |PGP )?PRIVATE KEY( BLOCK)?-----"), 5),
    ("Credentials in URL", re.compile(r"\b[a-z][a-z0-9+.\-]*://[A-Za-z0-9._%\-]{2,}:[^\s:@/'\"$<>{}]{6,}@[A-Za-z0-9.\-]+"), 2),
    ("Hard-coded secret assignment",
     re.compile(r"""(?i)\b(password|passwd|secret|api[_-]?key|access[_-]?token|auth[_-]?token|client[_-]?secret)\b["']?\s*[:=]\s*["']([A-Za-z0-9+/=_\-]{16,})["']"""), 1.5),
]
PLACEHOLDER_RE = re.compile(r"(?i)(example|dummy|fake|test|sample|placeholder|changeme|xxxx|your[_-]|redacted|<|\$\{|\{\{|0000|1234|abcd|mock)")
SENSITIVE_FILES = re.compile(r"(^|/)(\.env(\.[a-z]+)?|id_rsa|id_dsa|id_ecdsa|id_ed25519|.*\.p12|.*\.pfx|.*\.jks|.*\.keystore|\.npmrc|\.pypirc|credentials\.json|.*\.kdbx)$")


def check_secrets(repo):
    deductions, findings = [], []
    for f in repo.files:
        name = f.rsplit("/", 1)[-1]
        test = repo.is_test(f)
        if SENSITIVE_FILES.search(f) and not name.endswith((".example", ".sample", ".template")):
            text = repo.text(f) or ""
            if name in (".npmrc", ".pypirc"):
                if re.search(r"(_authToken|_auth|password)\s*=\s*[^$\s{]", text):
                    findings.append(finding(f"{name} contains a literal registry credential.", f, None))
                    deductions.append(("file", 4, 6))
                continue
            if name.startswith(".env") and not re.search(r"(?im)^(?!\s*#)\s*\w*(KEY|TOKEN|SECRET|PASSWORD)\w*\s*=\s*\S{8,}", text):
                continue
            findings.append(finding("Sensitive file type committed (key store / env file / private key).", f, None))
            deductions.append(("file", 1 if test else 3, 6))
        if Path(f).suffix.lower() in BINARY_EXT:
            continue
        text = repo.text(f, limit=1_000_000)
        if not text:
            continue
        for label, rx, weight in SECRET_PATTERNS:
            for m in rx.finditer(text):
                s = m.group(0)
                line_start = text.rfind("\n", 0, m.start()) + 1
                line_txt = text[line_start:text.find("\n", m.end()) if text.find("\n", m.end()) != -1 else None]
                if PLACEHOLDER_RE.search(s) or (label.startswith("Hard-coded") and PLACEHOLDER_RE.search(line_txt)):
                    continue
                if label == "Hard-coded secret assignment" and len(set(m.group(2))) < 8:
                    continue  # low entropy
                lineno = text.count("\n", 0, m.start()) + 1
                in_test = test or repo.is_test_line(f, lineno)
                if in_test:
                    deductions.append(("test-fixtures", 0.2 * weight, 1.0))
                else:
                    deductions.append((label, weight, 6))
                where = "test code/fixture" if in_test else "source"
                findings.append(finding(f"Possible {label} in {where}: {s[:12]}... (redacted)", f, lineno))
                if len(findings) > 300:
                    break
    if not findings:
        findings.append(finding("No hard-coded credentials or private keys detected by pattern scan."))
    return capped_score(deductions), findings


def _has_ancestor_file(repo, rel_dir, names):
    d = Path(rel_dir)
    while True:
        for n in names:
            if repo.exists((d / n).as_posix()):
                return (d / n).as_posix()
        if d.as_posix() in (".", ""):
            return None
        d = d.parent


def check_lockfiles(repo):
    rules = [
        ("package.json", ["pnpm-lock.yaml", "package-lock.json", "yarn.lock", "bun.lockb", "bun.lock", "npm-shrinkwrap.json"], "npm"),
        ("Cargo.toml", ["Cargo.lock"], "cargo"),
        ("pyproject.toml", ["uv.lock", "poetry.lock", "pdm.lock", "Pipfile.lock", "requirements.txt", "pylock.toml"], "python"),
        ("Pipfile", ["Pipfile.lock"], "python"),
        ("go.mod", ["go.sum"], "go"),
        ("Gemfile", ["Gemfile.lock"], "ruby"),
    ]
    total, locked, findings = 0, 0, []
    for manifest, locks, eco in rules:
        for f in repo.files:
            if f.rsplit("/", 1)[-1] != manifest or repo.is_test(f):
                continue
            text = repo.text(f) or ""
            if manifest == "package.json":
                try:
                    pj = json.loads(text)
                except json.JSONDecodeError:
                    continue
                if not any(pj.get(k) for k in ("dependencies", "devDependencies", "optionalDependencies")):
                    continue
            if manifest == "go.mod" and "require" not in text:
                continue
            if manifest == "pyproject.toml" and not re.search(r"(?m)^\s*dependencies\s*=|\[tool\.poetry\.dependencies\]", text):
                continue
            d = f.rsplit("/", 1)[0] if "/" in f else "."
            total += 1
            lock = _has_ancestor_file(repo, d, locks) if eco != "go" else (
                f"{d}/go.sum" if repo.exists(f"{d}/go.sum") else None)
            if lock:
                locked += 1
            else:
                findings.append(finding(f"{eco} manifest has no committed lockfile ({' / '.join(locks[:3])}); "
                                        "builds resolve unreviewed versions.", f, None))
    for f in repo.files:
        if re.search(r"(^|/)requirements[^/]*\.txt$", f) and not repo.is_test(f):
            for i, ln in enumerate((repo.text(f) or "").splitlines(), 1):
                s = ln.split("#")[0].strip()
                if s and not s.startswith("-") and "==" not in s and "@" not in s:
                    findings.append(finding(f"Unpinned requirement '{s}'.", f, i))
    if total == 0:
        return None, [finding("No dependency manifests found; check not applicable.")]
    findings.insert(0, finding(f"{locked}/{total} dependency manifests are covered by a lockfile."))
    unpinned = sum(1 for x in findings if x["message"].startswith("Unpinned"))
    return round(max(0.0, 10.0 * locked / total - min(3.0, 0.25 * unpinned)), 1), findings


def detect_ecosystems(repo):
    eco = {}
    def add(name, path):
        eco.setdefault(name, []).append(path)
    for f in repo.files:
        n = f.rsplit("/", 1)[-1]
        if repo.is_test(f):
            continue
        if n == "Cargo.toml":
            add("cargo", f)
        elif n in ("package.json",):
            add("npm", f)
        elif n in ("pyproject.toml", "Pipfile") or re.match(r"requirements[^/]*\.txt$", n):
            add("pip", f)
        elif n == "go.mod":
            add("gomod", f)
        elif n == "Gemfile":
            add("bundler", f)
        elif re.match(r"(Dockerfile|Containerfile)(\..*)?$", n) or n.endswith(".dockerfile"):
            add("docker", f)
    if repo.workflows():
        add("github-actions", ".github/workflows")
    return eco


ECO_ALIASES = {"npm": {"npm", "bun", "yarn", "pnpm"}, "pip": {"pip", "uv", "poetry", "pipenv", "pip-compile"},
               "cargo": {"cargo"}, "gomod": {"gomod"}, "docker": {"docker", "docker-compose"},
               "github-actions": {"github-actions"}, "bundler": {"bundler"}}


def check_dependency_updates(repo):
    eco = detect_ecosystems(repo)
    if not eco:
        return None, [finding("No dependency ecosystems detected; check not applicable.")]
    findings = []
    dep = next((f for f in (".github/dependabot.yml", ".github/dependabot.yaml") if repo.exists(f)), None)
    renovate = next((f for f in ("renovate.json", "renovate.json5", ".renovaterc", ".renovaterc.json",
                                 ".github/renovate.json", ".github/renovate.json5") if repo.exists(f)), None)
    if renovate:
        findings.append(finding("Renovate is configured; assuming it covers all detected ecosystems.", renovate, None))
        return 10.0, findings
    if not dep:
        findings.append(finding("No Dependabot or Renovate config: dependency and action updates (including "
                                "security fixes) depend on someone remembering.", None, None))
        return 0.0, findings
    text = repo.text(dep) or ""
    doc = load_yaml(text) or {}
    configured, dirs = set(), []
    for u in (doc.get("updates") or []) if isinstance(doc, dict) else []:
        if not isinstance(u, dict):
            continue
        configured.add(str(u.get("package-ecosystem")))
        for d in ([u.get("directory")] if u.get("directory") else []) + list(u.get("directories") or []):
            dirs.append((str(u.get("package-ecosystem")), str(d)))
    if not doc:
        configured = set(re.findall(r"package-ecosystem:\s*['\"]?([\w-]+)", text))
    # Stale entries (directory missing, or no manifest of that ecosystem in it) update nothing.
    import fnmatch
    live, stale = [], 0
    manifest_dirs = {name: sorted({p.rsplit("/", 1)[0] if "/" in p else "." for p in paths})
                     for name, paths in eco.items()}
    for e, d in dirs:
        pat = d.strip("/") or "."
        ln = next((i for i, l in enumerate(text.splitlines(), 1) if d in l), None)
        if "*" not in pat and not repo.exists(pat):
            stale += 1
            findings.append(finding(f"Dependabot '{e}' entry points at '{d}', which does not exist; it updates nothing.",
                                    dep, ln))
            continue
        eco_name = next((k for k, v in ECO_ALIASES.items() if e in v), e)
        if eco_name in ("docker", "npm", "pip", "cargo", "gomod", "bundler") and "*" not in pat \
                and not any(md == pat or md.startswith(pat + "/") or pat == "." for md in manifest_dirs.get(eco_name, [])):
            stale += 1
            findings.append(finding(f"Dependabot '{e}' entry points at '{d}', which holds no {eco_name} manifest; "
                                    "it updates nothing.", dep, ln))
            continue
        live.append((eco_name, pat))
    if not dirs:  # no YAML parser: fall back to ecosystem names only
        live = [(next((k for k, v in ECO_ALIASES.items() if e in v), e), "*") for e in configured]
    ratios = []
    for name, mdirs in sorted(manifest_dirs.items()):
        pats = [p for e, p in live if e == name]
        if name == "github-actions":
            ok = list(mdirs) if pats else []
        else:
            # Lockfile-based tools update a whole workspace from its root, so an ancestor entry covers members.
            ancestor_ok = name in ("cargo", "npm", "pip", "gomod", "bundler")
            ok = [md for md in mdirs if any(fnmatch.fnmatch(md, p) or p == "*" or
                                             (ancestor_ok and (p == "." or md.startswith(p + "/"))) for p in pats)]
        ratios.append(len(ok) / len(mdirs))
        missing = [md for md in mdirs if md not in ok]
        if missing:
            findings.append(finding(f"Ecosystem '{name}': {len(missing)}/{len(mdirs)} manifest directories get no "
                                    f"automated updates (e.g. {', '.join(missing[:5])}).", dep, None))
    findings.insert(0, finding(f"Update coverage by ecosystem: " + ", ".join(
        f"{n} {r:.0%}" for n, r in zip(sorted(manifest_dirs), ratios))))
    return round(max(0.0, 10.0 * sum(ratios) / len(ratios) - 0.5 * stale), 1), findings


def check_dependency_sources(repo):
    findings, deductions, lockfiles = [], [], 0
    for f in repo.files:
        n = f.rsplit("/", 1)[-1]
        text = None
        if n == "Cargo.lock":
            text = repo.text(f, limit=20_000_000) or ""
            lockfiles += 1
            for m in re.finditer(r'^source = "(git\+[^"]+|registry\+(?!https://github\.com/rust-lang/crates\.io-index)[^"]+|sparse\+(?!https://index\.crates\.io/)[^"]+)"', text, re.M):
                kind = "git" if m.group(1).startswith("git+") else "registry"
                block = text.rfind("[[package]]", 0, m.start())
                name = re.search(r'name = "([^"]+)"', text[block:m.start()])
                deductions.append((kind, 0.3 if kind == "git" else 1.0, 4))
                findings.append(finding(f"Crate '{name.group(1) if name else '?'}' comes from a {kind} source "
                                        f"({m.group(1).split('#')[0]}), outside crates.io advisories/yank checks.",
                                        f, text.count("\n", 0, m.start()) + 1))
        elif n in ("pnpm-lock.yaml", "package-lock.json", "yarn.lock", "uv.lock", "poetry.lock"):
            text = repo.text(f, limit=50_000_000) or ""
            lockfiles += 1
            pats = [(r"(tarball|resolved|url):\s*['\"]?(http://[^\s'\",]+)", "plaintext-http", 3.0),
                    (r"(git\+(ssh|https?)://[^\s'\",]+|github\.com/[^\s'\"]+/tar\.gz/[0-9a-f]+|codeload\.github\.com[^\s'\",]+)", "git", 0.3),
                    (r'"resolved":\s*"(http://[^"]+)"', "plaintext-http", 3.0),
                    (r'git = "([^"]+)"', "git", 0.3)]
            seen = set()
            for rx, kind, pts in pats:
                for m in re.finditer(rx, text):
                    key = (kind, m.group(0)[:80])
                    if key in seen:
                        continue
                    seen.add(key)
                    deductions.append((kind, pts, 4))
                    findings.append(finding(f"Dependency resolved from a {kind} source: {m.group(0)[:120]}",
                                            f, text.count("\n", 0, m.start()) + 1))
    for f in repo.files:
        if f.rsplit("/", 1)[-1] == ".npmrc":
            t = repo.text(f) or ""
            if re.search(r"(?m)^\s*registry\s*=\s*http://", t):
                deductions.append(("plaintext-http", 3, 4))
                findings.append(finding("npm registry configured over plain http.", f, None))
    if lockfiles == 0:
        return None, [finding("No lockfiles found; dependency sources cannot be evaluated.")]
    if not findings:
        findings.append(finding("All locked dependencies resolve from default registries over HTTPS."))
    return capped_score(deductions), findings


SCAN_CATEGORIES = {
    "dependency vulnerability audit": r"cargo[- ]audit|cargo[- ]deny|rustsec/audit-check|EmbarkStudios/cargo-deny-action|npm audit|pnpm audit|yarn (npm )?audit|pip-audit|osv-scanner|govulncheck|dependency-review-action|trivy|grype|snyk|safety check",
    "static analysis (SAST)": r"github/codeql-action|semgrep|gosec|bandit|ruff[^\n]*--select[^\n]*\bS\b|eslint-plugin-security|njsscan|sonar",
    "workflow security lint": r"zizmor|actionlint|poutine|octoscan",
    "secret scanning": r"gitleaks|trufflehog|detect-secrets|ggshield|secret[_-]scanning",
}


def check_security_tooling(repo):
    wfs = repo.workflows()
    findings = []
    nested = [f for f in repo.files if re.search(r".+/\.github/workflows/[^/]+\.ya?ml$", f)]
    for f in nested:
        findings.append(finding("Workflow lives in a nested .github/workflows directory; GitHub only runs workflows "
                                "from the repository root, so this one never executes.", f, None))
    ci_text = "\n".join(repo.text(f) or "" for f in wfs)
    extra = "\n".join(repo.text(f) or "" for f in repo.files if re.search(r"(^|/)(\.pre-commit-config\.ya?ml|justfile|Makefile|lefthook\.ya?ml)$", f))
    if not wfs:
        findings.append(finding("No CI workflows found; security tooling can only run locally."))
    score = 0.0
    for cat, rx in SCAN_CATEGORIES.items():
        m = re.search(rx, ci_text, re.I)
        if m:
            score += 2.5
            src = next((f for f in wfs if re.search(rx, repo.text(f) or "", re.I)), None)
            findings.append(finding(f"CI runs {cat} ('{m.group(0)}').", src, None))
        elif re.search(rx, extra, re.I):
            score += 1.0
            findings.append(finding(f"{cat} is configured only locally (pre-commit/justfile/Makefile), not enforced in CI."))
        else:
            findings.append(finding(f"No {cat} step found in CI."))
    if not re.search(SCAN_CATEGORIES["static analysis (SAST)"], ci_text, re.I):
        m = re.search(r"clippy[^\n]*(-D|--deny)\b|\beslint\b|golangci-lint|\bruff check\b", ci_text, re.I)
        if m:
            score += 1.25
            findings.append(finding(f"Only a general linter runs in CI ('{m.group(0)[:40]}'): half credit for SAST; "
                                    "it has few security-specific rules."))
    if not re.search(SCAN_CATEGORIES["secret scanning"], ci_text + extra, re.I):
        findings.append(finding("GitHub-native secret scanning / push protection may be enabled in repository "
                                "settings; that cannot be verified from the checkout."))
    if nested and score > 0:
        score -= 0.5
    return round(max(score, 0.0), 1), findings


def check_policy_ownership(repo):
    findings, score = [], 0.0
    sec = next((f for f in ("SECURITY.md", ".github/SECURITY.md", "docs/SECURITY.md", "SECURITY.rst", "SECURITY.txt")
                if repo.exists(f)), None)
    if sec:
        t = repo.text(sec) or ""
        if re.search(r"[\w.+-]+@[\w-]+\.[\w.]+|https?://|security advisor|/security/advisories", t, re.I):
            score += 4
            findings.append(finding("Security policy with a reporting channel is present.", sec, None))
        else:
            score += 2
            findings.append(finding("SECURITY.md has no reporting channel (email, URL or advisory link).", sec, None))
    else:
        findings.append(finding("No SECURITY.md: reporters have no private disclosure channel."))
    co = next((f for f in ("CODEOWNERS", ".github/CODEOWNERS", "docs/CODEOWNERS") if repo.exists(f)), None)
    if co:
        score += 3
        rules = [l.split()[0] for l in (repo.text(co) or "").splitlines() if l.strip() and not l.lstrip().startswith("#") and len(l.split()) > 1]
        covers = [r for r in rules if r in ("*", "/*", "**") or r.lstrip("/").startswith(".github")
                  and ("workflows" in r or r.rstrip("/") in ("/.github", ".github", "/.github/*", ".github/**"))]
        if covers:
            score += 3
            findings.append(finding(f"CODEOWNERS covers CI configuration ({covers[0]}).", co, None))
        else:
            findings.append(finding("CODEOWNERS does not cover .github/workflows: CI and release pipelines (which "
                                    "hold signing and publishing secrets) can change without an owner's review.", co, None))
    else:
        findings.append(finding("No CODEOWNERS file: no mandatory reviewers for sensitive paths."))
    return score, findings


def check_sast(repo):
    rules = HERE / "rules" / "semgrep.yml"
    excl = []
    for d in sorted(SKIP_DIRS | TEST_PARTS):
        excl += ["--exclude", d]
    for p in ("*_test.go", "*_test.rs", "*_tests.rs", "*.test.ts", "*.test.js", "*.spec.ts", "*.spec.js"):
        excl += ["--exclude", p]
    res = run_tool(["semgrep", "scan", "--config", str(rules), "--json", "--metrics=off", "--disable-version-check",
                    "--no-git-ignore", "--timeout", "30", "--max-target-bytes", "2000000", "-j", "4", *excl, "."],
                   repo.root, timeout=1800)
    if res is None:
        return None, [finding("semgrep is not installed.")]
    rc, out, err = res
    try:
        data = json.loads(out)
    except json.JSONDecodeError:
        return None, [finding(f"semgrep failed: {err.strip()[-300:]}")]
    pts = {"ERROR": 2.0, "WARNING": 0.5, "INFO": 0.1}
    deductions, findings = [], []
    for r in data.get("results", []):
        sev = r.get("extra", {}).get("severity", "WARNING")
        rid = r.get("check_id", "").split(".")[-1]
        deductions.append((rid, pts.get(sev, 0.5), 3.0 if sev == "ERROR" else 2.0))
        findings.append(finding(f"[{sev}] {rid}: {r.get('extra', {}).get('message')}", r.get("path"),
                                r.get("start", {}).get("line")))
    findings.sort(key=lambda f: ({"[ERROR]": 0, "[WARNING]": 1}.get(f["message"].split()[0], 2), f["path"] or ""))
    nerr = len(data.get("errors", []))
    if nerr:
        findings.append(finding(f"semgrep reported {nerr} parse/scan errors (those files were partly skipped)."))
    if not data.get("results"):
        findings.insert(0, finding("No matches for the scorecard's semgrep rules in non-test source."))
    return capped_score(deductions), findings


RUFF_IGNORE = "S101"  # assert: not a vulnerability in scripts/tools
RUFF_WEIGHTS = {"S102": 1.5, "S307": 1.5, "S301": 1.5, "S506": 1.5, "S602": 1.0, "S605": 1.0, "S501": 1.5,
                "S323": 1.5, "S105": 0.5, "S106": 0.5, "S107": 0.3, "S108": 0.2, "S110": 0.01, "S112": 0.01,
                "S603": 0.02, "S607": 0.02, "S311": 0.05, "S310": 0.05, "S202": 0.5, "S606": 0.1,
                "S113": 0.2, "S104": 0.3, "S324": 0.3, "S608": 1.0, "S701": 1.0}


def check_python(repo):
    py = [f for f in repo.files if f.endswith(".py") and not repo.is_test(f)]
    if not py:
        return None, [finding("No non-test Python files; check not applicable.")]
    excl = ",".join(sorted(SKIP_DIRS | TEST_PARTS) + ["test_*.py", "*_test.py", "conftest.py"])
    res = run_tool(["ruff", "check", "--isolated", "--no-cache", "--select", "S", "--ignore", RUFF_IGNORE,
                    "--extend-exclude", excl, "--output-format", "json", "--exit-zero", "."], repo.root)
    if res is None:
        return None, [finding("ruff is not installed.")]
    rc, out, err = res
    try:
        data = json.loads(out or "[]")
    except json.JSONDecodeError:
        return None, [finding(f"ruff failed: {err.strip()[-300:]}")]
    deductions, findings = [], []
    for d in data:
        rel = os.path.relpath(d["filename"], repo.root)
        if repo.is_test(rel):
            continue
        code = d.get("code")
        deductions.append((code, RUFF_WEIGHTS.get(code, 0.3), 2.5))
        findings.append(finding(f"[{code}] {d.get('message')}", rel, d.get("location", {}).get("row")))
    findings.sort(key=lambda f: -RUFF_WEIGHTS.get(f["message"][1:5], 0.3))
    findings.insert(0, finding(f"{len(findings)} flake8-bandit findings across {len(py)} non-test Python files "
                               f"({RUFF_IGNORE} ignored)."))
    return capped_score(deductions), findings


UNSAFE_RE = re.compile(r"\bunsafe\s*(\{|fn\b|impl\b|extern\b)")


def check_rust_unsafe(repo):
    rs = [f for f in repo.files if f.endswith(".rs") and not repo.is_test(f)]
    if not rs:
        return None, [finding("No non-test Rust code; check not applicable.")]
    total, documented, findings, files_with = 0, 0, [], set()
    for f in rs:
        text = repo.text(f) or ""
        if "unsafe" not in text:
            continue
        lines = text.splitlines()
        tstart = repo.test_start_line(f)
        for i, ln in enumerate(lines):
            s = ln.strip()
            if tstart is not None and i + 1 >= tstart:
                break
            if s.startswith("//") or not UNSAFE_RE.search(ln):
                continue
            if re.search(r"\bunsafe\s+extern\b", ln) and "{" not in ln and "fn" not in ln:
                continue
            total += 1
            files_with.add(f)
            # Justification = same line, or the contiguous comment/attribute block right above
            # (doc comments with a `# Safety` section count for unsafe fn/impl).
            ctx = [ln]
            j = i - 1
            while j >= 0 and i - j <= 20:
                prev = lines[j].strip()
                if prev.startswith(("//", "#[", "#![", "*", "/*")) or (prev and i - j <= 2 and not prev.endswith((";", "{", "}"))):
                    ctx.append(prev)
                    j -= 1
                    continue
                break
            if re.search(r"(?i)\bSAFETY\b|safe because|sound because", "\n".join(ctx)):
                documented += 1
            else:
                findings.append(finding("`unsafe` without a `// SAFETY:` justification nearby.", f, i + 1))
    crates = [f for f in repo.files if f.endswith("Cargo.toml")]
    forbid = sum(1 for f in rs if f.endswith(("lib.rs", "main.rs")) and "forbid(unsafe_code)" in (repo.text(f) or ""))
    lint = any(re.search(r"undocumented_unsafe_blocks", repo.text(f) or "") for f in repo.files
               if f.endswith(("Cargo.toml", "clippy.toml", ".clippy.toml", "lib.rs", "main.rs")))
    head = (f"{total} unsafe sites in {len(files_with)} files across {len(crates)} Cargo manifests; "
            f"{documented} carry a SAFETY justification; {forbid} crate roots forbid unsafe_code; "
            f"clippy::undocumented_unsafe_blocks {'enabled' if lint else 'not enabled'}.")
    findings.insert(0, finding(head))
    if total == 0:
        return 10.0, findings
    score = 10.0 * documented / total
    if lint:
        score = min(10.0, score + 1.0)
    return round(score, 1), findings


def check_containers(repo):
    dfs = [f for f in repo.files if re.search(r"(^|/)(Dockerfile|Containerfile)(\.[\w-]+)?$|\.dockerfile$", f)]
    if not dfs:
        return None, [finding("No Dockerfiles; check not applicable.")]
    deductions, findings = [], []
    for f in dfs:
        text = repo.text(f) or ""
        lines = text.splitlines()
        stages = set()
        user = None
        for i, ln in enumerate(lines, 1):
            m = re.match(r"\s*FROM\s+(--platform=\S+\s+)?(\S+)(\s+AS\s+(\S+))?", ln, re.I)
            if m:
                img = m.group(2)
                if m.group(4):
                    stages.add(m.group(4).lower())
                if img.lower() in stages or img == "scratch" or img.startswith("$"):
                    continue
                if "@sha256:" not in img:
                    deductions.append(("unpinned-base", 1.0, 4))
                    findings.append(finding(f"Base image '{img}' is not pinned by digest; rebuilds can pull "
                                            "a different (possibly compromised) image.", f, i))
                user = None  # USER resets per stage
            m = re.match(r"\s*USER\s+(\S+)", ln, re.I)
            if m:
                user = (m.group(1), i)
            if re.match(r"\s*ADD\s+https?://", ln, re.I):
                deductions.append(("add-url", 1.0, 2))
                findings.append(finding("ADD from a URL fetches unverified remote content; use curl + checksum.", f, i))
            if re.search(r"(?i)\b(ENV|ARG)\s+\w*(PASSWORD|SECRET|TOKEN|API_KEY)\w*[= ]\s*\S", ln):
                deductions.append(("secret-in-image", 2.0, 4))
                findings.append(finding("Secret-looking value baked into an image layer via ENV/ARG.", f, i))
        if user is None or user[0] in ("root", "0", "0:0"):
            deductions.append(("root-user", 1.5, 4))
            findings.append(finding("Final stage runs as root (no non-root USER).", f, user[1] if user else None))
    return capped_score(deductions), findings


def check_release_integrity(repo):
    wfs = repo.workflows()
    rel = []
    for f in wfs:
        t = repo.text(f) or ""
        if re.search(r"(?i)release|publish|deploy", f) or re.search(r"(?m)^\s*(release|tags):|npm publish|cargo publish|twine upload|gh-action-pypi-publish|gh release create|softprops/action-gh-release", t):
            if re.search(r"(?i)npm publish|cargo publish|twine upload|pypi-publish|gh release|action-gh-release|upload-release|docker/build-push-action|docker push|uv publish|pnpm publish", t):
                rel.append(f)
    if not rel:
        return None, [finding("No release/publish workflows found; check not applicable.")]
    text = "\n".join(repo.text(f) or "" for f in rel)
    findings, score = [finding(f"{len(rel)} release/publish workflows: {', '.join(rel)}")], 0.0
    checks = [
        ("build provenance attestation (SLSA / actions/attest)", r"actions/attest(-build-provenance)?@|slsa-github-generator|--provenance|attestations:\s*write", 4.0),
        ("artifact signing (sigstore/cosign/gpg/codesign/signtool)", r"cosign|sigstore|gpg\s+--(detach-)?sign|codesign\b|signtool|minisign|code-sign", 3.0),
        ("OIDC trusted publishing / short-lived credentials", r"id-token:\s*write", 2.0),
        ("checksums published with artifacts", r"sha256sum|shasum -a 256|SHA256SUMS|checksums", 1.0),
    ]
    for label, rx, pts in checks:
        m = re.search(rx, text, re.I)
        if m:
            score += pts
            src = next((f for f in rel if re.search(rx, repo.text(f) or "", re.I)), None)
            findings.append(finding(f"Has {label} ('{m.group(0)}').", src, None))
        else:
            findings.append(finding(f"Missing {label}; consumers cannot verify where artifacts came from."))
    for f in rel:
        t = repo.text(f) or ""
        for m in re.finditer(r"(?i)\b(NPM_TOKEN|PYPI_API_TOKEN|PYPI_TOKEN|CARGO_REGISTRY_TOKEN|DOCKERHUB_TOKEN)\b", t):
            findings.append(finding(f"Long-lived publish token '{m.group(1)}' used; prefer OIDC trusted publishing.",
                                    f, t.count("\n", 0, m.start()) + 1))
            break
    return round(score, 1), findings


AGENT_USES_RE = re.compile(r"uses:\s*['\"]?((openai/codex-action|anthropics/claude-code(-base)?-action|"
                           r"google-github-actions/run-gemini-cli|actions/ai-inference|coderabbitai/[\w-]+|"
                           r"qodo-ai/pr-agent|Codium-ai/pr-agent|warpdotdev/[\w-]+|cline/[\w-]+)[^\s'\"]*)", re.I)
AGENT_RUN_RE = re.compile(r"\b(codex\s+exec|claude\s+(-p|--print)\b|gemini\s+(-p|--prompt)\b|aider\s+--message|llm\s+prompt)\b")
UNTRUSTED_EVENTS = {"issues", "issue_comment", "pull_request_target", "pull_request_review",
                    "pull_request_review_comment", "discussion", "discussion_comment", "pull_request", "workflow_run"}
UNTRUSTED_EXPR_RE = re.compile(r"\$\{\{\s*(github\.event\.(issue\.(title|body)|comment\.body|review\.body|"
                               r"pull_request\.(title|body|head\.ref|head\.label)|discussion\.(title|body)|"
                               r"head_commit\.message|commits)|github\.head_ref)[^}]*\}\}")
WILDCARD_USERS_RE = re.compile(r"(allow-users|allowed_non_write_users|allowed_bots|allowed-users)\s*:\s*['\"]?\*['\"]?")


def _workflow_events(text):
    doc = load_yaml(text)
    if isinstance(doc, dict):
        on = doc.get("on", doc.get(True))
        if isinstance(on, str):
            return {on}
        if isinstance(on, list):
            return set(map(str, on))
        if isinstance(on, dict):
            return set(map(str, on.keys()))
    m = re.search(r"^on:(.*?)(?=^\S)", text, re.M | re.S)
    return set(re.findall(r"\b(" + "|".join(UNTRUSTED_EVENTS) + r")\b", m.group(1) if m else ""))


def check_ai_agents(repo):
    """LLM coding agents in CI: prompt injection from untrusted event text."""
    deductions, findings, agent_wfs = [], [], []
    for f in repo.workflows():
        text = repo.text(f) or ""
        uses = AGENT_USES_RE.search(text)
        runs = AGENT_RUN_RE.search(text)
        if not uses and not runs:
            continue
        agent_wfs.append(f)
        events = _workflow_events(text) & UNTRUSTED_EVENTS
        agent = uses.group(1).split("@")[0] if uses else runs.group(0)
        if not events:
            findings.append(finding(f"Runs an LLM agent ({agent}) only on trusted triggers.", f, None))
            continue
        deductions.append(("untrusted-trigger", 0.5, 1.0))  # agent reads attacker text even when fetched indirectly
        findings.append(finding(f"Runs an LLM agent ({agent}) on untrusted trigger(s): {', '.join(sorted(events))}; "
                                "treat everything it reads as hostile input.", f, None))
        for m in UNTRUSTED_EXPR_RE.finditer(text):
            ln = text.count("\n", 0, m.start()) + 1
            deductions.append((f"prompt:{f}", 2.0, 3.0))
            findings.append(finding(f"Attacker-controlled `{m.group(1)}` is interpolated into the agent workflow "
                                    "(prompt injection: anyone who can open an issue/PR can steer the agent and "
                                    "whatever it can reach, including its API key and job outputs).", f, ln))
        for m in WILDCARD_USERS_RE.finditer(text):
            ln = text.count("\n", 0, m.start()) + 1
            deductions.append(("wildcard-users", 1.0, 2.0))
            findings.append(finding(f"`{m.group(0)}` lets any GitHub user trigger the agent.", f, ln))
        writes = re.findall(r"^\s*(contents|pull-requests|issues|actions|packages|id-token):\s*write", text, re.M)
        if writes or re.search(r"permissions:\s*write-all", text):
            deductions.append(("write-token", 1.0, 2.0))
            findings.append(finding(f"Agent output feeds a workflow holding write scopes ({', '.join(sorted(set(writes))) or 'write-all'}); "
                                    "a steered agent can make it label, comment or push.", f, None))
        if re.search(r"sandbox:\s*read-only|safety-strategy:\s*(drop-sudo|unprivileged-user)", text):
            findings.append(finding("Mitigation present: the agent runs read-only / without sudo.", f, None))
    if not agent_wfs:
        return None, [finding("No LLM agent workflows found; check not applicable.")]
    return capped_score(deductions), findings


CHECKS = [
    # id, title, weight, function, why
    ("ci-workflow-audit", "GitHub Actions security audit (zizmor)", 3, check_ci_workflow_audit,
     "Runs zizmor over the workflows GitHub executes (template injection, dangerous triggers, credential "
     "persistence, cache poisoning, excessive permissions). CI holds release and signing secrets, so a workflow "
     "flaw is a supply-chain compromise. Deduction per finding by severity x confidence, capped at 3 points per audit."),
    ("action-pinning", "Third-party actions pinned to commit SHAs", 2, check_action_pinning,
     "A tag like @v4 can be moved by whoever controls the action (tj-actions/changed-files, 2025). Score is the "
     "share of external `uses:` references pinned to a 40-character SHA (or image digest)."),
    ("workflow-permissions", "Least-privilege GITHUB_TOKEN", 2, check_workflow_permissions,
     "Workflows without an explicit `permissions:` block inherit the repository default, which can be read-write "
     "on everything. Score is the share of workflows that scope the token; write-all is penalised, risky triggers are reported."),
    ("ai-agent-workflows", "LLM agents in CI resist prompt injection", 2, check_ai_agents,
     "Workflows that run coding agents (codex-action, claude-code-action, gemini-cli, ...) on issues, comments or "
     "PRs read attacker-written text. Interpolating that text into the prompt, letting any user trigger the agent, or "
     "giving the run write scopes turns the agent into a confused deputy. zizmor does not model this. Null if no agent workflows."),
    ("workflow-lint", "Workflow correctness (actionlint)", 1, check_workflow_lint,
     "actionlint finds broken expressions, bad contexts and shell bugs that make CI checks silently not run or "
     "behave unexpectedly. Unknown runner labels are reported but not scored (they are usually custom runners)."),
    ("secrets", "No committed credentials", 3, check_secrets,
     "Pattern scan of every non-vendored file for cloud/API tokens, private keys and hard-coded passwords, plus "
     "committed key stores and .env files. A leaked credential is usable by anyone who clones the repo. Hits in "
     "test code (test paths, Rust #[cfg(test)] modules) are reported but cost at most 1 point in total."),
    ("dependency-lockfiles", "Dependencies locked", 2, check_lockfiles,
     "Each package manifest should be covered by a committed lockfile so builds are reproducible and dependency "
     "changes are reviewable. Score is the share of manifests covered, minus unpinned requirements.txt lines."),
    ("dependency-updates", "Automated dependency updates", 1.5, check_dependency_updates,
     "Dependabot/Renovate should cover every ecosystem in the repo (including GitHub Actions and Docker) so "
     "security fixes arrive as PRs. Score is the share of detected ecosystems covered, minus stale entries."),
    ("dependency-sources", "Dependencies from trusted registries", 1, check_dependency_sources,
     "Git and non-default registry dependencies bypass registry advisories and yank checks; plain-http sources "
     "allow tampering in transit. Parsed from Cargo.lock, pnpm/npm/yarn and uv/poetry lockfiles."),
    ("security-tooling-in-ci", "Security scanning enforced in CI", 2, check_security_tooling,
     "Checks that CI runs a dependency vulnerability audit, SAST, workflow linting and secret scanning "
     "(2.5 points each; 1 if only local). Workflows in nested .github directories are flagged because GitHub never runs them."),
    ("policy-and-ownership", "Security policy and code ownership", 1, check_policy_ownership,
     "SECURITY.md with a reporting channel (4), a CODEOWNERS file (3) that covers CI configuration (3), so "
     "vulnerabilities have a disclosure path and pipeline changes need an owner's review."),
    ("sast", "Dangerous code patterns (semgrep, local rules)", 2, check_sast,
     "Bundled semgrep rules for JS/TS, Rust, Go, shell and Dockerfiles: disabled TLS verification, eval, "
     "shell command strings built from variables, SQL by string formatting, raw HTML sinks, curl|sh, weak hashes. "
     "Test code is excluded. Deduction by severity, capped per rule."),
    ("python-security-lint", "Python security lint (ruff flake8-bandit)", 1, check_python,
     "ruff's S rules (bandit) on non-test Python: exec/eval, shell=True, unsafe deserialisation, hard-coded "
     "passwords, insecure temp files, unverified URL opens. Weighted by rule severity, capped per rule."),
    ("rust-unsafe", "Rust unsafe code is justified", 1, check_rust_unsafe,
     "Memory-safety bugs live in `unsafe`. Each unsafe block/fn/impl should carry a `// SAFETY:` comment stating "
     "the invariant it relies on (clippy::undocumented_unsafe_blocks). Score is the documented share."),
    ("container-hardening", "Container images hardened", 1, check_containers,
     "Dockerfiles should pin base images by digest, drop root, avoid ADD from URLs and keep secrets out of layers."),
    ("release-integrity", "Signed, attested releases", 2, check_release_integrity,
     "Release workflows should attest build provenance, sign artifacts, publish with OIDC instead of long-lived "
     "tokens and ship checksums, so users can verify what they install."),
]


def main(argv):
    if len(argv) != 2 or argv[1] in ("-h", "--help"):
        print(__doc__.strip(), file=sys.stderr)
        return 2
    root = Path(argv[1])
    if not root.is_dir():
        print(f"not a directory: {root}", file=sys.stderr)
        return 2
    repo = Repo(root)
    log(f"{len(repo.files)} files indexed in {repo.root}")
    out_checks, wsum, ssum = [], 0.0, 0.0
    for cid, title, weight, fn, why in CHECKS:
        log(f"running {cid}")
        try:
            score, findings = fn(repo)
        except Exception as e:  # a broken check must not hide the others
            score, findings = None, [finding(f"check crashed: {type(e).__name__}: {e}")]
        if score is not None:
            score = max(0.0, min(10.0, float(score)))
            wsum += weight
            ssum += weight * score
        out_checks.append({"id": cid, "title": title, "score": score, "weight": weight, "why": why,
                           "findings": findings})
    result = {
        "repo": str(repo.root),
        "overall_score": round(ssum / wsum, 1) if wsum else None,
        "checks": out_checks,
    }
    json.dump(result, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
