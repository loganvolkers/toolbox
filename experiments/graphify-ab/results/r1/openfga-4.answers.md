# Answers

## Q1

**Short answer.** For a v1 Check, the request runs through a chain of `graph.CheckResolver`s built per request by `graph.CheckResolverOrderedBuilder.Build()`, in this order:

1. `CachedCheckResolver`, if it is enabled;
2. `DispatchThrottlingCheckResolver`, if it is enabled;
3. either `ShadowResolver`, which wraps a main and a shadow `LocalChecker`, or a plain `LocalChecker`.

The last resolver's delegate is set back to the first, so the chain is a ring. Only the `LocalChecker` reads tuples.

**Entry point (`pkg/server/check.go`, `Server.Check`).** Before any resolver runs, `Server.Check` does the following:

- It validates the request and runs `s.checkAuthz`.
- If the feature flag `ExperimentalWeightedGraphCheck` (`"weighted_graph_check"`) is on for the store, it first tries `s.v2Check`. That path uses `commands.CheckQueryV2` (`pkg/server/commands/check.go`) and `internal/check.Resolver`, not this resolver chain. It falls back to v1 on errors that `isV2TerminalError` does not treat as terminal.
- On the v1 path it calls `s.getCheckResolverBuilder(storeID).Build()`, resolves the typesystem, and runs `commands.NewCheckCommand(...).Execute`.
- `CheckQuery.Execute` (`pkg/server/commands/check_command.go`) calls `c.checkResolver.ResolveCheck`, which enters the chain at its head.

**Builder.** `Server.getCheckResolverBuilder` (`pkg/server/check.go`) calls `graph.NewOrderedCheckResolvers(...)` with these options: `WithLocalCheckerOpts`, `WithLocalShadowCheckerOpts`, `WithShadowResolverEnabled`, `WithShadowResolverOpts`, `WithCachedCheckResolverOpts` and `WithDispatchThrottlingCheckResolverOpts`. The options for the cache and the throttler come from `Server.getCheckResolverOptions` (`pkg/server/server.go`). `BatchCheck` uses the same builder (`pkg/server/batch_check.go`).

**`Build()` (`internal/graph/builder.go`, `CheckResolverOrderedBuilder.Build`).** It appends resolvers in a fixed order:

- `NewCachedCheckResolver`, if `cachedCheckResolverEnabled`;
- `NewDispatchThrottlingCheckResolver`, if `dispatchThrottlingCheckResolverEnabled`;
- `NewShadowChecker(main, shadow)` if `shadowResolverEnabled`, otherwise `NewLocalChecker`.

It then calls `SetDelegate` so that each resolver delegates to the next one and the last delegates to `c.resolvers[0]`. The order of the options passed in does not matter; the code order in `Build` decides.

**What decides whether each resolver is present:**

| Resolver | Present when | Source |
|---|---|---|
| `CachedCheckResolver` (`internal/graph/cached_resolver.go`) | `s.cacheSettings.ShouldCacheCheckQueries()`, which is `CheckCacheLimit > 0 && CheckQueryCacheEnabled`. Both come from config `checkQueryCache.enabled` (default `false`) and `checkCache.limit` (default 10000). | `pkg/server/config/cache_settings.go`, `pkg/server/config/config.go` |
| `DispatchThrottlingCheckResolver` (`internal/graph/dispatch_throttling_check_resolver.go`) | `s.checkDispatchThrottlingEnabled`, set by `WithDispatchThrottlingCheckResolverEnabled` from `checkDispatchThrottling.enabled` (default `false`) | `pkg/server/server.go`, `cmd/run/run.go` |
| `ShadowResolver` (`internal/graph/shadow_resolver.go`) instead of a bare `LocalChecker` | `s.featureFlagClient.Boolean(serverconfig.ExperimentalShadowCheck, storeID)`, flag `"shadow_check"`. With the default client (`pkg/featureflags/client.go`, `NewDefaultClient`), this means the flag is listed in `experimentals`. | `pkg/server/check.go` |
| `LocalChecker` (`internal/graph/check.go`) | Always present, either alone or as `ShadowResolver.main` | `builder.go` |

**Behaviour of each resolver:**

