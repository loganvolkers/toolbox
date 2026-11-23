#!/usr/bin/env python3
"""Render connector A/B prompts. python3 -I experiments/connector-ab/prompts/make.py [--check]"""

from __future__ import annotations

import difflib
import sys
from pathlib import Path

HERE = Path(__file__).parent
AB = "/tmp/connector-ab"

COMMON = """You are authoring a cloud/SaaS connector for Forge (a mock Jira Cloud API). Work only inside {arm_dir}. Do not commit; the harness commits when you finish.

## Environment

- Read {arm_dir}/FORGE.md for base URLs and tokens. Forge is already running on loopback.
- Budget: 50 tool calls. Network only to 127.0.0.1 (Forge). No package installs unless the scaffold's README already documented a one-line install that setup ran; prefer the stdlib and what is already present.
- Do **not** read `{ab}/fixtures/**/expected-*.ndjson`, `{ab}/../experiments/connector-ab/harness/score.py`, `{ab}/../experiments/connector-ab/harness/read-key.md`, `{toolbox}/experiments/`, or `{toolbox}/docs/research-log.md`. Those hold the scoring key.
- Write mapped catalog entities to {arm_dir}/OUT/entities.ndjson using exactly this schema (one JSON object per line, sorted by kind then id):

```json
{{"kind":"project","id":"ALPHA","data":{{"key":"ALPHA","name":"Alpha","type":"software"}}}}
{{"kind":"issue","id":"ALPHA-1","data":{{"key":"ALPHA-1","summary":"...","status":"To Do","project":"ALPHA","assignee":null}}}}
{{"kind":"component","id":"ALPHA:api","data":{{"id":"1","name":"api","project":"ALPHA"}}}}
```

`assignee` is the accountId string or null. `component` only from phase 3.

{arm_block}

## Task

{task}

When you finish, reply with:
1. What you built or changed, and how to run a resync (update RUN.md).
2. Anything you could not do.
3. **Developer-experience feedback**, most important first: anything confusing, missing, slow, wrong in docs, or that you had to work around (command or file). Say "none" if none.
"""

ARM = {
    "A": """## Your setup (Toolbox)

- Scaffold Cargo project at {arm_dir}; `toolbox-connect` is a path dependency.
- You may read only: {toolbox}/crates/toolbox-connect/, {toolbox}/crates/toolbox-connector-github/src/main.rs, {toolbox}/crates/toolbox-connector-git-files/src/main.rs, {toolbox}/docs/design/connector-control.md, {toolbox}/docs/design/connector-standards.md, {toolbox}/docs/v1-spec.md (connector sections), {toolbox}/schema/connector-messages.schema.json, {toolbox}/plugins/toolbox-operator/skills/connector-test/, {toolbox}/plugins/toolbox-operator/skills/author-connector/, {toolbox}/plugins/toolbox-operator/skills/connector-drive/, {toolbox}/examples/connector-scaffold/, {toolbox}/site/src/content/docs/authors/connectors.md.
- `export PATH={ab}/bin:$PATH` for `toolbox` if present. Prefer `toolbox connector drive` over hand-built stdio JSON; a portal stub is already under portal/.
- Do **not** read Ocean or Backstage trees under {ab}/pins/.
- Required shape: a stdio connector using `run_stdio` / `Source` that emits `record` + `snapshot_complete`, and OUT/entities.ndjson.""",
    "B": """## Your setup (Port Ocean)

- Scaffold at {arm_dir}. Ocean pin root is in OCEAN_ROOT.txt ({ab}/pins/ocean).
- You may read only that pin's `integrations/fake-integration/` and `integrations/jira/` plus Ocean framework modules those imports need.
- Do **not** read Toolbox crates/docs or the Backstage pin.
- Required shape: `@ocean.on_resync` handlers in integration.py for each kind, `.port/spec.yaml` updated, and `resync.py` (or equivalent) writing OUT/entities.ndjson without Port SaaS.""",
    "C": """## Your setup (Backstage)

- Scaffold at {arm_dir}. Backstage pin root is in BACKSTAGE_ROOT.txt ({ab}/pins/backstage).
- You may read only that pin's `plugins/catalog-backend-module-github/`, `plugins/catalog-node/`, `plugins/catalog-backend/`, `packages/catalog-model/`.
- Do **not** read Toolbox or Ocean.
- Required shape: `ForgeEntityProvider` implementing the EntityProvider pattern (getProviderName + connect), and resync writing OUT/entities.ndjson.""",
}

TASKS = {
    1: """**Author (tenant A).** Implement a connector that syncs Forge `project` and `issue` from tenant A (port 9743, token tenant-a-token). Map into the schema above (no components yet). Update RUN.md. Produce OUT/entities.ndjson.""",
    3: """**Change.** Forge now requires syncing `component` via GET /rest/api/3/project/{{key}}/components. Add the kind; keep projects and issues correct for tenant A. Update RUN.md. Produce OUT/entities.ndjson including components.""",
    4: """**Reuse (tenant B).** Point the same connector at tenant B (port 9744, token tenant-b-token) without rewriting sync logic. Produce OUT/entities.ndjson for tenant B (including components).""",
    5: """**Read.** Do not call Forge and do not regenerate entities. Using only files under {arm_dir} (RUN.md, source, OUT/entities.ndjson, config), answer:
1. How many projects does tenant B have?
2. List issue keys on tenant B.
3. Who is assigned to GAMMA-1 (account id)?
4. Name every component on GAMMA.
5. Which config fields switch tenants?
Write answers to OUT/read-answers.md. Do not change connector code.""",
}


def render(phase: int, arm: str) -> str:
    toolbox = "/agent/repos/toolbox"
    return COMMON.format(
        arm_dir=f"{AB}/arms/{arm}",
        arm_block=ARM[arm].format(arm_dir=f"{AB}/arms/{arm}", ab=AB, toolbox=toolbox),
        task=TASKS[phase].format(arm_dir=f"{AB}/arms/{arm}"),
        ab=AB,
        toolbox=toolbox,
    )


def main():
    check = "--check" in sys.argv
    for phase in (1, 3, 4, 5):
        for arm in ("A", "B", "C"):
            path = HERE / f"{phase}-{arm}.md"
            text = render(phase, arm)
            if check:
                old = path.read_text() if path.exists() else ""
                if old != text:
                    print("".join(difflib.unified_diff(old.splitlines(True), text.splitlines(True), path.name, "new")))
                    sys.exit(1)
            else:
                path.write_text(text)
                print("wrote", path.relative_to(HERE.parent))
    if check:
        print("ok")


if __name__ == "__main__":
    main()
