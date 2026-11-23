# Answers

## Q1

**Two Check engines.** `Server.Check` (`pkg/server/check.go`) first tests the per-store feature flag `ExperimentalWeightedGraphCheck` (`"weighted_graph_check"`, `pkg/server/config/config.go`). If that flag is on, the request goes to `Server.v2Check`, which runs `commands.CheckQueryV2` → `check.Resolver` (`internal/check`). That path does not use the resolver chain below. It falls back to the v1 path on any error except timeouts, cancellation and throttling (`isV2TerminalError`). Everything below describes the default v1 path.

**Where the chain is built.**
- `Server.getCheckResolverBuilder(storeID)` (`pkg/server/check.go`) calls `graph.NewOrderedCheckResolvers(...)` with these options:
  - `WithLocalCheckerOpts`
  - `WithLocalShadowCheckerOpts`
  - `WithShadowResolverEnabled`
  - `WithShadowResolverOpts`
  - `WithCachedCheckResolverOpts`
  - `WithDispatchThrottlingCheckResolverOpts`
- The cache and throttling options come from `Server.getCheckResolverOptions` (`pkg/server/server.go`).
- `CheckResolverOrderedBuilder.Build()` (`internal/graph/builder.go`) appends the resolvers in a fixed order, whatever order the options were given in. It then links each resolver to the next with `SetDelegate`. The last resolver's delegate is set back to the first one, so the chain is circular. `Build` returns the first resolver plus a closer.
- A new chain is built for every v1 Check call (`pkg/server/check.go`, `s.getCheckResolverBuilder(storeID).Build()`).

**Order, and what decides whether each resolver is present:**

| # | Type (file) | Present when | What it does before tuples are read |
|---|---|---|---|
| 1 | `CachedCheckResolver` (`internal/graph/cached_resolver.go`) | `cacheSettings.ShouldCacheCheckQueries()`, which is `CheckCacheLimit > 0 && CheckQueryCacheEnabled` (`pkg/server/config/cache_settings.go`). Configured with `checkQueryCache.enabled` and `checkCache.limit`. | Looks up `storage.CheckCacheKey(...)`. On a valid hit it returns the cached response. With `HIGHER_CONSISTENCY` it skips the lookup. |
| 2 | `DispatchThrottlingCheckResolver` (`internal/graph/dispatch_throttling_check_resolver.go`) | `s.checkDispatchThrottlingEnabled`, set from `checkDispatchThrottling.enabled` / `--check-dispatch-throttling-enabled` (`cmd/run/run.go`, `server.WithDispatchThrottlingCheckResolverEnabled`). | If `threshold.ShouldThrottle` is true for the request's `DispatchCounter`, it marks the request `DispatchThrottled` and calls `throttler.Throttle(ctx)`. Then it delegates. |
| 3 | Either `ShadowResolver` (`internal/graph/shadow_resolver.go`) or `LocalChecker` (`internal/graph/check.go`) | `ShadowResolver` when the per-store flag `ExperimentalShadowCheck` (`"shadow_check"`) is on, otherwise a plain `LocalChecker`. | `ShadowResolver` wraps two `LocalChecker`s. It returns the `main` result and runs the `shadow` one asynchronously to compare them. Only `main` gets the delegate (`ShadowResolver.SetDelegate`). |

**Where tuples are read.** `LocalChecker.ResolveCheck` does several steps before any tuple read:
1. It checks the depth limit (`ErrResolutionDepthExceeded`).
2. It checks for a cycle.
3. It handles self-defining tuples.
4. It checks that a typesystem and a datastore are in the context, and that a path exists (`PathExists`).

After that, it reads tuples through the reader taken from the context (for example, `checkDirectUserTuple` → `ReadUserTuple`). Sub-problems are dispatched to `c.delegate`, which is the head of the chain. Each sub-check therefore goes through the cache and the throttler again.

**Storage wrappers.** These are not resolvers, but they sit between the resolvers and the database. `CheckQuery.Execute` (`pkg/server/commands/check_command.go`) runs these steps before `ResolveCheck`:
1. It validates the request.
2. It gets the cache invalidation time.
3. It builds a `ResolveCheckRequest`.
4. It wraps the datastore with `storagewrappers.NewRequestStorageWrapperWithCache` (`pkg/storage/storagewrappers/request.go`) and stores the result in the context.

