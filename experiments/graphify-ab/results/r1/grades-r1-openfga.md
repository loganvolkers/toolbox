# Grades: OpenFGA, round 1 (blind)

The rubric at the top of `gold/openfga.md` was applied to all ten answers in `blind-r1/openfga/`.
Each key fact was scored covered (C = 1), partial (P = 0.5) or missed (M = 0). Coverage is
(C + 0.5 × P) / 22. The grader read only the gold key and the ten answer files. The OpenFGA
checkout at `r1/openfga-2/` was consulted for two things. It confirmed that
`pkg/typesystem/resolver.go` calls `NewAndValidate` and wraps errors as `ErrInvalidModel`
(gold O4d). It also confirmed that `commands.NewCheckQuery` returns `*CheckQueryV2`, so the two
names identify the same v2 path (gold O1a). No answer made a claim that contradicts a gold fact.

## Fact × label table

| Fact | 115 | 315 | 351 | 447 | 581 | 667 | 686 | 769 | 771 | 996 |
|---|---|---|---|---|---|---|---|---|---|---|
| O1a | C | C | C | C | C | C | C | C | C | C |
| O1b | C | C | C | C | C | C | C | C | C | C |
| O1c | C | C | C | C | C | C | C | C | C | C |
| O1d | C | C | C | C | C | C | C | C | C | C |
| O1e | C | C | C | C | C | C | C | C | C | C |
| O2a | C | C | C | C | C | C | C | C | C | C |
| O2b | C | C | C | C | C | C | C | C | C | C |
| O2c | C | C | C | C | C | C | C | C | C | C |
| O2d | C | C | C | C | C | C | C | C | C | C |
| O3a | C | C | C | C | C | C | C | C | C | C |
| O3b | C | C | C | C | C | C | C | C | C | C |
| O3c | C | C | C | C | C | C | C | C | C | C |
| O3d | C | C | C | C | C | C | C | C | C | C |
| O3e | C | C | C | C | C | C | C | C | C | C |
| O4a | C | C | C | C | C | C | C | C | C | C |
| O4b | C | C | C | C | C | C | C | C | C | C |
| O4c | C | C | C | C | C | C | C | C | C | C |
| O4d | M | M | M | M | M | M | M | M | M | M |
| O5a | C | C | C | C | C | C | C | C | C | C |
| O5b | C | C | C | C | C | C | C | C | C | C |
| O5c | C | C | C | C | C | C | C | C | C | C |
| O5d | C | C | C | C | C | C | C | C | C | C |

## Coverage per label

| Label | C | P | M | Coverage |
|---|---|---|---|---|
| 115 | 21 | 0 | 1 | 0.955 |
| 315 | 21 | 0 | 1 | 0.955 |
| 351 | 21 | 0 | 1 | 0.955 |
| 447 | 21 | 0 | 1 | 0.955 |
| 581 | 21 | 0 | 1 | 0.955 |
| 667 | 21 | 0 | 1 | 0.955 |
| 686 | 21 | 0 | 1 | 0.955 |
| 769 | 21 | 0 | 1 | 0.955 |
| 771 | 21 | 0 | 1 | 0.955 |
| 996 | 21 | 0 | 1 | 0.955 |

## Calibration decisions (applied to every label)

1. **O1a.** Covered needs three things: `Server.Check`, the weighted-graph flag (as
   `ExperimentalWeightedGraphCheck` or its string `weighted_graph_check`) routing to the v2 path,
   and the fallback to v1 on a non-terminal error. `CheckQueryV2` counts as naming the v2 command,
   because `NewCheckQuery` returns `*CheckQueryV2`. The v1 command may be named either as
   `NewCheckCommand` or as `CheckQuery.Execute`.
2. **O1b.** Covered needs `CheckResolverOrderedBuilder.Build` (builder.go) and the server's
   `getCheckResolverBuilder`.
3. **O1e.** Covered needs a `ShadowResolver` wrapping a main and a shadow `LocalChecker` in the
   last position, under the shadow-check flag. Naming `NewShadowChecker` is not required.
4. **O2a / O2b.** Covered needs all four datastores and all four storage wrappers named. Listing
   extra embedding-only types or mocks as well neither helps nor hurts.
5. **O2d.** Covered needs the interface (`RelationshipTupleReader`, storage.go), the regenerated
   mocks, and the implementations and wrappers that must be updated.
6. **O3b.** Covered needs the newest-first `ReadChanges` and the iterator-cache invalidation,
   with its split between the whole store and per object#relation or per user and object type.
   The clause "writes invalidation entries rather than deleting" is not graded separately: naming
   the `Invalid…CacheKey` markers, or saying "markers", is enough. An answer that said entries are
   deleted would have dropped to P; none did. The exact condition for invalidating the whole store
   ("all changes within the iterator TTL") is not required when the split is stated.
