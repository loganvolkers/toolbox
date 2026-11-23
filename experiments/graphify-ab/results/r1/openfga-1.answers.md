# Answers

## Q1

**Two Check paths exist.** `Server.Check` (`pkg/server/check.go`) first validates the request and runs `checkAuthz`. Next it checks the feature flag `serverconfig.ExperimentalWeightedGraphCheck` (`"weighted_graph_check"`, `pkg/server/config/config.go`).
- **Flag on:** it calls `Server.v2Check`. That path builds a `commands.CheckQueryV2` (`pkg/server/commands/check.go`), which uses a single `check.Resolver` from `internal/check/check.go` (`check.New`). There is no resolver chain on this path.
- **Fallback:** if `v2Check` returns an error that `isV2TerminalError` does not treat as terminal, Check falls back to v1.

The chain below is the v1 (default) path.

**Code that builds the chain.**
- `Server.getCheckResolverBuilder(storeID)` (`pkg/server/check.go`) calls `graph.NewOrderedCheckResolvers(...)` with these options: `WithLocalCheckerOpts`, `WithLocalShadowCheckerOpts`, `WithShadowResolverEnabled`, `WithShadowResolverOpts`, `WithCachedCheckResolverOpts` and `WithDispatchThrottlingCheckResolverOpts`.
- `Server.Check` then calls `.Build()`.
- `CheckResolverOrderedBuilder.Build` (`internal/graph/builder.go`) appends the resolvers in a fixed order. The order of the options does not matter.
  1. **`CachedCheckResolver`** (`internal/graph/cached_resolver.go`). It is added only if `cachedCheckResolverEnabled` is true. That value comes from `s.cacheSettings.ShouldCacheCheckQueries()` (`pkg/server/config/cache_settings.go`), which is `CheckCacheLimit > 0 && CheckQueryCacheEnabled`. The operator flags are `--check-query-cache-enabled` and `--check-cache-limit`; the cache is off by default (`DefaultCheckQueryCacheEnabled = false`). Its options come from `Server.getCheckResolverOptions` (`pkg/server/server.go`): the shared cache, the TTL and the jitter.
  2. **`DispatchThrottlingCheckResolver`** (`internal/graph/dispatch_throttling_check_resolver.go`). It is added only if `s.checkDispatchThrottlingEnabled` is true, set by `--check-dispatch-throttling-enabled` through `WithDispatchThrottlingCheckResolverEnabled`. Its `ResolveCheck` compares the request's `DispatchCounter` against a threshold, may call `throttler.Throttle`, and then delegates.
  3. **The terminal resolver**, which depends on the feature flag `ExperimentalShadowCheck` (`"shadow_check"`), read per store through `featureFlagClient.Boolean`:
     - **Flag on:** a **`ShadowResolver`** (`internal/graph/shadow_resolver.go`, `NewShadowChecker`) that wraps two `LocalChecker`s. The main checker uses the normal options; the shadow checker always has optimizations on. The main result is returned. The shadow check runs asynchronously in a goroutine and only logs differences. `ShadowResolver.SetDelegate` sets the delegate only on `main`.
     - **Flag off:** a plain **`LocalChecker`** (`internal/graph/check.go`).
- **Linking:** `Build` makes the list circular. Each resolver gets `SetDelegate(next)`, and the last one (the `LocalChecker` or `ShadowResolver`) gets `SetDelegate(resolvers[0])`. Subproblems that `LocalChecker` dispatches therefore go back through the cache and the throttler. `Build` returns `resolvers[0]` and a closer.
- The order is `[CachedCheckResolver]? → [DispatchThrottlingCheckResolver]? → (ShadowResolver | LocalChecker) ⟲`. This matches `AGENTS.md` (resolver chain section) and `docs/check/README.md` (CheckResolver section).

