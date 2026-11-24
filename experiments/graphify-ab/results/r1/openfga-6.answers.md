# Answers

## Q1

**Short answer.** In the default (v1) Check path, a request goes through up to three resolvers, in this order. The chain is a circular linked list, so every sub-problem that `LocalChecker` dispatches goes back to the head of the chain.

| # | Resolver type | Present when |
|---|---|---|
| 1 | `graph.CachedCheckResolver` (`internal/graph/cached_resolver.go`) | `CacheSettings.ShouldCacheCheckQueries()` is true, which means `CheckQueryCacheEnabled && CheckCacheLimit > 0` (`pkg/server/config/cache_settings.go`). The flags are `--check-query-cache-enabled` and `--check-cache-limit`. |
| 2 | `graph.DispatchThrottlingCheckResolver` (`internal/graph/dispatch_throttling_check_resolver.go`) | `Server.checkDispatchThrottlingEnabled` is true. The flag is `--check-dispatch-throttling-enabled` / `OPENFGA_CHECK_DISPATCH_THROTTLING_ENABLED` (`cmd/run/flags.go`). |
| 3 | `graph.ShadowResolver` wrapping two `graph.LocalChecker`s (main and shadow), **or** a plain `graph.LocalChecker` (`internal/graph/check.go`) | The `ShadowResolver` is used when the per-store feature flag `serverconfig.ExperimentalShadowCheck` (`"shadow_check"`) is on. Otherwise the plain `LocalChecker` is used. The terminal checker is always present. |

**The code that builds the chain:**
- `Server.Check` (`pkg/server/check.go`) calls `s.getCheckResolverBuilder(storeID).Build()`.
- `Server.getCheckResolverBuilder` (`pkg/server/check.go`) calls `graph.NewOrderedCheckResolvers(...)` with these options: `WithLocalCheckerOpts`, `WithLocalShadowCheckerOpts`, `WithShadowResolverEnabled(featureFlag ExperimentalShadowCheck)`, `WithShadowResolverOpts`, `WithCachedCheckResolverOpts(s.cacheSettings.ShouldCacheCheckQueries(), …)` and `WithDispatchThrottlingCheckResolverOpts(s.checkDispatchThrottlingEnabled, …)`.
- The cache and throttling options themselves come from `Server.getCheckResolverOptions` (`pkg/server/server.go`). That function sets the shared `CheckCache`, the TTL and jitter, the default and max dispatch thresholds, and a constant-rate throttler.

**How `CheckResolverOrderedBuilder.Build()` (`internal/graph/builder.go`) fixes the order:**
- It appends the resolvers in a fixed order regardless of option order: cached, then dispatch-throttling, then shadow or local.
- It then calls `SetDelegate` so that each resolver points to the next one, and the last one points back to `resolvers[0]`. It returns the head and a closer.
- `ShadowResolver.SetDelegate` sets the delegate only on `main` (`internal/graph/shadow_resolver.go`). The shadow checker runs asynchronously, and only its result is compared with the main result.

**What each resolver does before tuples are read:**
- **`CachedCheckResolver.ResolveCheck`** looks up `storage.CheckCacheKey(store, object, relation, user, invariantCacheKey)`.
  - It skips the lookup when the consistency is `HIGHER_CONSISTENCY`.
  - On a valid hit it returns the cached response. Otherwise it delegates the request and caches the result, unless a cycle was detected.
- **`DispatchThrottlingCheckResolver.ResolveCheck`** throttles the request when `threshold.ShouldThrottle(...)` fires on the request's `DispatchCounter`. It then delegates.
- **`LocalChecker.ResolveCheck`** does the actual work:
  - It checks the depth limit, cycles, self-defining tuples and `typesys.PathExists`.
  - It then runs `CheckRewrite` (for example `checkDirectUserTuple`, `checkDirectUsersetTuples` and `checkTTU`).
  - These read tuples from the `RelationshipTupleReader` stored in the context, using `storage.RelationshipTupleReaderFromContext`.
  - `LocalChecker.dispatch` sends sub-problems to `c.delegate`, which is the head of the chain again.

