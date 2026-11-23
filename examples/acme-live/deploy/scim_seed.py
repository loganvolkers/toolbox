#!/usr/bin/env python3
"""Push Acme's directory into the SCIM connector. Users, then groups.

The connector is push-only: this is the identity provider, not a fixture load into Postgres.
The same users and groups are in scim_seed.sh, which the portal image runs.
"""

import json
import os
import sys
import urllib.request

EXT = "urn:ietf:params:scim:schemas:extension:toolbox:2.0:Group"
BASE = sys.argv[1].rstrip("/")
TOKEN = os.environ.get("TOOLBOX_SCIM_TOKEN", "")

USERS = [
    ("alice", "Alice Ng", "alice@acme.test"),
    ("bob", "Bob Iqbal", "bob@acme.test"),
    ("carol", "Carol Ortiz", "carol@acme.test"),
    ("dave", "Dave Shah", "dave@acme.test"),
]
GROUPS = [
    ("platform", "Platform", "platform", ["bob"]),
    ("payments", "Payments", "payments", ["alice", "dave"]),
]


def put(path, body):
    headers = {"content-type": "application/scim+json"}
    if TOKEN:
        headers["authorization"] = f"Bearer {TOKEN}"
    req = urllib.request.Request(
        f"{BASE}/{path}", data=json.dumps(body).encode(), method="PUT", headers=headers
    )
    with urllib.request.urlopen(req, timeout=30) as resp:
        resp.read()


for login, name, email in USERS:
    put(
        f"Users/u-{login}",
        {
            "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
            "userName": login,
            "name": {"formatted": name},
            "emails": [{"value": email, "primary": True}],
            "active": True,
        },
    )
for slug, display, team, members in GROUPS:
    put(
        f"Groups/g-{slug}",
        {
            "schemas": ["urn:ietf:params:scim:schemas:core:2.0:Group", EXT],
            "displayName": display,
            "members": [{"value": f"u-{m}"} for m in members],
            EXT: {"slug": slug, "githubTeam": team},
        },
    )
print(f"provisioned {len(USERS)} users and {len(GROUPS)} groups")
