# Answers

## Q1

**Short answer.** On the default (v1) Check path, the request goes through at most three check resolvers, in this order. A resolver is left out when its setting is off:

1. `graph.CachedCheckResolver` (`internal/graph/cached_resolver.go`). It is added only when `cacheSettings.ShouldCacheCheckQueries()` is true. That requires `checkQueryCache.enabled` and `checkCache.limit > 0` (`pkg/server/config/cache_settings.go`, `ShouldCacheCheckQueries`).
2. `graph.DispatchThrottlingCheckResolver` (`internal/graph/dispatch_throttling_check_resolver.go`). It is added only when `s.checkDispatchThrottlingEnabled` is true, which is set by `checkDispatchThrottling.enabled`.
3. One of the following, which is always present:
   - `graph.ShadowResolver` (`internal/graph/shadow_resolver.go`), which wraps a main `LocalChecker` and a shadow `LocalChecker`. It is used when the per-store feature flag `serverconfig.ExperimentalShadowCheck` (`"shadow_check"`) is on.
   - Otherwise, a plain `graph.LocalChecker` (`internal/graph/check.go`).

Only `LocalChecker` reads tuples. For example, `LocalChecker.checkDirectUserTuple` calls `ReadUserTuple`, and other handlers read userset and tuple-to-userset (TTU) tuples.

**Code that builds the chain.**
- `Server.Check` (`pkg/server/check.go`) calls `s.getCheckResolverBuilder(storeID).Build()` for the "v1 Check".
- `Server.getCheckResolverBuilder` (`pkg/server/check.go`) calls `graph.NewOrderedCheckResolvers(...)` with these options:
  - `WithLocalCheckerOpts` and `WithLocalShadowCheckerOpts`
  - `WithShadowResolverEnabled(featureFlagClient.Boolean(ExperimentalShadowCheck, storeID))` and `WithShadowResolverOpts`
  - `WithCachedCheckResolverOpts(s.cacheSettings.ShouldCacheCheckQueries(), ...)`
  - `WithDispatchThrottlingCheckResolverOpts(s.checkDispatchThrottlingEnabled, ...)`
- The resolver options come from `Server.getCheckResolverOptions` (`pkg/server/server.go`). It supplies the cache, the TTL and the jitter for the cached resolver, and the threshold, the max threshold and a constant-rate throttler for the dispatch throttler.
- `CheckResolverOrderedBuilder.Build` (`internal/graph/builder.go`) sets the order. It appends the cached resolver, then the dispatch throttler, then the shadow resolver or the local checker. The order in which the options are passed does not matter. `Build` then links the resolvers with `SetDelegate` into a circular list: each resolver delegates to the next, and the last one delegates back to the first. It returns `resolvers[0]` and a closer.

**What happens during a request.**
- `commands.CheckQuery.Execute` (`pkg/server/commands/check_command.go`) runs `checkResolver.ResolveCheck` on the head of the chain.
- `CachedCheckResolver.ResolveCheck` returns a cached entry when one is valid. Otherwise it delegates.
- `DispatchThrottlingCheckResolver.ResolveCheck` throttles when the dispatch count is above the threshold (`threshold.ShouldThrottle`), then delegates.
- `ShadowResolver.ResolveCheck` runs the main checker, and runs the shadow checker in a background goroutine.
- When `LocalChecker` recurses, `LocalChecker.dispatch` calls `c.delegate.ResolveCheck`. Because of the circular list, each sub-problem goes back through the cache and the throttler again.
- `AGENTS.md`, section "Resolver Chain", documents the same order: `[CachedCheckResolver]? → [DispatchThrottlingCheckResolver]? → [ShadowResolver | LocalChecker] ⟲`.