**Storage wrappers below the resolvers.** Strictly speaking these are storage decorators, not resolvers. `CheckQuery.Execute` (`pkg/server/commands/check_command.go`) puts a reader into the context. It is built by `storagewrappers.NewRequestStorageWrapperWithCache` (`pkg/storage/storagewrappers/request.go`). Reads pass through these layers, outermost first:
1. `CombinedTupleReader`, which serves contextual tuples.
2. `sharediterator.IteratorDatastore` (built by `NewSharedIteratorDatastore`), only if `SharedIteratorEnabled`.
3. `CachedDatastore`, the iterator cache, only if `ShouldCacheCheckIterators()`.
4. `BoundedTupleReader`, which applies the concurrency limit and throttling.
5. The server datastore. `NewServerWithOpts` (`pkg/server/server.go`) wraps it in `cachedOpenFGADatastore` (from `NewCachedOpenFGADatastore`) and, unless context propagation is enabled, in `ContextTracerWrapper`.

**The v2 path bypasses this chain.** If the per-store flag `ExperimentalWeightedGraphCheck` (`"weighted_graph_check"`) is on, `Server.Check` first tries `s.v2Check`. That path uses `commands.CheckQueryV2` and `internal/check.Resolver`, and it does not use the `CheckResolverOrderedBuilder` chain at all. On a non-terminal error it falls back to the v1 chain described above (`isV2TerminalError`, `pkg/server/check.go`).

## Q2

**Where the method is declared.** `ReadUserTuple` is declared on the `storage.RelationshipTupleReader` interface (`pkg/storage/storage.go`). That interface is embedded in `TupleBackend` and therefore in `OpenFGADatastore`.
- Signature: `ReadUserTuple(ctx, store string, filter ReadUserTupleFilter, options ReadUserTupleOptions) (*openfgav1.Tuple, error)`.
- Contract: it returns an exact match, or `storage.ErrNotFound`.
- `ReadUserTupleFilter` is an alias for `ReadFilter`. `ReadUserTupleOptions` holds a `Consistency`.

**Types that implement it with their own method.** These are production code:

| Type | File | Notes |
|---|---|---|
| `postgres.Datastore` | `pkg/storage/postgres/postgres.go` | SQL backend |
| `mysql.Datastore` | `pkg/storage/mysql/mysql.go` | SQL backend |
| `sqlite.Datastore` | `pkg/storage/sqlite/sqlite.go` | SQL backend |
| `memory.MemoryBackend` | `pkg/storage/memory/memory.go` | in-memory backend |
| `storagewrappers.BoundedTupleReader` | `pkg/storage/storagewrappers/bounded_datastore.go` | Takes a limiter slot (`bound(ctx, storage.OperationReadUserTuple)`), delegates, and counts the item. |
| `storagewrappers.CombinedTupleReader` | `pkg/storage/storagewrappers/combinedtuplereader.go` | Checks contextual tuples first, then delegates. |
| `storagewrappers.CachedTupleReader` | `pkg/storage/storagewrappers/cached_reader.go` | Plain delegation, with no caching. Used by the v2 check. |
| `storagewrappers.ContextTracerWrapper` | `pkg/storage/storagewrappers/context.go` | Swaps in the query context, then delegates. |

**Types that satisfy it only by embedding.** These have no method of their own:
- `RequestStorageWrapper` (`request.go`)
- `CachedDatastore` (`cached_datastore.go`)
- `sharediterator.IteratorDatastore` (`shared_iterator_datastore.go`)
- `cachedOpenFGADatastore` (`model_caching.go`)

**Test and mock implementations:**
- `mocks.MockRelationshipTupleReader` and `mocks.MockOpenFGADatastore` in `internal/mocks/mock_storage.go`. This file is generated by `mockgen -source pkg/storage/storage.go`.
- `slowDataStorage` in `internal/mocks/mock_slow_storage.go`.

**Production callers.** These issue a lookup themselves:
1. `graph.LocalChecker.checkDirectUserTuple` in `internal/graph/check.go`. This is the v1 Check direct-tuple lookup. It reads through `storage.RelationshipTupleReaderFromContext`.
2. `check.Resolver.specificType` in `internal/check/check.go`. This is the v2 (weighted-graph) Check. It is called only if no matching contextual tuple was found.