**Where tuples are read.**
- `commands.CheckQuery.Execute` (`pkg/server/commands/check_command.go`) puts a `storagewrappers.RequestStorageWrapper` into the context with `storage.ContextWithRelationshipTupleReader`. It builds the wrapper with `NewRequestStorageWrapperWithCache` (`pkg/storage/storagewrappers/request.go`). It then calls `checkResolver.ResolveCheck`.
- Only `LocalChecker` reads tuples, for example in `checkDirectUserTuple` and `checkPublicAssignable`. It gets the reader with `storage.RelationshipTupleReaderFromContext`.
- From outermost to innermost, that reader is:
  1. `CombinedTupleReader`, which handles contextual tuples.
  2. `sharediterator.SharedIteratorDatastore`, only if `SharedIteratorEnabled`.
  3. `CachedDatastore`, the iterator cache, only if `ShouldCacheCheckIterators()`.
  4. `BoundedTupleReader`.
  5. The server datastore, which `NewServerWithOpts` in `pkg/server/server.go` wraps as `CachedOpenFGADatastore` around `ContextTracerWrapper`. The `ContextTracerWrapper` is only added when context propagation to the datastore is disabled.

These are storage wrappers, not `CheckResolver`s.

## Q2

**Interface.** `ReadUserTuple` is declared on `storage.RelationshipTupleReader` (`pkg/storage/storage.go`):
`ReadUserTuple(ctx, store string, filter ReadUserTupleFilter, options ReadUserTupleOptions) (*openfgav1.Tuple, error)`.
- It must return `storage.ErrNotFound` when no tuple matches.
- `ReadUserTupleFilter` is a type alias of `ReadFilter`.
- `ReadUserTupleOptions` holds `Consistency ConsistencyOptions`.
- `TupleBackend` embeds the interface, and `OpenFGADatastore` embeds `TupleBackend`.

**Types that define the method explicitly (non-mock):**

| Type | File | Role |
|---|---|---|
| `postgres.Datastore` | `pkg/storage/postgres/postgres.go` | backend |
| `mysql.Datastore` | `pkg/storage/mysql/mysql.go` | backend |
| `sqlite.Datastore` | `pkg/storage/sqlite/sqlite.go` | backend |
| `memory.MemoryBackend` | `pkg/storage/memory/memory.go` | backend |
| `storagewrappers.BoundedTupleReader` | `pkg/storage/storagewrappers/bounded_datastore.go` | concurrency bound and throttling, then delegates |
| `storagewrappers.CombinedTupleReader` | `pkg/storage/storagewrappers/combinedtuplereader.go` | answers from contextual tuples first, then delegates |
| `storagewrappers.ContextTracerWrapper` | `pkg/storage/storagewrappers/context.go` | detaches cancellation with `context.WithoutCancel`, then delegates |
| `storagewrappers.CachedTupleReader` | `pkg/storage/storagewrappers/cached_reader.go` | v2 iterator cache; passes straight through without caching |

**Mock and test types.**
- `MockTupleBackend`, `MockRelationshipTupleReader` and `MockOpenFGADatastore` are in `internal/mocks/mock_storage.go`, generated by mockgen from `pkg/storage/storage.go`.
- `slowDataStorage` is in `internal/mocks/mock_slow_storage.go`.

**Types that get the method only by embedding** a `RelationshipTupleReader` or `OpenFGADatastore`, without defining it:
- `CachedDatastore` (`cached_datastore.go`)
- `CachedOpenFGADatastore` (`model_caching.go`)
- `SharedIteratorDatastore` (`sharediterator/shared_iterator_datastore.go`)
- `RequestStorageWrapper` (`request.go`)

**Production callers** (excluding tests and mocks):
1. `LocalChecker.checkDirectUserTuple` in `internal/graph/check.go`. This is the v1 Check direct-assignment lookup. It gets `ds` from `storage.RelationshipTupleReaderFromContext` and treats `ErrNotFound` as "not allowed".
2. `Resolver.specificType` in `internal/check/check.go`. This is the v2 weighted-graph Check. It calls `r.datastore.ReadUserTuple` when no contextual tuple matched, and also treats `ErrNotFound` as "not allowed".
3. The delegating wrappers listed above, each forwarding to its inner reader: `BoundedTupleReader`, `CombinedTupleReader`, `ContextTracerWrapper` and `CachedTupleReader`.

