#!/usr/bin/env python3
"""Repository security scorecard.

Usage: run <repo-path>   (prints JSON to stdout)

Each check scores 0-10 (10 = no problems found) or null when the repository
has nothing the check applies to, or a required tool is missing. The overall
score is the mean of the evaluated checks. Standard library only; external
tools used when on PATH: zizmor, actionlint, semgrep, ruff.
"""
import json
import math
import os
import re
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
RULES = os.path.join(HERE, "rules", "semgrep.yml")

SKIP_DIRS = {".git", ".hg", ".svn", "node_modules", "vendor", "target", ".venv", "venv",
             "__pycache__", ".tox", ".mypy_cache", ".ruff_cache", ".pytest_cache",
             "bower_components", ".terraform", ".gradle", ".idea"}
TEST_RE = re.compile(
    r"(^|/)(tests?|testdata|testutils?|testing|test_?helpers?|fixtures?|mocks?|__tests__|"
    r"__mocks__|e2e|spec|examples?|samples?)(/|$)|_test\.(go|py)$|(^|/)test_[^/]*\.py$|"
    r"(^|/)conftest\.py$|\.(test|spec)\.[cm]?[jt]sx?$|(^|/)mock_[^/]*$", re.I)
MAX_FILE = 2 * 1024 * 1024


# ---------------------------------------------------------------- helpers
class Repo:
    def __init__(self, root):
        self.root = os.path.abspath(root)
        self.files = []
        for d, dirs, fs in os.walk(self.root):
            dirs[:] = sorted(x for x in dirs if x not in SKIP_DIRS)
            for f in sorted(fs):
                p = os.path.join(d, f)
                if os.path.isfile(p) and not os.path.islink(p):
                    self.files.append(os.path.relpath(p, self.root).replace(os.sep, "/"))
        self.fileset = set(self.files)
        self._text = {}

    def abs(self, rel):
        return os.path.join(self.root, rel)

    def text(self, rel):
        if rel not in self._text:
            t = None
            try:
                if os.path.getsize(self.abs(rel)) <= MAX_FILE:
                    with open(self.abs(rel), "rb") as fh:
                        b = fh.read()
                    if b"\0" not in b[:8192]:
                        t = b.decode("utf-8", "replace")
            except OSError:
                pass
            self._text[rel] = t
        return self._text[rel]

    def match(self, regex):
        r = re.compile(regex)
        return [f for f in self.files if r.search(f)]

    def workflows(self):
        return self.match(r"^\.github/workflows/[^/]+\.ya?ml$")

    def actions(self):
        return [f for f in self.match(r"(^|/)action\.ya?ml$") if not f.startswith(".github/workflows/")]

    def dockerfiles(self):
        return [f for f in self.match(r"(^|/)((Dockerfile|Containerfile)(\.[^/]*)?|[^/]+\.(Dockerfile|dockerfile))$")
                if not is_test(f)]


def F(message, path=None, line=None):
    return {"message": message, "path": path, "line": line}


def decay(weights):
    """10 * product(1 - w): each problem removes a share of what is left."""
    s = 10.0
    for w in weights:
        s *= (1.0 - min(max(w, 0.0), 0.95))
    return round(s, 1)


def capped(items, key, cap):
    """Keep at most `cap` items per key so one noisy rule cannot dominate."""
    seen = {}
    out = []
    for it in items:
        k = key(it)
        seen[k] = seen.get(k, 0) + 1
        if seen[k] <= cap:
            out.append(it)
    return out


def line_of(text, pos):
    return text.count("\n", 0, pos) + 1


def tool(name):
    return shutil.which(name)


def run_tool(args, cwd=None, timeout=900, env=None):
    try:
        p = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=timeout, env=env)
        return p.returncode, p.stdout, p.stderr
    except (OSError, subprocess.TimeoutExpired) as e:
        return -1, "", str(e)


def tool_version(name):
    if not tool(name):
        return None
    _, out, err = run_tool([name, "--version"], timeout=60)
    return ((out or err).strip().splitlines() or [""])[0]


def check(cid, title, why, score, findings):
    return {"id": cid, "title": title, "score": score, "why": why, "findings": findings}


def is_test(path):
    return bool(path and TEST_RE.search(path))


# ---------------------------------------------------------------- zizmor (shared)
ZIZ_W = {("High", "High"): .15, ("High", "Medium"): .10, ("High", "Low"): .05,
         ("Medium", "High"): .06, ("Medium", "Medium"): .04, ("Medium", "Low"): .02,
         ("Low", "High"): .02, ("Low", "Medium"): .01, ("Low", "Low"): .005}
_ZIZ = None


def zizmor(repo):
    """Returns list of (ident, severity, confidence, Finding) or None if unavailable."""
    global _ZIZ
    if _ZIZ is not None:
        return _ZIZ
    if not tool("zizmor"):
        _ZIZ = None
        return None
    rc, out, err = run_tool(["zizmor", "--offline", "--no-config", "--no-progress", "--format", "json", "."],
                            cwd=repo.root)
    try:
        data = json.loads(out)
    except ValueError:
        _ZIZ = []
        return _ZIZ
    res = []
    for d in data:
        if d.get("ignored"):
            continue
        det = d.get("determinations", {})
        locs = d.get("locations") or [{}]
        prim = next((l for l in locs if l.get("symbolic", {}).get("kind") == "Primary"), locs[0])
        key = prim.get("symbolic", {}).get("key", {}).get("Local", {})
        path = key.get("given_path") or key.get("verbatim_path") or None
        while path and path.startswith("./"):
            path = path[2:]
        if path and path.startswith("/"):
            path = os.path.relpath(path, repo.root)
        row = prim.get("concrete", {}).get("location", {}).get("start_point", {}).get("row")
        ann = prim.get("symbolic", {}).get("annotation", "")
        msg = "[zizmor %s, %s severity] %s%s" % (d["ident"], det.get("severity", "?").lower(), d.get("desc", ""),
                                                 (": " + ann) if ann else "")
        res.append((d["ident"], det.get("severity"), det.get("confidence"),
                    F(msg, path, row + 1 if isinstance(row, int) else None)))
    _ZIZ = res
    return res


