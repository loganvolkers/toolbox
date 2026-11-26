"""Phase 2: apply scripted changes one at a time to an arm's committed result, run its gate, reset.
Usage: gate_test.py <repo dir> <s1|s2|s3> '<gate command>' -> one JSON line per case."""
import json, subprocess, sys
repo, std, gate = sys.argv[1], sys.argv[2], sys.argv[3]
def append(path, text):
    with open(f"{repo}/{path}", "a") as f: f.write(text)
def replace_first(path, old, new):
    p = f"{repo}/{path}"; s = open(p).read(); assert old in s, (path, old); open(p, "w").write(s.replace(old, new, 1))
def drop_last(path, n):
    p = f"{repo}/{path}"; lines = open(p).read().split("\n"); open(p, "w").write("\n".join(lines[:-n-1]) + "\n")
cases = {
  "s1": [("new violation", "fail", lambda: append("posthog/api/team.py", "\n_probe = Person.objects.filter(pk=0)\n")),
         ("fix one", "pass", lambda: replace_first("posthog/management/commands/split_person.py", "Person.objects", "person_queries")),
         ("unrelated edit", "pass", lambda: append("posthog/api/team.py", "\n# unrelated\n"))],
  "s2": [("violator grows", "fail", lambda: append("codex-rs/core-plugins/src/marketplace.rs", "// x\n" * 5)),
         ("violator shrinks below 800", "pass", lambda: drop_last("codex-rs/linux-sandbox/src/proxy_routing.rs", 5)),
         ("unrelated edit", "pass", lambda: append("codex-rs/codex-client/src/retry.rs", "// unrelated\n")),
         ("file crosses 800", "fail", lambda: append("codex-rs/codex-client/src/custom_ca.rs", "// y\n" * 10))],
  "s3": [("violator grows", "fail", lambda: append("pkg/storage/test/tuples.go", "// x\n" * 5)),
         ("violator shrinks", "pass", lambda: drop_last("pkg/typesystem/typesystem.go", 5)),
         ("unrelated edit", "pass", lambda: append("cmd/openfga/main.go", "// unrelated\n")),
         ("file crosses 1000", "fail", lambda: append("pkg/server/config/config.go", "// y\n" * 10))],
}[std]
for name, want, change in cases:
    subprocess.run(["git", "-C", repo, "stash", "-u", "-q"], capture_output=True)  # start from the committed state
    change()
    r = subprocess.run(gate, shell=True, cwd=repo, capture_output=True, text=True, timeout=1800)
    got = "pass" if r.returncode == 0 else "fail"
    out = (r.stdout + r.stderr).strip().splitlines()
    print(json.dumps({"case": name, "want": want, "got": got, "exit": r.returncode, "ok": got == want, "tail": out[-4:]}))
    subprocess.run(["git", "-C", repo, "checkout", "-q", "--", "."]); subprocess.run(["git", "-C", repo, "clean", "-fdq", "-e", ".seikika", "-e", ".standards"])