- **`CachedCheckResolver.ResolveCheck`** returns a cached result when the request's consistency is not `HIGHER_CONSISTENCY` and the entry's `LastModified` is after `req.LastCacheInvalidationTime`. Otherwise it delegates the request.
- **`DispatchThrottlingCheckResolver.ResolveCheck`** calls `r.throttler.Throttle(ctx)` when `threshold.ShouldThrottle` decides the dispatch count is over the threshold, and then delegates.
- **`ShadowResolver.ResolveCheck`** returns the main checker's result and runs the shadow checker asynchronously only to compare results. `ShadowResolver.SetDelegate` sets only `main`'s delegate, so the shadow `LocalChecker` keeps itself as its delegate (`NewLocalChecker`).
- **`LocalChecker.ResolveCheck`** evaluates the rewrite. Its sub-problems go through `LocalChecker.dispatch` to `c.delegate`, which is the head of the chain again. As a result, sub-problems are also cache-checked and throttled (see also `docs/check/README.md`, "Delegate Composition").

**Tuple reads.** The `LocalChecker` gets its tuple reader from the context. `CheckQuery.Execute` puts it there through `storagewrappers.NewRequestStorageWrapperWithCache` (`pkg/storage/storagewrappers/request.go`). From outermost to innermost, the wrappers are:

1. `CombinedTupleReader`, which serves contextual tuples;
2. `sharediterator.IteratorDatastore`, if `SharedIteratorEnabled`;
3. `CachedDatastore`, the iterator cache, if `ShouldCacheCheckIterators()`;
4. `BoundedTupleReader`, which limits concurrency and throttles;
5. the server datastore.

The server datastore is itself wrapped in `NewServerWithOpts` (`pkg/server/server.go`). It is a `cachedOpenFGADatastore`, around a `ContextTracerWrapper` when `contextPropagationToDatastore` is false, around the real backend. These are storage wrappers, not resolvers.

## Q2

**Interface.** `ReadUserTuple(ctx, store string, filter ReadUserTupleFilter, options ReadUserTupleOptions) (*openfgav1.Tuple, error)` is declared on `storage.RelationshipTupleReader` in `pkg/storage/storage.go`. It is part of `TupleBackend` and therefore of `OpenFGADatastore`.

**Types that define the method explicitly:**

| Kind | Type and file |
|---|---|
| Backends | `memory.MemoryBackend` (`pkg/storage/memory/memory.go`), `mysql.Datastore` (`pkg/storage/mysql/mysql.go`), `postgres.Datastore` (`pkg/storage/postgres/postgres.go`), `sqlite.Datastore` (`pkg/storage/sqlite/sqlite.go`) |
| Wrappers | `storagewrappers.BoundedTupleReader` (`bounded_datastore.go`), `storagewrappers.CombinedTupleReader` (`combinedtuplereader.go`), `storagewrappers.ContextTracerWrapper` (`context.go`), `storagewrappers.CachedTupleReader` (`cached_reader.go`, pass-through) |
| Mocks and test helpers | `mocks.MockTupleBackend`, `mocks.MockRelationshipTupleReader`, `mocks.MockOpenFGADatastore` (`internal/mocks/mock_storage.go`, generated by mockgen from `pkg/storage/storage.go`), `mocks.slowDataStorage` (`internal/mocks/mock_slow_storage.go`) |

**Types that get the method by embedding, without overriding it:** `CachedDatastore` (`cached_datastore.go`), `cachedOpenFGADatastore` (`model_caching.go`), `RequestStorageWrapper` (`request.go`) and `sharediterator.IteratorDatastore` (`shared_iterator_datastore.go`).

**Production callers.** There are two call sites outside tests and mocks:

- `LocalChecker.checkDirectUserTuple` in `internal/graph/check.go`, the v1 Check direct-assignment path. It calls `ds.ReadUserTuple` on the reader taken from the context.
- `Resolver.specificType` in `internal/check/check.go`, the v2 weighted-graph Check. It calls `r.datastore.ReadUserTuple` only when no matching contextual tuple was found.