def ziz_weights(items):
    return [ZIZ_W.get((s, c), 0.0) for _, s, c, _ in capped(items, lambda x: x[0], 5)]


PINNING_IDENTS = {"unpinned-uses", "unpinned-images", "stale-action-refs", "impostor-commit",
                  "ref-confusion", "ref-version-mismatch"}
PERM_IDENTS = {"excessive-permissions", "undocumented-permissions"}
DEP_IDENTS = {"dependabot-cooldown", "dependabot-execution"}


# ---------------------------------------------------------------- 1. workflow audit
def check_ci_audit(repo):
    why = ("Audits CI/CD workflows with zizmor (template/script injection, dangerous triggers such as "
           "pull_request_target, cache poisoning, credential persistence, obfuscation) and actionlint "
           "(invalid workflows, untrusted expressions). CI holds release credentials and write tokens, so a "
           "workflow bug is a direct path to a supply-chain compromise.")
    wfs = repo.workflows() + repo.actions()
    if not wfs:
        return check("ci-workflow-audit", "CI workflow security audit", why, None,
                     [F("No GitHub Actions workflows or actions found; nothing to audit.")])
    findings, weights = [], []
    z = zizmor(repo)
    if z is None:
        findings.append(F("zizmor is not installed; injection/trigger audit skipped."))
    else:
        mine = [x for x in z if x[0] not in PINNING_IDENTS | PERM_IDENTS | DEP_IDENTS]
        weights += ziz_weights(mine)
        findings += [x[3] for x in mine]
    for wf in repo.workflows():
        t = repo.text(wf) or ""
        if not triggers(t) & {"pull_request_target", "workflow_run"}:
            continue
        for m in re.finditer(r"\$\{\{\s*(github\.event\.pull_request\.head\.(sha|ref)|github\.head_ref|"
                             r"github\.event\.workflow_run\.head_(sha|branch))\s*\}\}", t):
            weights.append(.30)
            findings.append(F("Privileged trigger (pull_request_target/workflow_run) references untrusted PR head "
                              "code (%s); checking it out and running it is a 'pwn request' that exposes secrets "
                              "and write tokens." % m.group(1), wf, line_of(t, m.start())))
    if tool("actionlint"):
        targets = [repo.abs(w) for w in repo.workflows()]
        if targets:
            rc, out, err = run_tool(["actionlint", "-format", "{{json .}}"] + targets, cwd=repo.root)
            try:
                errs = json.loads(out or "[]")
            except ValueError:
                errs = []
            al = []
            zlines = {(f["path"], f["line"]) for f in findings}
            for e in errs:
                fp = e.get("filepath") or ""
                fp = os.path.relpath(fp if os.path.isabs(fp) else os.path.join(repo.root, fp), repo.root) if fp else None
                dup = (fp, e.get("line")) in zlines
                untrusted = "untrusted" in e.get("message", "")
                al.append((e.get("kind", ""), 0.0 if dup else .10 if untrusted else .03,
                           F("[actionlint %s] %s" % (e.get("kind"), e.get("message")), fp, e.get("line"))))
            weights += [w for _, w, _ in capped(al, lambda x: x[0], 10)]
            findings += [f for _, _, f in al]
    else:
        findings.append(F("actionlint is not installed; workflow validation skipped."))
    if z is None and not tool("actionlint"):
        return check("ci-workflow-audit", "CI workflow security audit", why, None, findings)
    return check("ci-workflow-audit", "CI workflow security audit", why, decay(weights), findings)


# ---------------------------------------------------------------- 2. pinning
USES_RE = re.compile(r"""^\s*(?:-\s*)?uses\s*:\s*["']?([^\s"'#]+)""", re.M)
FROM_RE = re.compile(r"^\s*FROM\s+(?:--\S+\s+)*(\S+)(?:\s+AS\s+(\S+))?", re.I | re.M)
COPY_FROM_RE = re.compile(r"^\s*COPY\s+(?:--\S+\s+)*?--from=(\S+)", re.I | re.M)
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
# Reusable workflows that must be referenced by tag (their provenance verification checks the tag).
TAG_ONLY = ("slsa-framework/slsa-github-generator/.github/workflows/",)


def check_pinning(repo):
    why = ("Third-party GitHub Actions, reusable workflows and container base images should be pinned to an "
           "immutable commit SHA or image digest. Tags and branches can be moved by whoever controls the "
           "upstream, so an unpinned reference runs whatever code is there today (e.g. the tj-actions/"
           "changed-files compromise).")
    total, bad, findings = 0, 0.0, []
    for wf in repo.workflows() + repo.actions():
        t = repo.text(wf) or ""
        for m in USES_RE.finditer(t):
            ref = m.group(1)
            if ref.startswith("./") or ref.startswith("../"):
                continue
            total += 1
            ln = line_of(t, m.start(1))
            if ref.startswith("docker://"):
                if "@sha256:" not in ref:
                    bad += 1
                    findings.append(F("Container action %s is not pinned by digest." % ref, wf, ln))
                continue
            name, _, ver = ref.partition("@")
            if SHA_RE.match(ver) or name.startswith(TAG_ONLY):
                continue
            first_party = name.split("/")[0] in ("actions", "github")
            bad += 0.5 if first_party else 1.0
            kind = "Reusable workflow" if "/.github/workflows/" in name else "Action"
            findings.append(F("%s %s is referenced by mutable ref '%s', not a commit SHA%s." %
                              (kind, name, ver or "(none)", " (GitHub-owned)" if first_party else ""), wf, ln))
    for df in repo.dockerfiles():
        t = repo.text(df) or ""
        stages = set()
        for m in FROM_RE.finditer(t):
            img, alias = m.group(1), m.group(2)
            if alias:
                stages.add(alias.lower())
            if img.lower() == "scratch" or img.lower() in stages:
                continue
            total += 1
            if "@sha256:" not in img:
                bad += 1
                reason = "uses a build argument, so the base cannot be verified" if "$" in img else \
                    "is not pinned by digest (@sha256:...)"
                findings.append(F("Base image %s %s." % (img, reason), df, line_of(t, m.start(1))))
        for m in COPY_FROM_RE.finditer(t):
            src = m.group(1)
            if src.lower() in stages or src.isdigit() or "/" not in src and ":" not in src:
                continue
            total += 1
            if "@sha256:" not in src:
                bad += 1
                findings.append(F("COPY --from image %s is not pinned by digest." % src, df, line_of(t, m.start(1))))
    z = zizmor(repo) or []
    extra = [x for x in z if x[0] in ("unpinned-images", "impostor-commit", "ref-confusion", "stale-action-refs")]
    findings += [x[3] for x in extra]
    if total == 0 and not extra:
        return check("dependency-pinning", "Actions and images pinned to immutable refs", why, None,
                     [F("No GitHub Actions or Dockerfiles found.")])
    score = 10.0 * (1 - bad / total) if total else 10.0
    score = round(score * decay(ziz_weights(extra)) / 10.0, 1)
    return check("dependency-pinning", "Actions and images pinned to immutable refs", why, score, findings)