The wrapper methods listed above also call `ReadUserTuple` on the reader they wrap. Apart from these, the only callers are the datastore conformance suite (`pkg/storage/test/tuples.go`), `_test.go` files and mocks. No ListObjects, ListUsers, Read or Write command calls it directly.

**What a signature change would require:**
- `RelationshipTupleReader` in `pkg/storage/storage.go`. If the options or filter types change, also `ReadUserTupleFilter`/`ReadFilter` and `ReadUserTupleOptions`.
- All four backends: postgres, mysql, sqlite and memory.
- The four wrappers that override the method: `BoundedTupleReader`, `CombinedTupleReader`, `CachedTupleReader` and `ContextTracerWrapper`. The embedding wrappers pick up the change automatically.
- Both production call sites: `checkDirectUserTuple` and `Resolver.specificType`.
- Regenerating `internal/mocks/mock_storage.go` with mockgen, and updating `internal/mocks/mock_slow_storage.go`.
- The conformance tests in `pkg/storage/test/tuples.go` and the many `_test.go` files that call the method or mock it with `EXPECT().ReadUserTuple(...)`. Examples are `internal/graph/check_test.go`, `internal/check/check_test.go`, `pkg/server/server_test.go`, the storagewrappers tests and the backend tests.
- Any out-of-tree datastore that implements `OpenFGADatastore`. This last point is an inference: the interface is public under `pkg/`.

## Q3

**Short answer.** Staleness is decided by timestamps. Nothing is invalidated when the write happens. Instead, Check requests poll the store's changelog through a cache controller, and a cached result is trusted only if it was cached *after* the last known write to the store.

**Where it lives:**
- `internal/cachecontroller/cache_controller.go`: the `CacheController` interface, with `InMemoryCacheController` and `NoopCacheController`.
- It is created in `shared.NewSharedDatastoreResources` (`internal/shared/shared.go`), only when `CacheSettings.ShouldCreateCacheController()` is true. That requires a cache to exist and `--cache-controller-enabled`, which defaults to false.
- Otherwise the controller is a `NoopCacheController`. Its `DetermineInvalidationTime` always returns the zero time, so cached results expire only by TTL.

**How a Check uses it:**
1. **Getting the invalidation time.** `CheckQuery.Execute` (`pkg/server/commands/check_command.go`) calls `CacheController.DetermineInvalidationTime(ctx, storeID)`, unless the consistency is `HIGHER_CONSISTENCY`. It puts the result in `ResolveCheckRequest.LastCacheInvalidationTime`. `Server.v2Check` does the same for the v2 path.
2. **Reading the changelog entry.** `InMemoryCacheController.DetermineInvalidationTime` reads `storage.ChangelogCacheKey(storeID)`, a `ChangelogCacheEntry{LastModified, LastChecked}`.
   - On a miss, it starts an asynchronous `InvalidateIfNeeded` and returns the zero time, so existing cache entries are still used.
   - On a hit, it starts `InvalidateIfNeeded` only if `time.Since(entry.LastChecked) > minInvalidationInterval` (`--cache-controller-ttl`, default 10s). It then returns `entry.LastModified`.
3. **Polling the changelog.** `InvalidateIfNeeded` allows one in-flight refresh per store (`inflightInvalidations` sync.Map). It runs `findChangesAndInvalidateIfNecessary` in a goroutine, which does the following:
   - It calls `ds.ReadChanges` in descending order, with a 1s timeout.
   - It stores `ChangelogCacheEntry{LastModified: newest change timestamp, LastChecked: now}` with the query-cache TTL.
   - If there are new changes, it writes "invalid" markers for the iterator cache. If every change in the page is newer than `now - iteratorCacheTTL`, it marks the whole store with `InvalidIteratorCacheKey`. Otherwise it marks individual entries with `InvalidIteratorByObjectRelationCacheKey` and `InvalidIteratorByUserObjectTypeCacheKey`.
   - If `ReadChanges` errors, it invalidates the store's whole iterator cache.