**The storage wrappers in front of the datastore.** These are not check resolvers, but every tuple read goes through them. `storagewrappers.NewRequestStorageWrapperWithCache` (`pkg/storage/storagewrappers/request.go`) builds them, and `CheckQuery.Execute` puts the result in the context. From the outermost layer inwards:
1. `CombinedTupleReader`, which serves the request's contextual tuples.
2. `sharediterator.SharedIteratorDatastore`, only if `sharedIterator.enabled`.
3. `CachedDatastore`, only if `ShouldCacheCheckIterators()` is true.
4. `BoundedTupleReader`, which limits concurrency and applies datastore throttling.
5. The server's datastore. In `pkg/server/server.go`, `NewServerWithOpts` wraps it with `NewCachedOpenFGADatastore`, and with `NewContextWrapper` when context propagation is off.

**The alternative path.** If the per-store flag `ExperimentalWeightedGraphCheck` (`"weighted_graph_check"`) is on, `Server.Check` first calls `Server.v2Check`. That path builds `commands.CheckQueryV2` (`pkg/server/commands/check.go`) and uses `check.Resolver` (`internal/check`). It does not use this resolver chain. Its datastore wrappers are a `BoundedTupleReader` plus an optional `CachedTupleReader`. If v2 fails with an error that is not terminal (see `isV2TerminalError`), Check falls back to the v1 chain above.

## Q2

**Production types that implement `ReadUserTuple`.** The method is declared on `storage.RelationshipTupleReader` (`pkg/storage/storage.go`). That interface is embedded in `TupleBackend`, which is embedded in `OpenFGADatastore`. These types implement it:

| Type | File |
|---|---|
| `postgres.Datastore` | `pkg/storage/postgres/postgres.go` |
| `mysql.Datastore` | `pkg/storage/mysql/mysql.go` |
| `sqlite.Datastore` | `pkg/storage/sqlite/sqlite.go` |
| `memory.MemoryBackend` | `pkg/storage/memory/memory.go` |
| `storagewrappers.BoundedTupleReader` | `pkg/storage/storagewrappers/bounded_datastore.go` |
| `storagewrappers.CombinedTupleReader` | `pkg/storage/storagewrappers/combinedtuplereader.go` |
| `storagewrappers.CachedTupleReader` | `pkg/storage/storagewrappers/cached_reader.go` |
| `storagewrappers.ContextTracerWrapper` | `pkg/storage/storagewrappers/context.go` |

The three wrappers and the context wrapper each delegate to the type they wrap. `CombinedTupleReader` first checks the contextual tuples.

Some other wrappers get the method only by embedding an interface and do not override it:
- `CachedDatastore` (`cached_datastore.go`)
- `sharediterator.SharedIteratorDatastore` (`shared_iterator_datastore.go`)
- `RequestStorageWrapper` (`request.go`)
- the model-caching wrapper in `model_caching.go`

**Test and mock implementations.**
- In `internal/mocks/mock_storage.go`, the generated `MockTupleBackend`, `MockRelationshipTupleReader` and `MockOpenFGADatastore`. They are generated by mockgen from `pkg/storage/storage.go`.
- The hand-written `slowDataStorage` in `internal/mocks/mock_slow_storage.go`.

**Production callers.** Apart from the wrappers above delegating to each other, there are two:
- `LocalChecker.checkDirectUserTuple` in `internal/graph/check.go`, the v1 Check.
- `Resolver.specificType` in `internal/check/check.go`, the v2 weighted-graph Check.

`BoundedTupleReader.ReadUserTuple` also tags its limiter and metrics with `storage.OperationReadUserTuple` (`pkg/storage/keys.go`).

