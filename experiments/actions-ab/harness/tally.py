#!/usr/bin/env python3
"""README §6 from the adjudicator's records and the measures: each arm's runs, medians and W, and
H1 to H3 against each rival. Writes Markdown to stdout.

    tally.py <results/r<n>>

Reads `adjudication/<run>.json` (linesAfterStrikes, fullyPasses) and `<run>.count.json`
(toolCalls) for each `<arm>-<k>`.
"""
import json
import pathlib
import re
import statistics
import sys

ARMS = ["seikika", "gha", "backstage", "vanilla"]
PROBES = ["C1"]


def verdict(v) -> str:
    return v if isinstance(v, str) else v.get("verdict", "?")


def load(d: pathlib.Path):
    runs = {}
    records = [json.loads(p.read_text()) for p in sorted((d / "adjudication").glob("*-*.json"))]
    found = {k for a in records for k in a if re.fullmatch(r"P\d+", k)}
    PROBES[1:] = sorted(found, key=lambda k: int(k[1:]))
    for a, adj in zip(records, sorted((d / "adjudication").glob("*-*.json"))):
        name = a.get("run", adj.stem)
        count = json.loads((d / f"{name}.count.json").read_text())
        runs[name] = {
            "arm": name.rsplit("-", 1)[0],
            "lines": a["linesAfterStrikes"],
            "calls": count["toolCalls"],
            "passes": bool(a["fullyPasses"]),
            "probes": {p: verdict(a.get(p, "?")) for p in PROBES},
        }
    return runs


def compare(seikika_times: list, rival: list) -> str:
    w = max(max(seikika_times) - min(seikika_times), max(rival) - min(rival))
    diff = statistics.median(rival) - statistics.median(seikika_times)
    state = "holds" if diff > w else "lost" if -diff > w else "inconclusive"
    return f"{state} (Seikika {statistics.median(seikika_times):g}, rival {statistics.median(rival):g}, W {w:g})"


def main() -> None:
    d = pathlib.Path(sys.argv[1])
    runs = load(d)
    out = ["| Run | " + " | ".join(PROBES) + " | Fully passes | Lines | Calls |",
           "|---|" + "---|" * (len(PROBES) + 3)]
    for name, r in sorted(runs.items(), key=lambda kv: (ARMS.index(kv[1]["arm"]), kv[0])):
        out.append(f"| {name} | " + " | ".join(r["probes"][p] for p in PROBES)
                   + f" | {'yes' if r['passes'] else 'no'} | {r['lines']} | {r['calls']} |")
    by = {a: [r for r in runs.values() if r["arm"] == a] for a in ARMS}
    if any(len(v) < 3 for v in by.values()):
        out.append("\nIncomplete: every arm needs three adjudicated runs.")
        print("\n".join(out))
        return
    passes = {a: sum(r["passes"] for r in v) for a, v in by.items()}
    h1 = passes["seikika"] >= 2 and all(passes["seikika"] >= passes[a] for a in ARMS[1:])
    out += ["", f"**H1 (delivery):** {'holds' if h1 else 'fails'}. Fully passing runs: "
            + ", ".join(f"{a} {passes[a]}/3" for a in ARMS) + ".", "",
            "| Rival | H2 lines to maintain | H3 tool calls | Bar against this rival |",
            "|---|---|---|---|"]
    bar = h1
    for a in ARMS[1:]:
        h2 = compare([r["lines"] for r in by["seikika"]], [r["lines"] for r in by[a]])
        h3 = compare([r["calls"] for r in by["seikika"]], [r["calls"] for r in by[a]])
        ok = (h2.startswith("holds") or h3.startswith("holds")) and "lost" not in h2 + h3
        bar = bar and ok
        out.append(f"| {a} | {h2} | {h3} | {'met' if ok else 'not met'} |")
    out += ["", f"**The bar: {'passes' if bar else 'fails'}.**"]
    print("\n".join(out))


if __name__ == "__main__":
    main()