# ---------------------------------------------------------------- 3. workflow permissions
UNTRUSTED_TRIGGERS = ("pull_request_target", "issue_comment", "pull_request_review_comment",
                      "pull_request_review", "workflow_run", "issues", "discussion", "discussion_comment")
WRITE_RE = re.compile(r"^\s*([a-z][a-z-]*)\s*:\s*write\b|^\s*permissions\s*:\s*write-all\b", re.M)


def triggers(text):
    m = re.search(r"""^["']?(on|true)["']?\s*:(.*)$""", text, re.M)
    if not m:
        return set()
    inline = m.group(2).split("#")[0].strip()
    if inline:
        return set(re.findall(r"[a-z_]+", inline))
    block = []
    for ln in text[m.end() + 1:].splitlines():
        if ln and not ln[0].isspace() and not ln.startswith("#"):
            break
        block.append(ln)
    indent = min((len(l) - len(l.lstrip()) for l in block if l.strip() and not l.strip().startswith("#")),
                 default=0)
    names = set()
    for l in block:
        if l.strip() and len(l) - len(l.lstrip()) == indent:
            names.update(re.findall(r"^\s*-?\s*([a-z_]+)", l))
    return names


def check_permissions(repo):
    why = ("Every workflow should declare a top-level `permissions:` block (ideally read-only) and grant write "
           "scopes only to the jobs that need them. Without it, GITHUB_TOKEN inherits the repository default, "
           "which is often read-write. Write scopes in workflows that untrusted users can trigger "
           "(issue_comment, pull_request_target, workflow_run...) turn any injection into repository takeover.")
    wfs = repo.workflows()
    if not wfs:
        return check("workflow-permissions", "Least-privilege CI token permissions", why, None,
                     [F("No GitHub Actions workflows found.")])
    findings, weights = [], []
    for wf in wfs:
        t = repo.text(wf) or ""
        if not re.search(r"^permissions\s*:", t, re.M):
            weights.append(.08)
            findings.append(F("Workflow has no top-level `permissions:` block; GITHUB_TOKEN gets the repository "
                              "default scopes.", wf, 1))
        m = re.search(r"^permissions\s*:\s*write-all", t, re.M)
        if m:
            weights.append(.15)
            findings.append(F("Top-level `permissions: write-all` grants every scope to every job.", wf,
                              line_of(t, m.start())))
        trig = triggers(t) & set(UNTRUSTED_TRIGGERS)
        writes = list(WRITE_RE.finditer(t))
        if trig and writes:
            scopes = sorted({(w.group(1) or "write-all") for w in writes})
            weights.append(.20)
            findings.append(F("Workflow triggered by untrusted event(s) %s grants write scopes (%s)." %
                              (", ".join(sorted(trig)), ", ".join(scopes)), wf, line_of(t, writes[0].start())))
    z = zizmor(repo) or []
    extra = [x for x in z if x[0] in PERM_IDENTS]
    weights += ziz_weights(extra)
    findings += [x[3] for x in extra]
    return check("workflow-permissions", "Least-privilege CI token permissions", why, decay(weights), findings)


# ---------------------------------------------------------------- 4. secrets
SECRET_PATTERNS = [
    ("AWS access key id", r"(?<![A-Z0-9])(?:AKIA|ASIA)[0-9A-Z]{16}(?![A-Z0-9])", .35),
    ("GitHub token", r"\b(?:gh[pousr]_[A-Za-z0-9]{36,}|github_pat_[A-Za-z0-9_]{60,})\b", .35),
    ("GitLab token", r"\bglpat-[A-Za-z0-9_-]{20,}\b", .35),
    ("Slack token", r"\bxox[abprs]-[A-Za-z0-9-]{10,}\b", .35),
    ("Slack webhook URL", r"https://hooks\.slack\.com/services/T[A-Z0-9]+/B[A-Z0-9]+/[A-Za-z0-9]{20,}", .25),
    ("Google API key", r"\bAIza[0-9A-Za-z_-]{35}\b", .30),
    ("Stripe live key", r"\b(?:sk|rk)_live_[0-9a-zA-Z]{24,}\b", .35),
    ("npm token", r"\bnpm_[A-Za-z0-9]{36}\b", .35),
    ("PyPI token", r"\bpypi-AgE[A-Za-z0-9_-]{50,}", .35),
    ("Anthropic/OpenAI API key", r"\bsk-(?:ant-[A-Za-z0-9_-]{20,}|proj-[A-Za-z0-9_-]{20,}|[A-Za-z0-9]{48})\b", .35),
    ("Private key", r"-----BEGIN (?:RSA |EC |DSA |OPENSSH |PGP |ENCRYPTED )?PRIVATE KEY(?: BLOCK)?-----", .30),
    ("JSON Web Token", r"\beyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}", .10),
]
SECRET_RES = [(n, re.compile(p), w) for n, p, w in SECRET_PATTERNS]
GENERIC_RE = re.compile(
    r"""(?i)\b(password|passwd|pwd|secret|api[_-]?key|access[_-]?key|auth[_-]?token|access[_-]?token|"""
    r"""client[_-]?secret|private[_-]?key)\b["']?\s*[:=]\s*["']([^"'\s]{12,})["']""")