The wrappers also forward calls to the reader they wrap: `BoundedTupleReader`, `CombinedTupleReader`, `ContextTracerWrapper` and `CachedTupleReader`. They do this only as delegates on the path from those two callers.

**If the signature changed, these would have to change too:**

- the interface method in `pkg/storage/storage.go`, and `ReadUserTupleFilter`/`ReadUserTupleOptions` if those are what change;
- the four backends listed above;
- the four explicit wrappers listed above;
- the two production callers listed above;
- the generated mocks in `internal/mocks/mock_storage.go`, regenerated with `make generate-mocks` (`Makefile`; `AGENTS.md`), and the hand-written `slowDataStorage`;
- the shared datastore conformance tests in `pkg/storage/test/tuples.go`;
- the `_test.go` callers: `internal/check/check_test.go` (54 matches, mostly mock expectations), `pkg/storage/{mysql,postgres,sqlite}/*_test.go`, `pkg/storage/storagewrappers/{bounded_datastore,cached_reader,combinedtuplereader}_test.go`, `pkg/server/server_test.go` and `internal/graph/check_test.go`.

The embedding wrappers would need no edits. Because `pkg/storage` is a public package, any out-of-tree `OpenFGADatastore` implementation would also break; that is an inference, as nothing in the repo lists such implementations.

## Q3

**Short answer.** A write does not touch any cache: `pkg/server/commands/write.go` has no cache code. Staleness is detected lazily, on the read side, by comparing the time each cache entry was stored with the time of the store's last change. That time is read from the changelog by the cache controller.

**Where the comparison happens:**

- **v1.** `CheckQuery.Execute` (`pkg/server/commands/check_command.go`) calls `CacheController.DetermineInvalidationTime(ctx, storeID)`, unless the consistency is `HIGHER_CONSISTENCY`. It puts the result into `ResolveCheckRequest.LastCacheInvalidationTime`. `CachedCheckResolver.ResolveCheck` (`internal/graph/cached_resolver.go`) then uses a `CheckResponseCacheEntry` only if `res.LastModified.After(req.LastCacheInvalidationTime)`. Otherwise it counts `check_cache_invalid_hit_count`, recomputes the result and overwrites the entry. The cache key (`storage.CheckCacheKey` plus `GetInvariantCacheKey`) includes the store, model id, context and contextual tuples, so a new model never hits old entries.
- **v2.** `Server.v2Check` (`pkg/server/check.go`) passes the same time as `LastCacheInvalidationTime` to `internal/check`. There, a cached result is rejected when `!res.LastModified.After(r.lastCacheInvalidationTime)` (`internal/check/check.go`).

**The controller (`internal/cachecontroller/cache_controller.go`, `InMemoryCacheController`):**

- `DetermineInvalidationTime` reads the `ChangelogCacheEntry` under `storage.ChangelogCacheKey(storeID)`.
  - On a miss, it triggers `InvalidateIfNeeded` asynchronously and returns zero time, which makes every cached entry count as valid.
  - On a hit, it returns `entry.LastModified`, and it triggers `InvalidateIfNeeded` only if `LastChecked` is older than `minInvalidationInterval`, which is `cacheController.ttl`.
- `InvalidateIfNeeded` allows one invalidation per store at a time (`inflightInvalidations`) and runs `findChangesAndInvalidateIfNecessary` in a goroutine. That function:
  - reads the newest changes with `ReadChanges`, descending, page size `storage.DefaultPageSize` = 50, with a 1 s timeout;
  - stores a new `ChangelogCacheEntry{LastModified: newest change timestamp}` with TTL `queryCacheTTL`;
  - for the iterator cache, writes `InvalidEntityCacheEntry` markers. It writes a store-wide marker (`InvalidIteratorCacheKey`) if all fetched changes fall inside the iterator TTL or if `ReadChanges` fails. Otherwise it writes per object#relation and per user/objectType markers for the changes inside the window. `findInCache` in `pkg/storage/storagewrappers/cached_datastore.go` checks these markers.

**When it is created.** `internal/shared/shared.go` creates the controller only if `ShouldCreateCacheController()` holds: some cache is enabled and `cacheController.enabled` is true (default `false`). Otherwise the server uses `NoopCacheController`, whose `DetermineInvalidationTime` always returns zero time. ListObjects also calls `InvalidateIfNeeded` on every request when its iterator cache is on (`pkg/server/commands/list_objects.go`).