7. **O3c.** Any explicit statement that writes or the Write path do not invalidate, or that
   invalidation is pull-based or lazy, counts as covered.
8. **O4d.** Covered needs the claim that the same type-system validation runs again when a model
   is loaded to serve requests (`pkg/typesystem/resolver.go`, `ErrInvalidModel`). Partial needs
   that gist without the names. Mentioning `TypeSystem.relationInvolves` returning
   `ErrInvalidModel` earns neither: it is a different code path, a query-time helper inside the
   type system, and it does not re-run `NewAndValidate` on load. Every label scored M, with the
   reason "absent".
9. **O5c.** Covered needs the flag, the environment variable and the `math.MaxUint32` default.
   The config-file key is optional.
10. **O5d.** Covered needs datastore throttling described as optional or experimental, plus a
    delay once a request passes a read threshold. Saying that the request is marked throttled is
    not required.
11. Material beyond the key (BatchCheck not passing the limit, a value of 0, clock skew, lazy SQL
    iterators) is not graded. It would only matter if it contradicted a key fact, and none did.

## Per-label detail

Each label gets a verbatim quote for every fact. "…" marks an elision inside a quote. The
misses and close calls are listed first.

### 115 (0.955)

- **M, O4d.** Quote: "—". Absent. The nearest statement is "`relationInvolves` … also returns
  `ErrInvalidModel` … It is a query helper rather than part of `NewAndValidate`." That is not
  load-time validation.
- **Close call, none.**

| Fact | Call | Quote |
|---|---|---|
| O1a | C | "If v2 returns an error that `isV2TerminalError` does not treat as terminal, `Server.Check` falls back to the v1 chain." |
| O1b | C | "It is built by `CheckResolverOrderedBuilder.Build()` in `internal/graph/builder.go`." |
| O1c | C | "the chain is `[CachedCheckResolver]? → [DispatchThrottlingCheckResolver]? → (ShadowResolver \| LocalChecker)`" |
| O1d | C | "The last resolver delegates back to the first, so the chain is a circular list." |
| O1e | C | "`ShadowResolver` (`internal/graph/shadow_resolver.go`), wrapping two `LocalChecker`s \| The per-store feature flag `shadow_check` (`ExperimentalShadowCheck`) is on" |
| O2a | C | "All four datastore implementations: memory, postgres, mysql and sqlite." |
| O2b | C | "The four wrappers with explicit methods: `BoundedTupleReader`, `CombinedTupleReader`, `CachedTupleReader` and `ContextTracerWrapper`." |
| O2c | C | "Two production functions call it: `LocalChecker.checkDirectUserTuple` (v1) and `Resolver.specificType` (v2)." |
| O2d | C | "Regenerate `internal/mocks/mock_storage.go`. It is generated by `mockgen -source pkg/storage/storage.go`; use `make generate-mocks`." |
| O3a | C | "If `LastChecked` is older than `minInvalidationInterval` (`--cache-controller-ttl`, default 10s), it also starts `InvalidateIfNeeded`." |
| O3b | C | "If all 50 changes fall within the iterator TTL, it invalidates the whole store's iterator cache (`InvalidIteratorCacheKey`)." |
| O3c | C | "OpenFGA does not invalidate the Check cache when tuples are written." |
| O3d | C | "it uses it only if `res.LastModified.After(req.LastCacheInvalidationTime)`." |
| O3e | C | "`HIGHER_CONSISTENCY` requests skip the cache entirely." |
| O4a | C | "It then calls `typesystem.NewAndValidate(ctx, model)` and maps any error to `InvalidAuthorizationModelInput`." |
| O4b | C | "`Y` must be purely direct: its rewrite must be exactly `Userset_This`, with no union, computed userset or TTU" |
| O4c | C | "`Y`'s type restrictions may not be usersets (`type#rel`) or wildcards (`type:*`) \| `TypeSystem.validateTypeRestrictions`" |
| O4d | M | — |
| O5a | C | "`NewBoundedTupleReader` creates `limiter: make(chan struct{}, op.Concurrency)`." |
| O5b | C | "`CheckQueryV2.Execute` (`pkg/server/commands/check.go`) builds its own `BoundedTupleReader` from `datastoreOp`" |
| O5c | C | "The default is `DefaultMaxConcurrentReadsForCheck = math.MaxUint32`, which is effectively unlimited" |
| O5d | C | "If datastore throttling is enabled, `bound()` also sleeps for `ThrottleDuration` once the request's read count exceeds `ThrottleThreshold`, and marks the request as throttled." |

### 315 (0.955)

- **M, O4d.** Quote: "—". Absent. The nearest statement is "`ErrInvalidModel` … That is a
  defensive check on an already-built typesystem, not the write-time enforcement." That refers to
  `relationInvolves`, not to model loading.
