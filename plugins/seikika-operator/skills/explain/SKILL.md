---
name: explain
description: >-
  Explains why a field has its value, or why a rule passes or fails. Use when a score is surprising, and after the server is up.
---

# seikika explain

1. Pass the entity id and either --field or --rule scorecard/rule.
2. Read the explanation before changing the manifest.
3. Change the source the explanation names. Do not change a score by editing the results file.

```
seikika explain command:validate --rule operator-cli/has-help -o json
```

Do not pass both a guess and no flag. If it fails, the field or the rule name is wrong.
