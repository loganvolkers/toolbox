#!/usr/bin/env bash
# Build arm B's Graphify graphs the way `/graphify .` does in Cursor (graphify/skill.md, graphifyy
# 0.9.118), up to the semantic subagents (README §3).
#
#   build-graph.sh prepare   export both repos, write .graphifyignore, detect, AST pass, chunk the
#                            documents and render one subagent prompt per chunk
#   (then launch one general-purpose subagent per rendered prompt; each writes its chunk JSON)
#   build-graph.sh finish    finish-graph.py on both repos: coverage check, merge, cache, build,
#                            cluster, report, health check
#
# Env: SEIKIKA_CHECKOUT (a clone with a8f7e03), OPENFGA_CHECKOUT (a clone of the fork with
# 7d89a566), WORK (default /tmp/graphify-ab).
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
WORK=${WORK:-/tmp/graphify-ab}
B=$WORK/build
PY=$(dirname "$(readlink -f "$(command -v graphify)")")/python
[[ -x "$PY" ]] || PY=$(head -1 "$(command -v graphify)" | sed 's/^#!//')
export GRAPHIFY_BACKEND=none GRAPHIFY_QUERY_LOG_DISABLE=1
SPEC=$("$PY" -c "import graphify,os;print(os.path.join(os.path.dirname(graphify.__file__),'skills','agents','references','extraction-spec.md'))")

prepare() {
  rm -rf "$B" "$WORK/semantic-prompts"
  mkdir -p "$B/seikika" "$B/openfga" "$WORK/semantic-prompts"
  (cd "${SEIKIKA_CHECKOUT:?}" && git archive a8f7e03) | tar -x -C "$B/seikika"
  (cd "${OPENFGA_CHECKOUT:?}" && git archive 7d89a566) | tar -x -C "$B/openfga"
  cp "$HERE/graphifyignore.seikika" "$B/seikika/.graphifyignore"
  cp "$HERE/graphifyignore.openfga" "$B/openfga/.graphifyignore"
  for r in seikika openfga; do
    cd "$B/$r"
    mkdir -p graphify-out
    printf %s "$PY" > graphify-out/.graphify_python
    pwd | tr -d '\n' > graphify-out/.graphify_root
    # Step 2: detect, with the build's .graphifyignore and .gitignore.
    "$PY" - "$r" <<'EOF'
import json, sys
from pathlib import Path
from graphify.detect import detect
from graphify.paths import out_path
from graphify.watch import _read_build_excludes, _read_build_gitignore
root = Path('graphify-out/.graphify_root').read_text().strip()
r = detect(Path(root), extra_excludes=_read_build_excludes(out_path()), gitignore=_read_build_gitignore(out_path()))
Path('graphify-out/.graphify_detect.json').write_text(json.dumps(r, ensure_ascii=False))
print(sys.argv[1], 'detect:', r['total_files'], 'files,', r['total_words'], 'words', {k: len(v) for k, v in r['files'].items()})
EOF
    # Part A: tree-sitter AST pass over the code files.
    "$PY" - "$r" <<'EOF' 2>&1 | grep -v 'AST extraction:'
import json, sys, time
from pathlib import Path
from graphify.extract import collect_files, extract
detect = json.loads(Path('graphify-out/.graphify_detect.json').read_text())
root = Path(Path('graphify-out/.graphify_root').read_text().strip())
files = []
for f in detect['files'].get('code', []):
    files.extend(collect_files(Path(f)) if Path(f).is_dir() else [Path(f)])
t = time.time()
result = extract(files, cache_root=root, root=root)
Path('graphify-out/.graphify_ast.json').write_text(json.dumps(result, indent=2, ensure_ascii=False))
print(sys.argv[1], f'AST: {len(result["nodes"])} nodes, {len(result["edges"])} edges, {time.time() - t:.1f}s')
EOF
    # Part B0-B1: semantic cache check and chunking (<=22 files and ~6,000 lines a chunk; an image
    # or a file over 2,000 lines alone; files grouped by directory).
    "$PY" - "$r" "$SPEC" <<'EOF'
import json, sys
from pathlib import Path
from graphify.cache import check_semantic_cache
detect = json.loads(Path('graphify-out/.graphify_detect.json').read_text())
files = [f for cat in ('document', 'paper', 'image') for f in detect['files'].get(cat, [])]
root = Path('graphify-out/.graphify_root').read_text().strip()
_, _, _, uncached = check_semantic_cache(files, root=root, prompt_file=sys.argv[2])
Path('graphify-out/.graphify_uncached.txt').write_text('\n'.join(uncached))
def lines(p):
    try:
        return sum(1 for _ in open(p, encoding='utf-8', errors='replace'))
    except OSError:
        return 0
chunks, cur, n = [], [], 0
for f in sorted(uncached, key=lambda p: (str(Path(p).parent), p)):
    k = lines(f)
    if Path(f).suffix.lower() in ('.svg', '.png', '.jpg', '.jpeg', '.gif', '.webp') or k > 2000:
        chunks.append([f])
        continue
    if cur and (len(cur) >= 22 or n + k > 6000):
        chunks.append(cur)
        cur, n = [], 0
    cur.append(f)
    n += k
if cur:
    chunks.append(cur)
Path('graphify-out/.chunks.json').write_text(json.dumps(chunks, indent=1))
print(sys.argv[1], len(uncached), 'files ->', len(chunks), 'chunks')
EOF
  done
  # Part B2: one prompt per chunk, the extraction spec's template as the skill fills it.
  python3 - "$SPEC" "$B" "$WORK/semantic-prompts" <<'EOF'
import json, pathlib, sys
spec, build, out = pathlib.Path(sys.argv[1]).read_text(), pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3])
tmpl = spec.split('```\n', 1)[1].rsplit('\n```', 1)[0]
for r in ('seikika', 'openfga'):
    chunks = json.loads((build / r / 'graphify-out/.chunks.json').read_text())
    for i, c in enumerate(chunks, 1):
        p = (tmpl.replace('FILE_LIST', '\n'.join(c)).replace('CHUNK_NUM', str(i))
             .replace('TOTAL_CHUNKS', str(len(chunks))).replace('DEEP_MODE', 'false')
             .replace('CHUNK_PATH', str(build / r / 'graphify-out' / f'.graphify_chunk_{i:02d}.json')))
        (out / f'{r}-{i:02d}.md').write_text(p)
print(len(list(out.iterdir())), 'subagent prompts in', out)
EOF
}

finish() {
  for r in seikika openfga; do
    (cd "$B/$r" && "$PY" "$HERE/finish-graph.py" "$SPEC")
  done
}

case "${1:-}" in
  prepare) prepare ;;
  finish) finish ;;
  *) sed -n '2,12p' "$0"; exit 2 ;;
esac