- **Close call, none.**

| Fact | Call | Quote |
|---|---|---|
| O1a | C | "That path uses `commands.NewCheckQuery` … It falls back to the v1 chain on non-terminal errors (`isV2TerminalError`)." |
| O1b | C | "`internal/graph/builder.go`, `CheckResolverOrderedBuilder.Build`: this function fixes the order, whatever order the options were passed in." |
| O1c | C | "It appends `NewCachedCheckResolver(...)` if `cachedCheckResolverEnabled` is set. … It appends `NewDispatchThrottlingCheckResolver(...)` if `dispatchThrottlingCheckResolverEnabled` is set." |
| O1d | C | "The last resolver's delegate points back to the first one, so the chain is circular" |
| O1e | C | "it appends `NewShadowChecker(NewLocalChecker(main opts), NewLocalChecker(shadow opts))`" |
| O2a | C | "The 4 backend implementations: postgres, mysql, sqlite and memory." |
| O2b | C | "The four wrappers with their own method (`BoundedTupleReader`, `CombinedTupleReader`, `CachedTupleReader`, `ContextTracerWrapper`)" |
| O2c | C | "`internal/graph/check.go`, `(*LocalChecker).checkDirectUserTuple` … `internal/check/check.go`, `(*Resolver).specificType`" |
| O2d | C | "The generated mocks. Regenerate `internal/mocks/mock_storage.go` with `make generate-mocks`" |
| O3a | C | "If `LastChecked` is older than `minInvalidationInterval` (`cache-controller-ttl`), it also starts `InvalidateIfNeeded`." |
| O3b | C | "If every change on the page falls inside the iterator TTL, it invalidates everything for the store (`InvalidIteratorCacheKey`)." |
| O3c | C | "**Writes don't invalidate.** Invalidation is pull-based: only Check requests trigger it" |
| O3d | C | "treats the entry as valid only if `res.LastModified.After(req.LastCacheInvalidationTime)`" |
| O3e | C | "**`HIGHER_CONSISTENCY` bypasses the cache.**" |
| O4a | C | "`WriteAuthorizationModelCommand.Execute` (`pkg/server/commands/write_authzmodel.go`) → `typesystem.NewAndValidate`." |
| O4b | C | "the 'type#Y' relation is referenced in at least one tupleset and thus must be a direct relation" |
| O4c | C | "`Y`'s type restrictions must be **plain object types**, with no `type#relation` userset and no `type:*` wildcard." |
| O4d | M | — |
| O5a | C | "`NewBoundedTupleReader` creates `limiter: make(chan struct{}, op.Concurrency)`." |
| O5b | C | "`CheckQuery.Execute` passes it as `storagewrappers.Operation{Concurrency: c.maxConcurrentReads}` to `NewRequestStorageWrapperWithCache`" |
| O5c | C | "Env \| `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK` … Default \| `math.MaxUint32` (4294967295), i.e. effectively unbounded" |
| O5d | C | "each read after the threshold count is delayed by the throttle duration and the request is marked throttled." |

### 351 (0.955)

- **M, O4d.** Quote: "—". Absent. The nearest statement is "`relationInvolves` also rejects a
  `type#relation` restriction on a tupleset, with `ErrInvalidModel`." That is not model loading.
- **Close call, O3b (C).** The answer gives the split between the whole store and per
  object#relation and per user and object type, and the newest-first read. It does not state the
  "all within the TTL" condition. Covered under calibration 6.
- **Close call, O5d (C).** The answer gives the delay after the threshold behind
  `ExperimentalDatastoreThrottling`, but does not say the request is marked throttled. Covered
  under calibration 10.