URL_CRED_RE = re.compile(r"\b[a-z][a-z0-9+.-]{1,15}://([^/\s:@'\"]+):([^/\s@'\"]{3,})@[^\s'\"]+", re.I)
PLACEHOLDER_RE = re.compile(r"(?i)^(\$|\{|<|%|\*+$)|example|changeme|change_me|placeholder|dummy|sample|"
                            r"your[_-]|redacted|xxx|password|secret|test|fake|^pass$|^admin$|^root$|\.\.\.")
SENSITIVE_FILES = [
    (r"(^|/)\.env(\.[a-z]+)?$", r"\.env\.(example|sample|template|dist|defaults?)$", "dotenv file"),
    (r"(^|/)id_(rsa|dsa|ecdsa|ed25519)$", None, "SSH private key"),
    (r"\.(p12|pfx|jks|keystore)$", None, "key store"),
    (r"(^|/)\.htpasswd$", None, "htpasswd file"),
    (r"(^|/)terraform\.tfstate(\.backup)?$", None, "Terraform state (often holds secrets)"),
    (r"(^|/)credentials\.json$", None, "credentials file"),
]
SKIP_SECRET_FILES = re.compile(r"(^|/)(go\.sum|[^/]*\.lock|package-lock\.json|pnpm-lock\.yaml|yarn\.lock|"
                               r"npm-shrinkwrap\.json)$|\.(svg|min\.js|map)$")


def entropy(s):
    if not s:
        return 0.0
    counts = {c: s.count(c) for c in set(s)}
    return -sum(v / len(s) * math.log2(v / len(s)) for v in counts.values())


def check_secrets(repo):
    why = ("Scans every text file for credentials (cloud keys, VCS/package-registry tokens, private keys, "
           "high-entropy password assignments, credentials embedded in URLs) and for committed credential "
           "files (.env, key stores, Terraform state). A secret in the tree is exposed to everyone with read "
           "access and lives on in history, so it must be rotated, not just deleted. Findings in test/fixture "
           "paths weigh less.")
    findings, weights = [], []
    for rel in repo.files:
        for pat, allow, label in SENSITIVE_FILES:
            if re.search(pat, rel) and not (allow and re.search(allow, rel)):
                weights.append(("file", .2 * (.3 if is_test(rel) else 1)))
                findings.append(F("Committed %s." % label, rel, None))
        if SKIP_SECRET_FILES.search(rel):
            continue
        t = repo.text(rel)
        if not t:
            continue
        test = is_test(rel)
        scale = .3 if test else 1.0
        suffix = " (test/fixture path)" if test else ""
        hits = 0
        for name, rx, w in SECRET_RES:
            for m in rx.finditer(t):
                hits += 1
                if hits > 20:
                    break
                weights.append((name, w * scale))
                findings.append(F("Possible %s%s." % (name, suffix), rel, line_of(t, m.start())))
        for m in GENERIC_RE.finditer(t):
            val = m.group(2)
            if PLACEHOLDER_RE.search(val) or entropy(val) < 3.5 or re.fullmatch(r"[A-Za-z_.]+(\(\))?", val):
                continue
            weights.append(("generic", .08 * scale))
            findings.append(F("High-entropy value assigned to '%s'%s." % (m.group(1), suffix), rel,
                              line_of(t, m.start())))
        for m in URL_CRED_RE.finditer(t):
            pw = m.group(2)
            if pw.startswith(("$", "{", "<", "%")) or pw == "***":
                continue
            weak = bool(PLACEHOLDER_RE.search(pw)) or entropy(pw) < 3.0
            if weak and test:
                continue  # dev defaults like postgres:password@localhost in tests are expected
            weights.append(("url-weak" if weak else "url", (.005 if weak else .10) * scale))
            findings.append(F("Credentials embedded in a URL (user '%s')%s%s." %
                              (m.group(1), suffix, "; looks like a local/dev default" if weak else ""),
                              rel, line_of(t, m.start())))
    return check("secrets", "No secrets committed to the repository", why,
                 decay([w for _, w in capped(weights, lambda x: x[0], 10)]), findings)


# ---------------------------------------------------------------- 5. SAST
SEM_W = {"ERROR": .12, "WARNING": .05, "INFO": .01}
RUFF_HIGH = {"S102", "S301", "S307", "S501", "S502", "S503", "S506", "S602", "S605", "S608", "S701", "S702"}
RUFF_LOW = {"S110", "S112", "S311", "S603", "S607", "S404", "S403"}
TEST_GLOBS = ["*_test.go", "test_*.py", "*_test.py", "conftest.py", "*.test.js", "*.test.ts", "*.spec.js",
              "*.spec.ts", "*.test.tsx", "*.spec.tsx", "tests", "test", "testdata", "testutils", "mocks",
              "__tests__", "__mocks__", "fixtures", "e2e", "node_modules", "vendor", "target"]
SRC_RE = r"\.(go|py|js|jsx|mjs|cjs|ts|tsx|rs|sh|bash)$"


