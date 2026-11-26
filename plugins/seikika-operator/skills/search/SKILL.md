---
name: search
description: >-
  Finds entities by name or title. Use when the id is not known and a running portal should be searched.
---

# seikika search

1. Pass the text to find in ids and titles.
2. Add -o json so the ids can be passed to get.
3. Open one result with get before acting on it. Do not treat a title match as the only entity.

```
seikika search seikika -o json
```

Do not use search as a substitute for query when the kind and the filter are known. If it fails, the server is down.