| Fact | Call | Quote |
|---|---|---|
| O1a | C | "if `v2Check` returns an error that `isV2TerminalError` does not treat as terminal, Check falls back to v1." |
| O1b | C | "`CheckResolverOrderedBuilder.Build` (`internal/graph/builder.go`) appends the resolvers in a fixed order." |
| O1c | C | "The order is `[CachedCheckResolver]? → [DispatchThrottlingCheckResolver]? → (ShadowResolver \| LocalChecker) ⟲`." |
| O1d | C | "Subproblems that `LocalChecker` dispatches therefore go back through the cache and the throttler." |
| O1e | C | "a **`ShadowResolver`** (`internal/graph/shadow_resolver.go`, `NewShadowChecker`) that wraps two `LocalChecker`s." |
| O2a | C | "The four backends: postgres, mysql, sqlite and memory." |
| O2b | C | "`storagewrappers.BoundedTupleReader` … `storagewrappers.CombinedTupleReader` … `storagewrappers.ContextTracerWrapper` … `storagewrappers.CachedTupleReader`" |
| O2c | C | "`LocalChecker.checkDirectUserTuple` in `internal/graph/check.go` … `Resolver.specificType` in `internal/check/check.go`." |
| O2d | C | "The mocks: regenerate them with `make generate-mocks` (the Makefile has this target)" |
| O3a | C | "On a miss, it calls `InvalidateIfNeeded` asynchronously and returns the zero time." |
| O3b | C | "It writes \"invalid entity\" markers for the iterator cache: store-wide (`InvalidIteratorCacheKey`), or per object#relation and per user+object type for recent changes." |
| O3c | C | "**Nothing happens at write time.** No write path calls the controller." |
| O3d | C | "a cached `CheckResponseCacheEntry` is used only if `res.LastModified.After(req.LastCacheInvalidationTime)`." |
| O3e | C | "`HIGHER_CONSISTENCY` requests bypass the cache entirely." |
| O4a | C | "The path is `WriteAuthorizationModelCommand.Execute` (`pkg/server/commands/write_authzmodel.go`) → `typesystem.NewAndValidate`." |
| O4b | C | "**It must be a direct relation only.** Its rewrite must be exactly `this` (`Userset_This`)" |
| O4c | C | "it calls `IsTuplesetRelation(objectType, relationName)` and returns `InvalidRelationTypeError` if a related type has `GetRelationOrWildcard() != nil`." |
| O4d | M | — |
| O5a | C | "`NewBoundedTupleReader(ds, op)` creates `limiter: make(chan struct{}, op.Concurrency)`." |
| O5b | C | "A new bounded reader, and so a new semaphore, is created for each Check request." |
| O5c | C | "Environment variable \| `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK` … The default is `math.MaxUint32`, which is effectively unbounded" |
| O5d | C | "After `threshold` reads it sleeps `throttleTime` per read. This is controlled by the feature flag `ExperimentalDatastoreThrottling`" |

### 447 (0.955)

- **M, O4d.** Quote: "—". Absent. The nearest statement is "At runtime, `relationInvolves` …
  returns `ErrInvalidModel` if a tupleset relation has a `type#relation` restriction." That is a
  query-time helper, not model loading.
- **Close call, none.**

| Fact | Call | Quote |
|---|---|---|
| O1a | C | "`v2Check` falls back to the v1 chain on any error that is not terminal (`isV2TerminalError`)." |
| O1b | C | "The assembly itself is `CheckResolverOrderedBuilder.Build` (`internal/graph/builder.go`)." |
| O1c | C | "`CachedCheckResolver`, if it is enabled; … `DispatchThrottlingCheckResolver`, if it is enabled;" |
| O1d | C | "The chain is circular: the last resolver delegates back to the first" |
| O1e | C | "`graph.ShadowResolver` (`internal/graph/shadow_resolver.go`), built by `NewShadowChecker(main, shadow)`" |
| O2a | C | "The four backends: memory, postgres, mysql and sqlite." |
| O2b | C | "`storagewrappers.BoundedTupleReader` … `storagewrappers.CombinedTupleReader` … `storagewrappers.CachedTupleReader` … `storagewrappers.ContextTracerWrapper`" |
| O2c | C | "`LocalChecker.checkDirectUserTuple` (`internal/graph/check.go`), on the v1 Check path." |
| O2d | C | "The regenerated mocks (`make generate-mocks`) and `slowDataStorage`." |
| O3a | C | "If `LastChecked` is older than `--cache-controller-ttl` (`minInvalidationInterval`, default 10s), it calls `InvalidateIfNeeded`." |
| O3b | C | "That is either a full invalidation (`InvalidIteratorCacheKey`) or a partial one, keyed by object#relation and by user plus object type" |
| O3c | C | "The Write path does not call the controller." |
| O3d | C | "A cached entry is used only if `res.LastModified.After(req.LastCacheInvalidationTime)`" |
| O3e | C | "`HIGHER_CONSISTENCY` requests skip the Check cache entirely." |
| O4a | C | "All of this is enforced in `typesystem.NewAndValidate` when a model is written." |
| O4b | C | "`Y` must be a **direct relation only**: its rewrite must be exactly `this` (`Userset_This`)" |
| O4c | C | "**No userset (`type#rel`) and no wildcard (`type:*`)** is allowed when `IsTuplesetRelation` is true. \| `validateTypeRestrictions` \| `InvalidRelationTypeError`" |
| O4d | M | — |
| O5a | C | "`BoundedTupleReader` holds `limiter: make(chan struct{}, op.Concurrency)`." |
| O5b | C | "On the v2 path, `CheckQueryV2.Execute` (`pkg/server/commands/check.go`) builds it directly" |
| O5c | C | "Environment variable `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK` … The default is `math.MaxUint32`" |
| O5d | C | "Once a Check has made more than *threshold* reads, `bound()` adds a fixed delay to each further read and marks the request throttled." |