def check_sast(repo):
    why = ("Static analysis of non-test source code for injection and crypto/transport mistakes: semgrep with "
           "bundled local rules (Go, JS/TS, Rust, shell: disabled TLS verification, SQL built from strings, "
           "shell command injection, servers without timeouts, unescaped HTML, weak randomness) and ruff's "
           "flake8-bandit (S) rules for Python. These are the bug classes that most often become exploitable "
           "vulnerabilities.")
    srcs = [f for f in repo.match(SRC_RE) if not is_test(f)]
    if not srcs:
        return check("static-analysis", "Static analysis for vulnerable code patterns", why, None,
                     [F("No supported source files (Go, Python, JS/TS, Rust, shell) outside tests.")])
    findings, weighted = [], []
    ran = False
    if tool("semgrep") and os.path.exists(RULES):
        with tempfile.TemporaryDirectory() as home:
            env = dict(os.environ, HOME=home, SEMGREP_SEND_METRICS="off", SEMGREP_ENABLE_VERSION_CHECK="0",
                       XDG_CONFIG_HOME=home, XDG_CACHE_HOME=home)
            args = ["semgrep", "scan", "--config", RULES, "--metrics=off", "--disable-version-check", "--json",
                    "--quiet", "--timeout", "30"]
            for g in TEST_GLOBS:
                args += ["--exclude", g]
            rc, out, err = run_tool(args + ["."], cwd=repo.root, env=env)
        try:
            data = json.loads(out)
            ran = True
        except ValueError:
            data = {}
            findings.append(F("semgrep failed to run: %s" % (err.strip().splitlines() or ["?"])[-1]))
        for r in data.get("results", []):
            rid = r["check_id"].split(".")[-1]
            sev = r.get("extra", {}).get("severity", "INFO")
            path = r.get("path", "")
            path = path[2:] if path.startswith("./") else path
            if is_test(path):
                continue
            weighted.append((rid, SEM_W.get(sev, .01),
                             F("[semgrep %s, %s] %s" % (rid, sev.lower(), r.get("extra", {}).get("message", "")),
                               path, r.get("start", {}).get("line"))))
    else:
        findings.append(F("semgrep is not installed; Go/JS/Rust/shell analysis skipped."))
    pys = [f for f in srcs if f.endswith(".py")]
    if pys and tool("ruff"):
        args = ["ruff", "check", "--isolated", "--no-cache", "--select", "S", "--ignore", "S101",
                "--output-format", "json", "--exit-zero"]
        rc, out, err = run_tool(args + pys, cwd=repo.root)
        try:
            data = json.loads(out or "[]")
            ran = True
        except ValueError:
            data = []
        for r in data:
            code = r.get("code") or ""
            w = .12 if code in RUFF_HIGH else .01 if code in RUFF_LOW else .05
            path = os.path.relpath(r.get("filename", ""), repo.root)
            weighted.append((code, w, F("[ruff %s] %s" % (code, r.get("message")), path,
                                        (r.get("location") or {}).get("row"))))
    elif pys:
        findings.append(F("ruff is not installed; Python analysis skipped."))
    if not ran:
        return check("static-analysis", "Static analysis for vulnerable code patterns", why, None, findings)
    findings = [f for _, _, f in weighted] + findings
    return check("static-analysis", "Static analysis for vulnerable code patterns", why,
                 decay([w for _, w, _ in capped(weighted, lambda x: x[0], 5)]), findings)


# ---------------------------------------------------------------- 6. dependency management
MANIFESTS = {
    "go.mod": "gomod", "package.json": "npm", "Cargo.toml": "cargo", "pyproject.toml": "pip",
    "setup.py": "pip", "Pipfile": "pip", "Gemfile": "bundler", "pom.xml": "maven", "build.gradle": "gradle",
    "build.gradle.kts": "gradle", "composer.json": "composer",
}
ECO_ALIASES = {"pip": {"pip", "uv", "pipenv", "poetry"}, "npm": {"npm", "yarn", "pnpm", "bun"}}


def find_up(repo, d, names):
    while True:
        for n in names:
            if (d + "/" + n if d else n) in repo.fileset:
                return True
        if not d:
            return False
        d = d.rsplit("/", 1)[0] if "/" in d else ""


def parse_dependabot(text):
    entries, cur = [], None
    for ln in text.splitlines():
        m = re.match(r"""^\s*-?\s*package-ecosystem\s*:\s*["']?([\w-]+)""", ln)
        if m:
            cur = {"eco": m.group(1), "dirs": [], "line": None}
            entries.append(cur)
            continue
        if cur is None:
            continue
        m = re.match(r"""^\s*directory\s*:\s*["']?([^"'#\s]+)""", ln)
        if m:
            cur["dirs"].append(m.group(1))
        m = re.match(r"""^\s*-\s*["']?(/[^"'#\s]*)["']?\s*$""", ln)
        if m:
            cur["dirs"].append(m.group(1))
    return entries


