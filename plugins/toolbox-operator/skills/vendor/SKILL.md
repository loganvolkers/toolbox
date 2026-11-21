---
name: vendor
description: >-
  Copies a Toolbox package into a portal's packages/ directory, without its tests, so the portal needs no network. Use when CI or an agent can't fetch a github: package, or to pin a package's exact files in the repository.
---

# toolbox vendor

1. Pass the package as a directory, or as a `github:` spec that `toolbox install` already cached.
2. Pass --portal as the portal directory. The package lands in `<portal>/packages/<name>`, with `VENDORED.json` recording its source, version and content hash.
3. Include it with the printed entry (`package: packages/<name>`) and the package's parameters, then run `toolbox validate` on the portal.

```
toolbox vendor ../toolbox/packages/file-size --portal .toolbox
```

Vendoring again replaces the copy; review the diff before committing. If it fails, the package directory has no `toolbox-package.yaml`, or a `github:` package isn't in the cache yet.