### 581 (0.955)

- **M, O4d.** Quote: "—". Absent. The answer does not mention any validation outside model
  writes and tuple writes.
- **Close call, none.**

| Fact | Call | Quote |
|---|---|---|
| O1a | C | "If v2 fails with an error that is not terminal (see `isV2TerminalError`), Check falls back to the v1 chain above." |
| O1b | C | "`CheckResolverOrderedBuilder.Build` (`internal/graph/builder.go`) sets the order." |
| O1c | C | "It appends the cached resolver, then the dispatch throttler, then the shadow resolver or the local checker." |
| O1d | C | "each resolver delegates to the next, and the last one delegates back to the first." |
| O1e | C | "`graph.ShadowResolver` (`internal/graph/shadow_resolver.go`), which wraps a main `LocalChecker` and a shadow `LocalChecker`." |
| O2a | C | "The four datastore implementations: postgres, mysql, sqlite and memory." |
| O2b | C | "The four wrappers that override the method: `BoundedTupleReader`, `CombinedTupleReader`, `CachedTupleReader` and `ContextTracerWrapper`." |
| O2c | C | "`LocalChecker.checkDirectUserTuple` in `internal/graph/check.go`, the v1 Check. … `Resolver.specificType` in `internal/check/check.go`" |
| O2d | C | "The mocks: regenerate `internal/mocks/mock_storage.go` and edit `mock_slow_storage.go` by hand." |
| O3a | C | "If the entry is missing, it starts an asynchronous `InvalidateIfNeeded` and returns the zero time" |
| O3b | C | "It does this by writing `InvalidEntityCacheEntry` markers rather than deleting entries" |
| O3c | C | "The Write API itself does not trigger invalidation." |
| O3d | C | "reuses an entry only if `res.LastModified.After(req.LastCacheInvalidationTime)`." |
| O3e | C | "Requests with `HIGHER_CONSISTENCY` skip the cache entirely." |
| O4a | C | "All checks run in `typesystem.NewAndValidate` … wraps any error as `InvalidAuthorizationModelInput`." |
| O4b | C | "**`Y` must be a direct relation.** Its rewrite must be exactly `this` (`Userset_This`)" |
| O4c | C | "If `IsTuplesetRelation(T, Y)` is true and a restriction has a relation or a wildcard … it returns `InvalidRelationTypeError`." |
| O4d | M | — |
| O5a | C | "It holds a buffered channel `limiter` whose capacity is `Operation.Concurrency`." |
| O5b | C | "`CheckQueryV2.Execute` calls `NewBoundedTupleReader` directly (`pkg/server/commands/check.go`)." |
| O5c | C | "Environment variable \| `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK` … Default \| `math.MaxUint32`, which is effectively unlimited" |
| O5d | C | "Once the per-request read count passes a threshold, it sleeps for a set duration and marks the request as throttled." |

### 667 (0.955)

- **M, O4d.** Quote: "—". Absent. The nearest statement is "`TypeSystem.relationInvolves` (TTU
  case) also returns `ErrInvalidModel` … not part of the `validateRelation` pipeline." That is
  not model loading.
- **Close call, none.**

| Fact | Call | Quote |
|---|---|---|
| O1a | C | "On a non-terminal error it falls back to the v1 chain described above (`isV2TerminalError`, `pkg/server/check.go`)." |
| O1b | C | "**How `CheckResolverOrderedBuilder.Build()` (`internal/graph/builder.go`) fixes the order:**" |
| O1c | C | "It appends the resolvers in a fixed order regardless of option order: cached, then dispatch-throttling, then shadow or local." |
| O1d | C | "each resolver points to the next one, and the last one points back to `resolvers[0]`." |
| O1e | C | "`graph.ShadowResolver` wrapping two `graph.LocalChecker`s (main and shadow)" |
| O2a | C | "All four backends: postgres, mysql, sqlite and memory." |
| O2b | C | "The four wrappers that override the method: `BoundedTupleReader`, `CombinedTupleReader`, `CachedTupleReader` and `ContextTracerWrapper`." |
| O2c | C | "`graph.LocalChecker.checkDirectUserTuple` in `internal/graph/check.go` … `check.Resolver.specificType` in `internal/check/check.go`." |
| O2d | C | "Regenerating `internal/mocks/mock_storage.go` with mockgen" |
| O3a | C | "On a hit, it starts `InvalidateIfNeeded` only if `time.Since(entry.LastChecked) > minInvalidationInterval`" |
| O3b | C | "If every change in the page is newer than `now - iteratorCacheTTL`, it marks the whole store with `InvalidIteratorCacheKey`." |
| O3c | C | "Nothing is invalidated when the write happens." |
| O3d | C | "treats an entry as valid only if `CheckResponseCacheEntry.LastModified.After(req.LastCacheInvalidationTime)`." |
| O3e | C | "**The cache is bypassed entirely** when the consistency is `HIGHER_CONSISTENCY`." |
| O4a | C | "calls `typesystem.NewAndValidate(ctx, model)`. On error it returns `InvalidAuthorizationModelInput`." |
| O4b | C | "`Y` must be a pure direct relation: its rewrite must be `Userset_This`" |
| O4c | C | "`Y`'s type restrictions cannot be usersets (`type#rel`) or wildcards (`type:*`) … returns `InvalidRelationTypeError`." |
| O4d | M | — |
| O5a | C | "`BoundedTupleReader` holds a buffered channel `limiter` of size `op.Concurrency`." |
| O5b | C | "Each Check request builds its own `storagewrappers.BoundedTupleReader`" |
| O5c | C | "Its default is `DefaultMaxConcurrentReadsForCheck = math.MaxUint32`, which effectively means unlimited." |
| O5d | C | "it sleeps `throttleTime` before each further read and marks the request as throttled." |

