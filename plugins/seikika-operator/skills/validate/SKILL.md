---
name: validate
description: >-
  Validates a Seikika portal: manifest schema, cross-references, CEL, files and fixtures. Use when a portal manifest or an included file changed, and before trusting the result of a later test or plan.
---

# seikika validate

1. Run the command on the portal directory. Exit 0 means valid, 1 means errors, 2 means the manifest could not be loaded.
2. Read each error. The text includes the file and a suggested fix.
3. Fix the manifest and run the command again. Do not continue to test while exit code 1 remains.

```
seikika validate .seikika
seikika validate .seikika --format json
```

Do not treat a text summary as success. If it fails, the exit code is 1 or 2.