Reads then pass through these wrappers, outermost first:
1. `CombinedTupleReader` (contextual tuples)
2. `sharediterator.IteratorDatastore` (built by `NewSharedIteratorDatastore`), if `SharedIteratorEnabled`
3. `CachedDatastore`, if `ShouldCacheCheckIterators()`
4. `BoundedTupleReader`
5. The server datastore, which `NewServerWithOpts` in `pkg/server/server.go` has wrapped in `cachedOpenFGADatastore` and, unless context propagation is enabled, `ContextTracerWrapper`.

## Q2

**The interface.** `ReadUserTuple` is declared on `storage.RelationshipTupleReader` in `pkg/storage/storage.go`. Its signature is:

```go
ReadUserTuple(ctx context.Context, store string, filter ReadUserTupleFilter, options ReadUserTupleOptions) (*openfgav1.Tuple, error)
```

It must return `ErrNotFound` when no tuple matches. Two interfaces inherit it: `TupleBackend` embeds `RelationshipTupleReader`, and `OpenFGADatastore` includes it as well.

**Types that define the method explicitly (production code):**
- Backends:
  - `memory.MemoryBackend` (`pkg/storage/memory/memory.go`)
  - `postgres.Datastore` (`pkg/storage/postgres/postgres.go`)
  - `mysql.Datastore` (`pkg/storage/mysql/mysql.go`)
  - `sqlite.Datastore` (`pkg/storage/sqlite/sqlite.go`)
- Wrappers:
  - `storagewrappers.BoundedTupleReader` (`bounded_datastore.go`): concurrency bound and throttling.
  - `storagewrappers.CombinedTupleReader` (`combinedtuplereader.go`): returns a matching contextual tuple first.
  - `storagewrappers.CachedTupleReader` (`cached_reader.go`): plain delegation.
  - `storagewrappers.ContextTracerWrapper` (`context.go`): swaps in a detached query context.

**Types that get the method by embedding the interface:**
- `RequestStorageWrapper` (`request.go`)
- `CachedDatastore` (`cached_datastore.go`)
- `sharediterator.IteratorDatastore` (`shared_iterator_datastore.go`)
- `cachedOpenFGADatastore` (`model_caching.go`)

**Mocks and test helpers:** `MockTupleBackend`, `MockRelationshipTupleReader` and `MockOpenFGADatastore` (`internal/mocks/mock_storage.go`, generated by mockgen), and the hand-written `slowDataStorage` (`internal/mocks/mock_slow_storage.go`).

**Production callers outside the wrappers:**
- `LocalChecker.checkDirectUserTuple` (`internal/graph/check.go`). This is the v1 Check path. It is reached from `LocalChecker.checkDirect` when `shouldCheckDirectTuple` is true.
- `Resolver.specificType` (`internal/check/check.go`). This is the v2 weighted-graph Check path, used only if no matching contextual tuple was found.
- The wrapper methods listed above, each of which forwards to its inner reader.

No other production package calls `ReadUserTuple`. It does not appear in `cmd/`, ListObjects, ListUsers or the write path.

**If the signature changed, these would have to change too:**
- The interface in `pkg/storage/storage.go`. Note that `ReadUserTupleFilter` is an alias of `ReadFilter`, so changing the filter type would also change `Read` and `ReadPage`. `ReadUserTupleOptions` has its own type.
- The four backend implementations.
- The four wrappers that define the method. The embedding wrappers would pick up the new signature automatically.
- The two callers in `internal/graph/check.go` and `internal/check/check.go`.
- The mocks: regenerate `mock_storage.go` (`make generate-mocks`) and edit `mock_slow_storage.go` by hand.
- The tests that call the method or set mock expectations on it:
  - the shared datastore suite in `pkg/storage/test/tuples.go`
  - `pkg/storage/{postgres,mysql,sqlite}/*_test.go`
  - the `storagewrappers` tests
  - `internal/graph/check_test.go`
  - `internal/check/check_test.go`
  - `pkg/server/server_test.go`
