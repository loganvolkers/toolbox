---
name: derive
description: >-
  Recomputes derived state from fixtures and prints a summary, or writes the reference-format files. Use when checking a portal's derived output, and before regenerating expected files.
---

# seikika derive

1. Run it on the portal directory and read the summary.
2. Pass --out only after reviewing what would change.
3. Commit regenerated files in the same change as the manifest edit that caused them.

```
seikika derive examples/reference-portal
```

Do not pass --out until the diff has been reviewed. If it fails, the manifest does not load.
