---
name: incident-brief
description: >-
  Produces a cited, read-only incident brief when an on-call responder
  provides a service and environment.
---

# Incident brief

1. Confirm the service, environment, portal, and incident time. Retrieve
   both the readiness and health entities with the caller's existing
   read credentials.
2. Verify the observed service and environment match the request. Treat
   health as unknown when the record is absent, its source is
   unavailable, or its observation and expiry timestamps do not
   establish freshness.
3. Summarize the affected journey, current burn and its observation
   time, owner, page route, dependencies, recent changes, runbook, and
   last recovery drill. Cite the source per claim. Identify missing
   evidence and label hypotheses.
4. Identify the documented mitigation and the independent customer
   signal a responder could use to verify recovery. Link the incident
   command record. Leave paging, updates, approval and production
   changes to responders.

For a checkout incident in production, read these entities:

```sh
toolbox get 'sre-readiness:checkout@production' -o json
toolbox get 'sre-health:checkout@production' -o json
```