- Possibly the operation name constant `storage.OperationReadUserTuple` (`pkg/storage/keys.go`), which `BoundedTupleReader` uses as a metric label. A signature change alone would not force a change here.

## Q3

**The mechanism compares timestamps.** Writes do not invalidate the Check query cache. The write path (`pkg/server/write.go`, `pkg/server/commands/write.go`) contains no cache code. Instead, a cached entry counts as stale if it was stored before the last write that the server knows about.

**How a cached result is checked.**
- `CachedCheckResolver.ResolveCheck` (`internal/graph/cached_resolver.go`) stores `CheckResponseCacheEntry{LastModified: time.Now(), ...}` with TTL `storage.JitteredTTL(cacheTTL, jitterPercentage)`.
- On a lookup it accepts a hit only if `res.LastModified.After(req.LastCacheInvalidationTime)`. Otherwise it increments `check_cache_invalid_hit_count` and recomputes the result.
- Results that report a cycle are never cached.
- The v2 path does the same comparison in `Resolver.isCached` (`internal/check/check.go`).

**Where the invalidation time comes from.**
- `CheckQuery.Execute` (`pkg/server/commands/check_command.go`) and `Server.v2Check` (`pkg/server/check.go`) call `CacheController.DetermineInvalidationTime(ctx, storeID)`. With `HIGHER_CONSISTENCY` they skip the call and the cache altogether.
- The real implementation is `InMemoryCacheController` (`internal/cachecontroller/cache_controller.go`). It is created in `shared.NewSharedDatastoreResources` (`internal/shared/shared.go`) only if `ShouldCreateCacheController()` is true, which requires a cache to exist and `cacheController.enabled`.
- `DetermineInvalidationTime` reads the per-store changelog entry `storage.ChangelogCacheKey(storeID)` and returns its `LastModified`, which is the time of the store's last write.
- If the entry is missing, it returns the zero time, which makes every cached entry valid, and starts a refresh in the background.
- If the entry was last checked more than `minInvalidationInterval` ago (`cacheController.ttl`), it also starts a background refresh.
- `InvalidateIfNeeded` allows at most one in-flight refresh per store (`inflightInvalidations`).
- The refresh, `findChangesAndInvalidateIfNecessary`, works as follows:
  1. It calls `ReadChanges` in descending order, limited to one page (`storage.DefaultPageSize` = 50) and a 1-second timeout.
  2. It stores `ChangelogCacheEntry{LastModified: newest change timestamp, LastChecked: now}` with TTL `queryCacheTTL`.
  3. If there are newer changes, it also writes `invalid_entity` markers for the iterator cache. A full invalidation uses `InvalidIteratorCacheKey`. A partial one uses `InvalidIteratorByObjectRelationCacheKey` and `InvalidIteratorByUserObjectTypeCacheKey`, and is limited to changes within the iterator TTL.
  4. The iterator cache honours these markers through `isInvalidAt` / `findInCache` (`pkg/storage/storagewrappers/cached_datastore.go`).
- Refreshes are triggered by Check calls, rate-limited by `cacheController.ttl`. They are also triggered on every ListObjects call when the ListObjects iterator cache is enabled (`commands/list_objects.go`, `InvalidateIfNeeded`). `docs/caching.md` ("Cache Controller") describes the same design.

**Limits:**
- **Off by default.** Without the controller, `NoopCacheController` returns the zero time. A cached result is then used until its TTL expires (`checkQueryCache.ttl`, default 10s, plus jitter), whatever writes happen in between. The defaults are in `pkg/server/config/config.go`.
- **Eventually consistent.** A write becomes visible only after a later Check (or ListObjects) starts a refresh and the refresh finishes. The Check that starts it still uses the old time. As a result, staleness is bounded roughly by `cacheController.ttl` plus the refresh latency, and can be longer on stores with little traffic. `docs/caching.md` calls this "First Check Staleness".
- **All or nothing for query results.** Any write to the store invalidates every cached Check result for that store, because the comparison is against a single store-wide timestamp.
- **Per process.** The cache and controller are in memory, so replicas do not share invalidation state.
- **Missing changelog entry.** If the changelog entry has expired (its TTL is `queryCacheTTL`), the next lookup again returns the zero time until a refresh finishes.
- **Refresh failures.**
  - If the 1-second `ReadChanges` timeout fires, the refresh returns without updating anything.
  - If `ReadChanges` returns any other error, the controller invalidates only the iterator cache and does not set the changelog entry. This includes the case where `ReadChanges` errors because there are no changes, as the comment on `findChangesDescending` says.
