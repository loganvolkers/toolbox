---
name: vendor
description: >-
  Copies a Seikika package into a portal's packages/ directory, without its tests, so the portal needs no network. Use when CI or an agent can't fetch a github: package, or to pin a package's exact files in the repository.
---

# seikika vendor

1. Pass the package as a directory, or as a `github:` spec that `seikika install` already cached.
2. Pass --portal as the portal directory. The package lands in `<portal>/packages/<name>`, with `VENDORED.json` recording its source, version and content hash.
3. Include it with the printed entry (`package: packages/<name>`) and the package's parameters, then run `seikika validate` on the portal.

```
seikika vendor ../seikika/packages/file-size --portal .seikika
```

Vendoring again replaces the copy; review the diff before committing. If it fails, the package directory has no `seikika-package.yaml`, or a `github:` package isn't in the cache yet.
