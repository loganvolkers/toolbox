#!/usr/bin/env python3
"""Turn raw/ into the product records the vs-spicedb scorecards read, and print the tables
RESULTS.md is written from. Usage: python3 snapshot.py (after ./run.sh).

Writes examples/openfga-portal/comparisons/{openfga,spicedb}.yaml, then formats them with yamark.
Timed and memory facts are medians over the rounds; every other fact is from raw/facts.json, where
facts.py recorded what each probe, count or facts.yaml entry decided.

The tally follows README.md: on each rule OpenFGA is ahead, at parity or behind SpiceDB (a measured
value decides, within the rule's tolerance is parity; otherwise passing beats failing, as
toolbox-derive's rule_standing does). A factor is ahead when OpenFGA is ahead on more rules than it
is behind, parity when as many, behind otherwise.
"""
import glob
import json
import shutil
import statistics
import subprocess
import sys
from pathlib import Path

import yaml

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
OUT = ROOT / "examples" / "openfga-portal" / "comparisons"
STANDARDS = ROOT / "examples" / "openfga-portal" / "standards" / "vs-spicedb.yaml"
ARMS = {"openfga": ("OpenFGA", "v1.23.0"), "spicedb": ("SpiceDB", "v1.57.1")}
# record field: (path in a round's JSON, decimals)
TIMED = {
    "readyS": (("readyS",), 3),
    "writeTuplesPerS": (("write", "perS"), 0),
    "checkP95Ms": (("check", "p95Ms"), 2),
    "checkRps": (("check", "rps"), 0),
    "listP95Ms": (("list", "p95Ms"), 2),
    "rssIdleMb": (("rssIdleMb",), 1),
    "rssLoadedMb": (("rssLoadedMb",), 1),
}
COUNTED = ["binaryMb", "goModules", "knownVulns", "provenance", "sbom"]
DECLARED = ["conditions", "contextualTuples", "wildcards", "exclusion", "listUsers",
            "consistencyTokens", "changeFeed", "modelTests", "multiTenant", "prometheusMetrics",
            "otelTracing", "httpByDefault", "migrateCommand", "datastores"]
ECOSYSTEM = ["contributors", "releasesLastYear", "officialSdks", "foundation"]


def get(d, path):
    for p in path:
        d = d[p]
    return d


def num(v, decimals):
    v = round(v, decimals)
    return int(v) if decimals == 0 else v


def rounds(arm):
    return [json.loads(Path(p).read_text()) for p in sorted(glob.glob(str(HERE / "raw" / f"round-*-{arm}.json")))]


def record(arm, rs, facts, env):
    title, version = ARMS[arm]
    f = facts["arms"][arm]
    decided = {k: v["value"] for k, v in f["decided"].items()}
    rec = {"id": arm, "title": title, "version": version, "harness": env["harness"],
           "corpus": rs[0]["write"]["relationships"], "runs": len(rs)}
    for name, (path, dec) in TIMED.items():
        rec[name] = num(statistics.median(get(r, path) for r in rs), dec)
    rec["binaryMb"] = f["binaryMb"]
    rec["goModules"] = f["goModules"]
    rec["knownVulns"] = f["knownVulns"]
    rec["provenance"] = decided["provenance"]
    rec["sbom"] = decided["sbom"]
    for name in DECLARED:
        rec[name] = decided[name]
    rec["contributors"] = f["contributors"]
    rec["releasesLastYear"] = f["releasesLastYear"]
    rec["officialSdks"] = decided["officialSdks"]
    rec["foundation"] = decided["foundation"]
    return rec


def in_kind_order(recs):
    """Keys exactly the Product kind's fields, in its order, after `id`."""
    kind = next(k for k in yaml.safe_load(STANDARDS.read_text())["kinds"] if k["name"] == "Product")
    want = list(kind["fields"])
    out = {}
    for arm, rec in recs.items():
        got = set(rec) - {"id"}
        if got != set(want):
            sys.exit(f"snapshot.py: {arm} keys differ from the Product kind's fields: "
                     f"missing {sorted(set(want) - got)}, extra {sorted(got - set(want))}")
        out[arm] = {"id": rec["id"], **{k: rec[k] for k in want}}
    return out


def write(arm, rec):
    title = ARMS[arm][0]
    text = (f"# {title}, as measured by experiments/openfga-spicedb-compare.\n"
            "# Written by snapshot.py; rerun the harness rather than editing it.\n"
            + yaml.safe_dump({"records": [rec]}, sort_keys=False, allow_unicode=True, width=1000))
    path = OUT / f"{arm}.yaml"
    path.write_text(text)
    return path


def bar_pass(rule, rec):
    """Evaluate the rule's check, which in vs-spicedb.yaml is one comparison or a boolean."""
    expr = rule["check"].replace("entity.", "")
    return bool(eval(expr, {}, dict(rec)))  # the checks are simple comparisons over record fields


