---
name: add-permission
description: Change who can see or do what in a Seikika portal. Edits OpenFGA model modules (authz/*.fga, fga.mod), .fga.yaml tests, derivation rules from catalog relations, facets and CODEOWNERS-backed claims. Use when changing permissions, roles, access control, ReBAC, OpenFGA models or fga.yaml tests.
metadata:
  status: pre-release
---

# Change permissions

1. **Write the tests first:**
   - **Level 1 (model):** in `authz/<module>.fga.yaml`, write `check`, `list_objects` and `list_users`
     assertions for each relation you add. Keep the file beside `fga.mod`, because `fga model test`
     refuses `../` paths.
   - **Level 2 (derivation):** catalog fixtures (`*.catalog.yaml`) that prove the derivation rule maps
     catalog data to the right tuples.
   - **Level 3 (surfaces):** what API and MCP return for two different principals.
2. **Edit the model.** Prefer `extend type` in your own module over editing core types. Ownership
   claims only count when CODEOWNERS backs them; don't work around that.
3. **Run** both `seikika test` and `fga model test --tests "authz/*.fga.yaml"`. They use the same
   OpenFGA engine.
4. **Run `plan`** and read the authorization diff: who gains or loses which permission. A large loss
   trips the `authzLoss` gate, which needs an owner's acknowledgement.