- **Two different clocks.** Entry times come from the server's `time.Now()`, but the comparison time comes from the datastore's changelog timestamps. Clock skew between the two could misclassify entries. I infer this from the code; it is not documented.
- **Iterator cache only sees the latest 50 changes.** Partial iterator invalidation considers only the newest 50 changes. When all 50 fall within the iterator TTL, it falls back to a full invalidation.

## Q4

**Where the checks run when a model is written.**
- `Server.WriteAuthorizationModel` (`pkg/server/authorization_models.go`) calls `WriteAuthorizationModelCommand.Execute` (`pkg/server/commands/write_authzmodel.go`).
- `Execute` calls `typesystem.NewAndValidate(ctx, model)` (`pkg/typesystem/typesystem.go`).
- Any error is returned as `serverErrors.InvalidAuthorizationModelInput`, and the model is not stored.
- `NewAndValidate` calls `validateRelation` for every relation in every type, in sorted order. `validateRelation` runs `isUsersetRewriteValid`, then `validateTypeRestrictions`, then `hasEntrypoints`, then `HasCycle`.

**Constraints on the tupleset relation `Y` in `X from Y`, defined on type `T`:**

1. **`Y` must be a relation on the same type `T`.** `isUsersetRewriteValid`, case `*openfgav1.Userset_TupleToUserset`, calls `t.GetRelation(objectType, tupleset)`. If that fails it returns `RelationUndefinedError{... ErrRelationUndefined}`.
2. **`Y` must be a pure direct relation.** Its rewrite must be exactly `this`, meaning only `[types]`, with no `or`/`and`/`but not`, computed usersets or nested TTUs. The same case in `isUsersetRewriteValid` compares the rewrite against `&openfgav1.Userset_This{}` with `reflect.TypeOf`. On failure the error is "the 'T#Y' relation is referenced in at least one tupleset and thus must be a direct relation".
3. **`Y`'s type restrictions must be plain object types.** `type#relation` usersets and `type:*` wildcards are not allowed.
   - `validateTypeRestrictions` checks `related.GetRelationOrWildcard() != nil && t.IsTuplesetRelation(objectType, relationName)` and returns `InvalidRelationTypeError`.
   - `IsTuplesetRelation` looks through `ttuRelations`, which `New` fills in through `flattenUserset`. So a TTU nested inside a union, intersection or exclusion also marks `Y` as a tupleset relation.
   - The same rule applies to every assignable relation: each restricted type, and each referenced relation, must exist. Conditions on the restrictions must be defined in the model.
   - The rule doc comment on `validateTypeRestrictions` (item 4) states this constraint.
   - `relationInvolves` repeats the assumption ("we disallow tupleset relations of the form objectType:id#relation"), but it is a later helper, not a write-time check.
4. **The computed relation `X` must exist on at least one of `Y`'s directly related types.** This is checked in `isUsersetRewriteValid` for schema 1.1/1.2. Otherwise the error is "`X` does not appear as a relation in any of the directly related user types". The schema 1.0 branch only requires `X` to exist somewhere in the model. However, `NewAndValidate` already rejects unsupported schema versions (`IsSchemaVersionSupported` accepts only 1.1 and 1.2), so I believe that branch cannot be reached when a model is written.
5. **Entrypoints.** `hasEntrypoints`, case `Userset_TupleToUserset`, requires that some directly related type of `Y` that defines `X` has an entrypoint. Otherwise `validateRelation` returns `InvalidRelationError{Cause: ErrNoEntrypoints}` (or `ErrNoEntryPointsLoop`). If `Y` is missing there, the error is "undefined type definition for 'T#Y'".