In `internal/graph/check.go`, `checkPublicAssignable` deliberately uses `ReadUsersetTuples` instead, so it can hit the iterator cache. A code comment there says so.

**Impact of a signature change:**
- The interface in `pkg/storage/storage.go`, and possibly `ReadUserTupleFilter`/`ReadFilter` and `ReadUserTupleOptions` in the same file.
- The four backends: postgres, mysql, sqlite and memory.
- The four explicit wrappers: bounded, combined, context and cached reader.
- The two production callers: `internal/graph/check.go` and `internal/check/check.go`.
- The mocks: regenerate them with `make generate-mocks` (the Makefile has this target), and hand-edit `internal/mocks/mock_slow_storage.go`.
- The shared datastore conformance suite in `pkg/storage/test/` (for example `tuples.go`), which calls `ReadUserTuple`.
- The tests that call it: `internal/check/check_test.go`, `internal/graph/check_test.go`, `pkg/server/server_test.go`, the postgres, mysql and sqlite `*_test.go` files, and the storagewrappers tests (`combinedtuplereader_test.go`, `cached_reader_test.go`, `bounded_datastore_test.go`).
- Any out-of-tree datastore implementation.

Types that only embed the interface would not need edits.

## Q3

**Mechanism: a per-store "last write time" compared against each entry's timestamp.**

1. **Cache controller.** `cachecontroller.CacheController` lives in `internal/cachecontroller/cache_controller.go`.
   - `SharedDatastoreResources` (`internal/shared/shared.go`) creates an `InMemoryCacheController` only if `CacheSettings.ShouldCreateCacheController()` is true. That requires a cache to be enabled and `CacheControllerEnabled`. The flags are `--cache-controller-enabled` (default `false`) and `--cache-controller-ttl` (default 10s), defined in `cmd/run/run.go`.
   - Otherwise a `NoopCacheController` is used. Its `DetermineInvalidationTime` always returns the zero time.

2. **Per request.** Unless the request uses `HIGHER_CONSISTENCY`, `CheckQuery.Execute` (`pkg/server/commands/check_command.go`) calls `CacheController.DetermineInvalidationTime(ctx, storeID)` and puts the result in `ResolveCheckRequest.LastCacheInvalidationTime`. `Server.v2Check` in `pkg/server/check.go` does the same.

3. **Staleness test.** In `CachedCheckResolver.ResolveCheck` (`internal/graph/cached_resolver.go`), a cached `CheckResponseCacheEntry` is used only if `res.LastModified.After(req.LastCacheInvalidationTime)`. Otherwise the entry counts as an invalid hit and the subproblem is resolved again.
   - `LastModified` is `time.Now()` when the entry was stored.
   - The v2 resolver applies the same rule in `Resolver.isCached` (`internal/check/check.go`).
   - The cache key is `storage.CheckCacheKey` (store, object, relation, user, plus an invariant hash). The invariant covers store, model ID, context and contextual tuples (`InvariantCacheKey`, `pkg/storage/cache.go`). A new model therefore gets new keys.

4. **Where the last write time comes from.**
   - `DetermineInvalidationTime` reads a `ChangelogCacheEntry{LastModified, LastChecked}` stored under `ChangelogCacheKey(storeID)`.
   - On a miss, it calls `InvalidateIfNeeded` asynchronously and returns the zero time.
   - On a hit, it returns `entry.LastModified`. If `LastChecked` is older than `minInvalidationInterval` (the controller TTL), it also triggers `InvalidateIfNeeded`.

5. **The refresh.** `InvalidateIfNeeded` allows at most one in-flight refresh per store (`inflightInvalidations`), started from a goroutine. That goroutine runs `findChangesAndInvalidateIfNecessary`:
   - It reads the store changelog newest-first through `ReadChanges` (`findChangesDescending`, one page of `storage.DefaultPageSize`), with a 1-second timeout.
   - It stores a new `ChangelogCacheEntry` whose `LastModified` is the newest change's timestamp, with TTL `queryCacheTTL`.
   - It writes "invalid entity" markers for the iterator cache: store-wide (`InvalidIteratorCacheKey`), or per object#relation and per user+object type for recent changes.
   - `CachedDatastore` checks those markers in `findInCache` and `isInvalidAt` (`pkg/storage/storagewrappers/cached_datastore.go`).

