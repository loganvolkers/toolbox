---
name: token-create
description: >-
  Creates a personal access token for a user and prints it once. Use when a person needs a token for a running portal that stores token hashes in Postgres.
---

# toolbox token create

1. Pass --user as a login or an email, and --database-url as the portal's Postgres.
2. Copy the token from the output at once. It is not shown again.
3. Store it in a secret manager. Do not commit it or pass it as a chat message.

```
toolbox token create --user alice --label laptop --database-url "$DATABASE_URL"
```

Do not log the printed token. If it fails, the database URL is wrong or Postgres is down.
