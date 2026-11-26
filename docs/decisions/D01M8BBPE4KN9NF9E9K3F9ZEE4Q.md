---
id: D01M8BBPE4KN9NF9E9K3F9ZEE4Q
decided: 2026-11-26T07:21:38Z
---

# Publish npm packages with GitHub Actions OIDC

Publish `@seikika/ui` and `@seikika/astro-report` from the `publish-npm.yml` workflow on
`main` in `seikika/seikika`. Use the `npm-publish` GitHub environment and configure each
package to trust that workflow and environment. npm generates provenance for public packages
published through this identity.

Build and pack with pnpm in a job with read permissions. The publish job receives only those
tarballs and an OIDC identity. Publish the UI first so the report's workspace dependency can
resolve. Skip versions already in the registry so partially completed releases can be retried.

Keep npm versions in their package manifests. Maintainers dispatch the workflow separately
from Rust tags. New package names require an initial maintainer publish before npm can attach a
trusted publisher. See [the release guide](../RELEASING.md#public-frontend-packages) for setup
and release commands.
