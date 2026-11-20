---
name: assess-submit
description: >-
  Validates assessment files and pushes them to a running server as assessment records. Use when a portal stores assessments in Postgres rather than in the repository.
---

# toolbox assess submit

1. Check the files locally first.
2. Pass the files, --portal, --url and the connector token.
3. Confirm the server accepted them. Do not submit a file that was not judged in this session.

```
toolbox assess submit portal/assessments/rule-quality/rule/toolbox/maintainer-crate/has-tests.yaml --portal portal --url http://127.0.0.1:8080
```

Do not put the connector token on the command line of a shared log. If it fails, the file is invalid or the token was rejected.