**Limits.** These are documented in `docs/caching.md`, "Cache Controller" and "Caveats", and visible in the code above:

- It is eventually consistent. After a write, a Check can return a stale result for up to `cacheController.ttl`, because the changelog is re-read at most once per that interval per store.
- The first Check after a long idle period can still be stale ("First Check Staleness"). Invalidation is asynchronous, and a missing or expired changelog entry returns zero time.
- With the controller disabled, which is the default, nothing detects writes. Entries stay valid until the query-cache TTL expires (`checkQueryCache.ttl`, default 10 s, plus `CacheTTLJitterPercentage` jitter).
- Caches are in-memory and per replica, so a write is noticed by each replica independently.
- Any write to the store invalidates all query-cache entries, not just the affected ones. Only the iterator cache is invalidated selectively, and only from the last 50 changes.
- `HIGHER_CONSISTENCY` bypasses the caches entirely.
- The check compares a server-local `time.Now()`, which is when the entry was stored, with changelog timestamps that come from the datastore. Clock skew between the two could therefore affect it. This is an inference from the code; the docs do not discuss it.

## Q4

**Short answer.** For `X from Y`, the tupleset relation `Y` must be defined on the same type, must be a purely direct relation, and may only be restricted to plain object types. At least one of those types must define `X` and give it an entrypoint. All of this is enforced when a model is written, through `WriteAuthorizationModelCommand.Execute` (`pkg/server/commands/write_authzmodel.go`). That function calls `typesystem.NewAndValidate`, which runs `TypeSystem.validateRelation` (`pkg/typesystem/typesystem.go`) for every relation, in sorted order. A failure is returned as `serverErrors.InvalidAuthorizationModelInput`.

**Constraints on the tupleset `Y`:**

1. **`Y` must exist on the same object type.** `isUsersetRewriteValid`, case `Userset_TupleToUserset`, returns `RelationUndefinedError` if `t.GetRelation(objectType, tupleset)` fails.
2. **`Y` must be a purely direct relation.** Its rewrite must be exactly `Userset_This`, with no union, intersection, exclusion or computed userset. The same `isUsersetRewriteValid` case returns "the '%s#%s' relation is referenced in at least one tupleset and thus must be a direct relation".
3. **`X` must be defined on at least one of `Y`'s directly related types.** For schema 1.1 and 1.2, `isUsersetRewriteValid` checks the computed relation against `tuplesetRelation.GetTypeInfo().GetDirectlyRelatedUserTypes()`. One match is enough; the types do not all need to define `X`. The 1.0 branch, which only requires `X` somewhere in the model, cannot be reached through `NewAndValidate`, because `IsSchemaVersionSupported` accepts only 1.1 and 1.2.
4. **`Y`'s type restrictions must be plain object types.** A userset (`type#rel`) or wildcard (`type:*`) restriction is rejected. `validateTypeRestrictions` returns `InvalidRelationTypeError` when `related.GetRelationOrWildcard() != nil` and `t.IsTuplesetRelation(objectType, relationName)` is true. Because `Y` is assignable, rule 1 of the same function also requires it to have at least one type restriction. Conditions on `Y`'s type restrictions are not rejected there; they only have to name a condition that exists in the model.
5. **The rewrite must have an entrypoint and no cycle.** `hasEntrypoints`, case `Userset_TupleToUserset`, requires at least one of `Y`'s directly related types whose relation `X` has an entrypoint. `validateRelation` then rejects cycles through `HasCycle` (`ErrCycle`).

**Related checks outside model writes:**

- `TypeSystem.relationInvolves`, used by `RelationInvolvesIntersection`/`RelationInvolvesExclusion`, also returns an `ErrInvalidModel` error if it meets a `type#relation` restriction on a tupleset. That is a query-time helper, not model-write validation.
- At tuple-write time, `validateTuplesetRestrictions` (`internal/validation/validation.go`, called from `ValidateTupleForRead`/`ValidateTupleForWrite`) rejects tuples whose user, on a tupleset relation, is a wildcard or a userset.

