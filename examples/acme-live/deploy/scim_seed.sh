#!/bin/sh
# Push Acme's directory into the SCIM connector. Users, then groups.
# The same list is in scim_seed.py. This is the identity provider, not a fixture load.
set -eu
base="${1:?scim base url}"
base="${base%/}"
token="${TOOLBOX_SCIM_TOKEN:-}"

put() {
  path="$1"
  body="$2"
  if [ -n "$token" ]; then
    curl -fsS -X PUT -H "content-type: application/scim+json" -H "Authorization: Bearer ${token}" \
      --data "$body" "${base}/${path}" >/dev/null
  else
    curl -fsS -X PUT -H "content-type: application/scim+json" --data "$body" "${base}/${path}" >/dev/null
  fi
}

put "Users/u-alice" '{"schemas":["urn:ietf:params:scim:schemas:core:2.0:User"],"userName":"alice","name":{"formatted":"Alice Ng"},"emails":[{"value":"alice@acme.test","primary":true}],"active":true}'
put "Users/u-bob" '{"schemas":["urn:ietf:params:scim:schemas:core:2.0:User"],"userName":"bob","name":{"formatted":"Bob Iqbal"},"emails":[{"value":"bob@acme.test","primary":true}],"active":true}'
put "Users/u-carol" '{"schemas":["urn:ietf:params:scim:schemas:core:2.0:User"],"userName":"carol","name":{"formatted":"Carol Ortiz"},"emails":[{"value":"carol@acme.test","primary":true}],"active":true}'
put "Users/u-dave" '{"schemas":["urn:ietf:params:scim:schemas:core:2.0:User"],"userName":"dave","name":{"formatted":"Dave Shah"},"emails":[{"value":"dave@acme.test","primary":true}],"active":true}'

ext="urn:ietf:params:scim:schemas:extension:toolbox:2.0:Group"
put "Groups/g-platform" "{\"schemas\":[\"urn:ietf:params:scim:schemas:core:2.0:Group\",\"${ext}\"],\"displayName\":\"Platform\",\"members\":[{\"value\":\"u-bob\"}],\"${ext}\":{\"slug\":\"platform\",\"githubTeam\":\"platform\"}}"
put "Groups/g-payments" "{\"schemas\":[\"urn:ietf:params:scim:schemas:core:2.0:Group\",\"${ext}\"],\"displayName\":\"Payments\",\"members\":[{\"value\":\"u-alice\"},{\"value\":\"u-dave\"}],\"${ext}\":{\"slug\":\"payments\",\"githubTeam\":\"payments\"}}"

echo "provisioned 4 users and 2 groups"