def check_dependencies(repo):
    why = ("Dependencies should be locked (go.sum, package-lock, Cargo.lock, poetry/uv locks, pinned "
           "requirements) so builds are reproducible and cannot silently pull a newly published malicious "
           "version, and kept current by automation (Dependabot or Renovate) covering every ecosystem in the "
           "repo, so security fixes actually land. Stale update config fails silently.")
    findings, extra_w = [], []
    ecos, manifests = {}, []
    for rel in repo.files:
        base = rel.rsplit("/", 1)[-1]
        eco = MANIFESTS.get(base) or ("pip" if re.match(r"requirements[^/]*\.txt$", base) else None)
        if eco and not is_test(rel):
            ecos.setdefault(eco, rel)
            manifests.append((rel, eco))
    if repo.dockerfiles():
        ecos.setdefault("docker", repo.dockerfiles()[0])
    if repo.workflows():
        ecos.setdefault("github-actions", repo.workflows()[0])
    if not ecos:
        return check("dependency-management", "Locked and automatically updated dependencies", why, None,
                     [F("No package manifests, Dockerfiles or workflows found.")])
    # lockfiles
    need, locked = 0, 0
    for rel, _eco in manifests:
        d = rel.rsplit("/", 1)[0] if "/" in rel else ""
        base = rel.rsplit("/", 1)[-1]
        t = repo.text(rel) or ""
        ok, applies = True, True
        if base == "go.mod":
            applies = "require" in t
            ok = find_up(repo, d, ["go.sum"]) if d == "" else (d + "/go.sum") in repo.fileset
        elif base == "package.json":
            try:
                pj = json.loads(t)
                applies = bool(pj.get("dependencies") or pj.get("devDependencies") or pj.get("optionalDependencies"))
            except ValueError:
                applies = False
            ok = find_up(repo, d, ["package-lock.json", "npm-shrinkwrap.json", "yarn.lock", "pnpm-lock.yaml",
                                   "bun.lockb", "bun.lock"])
        elif base == "Cargo.toml":
            applies = "[dependencies]" in t or "[workspace" in t
            ok = find_up(repo, d, ["Cargo.lock"])
        elif base in ("pyproject.toml", "setup.py", "Pipfile"):
            applies = bool(re.search(r"(?m)^\s*dependencies\s*=\s*\[\s*\S|\[tool\.poetry\.dependencies\]|"
                                     r"install_requires|\[packages\]", t))
            ok = find_up(repo, d, ["poetry.lock", "uv.lock", "pdm.lock", "Pipfile.lock", "requirements.lock"]) or \
                any(re.match(re.escape(d + "/" if d else "") + r"requirements[^/]*\.(txt|lock)$", f)
                    for f in repo.files)
        elif base.startswith("requirements"):
            loose = [i + 1 for i, l in enumerate(t.splitlines())
                     if l.strip() and not l.strip().startswith(("#", "-")) and "==" not in l and "@" not in l]
            ok = not loose
            if loose:
                findings.append(F("%d requirement(s) not pinned with '=='." % len(loose), rel, loose[0]))
        elif base == "Gemfile":
            ok = find_up(repo, d, ["Gemfile.lock"])
        elif base == "composer.json":
            ok = find_up(repo, d, ["composer.lock"])
        elif base in ("pom.xml", "build.gradle", "build.gradle.kts"):
            applies = False  # lockfiles are optional in Maven/Gradle; not judged
        if not applies:
            continue
        need += 1
        if ok:
            locked += 1
        elif not base.startswith("requirements"):
            findings.append(F("Manifest has no lockfile next to it or in a parent directory.", rel, None))
    lockfrac = locked / need if need else 1.0
    # update automation
    dbot = next((f for f in (".github/dependabot.yml", ".github/dependabot.yaml") if f in repo.fileset), None)
    renov = next((f for f in ("renovate.json", "renovate.json5", ".renovaterc", ".renovaterc.json",
                              ".github/renovate.json", ".github/renovate.json5", ".gitlab/renovate.json")
                  if f in repo.fileset), None)
    if not renov and "package.json" in repo.fileset and '"renovate"' in (repo.text("package.json") or ""):
        renov = "package.json"
    auto, coverage = 0.0, 0.0
    if renov:
        auto, coverage = 1.0, 1.0
    elif dbot:
        auto = 1.0
        t = repo.text(dbot) or ""
        entries = parse_dependabot(t)
        configured = {e["eco"] for e in entries}
        covered = 0
        for eco, example in sorted(ecos.items()):
            if configured & ECO_ALIASES.get(eco, {eco}):
                covered += 1
            else:
                findings.append(F("Ecosystem '%s' (e.g. %s) is not covered by Dependabot." % (eco, example), dbot, None))
        coverage = covered / len(ecos)
        for e in entries:
            for d in e["dirs"]:
                if "*" in d or e["eco"] == "github-actions":
                    continue
                p = d.strip("/")
                if p and not any(f == p or f.startswith(p + "/") for f in repo.files):
                    extra_w.append(.05)
                    m = re.search(r"""directory\s*:\s*["']?%s""" % re.escape(d), t)
                    findings.append(F("Dependabot '%s' entry points at directory '%s', which does not exist; "
                                      "it updates nothing." % (e["eco"], d), dbot, line_of(t, m.start()) if m else None))
    else:
        findings.append(F("No Dependabot or Renovate configuration; dependency updates (and security fixes) "
                          "depend on someone remembering.", None, None))
    z = zizmor(repo) or []
    dz = [x for x in z if x[0] in DEP_IDENTS]
    findings += [x[3] for x in dz]
    score = 4 * auto + 3 * coverage + 3 * lockfrac
    score = round(score * decay(extra_w + [w * .5 for w in ziz_weights(dz)]) / 10.0, 1)
    return check("dependency-management", "Locked and automatically updated dependencies", why, score, findings)


# ---------------------------------------------------------------- 7. known vulnerabilities
def check_known_vulns(repo):
    why = ("Whether locked dependency versions have published advisories (OSV/GHSA/RustSec/Go vulndb). This "
           "needs a vulnerability database; none of the allowed tools carries one offline, so the check reports "
           "null rather than guessing. Run govulncheck/osv-scanner in CI to cover it (see security-tooling-in-ci).")
    return check("known-vulnerabilities", "Dependencies free of known vulnerabilities", why, None,
                 [F("Not evaluated: no offline vulnerability database is available to this scorecard.")])


# ---------------------------------------------------------------- 8. security tooling in CI
TOOLING = {
    "SAST": [r"github/codeql-action/(init|analyze)", r"\bsemgrep\b", r"\bgosec\b", r"\bbandit\b",
             r"sonarsource|sonarcloud|sonar-scanner", r"snyk code", r"\bbrakeman\b", r"eslint-plugin-security"],
    "dependency vulnerability scanning": [r"\bgovulncheck\b", r"osv-scanner", r"dependency-review-action",
                                          r"snyk (test|monitor)|snyk/actions/(?!docker)", r"\btrivy\b", r"\bgrype\b", r"npm audit", r"pnpm audit",
                                          r"yarn (npm )?audit", r"cargo[- ]audit", r"cargo[- ]deny",
                                          r"pip-audit", r"\bsafety check", r"anchore/scan-action"],
    "secret scanning": [r"\bgitleaks\b", r"\btrufflehog\b", r"detect-secrets", r"\bggshield\b", r"secretlint"],
    "CI/supply-chain linting": [r"\bzizmor\b", r"\bactionlint\b", r"ossf/scorecard-action",
                                r"step-security/harden-runner"],
}
CONTAINER_SCAN = [r"\btrivy\b", r"\bgrype\b", r"docker scout", r"snyk container", r"snyk/actions/docker",
                  r"anchore/scan-action"]
