# Seeded bugs — arm B. Agents must not read this file.

| Id | Symptom | Cause |
|---|---|---|
| BUG-ID | Component catalog ids are bare names (`api`,`web`,…) not `{project}:{name}` | Mapping uses name-only id |
| BUG-EXTRA | Component `data` has extra `description` field | Mapper includes `description` |
| BUG-ASSIGNEE | Issues that should have assignees are null | Assignee forced to null / not read from accountId |