def standing(rule, a, b):
    if "value" in rule:
        va, vb = a[rule["value"].replace("entity.", "")], b[rule["value"].replace("entity.", "")]
        tol = rule.get("tolerance", 0.0) * max(abs(va), abs(vb))
        if abs(va - vb) <= tol:
            return "parity"
        better = va < vb if rule.get("better") == "lower" else va > vb
        return "ahead" if better else "behind"
    pa, pb = bar_pass(rule, a), bar_pass(rule, b)
    return "parity" if pa == pb else ("ahead" if pa else "behind")


def tally(recs):
    cards = yaml.safe_load(STANDARDS.read_text())["scorecards"]
    lines = ["| Factor | Rule | OpenFGA | SpiceDB | OpenFGA bar | SpiceDB bar | OpenFGA is |",
             "|---|---|---|---|---|---|---|"]
    verdicts = []
    for card in cards:
        n = {"ahead": 0, "parity": 0, "behind": 0}
        for rule in card["rules"]:
            s = standing(rule, recs["openfga"], recs["spicedb"])
            n[s] += 1
            field = (rule.get("value") or rule["check"]).replace("entity.", "").split()[0]
            pa, pb = bar_pass(rule, recs["openfga"]), bar_pass(rule, recs["spicedb"])
            lines.append(f"| {card['title']} | `{rule['name']}` | {recs['openfga'][field]} | "
                         f"{recs['spicedb'][field]} | {'pass' if pa else 'fail'} | "
                         f"{'pass' if pb else 'fail'} | {s} |")
        v = "ahead" if n["ahead"] > n["behind"] else "parity" if n["ahead"] == n["behind"] else "behind"
        verdicts.append(f"| {card['title']} | {n['ahead']} | {n['parity']} | {n['behind']} | **{v}** |")
    return (["| Factor | Ahead | Parity | Behind | OpenFGA is |", "|---|---|---|---|---|"] + verdicts,
            lines)


def spread(rs_by_arm):
    lines = ["| Fact | OpenFGA min / median / max | SpiceDB min / median / max |", "|---|---|---|"]
    extra = {"checkP50Ms": (("check", "p50Ms"), 2), "checkP99Ms": (("check", "p99Ms"), 2),
             "listP50Ms": (("list", "p50Ms"), 2), "rssAfterCheckMb": (("rssAfterCheckMb",), 1),
             "equivalenceS": (("equivalence", "seconds"), 2)}
    for name, (path, dec) in {**TIMED, **extra}.items():
        cells = []
        for arm in ARMS:
            vals = [get(r, path) for r in rs_by_arm[arm]]
            cells.append(f"{num(min(vals), dec)} / {num(statistics.median(vals), dec)} / {num(max(vals), dec)}")
        lines.append(f"| `{name}` | {cells[0]} | {cells[1]} |")
    return lines


def main():
    facts = json.loads((HERE / "raw" / "facts.json").read_text())
    env = json.loads((HERE / "raw" / "env.json").read_text())
    rs = {arm: rounds(arm) for arm in ARMS}
    for arm, r in rs.items():
        if len(r) != env["rounds"]:
            sys.exit(f"snapshot.py: {len(r)} rounds for {arm}, expected {env['rounds']}")
    digests = {(r["equivalence"]["decisionsDigest"], r["equivalence"]["listsDigest"])
               for arm in ARMS for r in rs[arm]}
    if len(digests) != 1:
        sys.exit("snapshot.py: the rounds' answers differ; the run should have stopped")
    recs = in_kind_order({arm: record(arm, rs[arm], facts, env) for arm in ARMS})
    OUT.mkdir(parents=True, exist_ok=True)
    paths = [write(arm, rec) for arm, rec in recs.items()]
    yamark = shutil.which("yamark") or str(Path.home() / ".local" / "bin" / "yamark")
    subprocess.run([yamark, "format", *map(str, paths)], check=True)
    verdicts, rules = tally(recs)
    print("## Verdicts\n\n" + "\n".join(verdicts))
    print("\n## Rules\n\n" + "\n".join(rules))
    print("\n## Spread over rounds\n\n" + "\n".join(spread(rs)))
    d = digests.pop()
    print(f"\ndecisionsDigest {d[0]}\nlistsDigest {d[1]}")
    print("\nOracle match in every round: "
          + str(all(r["equivalence"]["oracleMatch"] for arm in ARMS for r in rs[arm])))
    print("Check mismatches / errors: " + ", ".join(
        f"{arm} {sum(r['check']['mismatches'] for r in rs[arm])} / {sum(r['check']['errors'] for r in rs[arm])}"
        for arm in ARMS))
    print("Load average start..end per round: " + "; ".join(
        f"{r['round']}-{r['arm']} {r['loadavgStart'][0]}..{r['loadavgEnd'][0]} (waited {r.get('loadWaitS')} s"
        f"{', gave up' if r.get('loadWaitGaveUp') else ''})"
        for r in sorted((r for arm in ARMS for r in rs[arm]), key=lambda r: (r["round"], r["order"]))))
    for p in paths:
        print(f"wrote {p.relative_to(ROOT)}", file=sys.stderr)


if __name__ == "__main__":
    main()