### 686 (0.955)

- **M, O4d.** Quote: "—". Absent. The nearest statement is "`relationInvolves` also returns
  `ErrInvalidModel` … It is a query-time helper, not part of `NewAndValidate`". That is not model
  loading.
- **Close call, O1a (C).** The answer names the flag only by its string, `weighted_graph_check`,
  not by the constant. It does give `s.v2Check`, `CheckQueryV2` and the fallback on non-terminal
  errors. Covered under calibration 1.
- **Close call, O5d (C).** The answer gives the delay after the threshold behind
  `datastore_throttling`, but does not say the request is marked throttled. Covered under
  calibration 10.

| Fact | Call | Quote |
|---|---|---|
| O1a | C | "If the `weighted_graph_check` experimental flag is on for the store, `Server.Check` first tries `s.v2Check`." |
| O1b | C | "a chain built by `graph.CheckResolverOrderedBuilder.Build` (`internal/graph/builder.go`)." |
| O1c | C | "`[CachedCheckResolver]? → [DispatchThrottlingCheckResolver]? → [ShadowResolver \| LocalChecker] ⟲`" |
| O1d | C | "the last resolver's delegate back to the first (`resolver.SetDelegate(c.resolvers[0])`)." |
| O1e | C | "a `ShadowResolver` (`internal/graph/shadow_resolver.go`) that wraps two `LocalChecker`s (main and shadow)" |
| O2a | C | "The four backend implementations: memory, postgres, mysql and sqlite." |
| O2b | C | "`storagewrappers.BoundedTupleReader` … `storagewrappers.CombinedTupleReader` … `storagewrappers.CachedTupleReader` … `storagewrappers.ContextTracerWrapper`" |
| O2c | C | "`LocalChecker.checkDirectUserTuple` in `internal/graph/check.go` (v1 Check). … `Resolver.specificType` in `internal/check/check.go`" |
| O2d | C | "The generated mocks. Regenerate them with `make generate-mocks`; don't edit them by hand" |
| O3a | C | "If the entry is missing, it starts an asynchronous invalidation and returns the zero time" |
| O3b | C | "if every fetched change is within the iterator TTL, it invalidates the whole store with `invalidateIteratorCache`;" |
| O3c | C | "**The write path does no invalidation itself.**" |
| O3d | C | "it uses it only if `res.LastModified.After(req.LastCacheInvalidationTime)`." |
| O3e | C | "`HIGHER_CONSISTENCY` bypasses the cache entirely." |
| O4a | C | "Every `WriteAuthorizationModel` request enforces the constraints below in `typesystem.NewAndValidate`" |
| O4b | C | "`Y` must be purely direct: its rewrite must be exactly `this` (`Userset_This`)" |
| O4c | C | "`Y`'s type restrictions must be plain object types: no userset (`type#relation`) and no wildcard (`type:*`)." |
| O4d | M | — |
| O5a | C | "Its `limiter` is a buffered channel of capacity `Operation.Concurrency`." |
| O5b | C | "`NewRequestStorageWrapperWithCache` (`request.go`) puts `NewBoundedTupleReader` innermost" |
| O5c | C | "the `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK` environment variable … `DefaultMaxConcurrentReadsForCheck = math.MaxUint32`" |
| O5d | C | "After that many reads in one request, each further read is delayed by `check-datastore-throttle-duration`." |

### 769 (0.955)

- **M, O4d.** Quote: "—". Absent. The nearest statement is "`relationInvolves` repeats the
  assumption … but it is a later helper, not a write-time check." That is not model loading.
- **Close call, O5d (C).** The answer gives the delay after the threshold behind
  `datastore_throttling`, but does not say the request is marked throttled. Covered under
  calibration 10.

