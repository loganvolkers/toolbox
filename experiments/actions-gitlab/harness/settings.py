#!/usr/bin/env python3
"""GitLab configuration outside files (README §5.1, reported): what a run set through the API or
the UI that someone must keep, which lines to maintain can't see.

    settings.py <dest>         snapshot the run's group (setup.sh writes <dest>.settings.json)
    settings.py diff <dest>    what changed since setup: {"changed": n, "items": [...]}

Per project in the group, subgroups included: protected branches and tags, CI/CD variables (keys
only), webhooks, pipeline triggers and schedules, deploy keys and tokens, access tokens,
environments, the job token's settings and the CI and merge settings below. Per group: variables,
access tokens, runners, and its members. New projects and groups are items too.
"""
import json
import os
import sys
import urllib.parse

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from score import api, group  # noqa: E402

PROJECT_KEYS = ["ci_push_repository_for_job_token_allowed", "ci_pipeline_variables_minimum_override_role",
                "only_allow_merge_if_pipeline_succeeds", "only_allow_merge_if_all_discussions_are_resolved",
                "merge_method", "squash_option", "shared_runners_enabled", "ci_config_path",
                "remove_source_branch_after_merge", "builds_access_level", "merge_requests_access_level",
                "restrict_user_defined_variables", "ci_job_token_scope_enabled", "default_branch",
                "auto_devops_enabled", "visibility"]


def names(items, key="name"):
    return sorted(str(i.get(key)) for i in items)


def safe(path):
    try:
        return api(path)
    except Exception as e:  # an endpoint CE or this role doesn't offer
        return {"unavailable": str(e)}


def snapshot(dest: str) -> dict:
    g = group(dest)
    out = {}
    groups = [api(f"/groups/{urllib.parse.quote(g, safe='')}")] + api(
        f"/groups/{urllib.parse.quote(g, safe='')}/descendant_groups?per_page=100")
    for gr in groups:
        gid = gr["id"]
        out[f"group {gr['full_path']}"] = {
            "variables": names(safe(f"/groups/{gid}/variables?per_page=100"), "key"),
            "access_tokens": names(safe(f"/groups/{gid}/access_tokens?per_page=100")),
            "runners": names(safe(f"/groups/{gid}/runners?type=group_type&per_page=100"), "description"),
            "members": sorted(f"{m['username']}:{m['access_level']}" for m in api(f"/groups/{gid}/members?per_page=100")),
        }
    for p in api(f"/groups/{urllib.parse.quote(g, safe='')}/projects?include_subgroups=true&per_page=100"):
        pid = p["id"]
        full = api(f"/projects/{pid}")
        out[f"project {p['path_with_namespace']}"] = {
            "settings": {k: full.get(k) for k in PROJECT_KEYS},
            "protected_branches": sorted(json.dumps({"name": b["name"],
                "push": sorted(a["access_level"] for a in b.get("push_access_levels", [])),
                "merge": sorted(a["access_level"] for a in b.get("merge_access_levels", [])),
                "force": b.get("allow_force_push")}, sort_keys=True)
                for b in api(f"/projects/{pid}/protected_branches?per_page=100")),
            "protected_tags": names(safe(f"/projects/{pid}/protected_tags?per_page=100")),
            "variables": names(safe(f"/projects/{pid}/variables?per_page=100"), "key"),
            "hooks": names(safe(f"/projects/{pid}/hooks?per_page=100"), "url"),
            "triggers": names(safe(f"/projects/{pid}/triggers?per_page=100"), "description"),
            "schedules": names(safe(f"/projects/{pid}/pipeline_schedules?per_page=100"), "description"),
            "deploy_keys": names(safe(f"/projects/{pid}/deploy_keys?per_page=100"), "title"),
            "deploy_tokens": names(safe(f"/projects/{pid}/deploy_tokens?per_page=100")),
            "access_tokens": names(safe(f"/projects/{pid}/access_tokens?per_page=100")),
            "environments": names(safe(f"/projects/{pid}/environments?per_page=100")),
            "job_token_allowlist": names(safe(f"/projects/{pid}/job_token_scope/allowlist?per_page=100"),
                                         "path_with_namespace"),
            "members": sorted(f"{m['username']}:{m['access_level']}" for m in api(f"/projects/{pid}/members?per_page=100")),
        }
    return out


def diff(dest: str) -> dict:
    before = json.load(open(dest.rstrip("/") + ".settings.json"))
    now = snapshot(dest)
    items = []
    for name in sorted(set(before) | set(now)):
        if name not in before:
            items.append({"what": name, "change": "created"})
            continue
        if name not in now:
            items.append({"what": name, "change": "deleted"})
            continue
        for key in sorted(set(before[name]) | set(now[name])):
            a, b = before[name].get(key), now[name].get(key)
            if key == "settings":
                for s in sorted(set(a or {}) | set(b or {})):
                    if (a or {}).get(s) != (b or {}).get(s):
                        items.append({"what": f"{name} {s}", "from": (a or {}).get(s), "to": (b or {}).get(s)})
            elif isinstance(a, list) and isinstance(b, list):
                for x in sorted(set(b) - set(a)):
                    items.append({"what": f"{name} {key}", "added": x})
                for x in sorted(set(a) - set(b)):
                    items.append({"what": f"{name} {key}", "removed": x})
            elif a != b:
                items.append({"what": f"{name} {key}", "from": a, "to": b})
    return {"changed": len(items), "items": items}


if __name__ == "__main__":
    if sys.argv[1] == "diff":
        json.dump(diff(sys.argv[2]), sys.stdout, indent=1)
    else:
        json.dump(snapshot(sys.argv[1]), sys.stdout, indent=1)
    print()