CI_FILES = r"^(\.github/workflows/[^/]+\.ya?ml|\.gitlab-ci\.yml|\.circleci/config\.yml|Jenkinsfile|" \
           r"azure-pipelines\.yml|bitbucket-pipelines\.yml|\.pre-commit-config\.yaml|\.buildkite/.*\.ya?ml)$"


def check_ci_tooling(repo):
    why = ("Point-in-time scans decay; the repo should run security tools on every change. Looks in CI and "
           "pre-commit config for four categories: SAST, dependency vulnerability scanning, secret scanning, and "
           "CI/supply-chain linting, plus container image scanning when the repo builds images.")
    ci = repo.match(CI_FILES)
    if not ci:
        return check("security-tooling-in-ci", "Security scanning runs continuously in CI", why, 0.0,
                     [F("No CI configuration found; nothing runs security checks on changes.")])
    blob = "\n".join((repo.text(f) or "") for f in ci)
    for cfg in repo.match(r"^\.golangci\.ya?ml$"):
        if re.search(r"\bgosec\b", repo.text(cfg) or "") and re.search(r"golangci", blob):
            blob += "\ngosec"
    cats = dict(TOOLING)
    if repo.dockerfiles():
        cats["container image scanning"] = CONTAINER_SCAN
    findings, have = [], 0
    for cat, pats in cats.items():
        if any(re.search(p, blob, re.I) for p in pats):
            have += 1
        else:
            findings.append(F("No %s found in CI configuration." % cat))
    return check("security-tooling-in-ci", "Security scanning runs continuously in CI", why,
                 round(10.0 * have / len(cats), 1), findings)


# ---------------------------------------------------------------- 9. container hardening
SECRETY = re.compile(r"(?i)(password|passwd|secret|token|api_?key|private_?key|credentials)")
NONROOT_BASE = re.compile(r"(?i)cgr\.dev/chainguard/(?!.*-dev)|distroless/.*nonroot|:nonroot|nonroot@|rootless")


def docker_instructions(text):
    out, buf, start = [], "", None
    for i, ln in enumerate(text.splitlines(), 1):
        s = ln.strip()
        if not buf and (not s or s.startswith("#")):
            continue
        if start is None:
            start = i
        buf += " " + s.rstrip("\\")
        if not s.endswith("\\"):
            out.append((start, buf.strip()))
            buf, start = "", None
    if buf:
        out.append((start, buf.strip()))
    return out


def check_containers(repo):
    why = ("Container images should run as a non-root user, avoid fetching unverified remote code at build time "
           "(ADD <url>, curl | sh), never bake secrets into ENV/ARG layers, and ship a .dockerignore so .git and "
           ".env files stay out of the build context. Compose files should not run privileged or mount the "
           "Docker socket. These limit what an attacker gets from a compromised process.")
    dfs = repo.dockerfiles()
    composes = [f for f in repo.match(r"(^|/)(docker-)?compose[^/]*\.ya?ml$") if not is_test(f)]
    if not dfs and not composes:
        return check("container-hardening", "Hardened container builds", why, None,
                     [F("No Dockerfiles or compose files found.")])
    findings, weights = [], []
    for df in dfs:
        t = repo.text(df) or ""
        ins = docker_instructions(t)
        last_from = max((i for i, (_, s) in enumerate(ins) if s.upper().startswith("FROM ")), default=-1)
        final = ins[last_from:] if last_from >= 0 else ins
        users = [(ln, s) for ln, s in final if s.upper().startswith("USER ")]
        base = final[0][1] if final else ""
        if users:
            ln, s = users[-1]
            u = s.split(None, 1)[1].split(":")[0].strip() if len(s.split()) > 1 else ""
            if u in ("root", "0"):
                weights.append(.25)
                findings.append(F("Final stage switches to USER %s." % u, df, ln))
        elif not NONROOT_BASE.search(base):
            weights.append(.20)
            findings.append(F("Final stage sets no USER and its base image is not a known non-root image, so the "
                              "container runs as root.", df, final[0][0] if final else None))
        for ln, s in ins:
            up = s.upper()
            if up.startswith("ADD ") and re.search(r"https?://", s):
                weights.append(.10)
                findings.append(F("ADD fetches a remote URL without checksum verification; use curl + sha256sum "
                                  "or ADD --checksum.", df, ln))
            if up.startswith("RUN ") and re.search(r"(curl|wget)[^|;&]*\|\s*(sudo\s+)?(ba|z|da)?sh\b", s):
                weights.append(.15)
                findings.append(F("RUN pipes a download straight into a shell.", df, ln))
            if up.startswith(("ENV ", "ARG ")):
                for name in re.findall(r"([A-Za-z_][A-Za-z0-9_]*)\s*=\s*\S+", s[4:]) or s[4:].split()[:1]:
                    if SECRETY.search(name) and re.search(re.escape(name) + r"\s*=?\s*\S", s[4:]):
                        weights.append(.20)
                        findings.append(F("%s %s puts a secret-like value in an image layer; use build secrets "
                                          "(RUN --mount=type=secret)." % (up[:3], name), df, ln))
            if up.startswith("RUN ") and re.search(r"chmod\s+(-R\s+)?0?777", s):
                weights.append(.05)
                findings.append(F("RUN makes files world-writable (chmod 777).", df, ln))
        d = df.rsplit("/", 1)[0] if "/" in df else ""
        if not ((d + "/.dockerignore") if d else ".dockerignore") in repo.fileset and ".dockerignore" not in repo.fileset:
            weights.append(.05)
            findings.append(F("No .dockerignore; .git, .env and local files may be copied into the image.", df, None))
    for c in composes:
        t = repo.text(c) or ""
        for rx, w, msg in [(r"privileged\s*:\s*true", .10, "Service runs privileged."),
                           (r"/var/run/docker\.sock", .10, "Docker socket mounted into a container (root on host)."),
                           (r"network_mode\s*:\s*[\"']?host", .05, "Service uses host networking.")]:
            for m in re.finditer(rx, t):
                weights.append(w)
                findings.append(F(msg, c, line_of(t, m.start())))
    return check("container-hardening", "Hardened container builds", why, decay(weights), findings)