**What a signature change would affect.**
- The interface method in `pkg/storage/storage.go`. If the change is to the filter or options types, that also means `ReadUserTupleFilter` (an alias of `ReadFilter`, so changing it would also affect `Read` and `ReadPage`) and `ReadUserTupleOptions`.
- The four datastore implementations: postgres, mysql, sqlite and memory.
- The four wrappers that override the method: `BoundedTupleReader`, `CombinedTupleReader`, `CachedTupleReader` and `ContextTracerWrapper`.
- The two production callers.
- The mocks: regenerate `internal/mocks/mock_storage.go` and edit `mock_slow_storage.go` by hand.
- The shared datastore conformance suite in `pkg/storage/test/tuples.go`, which calls the method many times.
- The unit tests that call or mock it. There are about 174 references across `server_test.go`, the postgres, mysql and sqlite tests, the storage-wrapper tests, `internal/graph/check_test.go` and `internal/check/check_test.go`.

Because `pkg/storage` is a public package, any datastore implemented outside this repository would also break. The wrappers that only embed the interface would need no change.

## Q3

**Short answer.** Staleness is decided by comparing timestamps. Each cached Check result is stored as a `CheckResponseCacheEntry` with `LastModified: time.Now()`. A per-request "last cache invalidation time" is the time of the store's latest write as last seen by the cache controller. An entry is used only if it was cached after that time.

**How it works.**
- `CachedCheckResolver.ResolveCheck` (`internal/graph/cached_resolver.go`) reuses an entry only if `res.LastModified.After(req.LastCacheInvalidationTime)`. Otherwise it counts an invalid hit and recomputes. Requests with `HIGHER_CONSISTENCY` skip the cache entirely.
- The v2 resolver applies the same rule in `internal/check/check.go`, line 169: `!res.LastModified.After(r.lastCacheInvalidationTime)`.
- The invalidation time comes from `CacheController.DetermineInvalidationTime`. `CheckQuery.Execute` (`pkg/server/commands/check_command.go`) and `Server.v2Check` (`pkg/server/check.go`) call it, unless the request asks for `HIGHER_CONSISTENCY`.

**The cache controller** is `InMemoryCacheController` in `internal/cachecontroller/cache_controller.go`. `shared.NewSharedDatastoreResources` (`internal/shared/shared.go`) creates it when `cacheController.enabled` is set and a cache is in use (`ShouldCreateCacheController`). Otherwise a `NoopCacheController` is used, which always returns the zero time. In that case cached entries are never invalidated by writes, only by their TTL.

- **`DetermineInvalidationTime`** reads a `ChangelogCacheEntry` (with `LastModified` and `LastChecked`) under the key `ChangelogCacheKey(storeID)`, defined in `pkg/storage/cache.go`.
  - If the entry is missing, it starts an asynchronous `InvalidateIfNeeded` and returns the zero time, so cached entries are treated as valid.
  - If the entry is present, it starts `InvalidateIfNeeded` only when `LastChecked` is older than `minInvalidationInterval` (`cacheController.ttl`, default 10s), and returns `entry.LastModified`.
- **`InvalidateIfNeeded`** allows only one invalidation at a time per store (`inflightInvalidations`). It starts `findChangesAndInvalidateIfNecessary` in a goroutine.
- **`findChangesAndInvalidateIfNecessary`**:
  - It calls `ds.ReadChanges` in descending order, with one page of `storage.DefaultPageSize` = 50 changes and a 1-second timeout.
  - It stores a new `ChangelogCacheEntry` with the timestamp of the newest change, using a TTL of `queryCacheTTL`.
  - It also invalidates the iterator cache. It does this by writing `InvalidEntityCacheEntry` markers rather than deleting entries:
    - for the whole store (`InvalidIteratorCacheKey`), if all 50 changes fall within the iterator cache TTL;
    - otherwise, for each recent change, by object and relation (`InvalidIteratorByObjectRelationCacheKey`) and by user and object type (`InvalidIteratorByUserObjectTypeCacheKey`).
  - `CachedDatastore` checks these markers through `findInCache` and `isInvalidAt` (`pkg/storage/storagewrappers/cached_datastore.go`).
