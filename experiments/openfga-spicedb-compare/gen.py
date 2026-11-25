#!/usr/bin/env python3
"""Generate the corpus, the check sample and the list users (README.md, seed 7).

Writes corpus.json in a neutral form both arms load:

  relationships: [resource_type, resource_id, relation, subject_type, subject_id], in write order;
                 subject_id "*" is the wildcard (every user)
  checks:        2,000 [user, document] pairs
  listUsers:     50 users whose viewable documents are listed
  expected:      what an independent Python evaluation of the model gives for the checks and
                 lists, with the same digests the load generator computes

The expected answers are a third opinion. The run gates on the two arms agreeing with each other
(README.md); agreement with this evaluation is recorded, not gated on.
"""
import functools
import hashlib
import json
import random
from pathlib import Path

SEED = 7
USERS, GROUPS, FOLDERS, DOCUMENTS = 1000, 100, 1000, 10000
GROUPS_PER_USER = 2
NESTED_FOLDERS = FOLDERS // 10
PUBLIC_DOCUMENTS = DOCUMENTS // 100
CHECKS, LIST_USERS = 2000, 50


def generate():
    rng = random.Random(SEED)
    users = [f"u{i}" for i in range(USERS)]
    rels = []
    member_of = {}
    for u in users:
        gs = [f"g{g}" for g in rng.sample(range(GROUPS), GROUPS_PER_USER)]
        member_of[u] = gs
        rels += [["group", g, "member", "user", u] for g in gs]
    folder_owner = {}
    for f in range(FOLDERS):
        folder_owner[f"f{f}"] = f"g{rng.randrange(GROUPS)}"
        rels.append(["folder", f"f{f}", "owner", "group", folder_owner[f"f{f}"]])
    folder_parent = {}
    # A parent always has a lower index, so nesting has no cycles.
    for f in sorted(rng.sample(range(1, FOLDERS), NESTED_FOLDERS)):
        folder_parent[f"f{f}"] = f"f{rng.randrange(f)}"
        rels.append(["folder", f"f{f}", "parent", "folder", folder_parent[f"f{f}"]])
    doc_folder, doc_viewer, doc_owner = {}, {}, {}
    for d in range(DOCUMENTS):
        doc = f"d{d}"
        doc_folder[doc] = f"f{rng.randrange(FOLDERS)}"
        doc_viewer[doc] = f"u{rng.randrange(USERS)}"
        doc_owner[doc] = f"u{rng.randrange(USERS)}"
        rels.append(["document", doc, "folder", "folder", doc_folder[doc]])
        rels.append(["document", doc, "viewer", "user", doc_viewer[doc]])
        rels.append(["document", doc, "owner", "user", doc_owner[doc]])
    public = sorted(f"d{d}" for d in rng.sample(range(DOCUMENTS), PUBLIC_DOCUMENTS))
    rels += [["document", d, "viewer", "user", "*"] for d in public]

    children = {}
    for f, p in folder_parent.items():
        children.setdefault(p, []).append(f)
    docs_in = {}
    for d, f in doc_folder.items():
        docs_in.setdefault(f, []).append(d)
    by_viewer, by_owner = {}, {}
    for d, u in doc_viewer.items():
        by_viewer.setdefault(u, []).append(d)
    for d, u in doc_owner.items():
        by_owner.setdefault(u, []).append(d)
    owned_by = {}
    for f, g in folder_owner.items():
        owned_by.setdefault(g, []).append(f)

    @functools.cache
    def viewable(u):
        """The model, evaluated directly: folders the user's groups own and their descendants,
        then documents in them, plus direct viewer, owner and public documents."""
        stack = [f for g in member_of[u] for f in owned_by.get(g, [])]
        folders = set()
        while stack:
            f = stack.pop()
            if f not in folders:
                folders.add(f)
                stack += children.get(f, [])
        docs = {d for f in folders for d in docs_in.get(f, [])}
        docs |= set(by_viewer.get(u, [])) | set(by_owner.get(u, [])) | set(public)
        return docs

    # Half the sample is uniform pairs, mostly denied; half picks a document the user can view,
    # so both answers are exercised.
    checks = []
    for i in range(CHECKS):
        u = users[rng.randrange(USERS)]
        if i % 2 == 0:
            d = f"d{rng.randrange(DOCUMENTS)}"
        else:
            d = rng.choice(sorted(viewable(u)))
        checks.append([u, d])
    list_users = [users[i] for i in rng.sample(range(USERS), LIST_USERS)]

    decisions = [d in viewable(u) for u, d in checks]
    lists = {u: sorted(viewable(u)) for u in list_users}
    return {
        "seed": SEED,
        "relationships": rels,
        "checks": checks,
        "listUsers": list_users,
        "expected": {
            "decisionsDigest": decisions_digest(checks, decisions),
            "listsDigest": lists_digest(list_users, lists),
            "allowed": sum(decisions),
            "decisions": decisions,
            "listSizes": {u: len(v) for u, v in lists.items()},
        },
    }


def decisions_digest(checks, decisions):
    lines = "".join(f"{u} {d} {int(a)}\n" for (u, d), a in zip(checks, decisions))
    return hashlib.sha256(lines.encode()).hexdigest()


def lists_digest(users, lists):
    lines = "".join(f"{u} {','.join(sorted(lists[u]))}\n" for u in users)
    return hashlib.sha256(lines.encode()).hexdigest()


def main():
    corpus = generate()
    out = Path(__file__).resolve().parent / "corpus.json"
    out.write_text(json.dumps(corpus, separators=(",", ":")) + "\n")
    e = corpus["expected"]
    sizes = sorted(e["listSizes"].values())
    print(f"{len(corpus['relationships'])} relationships, {len(corpus['checks'])} checks "
          f"({e['allowed']} allowed), {len(corpus['listUsers'])} list users "
          f"(documents per user {sizes[0]}..{sizes[-1]}); wrote {out.name}")


if __name__ == "__main__":
    main()