# ---------------------------------------------------------------- 10. security policy & ownership
def check_policy(repo):
    why = ("A SECURITY.md tells researchers how to report a vulnerability privately; without one, reports land "
           "in public issues or nowhere. CODEOWNERS makes review by the right people mandatory (with branch "
           "protection), and covering CI config and itself stops a contributor from quietly changing release "
           "workflows. Score: SECURITY.md with a reporting channel 5, CODEOWNERS 3, CODEOWNERS covers .github/ 2.")
    findings, score = [], 0.0
    sec = [f for f in repo.files if re.match(r"^(\.github/|docs/)?security\.(md|rst|txt|adoc)$", f, re.I)]
    if sec:
        t = repo.text(sec[0]) or ""
        if re.search(r"(?i)@|security/advisories|report|hackerone|bugcrowd|disclos", t):
            score += 5
        else:
            score += 3
            findings.append(F("SECURITY.md does not say how to report a vulnerability.", sec[0], None))
    else:
        findings.append(F("No SECURITY.md (root, .github/ or docs/). GitHub may fall back to an org-level policy, "
                          "which this scorecard cannot see."))
    co = next((f for f in ("CODEOWNERS", ".github/CODEOWNERS", "docs/CODEOWNERS", ".gitlab/CODEOWNERS")
               if f in repo.fileset), None)
    if co:
        score += 3
        rules = [l.split() for l in (repo.text(co) or "").splitlines() if l.strip() and not l.startswith("#")]
        if any(r[0] in ("*", "/*", "/**", "**") or r[0].lstrip("/").startswith(".github") for r in rules if len(r) > 1):
            score += 2
        else:
            findings.append(F("CODEOWNERS has no rule covering .github/ (workflows and CODEOWNERS itself).", co, None))
    else:
        findings.append(F("No CODEOWNERS file; no required reviewers for sensitive paths."))
    return check("security-policy", "Vulnerability disclosure policy and code ownership", why, score, findings)


# ---------------------------------------------------------------- 11. release integrity
RELEASE_HINTS = r"goreleaser|npm publish|cargo publish|twine upload|uv publish|gh release create|" \
                r"softprops/action-gh-release|docker/build-push-action|docker push|pypa/gh-action-pypi-publish|" \
                r"ko build|jib"


def check_release(repo):
    why = ("Published artifacts should be verifiable: build provenance (SLSA / GitHub attestations) proves which "
           "source and workflow produced them, signatures (Sigstore/cosign, GPG) prove who released them, and an "
           "SBOM lets consumers find vulnerable components. Without these, a tampered release is "
           "indistinguishable from a real one. Score: provenance 4, signing 4, SBOM 2.")
    files = repo.match(CI_FILES) + repo.match(r"^\.goreleaser[^/]*\.ya?ml$|^goreleaser[^/]*\.ya?ml$")
    blob = "\n".join((repo.text(f) or "") for f in files)
    if not re.search(RELEASE_HINTS, blob, re.I):
        return check("release-integrity", "Signed releases with provenance and SBOM", why, None,
                     [F("No release/publish automation detected.")])
    findings, score = [], 0.0
    for label, pts, rx in [
        ("build provenance (slsa-github-generator, actions/attest-build-provenance, --provenance)", 4,
         r"slsa-github-generator|attest-build-provenance|actions/attest@|--provenance|provenance:\s*true"),
        ("artifact signing (cosign/sigstore, goreleaser signs, GPG)", 4,
         r"\bcosign\b|sigstore|^\s*signs\s*:|docker_signs\s*:|gpg --detach-sign|npm publish[^\n]*--provenance"),
        ("SBOM generation (syft, CycloneDX, SPDX, goreleaser sboms)", 2,
         r"\bsyft\b|cyclonedx|\bspdx\b|^\s*sboms\s*:|sbom-action|sbom:\s*true")]:
        if re.search(rx, blob, re.I | re.M):
            score += pts
        else:
            findings.append(F("Release automation has no %s." % label))
    return check("release-integrity", "Signed releases with provenance and SBOM", why, score, findings)


# ---------------------------------------------------------------- main
CHECKS = [check_ci_audit, check_permissions, check_pinning, check_secrets, check_sast, check_dependencies,
          check_known_vulns, check_ci_tooling, check_containers, check_policy, check_release]


def main(argv):
    if len(argv) != 2 or not os.path.isdir(argv[1]):
        sys.stderr.write("usage: run <repo-path>\n")
        return 2
    repo = Repo(argv[1])
    checks = []
    for fn in CHECKS:
        try:
            checks.append(fn(repo))
        except Exception as e:  # one broken check must not sink the report
            checks.append(check(fn.__name__.replace("check_", ""), fn.__name__, "internal error", None,
                                [F("Check crashed: %r" % e)]))
    scored = [c["score"] for c in checks if c["score"] is not None]
    out = {
        "repo": repo.root,
        "checks": checks,
        "summary": {
            "score": round(sum(scored) / len(scored), 1) if scored else None,
            "method": "mean of evaluated checks (null checks excluded)",
            "evaluated": len(scored),
            "not_evaluated": [c["id"] for c in checks if c["score"] is None],
            "files_scanned": len(repo.files),
        },
        "tools": {t: tool_version(t) for t in ("zizmor", "actionlint", "semgrep", "ruff")},
    }
    json.dump(out, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