- **Other triggers.** ListObjects also calls `InvalidateIfNeeded` when its iterator cache is enabled (`pkg/server/commands/list_objects.go`). The Write API itself does not trigger invalidation.
- **Model changes.** The model id is part of the cache key: `CheckCacheKey` includes `InvariantCacheKey(storeID, modelID, context, contextual tuples)`. So a new model never reads entries cached under another model.

**Limits.** The code comments and `docs/caching.md` (section "Cache Controller", "Caveats") describe these:
- **Eventual consistency.** Results can be stale for up to `cacheController.ttl` after a write.
- **The first request after a write.** Because invalidation runs asynchronously, the first Check after a write can return a stale result, as can later ones until the job finishes. This happens even if more than the TTL has passed since the write ("First Check Staleness" in `docs/caching.md`). A missing changelog entry also returns the zero time.
- **Whole-store granularity for query results.** Any write to the store invalidates every cached Check result in that store.
- **Iterator invalidation only sees the latest 50 changes.** More than 50 recent changes falls back to invalidating the whole store.
- **Failed changelog reads.**
  - If `ReadChanges` times out (1s), the function returns without changing anything.
  - If `ReadChanges` returns any other error (for example `ErrNotFound` when there are no changes), the iterator cache for the store is invalidated, but the changelog entry is not updated.
- **Per-replica state.** The cache and the controller live in each process's memory, so each replica invalidates on its own. `docs/caching.md` describes the single-replica case.
- **Inflated metrics.** Because invalidation adds marker entries, cache size and removal metrics can be inflated (`docs/caching.md`).

## Q4

**Short answer.** All checks run in `typesystem.NewAndValidate` (`pkg/typesystem/typesystem.go`). `WriteAuthorizationModelCommand.Execute` (`pkg/server/commands/write_authzmodel.go`) calls it after checking the type count and model-size limits, and wraps any error as `InvalidAuthorizationModelInput`. `NewAndValidate` calls `validateRelation` for every relation, in sorted order. For the tupleset `Y` of `X from Y` on type `T`, the rules are below.

**In `TypeSystem.isUsersetRewriteValid`, `Userset_TupleToUserset` case.** This function also recurses into union, intersection and difference children.
1. **`Y` must be defined on the same object type `T`.** Otherwise it returns `RelationUndefinedError` / `ErrRelationUndefined`.
2. **`Y` must be a direct relation.** Its rewrite must be exactly `this` (`Userset_This`), with no union, computed userset or other rewrite. Otherwise it returns the error "the 'T#Y' relation is referenced in at least one tupleset and thus must be a direct relation".
3. **For 1.1 and 1.2 schemas, `X` must be a relation on at least one of the types that `Y` allows directly** (`GetDirectlyRelatedUserTypes`). Otherwise it returns `ErrRelationUndefined` with "… does not appear as a relation in any of the directly related user types …".
   - The 1.0 rule only requires `X` to exist on some type in the model.
   - `NewAndValidate` already rejects unsupported schema versions (`IsSchemaVersionSupported` accepts only 1.1 and 1.2), so I believe the 1.0 branch cannot be reached on this path.

**In `TypeSystem.validateTypeRestrictions`.**
4. **`Y`'s type restrictions must be plain object types,** such as `[folder]`, and not usersets (`folder#member`) or wildcards (`folder:*`). If `IsTuplesetRelation(T, Y)` is true and a restriction has a relation or a wildcard (`GetRelationOrWildcard() != nil`), it returns `InvalidRelationTypeError`.
   - `IsTuplesetRelation` is in the same file. It looks `Y` up in `ttuRelations`, which `New` builds with `flattenUserset`.
5. **The general rules also apply to `Y`.** Because it is assignable, it must list at least one type. Every listed type must exist. Any condition it references must be defined in the model.

**Other model-level checks.** Entry points (`hasEntrypoints`) and cycles (`HasCycle`) also run, but they are not specific to tuplesets.

