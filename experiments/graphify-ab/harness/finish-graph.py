#!/usr/bin/env python3
"""Finish a `/graphify .` build after its semantic subagents wrote their chunk JSONs: the skill's
steps B3 to 4.5 (graphify/skill.md, graphifyy 0.9.118), run in the repo with Graphify's interpreter.

    finish-graph.py <extraction-spec.md>

Two departures from the skill, both in README §3: communities are named by Graphify's own
LLM-free labeller (`label_communities_by_hub`) instead of by the host agent (step 5), and the
graph is built undirected, the skill's default. Steps 6 to 9 (HTML, manifest, cost tracker) don't
change what `graphify query`, `path` or `explain` read, and are skipped.
"""
import glob
import json
import sys
from pathlib import Path

from graphify.analyze import god_nodes, suggest_questions, surprising_connections
from graphify.build import build_from_json
from graphify.cache import drop_unreported_chunk_sources, save_semantic_cache
from graphify.cluster import cluster, label_communities_by_hub, score_all
from graphify.diagnostics import diagnose_extraction, format_diagnostic_report
from graphify.export import to_json
from graphify.report import generate

SPEC = sys.argv[1]
OUT = Path("graphify-out")
ROOT = (OUT / ".graphify_root").read_text(encoding="utf-8-sig").strip()


def load(p: Path) -> dict:
    return json.loads(p.read_text(encoding="utf-8"))


# B3: every chunk on disk and parseable; more than half missing stops the build.
expected = len(load(OUT / ".chunks.json"))
chunks, bad = [], []
for i in range(1, expected + 1):
    p = OUT / f".graphify_chunk_{i:02d}.json"
    try:
        d = load(p)
        assert isinstance(d.get("nodes"), list) and isinstance(d.get("edges"), list)
        chunks.append(p)
    except (OSError, ValueError, AssertionError):
        bad.append(p.name)
print(f"B3: {len(chunks)} of {expected} chunks valid" + (f"; missing or invalid: {bad}" if bad else ""))
if len(bad) * 2 > expected:
    sys.exit("more than half the chunks failed")

# B3.5: compare each file's reported coverage with its length; drop items for unread files.
covered: dict[str, int] = {}
for c in chunks:
    for src, n in (load(c).get("coverage") or {}).items():
        covered[src] = max(covered.get(src, 0), int(n))
partial = []
for src, n in sorted(covered.items()):
    p = Path(src) if Path(src).is_absolute() else Path(ROOT) / src
    if p.is_file():
        total = sum(1 for _ in p.open(encoding="utf-8", errors="replace"))
        if total and n < total:
            partial.append((src, n, total))
unread, dropped = drop_unreported_chunk_sources(OUT, Path(ROOT))
(OUT / ".graphify_partial.txt").write_text(
    "\n".join(dict.fromkeys([s for s, _, _ in partial] + unread)), encoding="utf-8")
print(f"B3.5: {len(partial)} file(s) partly read, {len(unread)} unread ({dropped} item(s) dropped)")
for src, n, total in partial:
    print(f"  partly read: {src}: {n} of {total} lines")
for src in unread:
    print(f"  unread: {src}")

# B3: merge the chunks, cache them under the spec, merge cached + new.
nodes, edges, hyper = [], [], []
for c in sorted(glob.glob(str(OUT / ".graphify_chunk_*.json"))):
    d = load(Path(c))
    nodes += d.get("nodes", [])
    edges += d.get("edges", [])
    hyper += d.get("hyperedges", [])
uncached = [x for x in (OUT / ".graphify_uncached.txt").read_text().splitlines() if x]
saved = save_semantic_cache(nodes, edges, hyper, root=ROOT, allowed_source_files=uncached, prompt_file=SPEC,
                            partial_source_files=[s for s, _, _ in partial] + unread)
seen, sem_nodes = set(), []
for n in nodes:
    if n["id"] not in seen:
        seen.add(n["id"])
        sem_nodes.append(n)
print(f"B3: semantic {len(sem_nodes)} nodes, {len(edges)} edges, {len(hyper)} hyperedges; cached {saved} files")

# Part C: AST first, semantic deduplicated by id and stamped as semantic.
ast = load(OUT / ".graphify_ast.json")
for x in sem_nodes + edges:
    x.setdefault("_origin", "semantic")
ids = {n["id"] for n in ast["nodes"]}
merged_nodes = list(ast["nodes"]) + [n for n in sem_nodes if n["id"] not in ids]
extraction = {"nodes": merged_nodes, "edges": ast["edges"] + edges, "hyperedges": hyper,
              "input_tokens": 0, "output_tokens": 0}
(OUT / ".graphify_extract.json").write_text(json.dumps(extraction, indent=2, ensure_ascii=False), encoding="utf-8")
print(f"C: {len(merged_nodes)} nodes ({len(ast['nodes'])} AST + {len(sem_nodes)} semantic), "
      f"{len(extraction['edges'])} edges")

# Step 4: build, cluster, analyse, export, report (labels from step 5's LLM-free default).
detection = load(OUT / ".graphify_detect.json")
G = build_from_json(extraction, root=ROOT, directed=False)
if G.number_of_nodes() == 0:
    sys.exit("graph is empty")
communities = cluster(G)
cohesion = score_all(G, communities)
labels = label_communities_by_hub(G, communities)
gods = god_nodes(G)
surprises = surprising_connections(G, communities)
questions = suggest_questions(G, communities, labels)
if not to_json(G, communities, str(OUT / "graph.json"), force=True, community_labels=labels):
    sys.exit("to_json refused to write graph.json")
tokens = {"input": 0, "output": 0}
(OUT / "GRAPH_REPORT.md").write_text(
    generate(G, communities, cohesion, labels, gods, surprises, detection, tokens, ROOT, suggested_questions=questions),
    encoding="utf-8")
(OUT / ".graphify_labels.json").write_text(json.dumps({str(k): v for k, v in labels.items()}, ensure_ascii=False))
print(f"4: {G.number_of_nodes()} nodes, {G.number_of_edges()} edges, {len(communities)} communities")

# Step 4.5: health check (read-only).
print(format_diagnostic_report(diagnose_extraction(extraction, directed=False, root=ROOT)))