**Limits:**
- **Nothing happens at write time.** No write path calls the controller. Invalidation is driven only by Check, which calls `DetermineInvalidationTime`, and by ListObjects with the iterator cache, which calls `InvalidateIfNeeded` in `pkg/server/commands/list_objects.go`.
- **Eventual consistency.** The type comment says so. Changes can go undetected for up to the controller TTL (default 10s). A cache miss on the changelog entry returns the zero time, so stale hits are possible while the asynchronous refresh runs. Code comments call this an accepted trade-off.
- **Controller disabled (the default).** The invalidation time is always zero, so cached Check results stay valid until their TTL expires: `--check-query-cache-ttl`, default 10s, jittered by `storage.JitteredTTL`.
- **Coarse grain for Check results.** A single write anywhere in the store invalidates every cached Check result older than that write. Only the iterator cache gets partial, per-tuple invalidation.
- **Local only.** The cache and the controller are in-memory and per process. Each replica discovers writes on its own.
- **Refresh failures.** If `ReadChanges` errors (the code says it returns an error when there are no changes) or times out after 1s, the changelog entry is not updated. On an error the whole store's iterator cache is invalidated; on a timeout nothing is invalidated.
- **Two clocks.** Entries are stamped with the server's `time.Now()`, and that is compared with changelog timestamps from the datastore. I did not find any handling of clock skew between the two. This is an inference from the code, not documented behaviour.
- **Exempt results.** Responses with `CycleDetected` are never cached. `HIGHER_CONSISTENCY` requests bypass the cache entirely.

## Q4

**Constraints on the tupleset relation `Y` in `X from Y`, checked when a model is written:**

The path is `WriteAuthorizationModelCommand.Execute` (`pkg/server/commands/write_authzmodel.go`) → `typesystem.NewAndValidate`. `NewAndValidate` runs `validateRelation` for every relation, in sorted order. Errors are mapped to `serverErrors.InvalidAuthorizationModelInput`. All functions below are in `pkg/typesystem/typesystem.go`.

1. **It must exist on the same object type.** In `isUsersetRewriteValid`, the `*openfgav1.Userset_TupleToUserset` case calls `t.GetRelation(objectType, tupleset)`. If that fails, it returns `RelationUndefinedError`.
2. **It must be a direct relation only.** Its rewrite must be exactly `this` (`Userset_This`), with no union, computed userset or other rewrite. If not, `isUsersetRewriteValid` returns `"the '<type>#<rel>' relation is referenced in at least one tupleset and thus must be a direct relation"`.
3. **Its type restrictions must be plain object types,** with no `type#relation` usersets and no `type:*` wildcards. `validateTypeRestrictions` enforces this: it calls `IsTuplesetRelation(objectType, relationName)` and returns `InvalidRelationTypeError` if a related type has `GetRelationOrWildcard() != nil`. As an assignable relation, `Y` also needs at least one type restriction, and every referenced type must exist; that rule applies to all assignable relations. Conditions on the restrictions are not rejected.
4. **The computed relation `X` must exist somewhere reachable.**
   - For schema 1.1, `X` must be defined on at least one of `Y`'s directly related user types. Otherwise `isUsersetRewriteValid` returns `ErrRelationUndefined: ... does not appear as a relation in any of the directly related user types`.
   - For 1.0 models, `X` only has to exist on some type in the model.
5. **General checks.** `validateRelation` then applies the usual entrypoint check (`hasEntrypoints`) and cycle check (`HasCycle`).

`relationInvolves` also rejects a `type#relation` restriction on a tupleset, with `ErrInvalidModel`. A comment there says such restrictions are already disallowed by that point.