`NewAndValidate` does not check whether `Y`'s type restrictions carry conditions. I found nothing that forbids conditional tupleset tuples at write time.

## Q5

**The mechanism.** The limit is a per-request semaphore, implemented by `storagewrappers.BoundedTupleReader` (`pkg/storage/storagewrappers/bounded_datastore.go`).
- `NewBoundedTupleReader` creates `limiter: make(chan struct{}, op.Concurrency)`.
- `Read`, `ReadUserTuple`, `ReadUsersetTuples` and `ReadStartingWithUser` each call `bound()` first. `bound()` calls `waitForLimiter`, which sends into the channel and gives up if the context is done.
- When the call returns, `defer b.done()` frees the slot.
- If the wait was longer than 1 ms, it is recorded in the histogram `datastore_bounded_read_delay_ms` and in the span attribute `datastore_time_waiting`.
- The same wrapper counts queries and items for the request metadata. It also applies the optional datastore throttle: after `ThrottleThreshold` reads it sleeps `ThrottleDuration` before each further read. The throttle is enabled by the per-store `"datastore_throttling"` experimental flag and `checkDatastoreThrottle.threshold`/`.duration`. Both default to 0, which means off.

**How the limit reaches each Check.**
- v1: `Server.Check` passes `commands.WithCheckCommandMaxConcurrentReads(s.maxConcurrentReadsForCheck)`. `CheckQuery.Execute` then builds a new `BoundedTupleReader` for each request through `NewRequestStorageWrapperWithCache` (`Operation{Method: apimethod.Check, Concurrency: c.maxConcurrentReads, ...}`, `pkg/storage/storagewrappers/request.go`).
- v2: `Server.v2Check` passes `commands.WithCheckQueryV2MaxConcurrentReads(...)`, and `CheckQueryV2.Execute` wraps the datastore with `NewBoundedTupleReader` (`pkg/server/commands/check.go`).
- Because the wrapper is per request, the limit applies to each Check, not to the whole server.

**Operator configuration.**

| Setting | Value |
|---|---|
| Config file key | `maxConcurrentReadsForCheck` (`Config.MaxConcurrentReadsForCheck`, `pkg/server/config/config.go`; `.config-schema.json`) |
| Flag | `--max-concurrent-reads-for-check` (`cmd/run/run.go`) |
| Environment variable | `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK`, or the alias `OPENFGA_MAXCONCURRENTREADSFORCHECK` (`cmd/run/flags.go`) |
| Default | `DefaultMaxConcurrentReadsForCheck = math.MaxUint32` (4294967295), effectively unbounded |
| Library option | `server.WithMaxConcurrentReadsForCheck(n)` (`pkg/server/server.go`) |

The doc comment on `WithMaxConcurrentReadsForCheck` gives sizing guidance. The limit should be chosen from `Datastore.MaxOpenConns`, the expected RPS and the replica count. For example, with 100 connections, one replica and 1 RPS, set it to 100.

**Limits and caveats (from the code):**
- **Only the call is bounded.** The slot is freed as soon as the read call returns an iterator, not when the iterator has been consumed. Iterators that fetch more pages lazily are therefore not covered. `ReadPage` and the other `OpenFGADatastore` methods are passed through by embedding and are not bounded at all.
- **BatchCheck ignores the setting.** `BatchCheckQuery` (`pkg/server/commands/batch_check_command.go`) builds each `NewCheckCommand` without `WithCheckCommandMaxConcurrentReads`, so every check in a batch keeps the MaxUint32 default. Parallelism in a batch is governed instead by `maxConcurrentChecksPerBatchCheck` (`WithBatchCheckMaxConcurrentChecks`).
- **Zero is not rejected.** `VerifyServerSettings` rejects `maxConcurrentReadsForListUsers == 0` but has no such check for Check. With 0, the channel would be unbuffered, so I expect every bounded read to block until the context is cancelled. I inferred this from the code and have not tested it.
- **A related but separate knob.** `resolveNodeBreadthLimit` (`--resolve-node-breadth-limit`, default 10) caps how many sub-problems at one level are evaluated concurrently. It is not a cap on datastore reads.