4. **Comparing on lookup.** For cached Check results, `CachedCheckResolver.ResolveCheck` (`internal/graph/cached_resolver.go`) treats an entry as valid only if `CheckResponseCacheEntry.LastModified.After(req.LastCacheInvalidationTime)`. Here `LastModified` is `time.Now()` when the entry was stored. Invalid hits are counted (`check_cache_invalid_hit_count`) and recomputed.
5. **Iterator cache.** Cached tuple iterators are checked separately against the invalid markers by `isInvalidAt` and `findInCache` (`pkg/storage/storagewrappers/cached_datastore.go`).
6. **ListObjects.** When the ListObjects iterator cache is on, ListObjects calls `InvalidateIfNeeded` directly on every request (`pkg/server/commands/list_objects.go`).

**Limits:**
- **Eventually consistent.** The first Check after a miss, and every Check within `cache-controller-ttl` of the last poll, can see stale results. The code comments say so (`DetermineInvalidationTime`, and the `InMemoryCacheController` doc comment). The flag help for `--check-query-cache-enabled` (`cmd/run/run.go`) says the cache "turns Check and ListObjects into eventually consistent APIs".
- **Store-level granularity for Check results.** Any write to the store invalidates every cached Check result for that store that is older than the write. Only the iterator cache gets the finer, per-object-relation and per-user-object-type invalidation.
- **Process-local.** The cache and controller are in-memory and belong to one instance. Each replica polls the changelog on its own, and writes do not push invalidations anywhere.
- **One page of changes.** Only one page (`storage.DefaultPageSize`) of the changelog is read. A changelog query that times out leaves the state unchanged until the next attempt.
- **The cache is bypassed entirely** when the consistency is `HIGHER_CONSISTENCY`.
- **Possible clock comparison (unverified).** Validity compares the server's `time.Now()` at caching time with the changelog timestamps stored by the datastore. Clock skew between server and database could matter. I did not verify how each backend stamps changelog rows.
- **Default TTL bounds.** Without the controller, staleness is bounded only by the query-cache TTL, `--check-query-cache-ttl`, which defaults to 10s (`pkg/server/config/config.go`).

## Q4

**When the model is written.** `WriteAuthorizationModelCommand.Execute` (`pkg/server/commands/write_authzmodel.go`) calls `typesystem.NewAndValidate(ctx, model)`. On error it returns `InvalidAuthorizationModelInput`. `NewAndValidate` (`pkg/typesystem/typesystem.go`) calls `TypeSystem.validateRelation` for every relation. The tupleset relation `Y` in `X from Y` is constrained in these places:

| Constraint | Where enforced |
|---|---|
| `Y` must be defined on the same object type. Otherwise the error is `RelationUndefinedError`. | `TypeSystem.isUsersetRewriteValid`, case `*openfgav1.Userset_TupleToUserset` |
| `Y` must be a pure direct relation: its rewrite must be `Userset_This`, with no computed userset, union, TTU and so on. The error is "the '%s#%s' relation is referenced in at least one tupleset and thus must be a direct relation". | `isUsersetRewriteValid` (TTU case) |
| For schema 1.1/1.2, `X` must be a relation on at least one of `Y`'s directly related user types. For schema 1.0, `X` must exist on some type in the model. | `isUsersetRewriteValid` (TTU case, `IsSchemaVersionSupported` branch) |
| `Y`'s type restrictions cannot be usersets (`type#rel`) or wildcards (`type:*`); they must be plain object types. The check uses `IsTuplesetRelation` and returns `InvalidRelationTypeError`. | `TypeSystem.validateTypeRestrictions`, rule 4 |
| `Y` is assignable, so, as for any assignable relation, it must have at least one type restriction, and every referenced type must exist. | `validateTypeRestrictions`, rules 1 and 3 |
| The TTU must yield an entrypoint through at least one of `Y`'s user types that has `X`. Otherwise the error is `ErrNoEntrypoints` or `ErrNoEntryPointsLoop`. | `hasEntrypoints`, TTU case, called from `validateRelation` |
| No cycles. | `TypeSystem.HasCycle`, called from `validateRelation` |

`TypeSystem.relationInvolves` (TTU case) also returns `ErrInvalidModel` if a tupleset relation has a `type#relation` restriction. It is a helper for `RelationInvolvesIntersection` and `RelationInvolvesExclusion`, not part of the `validateRelation` pipeline.