**Related checks at tuple-write and read time, not model-write time.** `validation.validateTuplesetRestrictions` (`internal/validation/validation.go`) is called from `ValidateTupleForWrite` → `ValidateTupleForRead`. For a tupleset relation it rejects:
- a rewrite other than `this`;
- a wildcard user (`*` or `type:*`);
- a user that is not a plain object, such as a userset like `folder:1#parent`.

## Q5

**Mechanism: a per-request semaphore in `storagewrappers.BoundedTupleReader`** (`pkg/storage/storagewrappers/bounded_datastore.go`).
- `NewBoundedTupleReader(ds, op)` creates `limiter: make(chan struct{}, op.Concurrency)`.
- `Read`, `ReadUserTuple`, `ReadUsersetTuples` and `ReadStartingWithUser` each call `bound()`, which uses `waitForLimiter` to block on sending to the channel, respecting context cancellation. They release the slot with `defer b.done()`.
- `ReadPage` is not overridden, so it is not bounded.
- Waits longer than 1ms are recorded in the `datastore_bounded_read_delay_ms` histogram.
- The same `bound()` also does optional datastore throttling. After `threshold` reads it sleeps `throttleTime` per read. This is controlled by the feature flag `ExperimentalDatastoreThrottling` plus `--check-datastore-throttle-threshold` and `--check-datastore-throttle-duration`.

**Wiring for Check:**
- **v1:** `Server.Check` passes `commands.WithCheckCommandMaxConcurrentReads(s.maxConcurrentReadsForCheck)` (`pkg/server/check.go`). `CheckQuery.Execute` then builds `NewRequestStorageWrapperWithCache` with `Operation{Method: apimethod.Check, Concurrency: c.maxConcurrentReads, ...}`, which calls `NewBoundedTupleReader`.
  - A new bounded reader, and so a new semaphore, is created for each Check request. The limit is per Check, not global.
  - The iterator cache (`CachedDatastore`) and the shared-iterator wrapper sit outside the bounded reader. Cache hits therefore do not take a slot. `CombinedTupleReader` answers contextual tuples without a datastore read.
- **v2 (weighted graph):** `Server.v2Check` passes `WithCheckQueryV2MaxConcurrentReads(s.maxConcurrentReadsForCheck)`. `CheckQueryV2.Execute` (`pkg/server/commands/check.go`) wraps the datastore in `NewBoundedTupleReader(q.datastore, &q.datastoreOp)`.

**Operator configuration:**

| Source | Name |
|---|---|
| CLI flag | `--max-concurrent-reads-for-check` (`cmd/run/run.go`) |
| Config key | `maxConcurrentReadsForCheck` (`cmd/run/flags.go`, `.config-schema.json`) |
| Environment variable | `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK` (also `OPENFGA_MAXCONCURRENTREADSFORCHECK`) |
| Library option | `server.WithMaxConcurrentReadsForCheck(...)` (`pkg/server/server.go`), set from `config.MaxConcurrentReadsForCheck` in `cmd/run/run.go` |

The default is `math.MaxUint32`, which is effectively unbounded (`DefaultMaxConcurrentReadsForCheck` in `pkg/server/config/config.go`).

**Limits:**
- **Slots are held only until the iterator is returned.** The slot is released when the read method returns (`defer b.done()`), not when the caller finishes iterating. The setting bounds concurrent query starts, not open cursors.
- **BatchCheck does not use this setting.** `BatchCheckQuery` builds its per-item `NewCheckCommand` without `WithCheckCommandMaxConcurrentReads` (`pkg/server/commands/batch_check_command.go`), so each item gets the default `MaxUint32`.
- **A value of 0 is not rejected.** `Config.Verify` rejects 0 for `maxConcurrentReadsForListUsers` but not for Check. With 0 the channel is unbuffered, so reads would appear to block until the context ends. I have not confirmed this at runtime.
- **A separate concurrency knob.** Resolution fan-out is limited by `resolveNodeBreadthLimit`, which goes to `LocalChecker` through `WithResolveNodeBreadthLimit` and to v2 through `WithCheckQueryV2ConcurrencyLimit`. That limit caps concurrent goroutines and subproblems, not datastore reads directly.