| Fact | Call | Quote |
|---|---|---|
| O1a | C | "It falls back to the v1 path on any error except timeouts, cancellation and throttling (`isV2TerminalError`)." |
| O1b | C | "`CheckResolverOrderedBuilder.Build()` (`internal/graph/builder.go`) appends the resolvers in a fixed order" |
| O1c | C | "1 \| `CachedCheckResolver` … 2 \| `DispatchThrottlingCheckResolver` … 3 \| Either `ShadowResolver` … or `LocalChecker`" |
| O1d | C | "The last resolver's delegate is set back to the first one, so the chain is circular." |
| O1e | C | "`ShadowResolver` wraps two `LocalChecker`s. It returns the `main` result and runs the `shadow` one asynchronously" |
| O2a | C | "`memory.MemoryBackend` … `postgres.Datastore` … `mysql.Datastore` … `sqlite.Datastore`" |
| O2b | C | "`storagewrappers.BoundedTupleReader` … `storagewrappers.CombinedTupleReader` … `storagewrappers.CachedTupleReader` … `storagewrappers.ContextTracerWrapper`" |
| O2c | C | "`LocalChecker.checkDirectUserTuple` (`internal/graph/check.go`). … `Resolver.specificType` (`internal/check/check.go`)." |
| O2d | C | "The mocks: regenerate `mock_storage.go` (`make generate-mocks`) and edit `mock_slow_storage.go` by hand." |
| O3a | C | "If the entry was last checked more than `minInvalidationInterval` ago (`cacheController.ttl`), it also starts a background refresh." |
| O3b | C | "A full invalidation uses `InvalidIteratorCacheKey`. A partial one uses `InvalidIteratorByObjectRelationCacheKey` and `InvalidIteratorByUserObjectTypeCacheKey`" |
| O3c | C | "Writes do not invalidate the Check query cache." |
| O3d | C | "it accepts a hit only if `res.LastModified.After(req.LastCacheInvalidationTime)`." |
| O3e | C | "With `HIGHER_CONSISTENCY` they skip the call and the cache altogether." |
| O4a | C | "`Execute` calls `typesystem.NewAndValidate(ctx, model)` … Any error is returned as `serverErrors.InvalidAuthorizationModelInput`" |
| O4b | C | "**`Y` must be a pure direct relation.** Its rewrite must be exactly `this`" |
| O4c | C | "`validateTypeRestrictions` checks `related.GetRelationOrWildcard() != nil && t.IsTuplesetRelation(objectType, relationName)` and returns `InvalidRelationTypeError`." |
| O4d | M | — |
| O5a | C | "`NewBoundedTupleReader` creates `limiter: make(chan struct{}, op.Concurrency)`." |
| O5b | C | "`CheckQuery.Execute` then builds a new `BoundedTupleReader` for each request through `NewRequestStorageWrapperWithCache`" |
| O5c | C | "Environment variable \| `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK` … Default \| `DefaultMaxConcurrentReadsForCheck = math.MaxUint32` (4294967295), effectively unbounded" |
| O5d | C | "after `ThrottleThreshold` reads it sleeps `ThrottleDuration` before each further read. The throttle is enabled by the per-store `\"datastore_throttling\"` experimental flag" |

### 771 (0.955)

- **M, O4d.** Quote: "—". Absent. The nearest statement is "`TypeSystem.relationInvolves` …
  returns `ErrInvalidModel` at evaluation time … This is a defensive duplicate of rule 4." That is
  not model loading.
- **Close call, none.**