**The related check when tuples are written.** When tuples, rather than models, are written, `validateTuplesetRestrictions` (`internal/validation/validation.go`) enforces the matching rules on each tuple. It rejects a userset, a wildcard, or a non-object user on a tupleset relation. That function is not part of model writes.

I did not check whether the protobuf request validation (`req.Validate()`, from generated code outside this repository) adds any tupleset-specific rule. I am not aware of one.

## Q5

**Short answer.** Each Check gets its own `storagewrappers.BoundedTupleReader` (`pkg/storage/storagewrappers/bounded_datastore.go`). It holds a buffered channel `limiter` whose capacity is `Operation.Concurrency`. Before each `Read`, `ReadUserTuple`, `ReadUsersetTuples` and `ReadStartingWithUser`, `bound()` calls `waitForLimiter()`. That sends into the channel, so it blocks while the channel is full, and returns `ctx.Err()` if the context ends first. `done()` releases the slot when the wrapped call returns.

**Behaviour while waiting and after the threshold.**
- Waits longer than 1ms are recorded in `openfga_datastore_bounded_read_delay_ms`.
- The same wrapper can also throttle. Once the per-request read count passes a threshold, it sleeps for a set duration and marks the request as throttled. This happens only when the per-store flag `ExperimentalDatastoreThrottling` (`"datastore_throttling"`) is on, and it uses `checkDatastoreThrottle.threshold` and `checkDatastoreThrottle.duration`.

**How the limit gets to the wrapper.**
- In v1, `Server.Check` passes `commands.WithCheckCommandMaxConcurrentReads(s.maxConcurrentReadsForCheck)`. `CheckQuery.Execute` (`pkg/server/commands/check_command.go`) then builds `NewRequestStorageWrapperWithCache` with `Operation{Method: Check, Concurrency: c.maxConcurrentReads, ...}`, and that calls `NewBoundedTupleReader` (`request.go`).
- In v2, `Server.v2Check` passes `WithCheckQueryV2MaxConcurrentReads`, and `CheckQueryV2.Execute` calls `NewBoundedTupleReader` directly (`pkg/server/commands/check.go`).

**How an operator configures it.**

| Setting | Value |
|---|---|
| Config key | `maxConcurrentReadsForCheck` (`Config.MaxConcurrentReadsForCheck`, `pkg/server/config/config.go`) |
| Flag | `--max-concurrent-reads-for-check` (`cmd/run/run.go`) |
| Environment variable | `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK`, or `OPENFGA_MAXCONCURRENTREADSFORCHECK` (`cmd/run/flags.go`) |
| Default | `math.MaxUint32`, which is effectively unlimited (`DefaultMaxConcurrentReadsForCheck`; `.config-schema.json`) |

`cmd/run/run.go` passes the value to `server.WithMaxConcurrentReadsForCheck`.

The doc comment on `WithMaxConcurrentReadsForCheck` (`pkg/server/server.go`) advises sizing the limit from three things: the expected Check and ListObjects request rate, the number of replicas, and `Datastore.MaxOpenConns`. Its example: with 100 connections, 1 replica and 100 RPS, set it to 1.

**Limits and caveats.**
- **The limit is per Check request, not global.** Each Check, including its sub-problems, shares one `BoundedTupleReader`.
- **It covers opening reads, not consuming them.** The slot is released when the read call returns, before the returned iterator has been consumed. How much database work is still in flight at that point depends on the datastore implementation; I did not verify this.
- **A value of 0 is not rejected for Check.** `VerifyServerSettings` rejects 0 only for `maxConcurrentReadsForListUsers`. With capacity 0 the channel is unbuffered and nothing reads from it, so every read would block until the context ends. This is inferred from the code and was not tested.
- **BatchCheck appears not to apply the setting.** `BatchCheckQuery` (`pkg/server/commands/batch_check_command.go`) calls `NewCheckCommand` without `WithCheckCommandMaxConcurrentReads`. Each item in a batch therefore seems to use the default of `MaxUint32`.