**At tuple write and read time.** This is related but separate from model writes. `validateTuplesetRestrictions` (`internal/validation/validation.go`, called by `ValidateTupleForRead` and `ValidateTupleForWrite`) rejects three kinds of tuples on a tupleset relation:
- tuples whose user is a wildcard;
- tuples whose user is a userset or not a valid `type:id` object;
- tuples whose tupleset relation has a rewrite other than `This`.

## Q5

**Mechanism.** Each Check request builds its own `storagewrappers.BoundedTupleReader` (`pkg/storage/storagewrappers/bounded_datastore.go`):
- In v1, `CheckQuery.Execute` calls `NewRequestStorageWrapperWithCache` with `Operation{Method: Check, Concurrency: c.maxConcurrentReads, …}` (`pkg/storage/storagewrappers/request.go`).
- In v2, `CheckQueryV2.Execute` (`pkg/server/commands/check.go`) calls `NewBoundedTupleReader` directly.

**How the limit works.** `BoundedTupleReader` holds a buffered channel `limiter` of size `op.Concurrency`.
- Every `Read`, `ReadUserTuple`, `ReadUsersetTuples` and `ReadStartingWithUser` call first runs `bound()`. `bound()` blocks in `waitForLimiter` until it can put a token in the channel, or until the context is done.
- `done()` removes the token, deferred, when the datastore call returns.
- Waits longer than a threshold are recorded in a delay histogram.

**Optional throttling in the same wrapper.** `bound()` also counts reads. If `ThrottlingEnabled && threshold > 0 && reads > threshold`, it sleeps `throttleTime` before each further read and marks the request as throttled. `CheckQuery.Execute` turns a deadline-exceeded error on a throttled request into `ThrottledError`.

**Configuration.** The limit flows from the server option into the command:
- The config field is `Config.MaxConcurrentReadsForCheck` (`pkg/server/config/config.go`). Its default is `DefaultMaxConcurrentReadsForCheck = math.MaxUint32`, which effectively means unlimited.
- It can be set with any of these:
  - the flag `--max-concurrent-reads-for-check` (`cmd/run/run.go`);
  - the env vars `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK` / `OPENFGA_MAXCONCURRENTREADSFORCHECK` (`cmd/run/flags.go`);
  - the config-file key `maxConcurrentReadsForCheck` (`.config-schema.json`).
- `cmd/run/run.go` passes it as `server.WithMaxConcurrentReadsForCheck(config.MaxConcurrentReadsForCheck)`. That option sets `Server.maxConcurrentReadsForCheck` (`pkg/server/server.go`; its doc comment explains how to size it against `Datastore.MaxOpenConns`, the replica count and RPS).
- `Server.Check` passes it with `commands.WithCheckCommandMaxConcurrentReads(...)`, and `Server.v2Check` with `WithCheckQueryV2MaxConcurrentReads(...)`.

**Related knobs.** Datastore throttling is turned on per store by the experimental flag `ExperimentalDatastoreThrottling` (`--experimentals`). It is tuned with `--check-datastore-throttle-threshold` and `--check-datastore-throttle-duration` (env `OPENFGA_CHECK_DATASTORE_THROTTLE_THRESHOLD` / `_DURATION`), which feed `server.WithCheckDatabaseThrottle`. Separately, `--resolve-node-breadth-limit` (`WithResolveNodeBreadthLimit`) caps resolver fan-out, not datastore reads directly.

**Limits of the mechanism:**
- **Per request, not global.** The limit applies to one Check request, not the whole server. Total load is still bounded by the datastore connection pool, for example `MaxOpenConns`.
- **Calls, not open iterators.** The token is released when the `Read…` call returns an iterator, not when the iterator is drained. So the limit bounds concurrent datastore *calls*, not open iterators.
- **ListObjects uses its own limit.** When ListObjects runs internal Checks, it passes `maxConcurrentReadsForListObjects` (`pkg/server/commands/list_objects.go`).
- **BatchCheck appears unlimited.** In `BatchCheckQuery` (`pkg/server/commands/batch_check_command.go`), the per-item `NewCheckCommand` call does not pass `WithCheckCommandMaxConcurrentReads`. Each item therefore appears to fall back to the `math.MaxUint32` default. I saw no other place where BatchCheck sets this limit, but I have not traced it exhaustively.