| Fact | Call | Quote |
|---|---|---|
| O1a | C | "then `commands.NewCheckCommand(...).Execute` … If v2 fails with an error that `isV2TerminalError` does not treat as terminal, Check falls back to v1." |
| O1b | C | "`CheckResolverOrderedBuilder.Build` (`internal/graph/builder.go`) appends the resolvers in a fixed order and links them with `SetDelegate`." |
| O1c | C | "`CachedCheckResolver`, if enabled. … `DispatchThrottlingCheckResolver`, if enabled. … Either `ShadowResolver` … or a plain `LocalChecker`." |
| O1d | C | "the last one delegates back to the first, so the list is circular." |
| O1e | C | "`ShadowResolver` (`internal/graph/shadow_resolver.go`), built by `NewShadowChecker(main, shadow)` with two `LocalChecker`s" |
| O2a | C | "The four backends: memory, mysql, postgres and sqlite." |
| O2b | C | "`storagewrappers.BoundedTupleReader` … `storagewrappers.CombinedTupleReader` … `storagewrappers.CachedTupleReader` … `storagewrappers.ContextTracerWrapper`" |
| O2c | C | "`LocalChecker.checkDirectUserTuple` in `internal/graph/check.go` … `Resolver.specificType` in `internal/check/check.go`" |
| O2d | C | "`internal/mocks/mock_storage.go`, which should be regenerated." |
| O3a | C | "**On a miss**, it triggers an async `InvalidateIfNeeded` and returns the zero time" |
| O3b | C | "If every change in the page falls inside the iterator TTL window, it writes a store-wide marker (`InvalidIteratorCacheKey`)." |
| O3c | C | "OpenFGA does not invalidate the cache when tuples are written." |
| O3d | C | "accepts an entry only if `res.LastModified.After(req.LastCacheInvalidationTime)`." |
| O3e | C | "**`HIGHER_CONSISTENCY` bypasses the cache entirely.**" |
| O4a | C | "`WriteAuthorizationModelCommand.Execute` … calls `typesystem.NewAndValidate`. Any error is returned as `serverErrors.InvalidAuthorizationModelInput`." |
| O4b | C | "**`Y` must be a pure direct relation**, meaning its rewrite is exactly `this`" |
| O4c | C | "**`Y`'s type restrictions cannot be usersets (`type#rel`) or wildcards (`type:*`).** `validateTypeRestrictions` handles this" |
| O4d | M | — |
| O5a | C | "`NewBoundedTupleReader` creates a buffered channel, `limiter := make(chan struct{}, op.Concurrency)`." |
| O5b | C | "A new wrapper, and therefore a new limiter, is built for each Check." |
| O5c | C | "Environment variable `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK` … The default is `DefaultMaxConcurrentReadsForCheck = math.MaxUint32`" |
| O5d | C | "the request's read count exceeds the threshold, it sleeps for `throttleTime` and marks the request as throttled." |

### 996 (0.955)

- **M, O4d.** Quote: "—". Absent. The nearest statement is "`TypeSystem.relationInvolves` … That
  is a query-time helper, not model-write validation." That is not model loading.
- **Close call, none.**

| Fact | Call | Quote |
|---|---|---|
| O1a | C | "runs `commands.NewCheckCommand(...).Execute` … It falls back to v1 on errors that `isV2TerminalError` does not treat as terminal." |
| O1b | C | "**`Build()` (`internal/graph/builder.go`, `CheckResolverOrderedBuilder.Build`).** It appends resolvers in a fixed order" |
| O1c | C | "`NewCachedCheckResolver`, if `cachedCheckResolverEnabled`; … `NewDispatchThrottlingCheckResolver`, if `dispatchThrottlingCheckResolverEnabled`;" |
| O1d | C | "The last resolver's delegate is set back to the first, so the chain is a ring." |
| O1e | C | "`NewShadowChecker(main, shadow)` if `shadowResolverEnabled`, otherwise `NewLocalChecker`." |
| O2a | C | "`memory.MemoryBackend` … `mysql.Datastore` … `postgres.Datastore` … `sqlite.Datastore`" |
| O2b | C | "`storagewrappers.BoundedTupleReader` … `storagewrappers.CombinedTupleReader` … `storagewrappers.ContextTracerWrapper` … `storagewrappers.CachedTupleReader`" |
| O2c | C | "`LocalChecker.checkDirectUserTuple` in `internal/graph/check.go` … `Resolver.specificType` in `internal/check/check.go`" |
| O2d | C | "the generated mocks in `internal/mocks/mock_storage.go`, regenerated with `make generate-mocks`" |
| O3a | C | "On a miss, it triggers `InvalidateIfNeeded` asynchronously and returns zero time" |
| O3b | C | "It writes a store-wide marker (`InvalidIteratorCacheKey`) if all fetched changes fall inside the iterator TTL or if `ReadChanges` fails." |
| O3c | C | "A write does not touch any cache: `pkg/server/commands/write.go` has no cache code. Staleness is detected lazily" |
| O3d | C | "uses a `CheckResponseCacheEntry` only if `res.LastModified.After(req.LastCacheInvalidationTime)`." |
| O3e | C | "`HIGHER_CONSISTENCY` bypasses the caches entirely." |
| O4a | C | "through `WriteAuthorizationModelCommand.Execute` … That function calls `typesystem.NewAndValidate`" |
| O4b | C | "Its rewrite must be exactly `Userset_This`, with no union, intersection, exclusion or computed userset." |
| O4c | C | "A userset (`type#rel`) or wildcard (`type:*`) restriction is rejected. `validateTypeRestrictions` returns `InvalidRelationTypeError`" |
| O4d | M | — |
| O5a | C | "a buffered channel `limiter` of capacity `op.Concurrency`." |
| O5b | C | "`CheckQueryV2.Execute` calls `NewBoundedTupleReader` directly (`pkg/server/commands/check.go`)." |
| O5c | C | "Environment \| `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK` … The default is `math.MaxUint32`, which is effectively unbounded" |
| O5d | C | "every read after the first `threshold` reads in the request waits an extra `throttleTime`. The request is then marked `WasThrottled`." |