This answer covers only the server-side checks in this repo. I did not verify any extra validation the DSL tooling in the external `openfga/language` module may do.

## Q5

**Short answer.** Each Check request gets its own `storagewrappers.BoundedTupleReader` (`pkg/storage/storagewrappers/bounded_datastore.go`). It is a counting semaphore: a buffered channel `limiter` of capacity `op.Concurrency`. Before calling the wrapped datastore, `Read`, `ReadUserTuple`, `ReadUsersetTuples` and `ReadStartingWithUser` each call `bound()`, which blocks in `waitForLimiter` until a slot is free or the context ends. When the call returns, they release the slot with a deferred `done()`.

**How the limit reaches the reader:**

- **v1.** `Server.Check` passes `commands.WithCheckCommandMaxConcurrentReads(s.maxConcurrentReadsForCheck)` (`pkg/server/check.go`). `CheckQuery.Execute` puts the value in `storagewrappers.Operation{Concurrency: c.maxConcurrentReads}` and builds the reader with `NewRequestStorageWrapperWithCache` → `NewBoundedTupleReader` (`pkg/server/commands/check_command.go`, `pkg/storage/storagewrappers/request.go`).
- **v2 (weighted graph).** `WithCheckQueryV2MaxConcurrentReads` passes the same value, and `CheckQueryV2.Execute` calls `NewBoundedTupleReader` directly (`pkg/server/commands/check.go`).

**Optional datastore throttling in the same wrapper.** When the feature flag `datastore_throttling` (`ExperimentalDatastoreThrottling`) is on for the store and `threshold > 0`, every read after the first `threshold` reads in the request waits an extra `throttleTime`. The request is then marked `WasThrottled`. These values come from `checkDatastoreThrottle.threshold` and `checkDatastoreThrottle.duration` (flags `--check-datastore-throttle-threshold` and `--check-datastore-throttle-duration`). If the deadline is then exceeded, the error is reported as `ThrottledError`.

**How an operator configures the limit:**

| Where | Name |
|---|---|
| Config file | `maxConcurrentReadsForCheck` (`.config-schema.json`) |
| Flag | `--max-concurrent-reads-for-check` (`cmd/run/run.go`) |
| Environment | `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK` or `OPENFGA_MAXCONCURRENTREADSFORCHECK` (`cmd/run/flags.go`) |
| Embedding | the server option `server.WithMaxConcurrentReadsForCheck(n)` (`pkg/server/server.go`) |

`cmd/run/run.go` passes `config.MaxConcurrentReadsForCheck` to that option. The default is `math.MaxUint32`, which is effectively unbounded (`DefaultMaxConcurrentReadsForCheck` in `pkg/server/config/config.go`). The doc comment on `WithMaxConcurrentReadsForCheck` explains how to size it from `Datastore.MaxOpenConns`, expected RPS and replica count.

**Limits:**

- **It is per request, not global.** A new limiter is created per Check.
- **BatchCheck does not pass the setting.** `BatchCheckQuery.Execute` (`pkg/server/commands/batch_check_command.go`) builds each `NewCheckCommand` without `WithCheckCommandMaxConcurrentReads`, so checks inside BatchCheck use the unbounded default. BatchCheck parallelism is limited separately by `maxConcurrentChecksPerBatchCheck`.
- **For iterator-returning methods, the slot only covers creating the iterator.** The slot is released when `Read`/`ReadUsersetTuples`/`ReadStartingWithUser` return their iterator. The SQL backends build lazy iterators: `sqlcommon.SQLTupleIterator` and `sqlite.SQLTupleIterator` run the query in `fetchBuffer` on the first `Next`/`Head`. So for those three methods, the limiter appears to bound iterator creation rather than in-flight SQL queries. `ReadUserTuple` runs its query inside the bound. This is inferred from the code; no doc states it.
- **Breadth limits concurrency only indirectly.** The number of sub-problems a `LocalChecker` evaluates in parallel at each node is limited separately by `resolveNodeBreadthLimit` (default 10; `concurrency.NewPool` in `union`/`intersection`, `internal/graph/check.go`). That indirectly limits how many reads are attempted at once.
