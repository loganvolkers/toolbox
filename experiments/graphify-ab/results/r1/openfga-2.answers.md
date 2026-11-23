# Answers

## Q1

**Short answer.** On the default (v1) path, a Check passes through up to three resolvers, in this order:

1. `CachedCheckResolver`, if it is enabled;
2. `DispatchThrottlingCheckResolver`, if it is enabled;
3. either `ShadowResolver` (which wraps two `LocalChecker`s) or a plain `LocalChecker`.

Only the `LocalChecker` reads tuples. The chain is circular: the last resolver delegates back to the first, so every sub-problem the `LocalChecker` dispatches starts again at the top of the chain.

**Entry point and the bypass.** `Server.Check` (`pkg/server/check.go`) first validates the request and runs `checkAuthz`. It then looks at the feature flag `ExperimentalWeightedGraphCheck` (`"weighted_graph_check"`, `pkg/server/config/config.go`).
- If that flag is on for the store, the request goes to `Server.v2Check`, which runs `commands.CheckQueryV2` (`pkg/server/commands/check.go`) and the `internal/check` `Resolver`. That path does not use the resolver chain at all.
- `v2Check` falls back to the v1 chain on any error that is not terminal (`isV2TerminalError`).

**Who builds the chain.** On the v1 path, `Server.Check` calls `s.getCheckResolverBuilder(storeID).Build()`. The chain is therefore rebuilt for every request, and closed afterwards with `checkResolverCloser`.
- `getCheckResolverBuilder` (`pkg/server/check.go`) calls `graph.NewOrderedCheckResolvers(...)`, passing:
  - `WithLocalCheckerOpts`;
  - `WithLocalShadowCheckerOpts`, with optimizations forced on;
  - `WithShadowResolverEnabled`;
  - `WithShadowResolverOpts`;
  - `WithCachedCheckResolverOpts`;
  - `WithDispatchThrottlingCheckResolverOpts`.
- The options for the cache and the throttler come from `Server.getCheckResolverOptions` (`pkg/server/server.go`).
- The assembly itself is `CheckResolverOrderedBuilder.Build` (`internal/graph/builder.go`). It appends the resolvers in a fixed order, then links them with `SetDelegate`: each resolver delegates to the next one, and the last delegates to `resolvers[0]`.

**What decides whether each resolver is present.**

| Position | Type | Present when | Where the condition comes from |
|---|---|---|---|
| 1 | `graph.CachedCheckResolver` (`internal/graph/cached_resolver.go`) | `cacheSettings.ShouldCacheCheckQueries()` is true, i.e. `CheckCacheLimit > 0 && CheckQueryCacheEnabled` (`pkg/server/config/cache_settings.go`) | The flags `--check-query-cache-enabled` (default `false`) and `--check-cache-limit` (default 10000), set in `cmd/run/run.go` |
| 2 | `graph.DispatchThrottlingCheckResolver` (`internal/graph/dispatch_throttling_check_resolver.go`) | `s.checkDispatchThrottlingEnabled` is true | The flag `--check-dispatch-throttling-enabled` (default `false`), applied via `server.WithDispatchThrottlingCheckResolverEnabled` |
| 3 | `graph.ShadowResolver` (`internal/graph/shadow_resolver.go`), built by `NewShadowChecker(main, shadow)` | `featureFlagClient.Boolean(ExperimentalShadowCheck, storeID)` is true. `ExperimentalShadowCheck` is `"shadow_check"`. | The default client is `featureflags.NewDefaultClient(s.experimentals)` (`pkg/featureflags/client.go`). It is driven by `--experimentals` or `OPENFGA_EXPERIMENTALS` and ignores the store id. An embedder can supply its own client with `WithFeatureFlagClient`. |
| 3 (otherwise) | `graph.LocalChecker` (`internal/graph/check.go`) | Always, whenever there is no shadow resolver | Built by `NewLocalChecker(c.localCheckerOptions...)` |

**What each resolver does with the request.**
- **`CachedCheckResolver.ResolveCheck`**
  - It skips the cache when the consistency is `HIGHER_CONSISTENCY`.
  - Otherwise it returns a cached entry if that entry's `LastModified` is after `req.LastCacheInvalidationTime`.
  - On a miss it delegates down the chain, then stores the result unless a cycle was detected.
- **`DispatchThrottlingCheckResolver.ResolveCheck`** compares the dispatch count in the request metadata against a threshold, using `threshold.ShouldThrottle`. If the count is over the threshold, it waits on the throttler and sets `DispatchThrottled`. Then it delegates.
- **`ShadowResolver.ResolveCheck`**
  - It runs `main` synchronously and returns that result.
  - It then runs `shadow` in a goroutine, with the timeout `shadowCheckResolverTimeout` (default 1s), and only logs any difference between the two results.
  - `ShadowResolver.SetDelegate` sets the delegate of `main` only. The shadow `LocalChecker` keeps the default from `NewLocalChecker`, which is to delegate to itself.
- **`LocalChecker.ResolveCheck`**
  - It checks the depth limit (`ErrResolutionDepthExceeded`) and detects cycles (`hasCycle`).
  - It answers self-defining tuples, then checks that the typesystem and tuple reader are in the context.
  - It calls `typesys.PathExists`, then `CheckRewrite`.
  - Sub-problems go through `LocalChecker.dispatch`, which calls `c.delegate.ResolveCheck`, i.e. back to the head of the chain.
  - The tuple reads themselves are in its handlers. For example, `checkDirectUserTuple` calls `ReadUserTuple` on the reader taken from `storage.RelationshipTupleReaderFromContext`.
  - The other strategy code in `internal/graph` (`weight_two_resolver.go`, `recursive_resolver.go`, planner selection) runs inside `LocalChecker`. It is not a link in the chain. I did not trace in detail which strategy runs when.

**What runs between the chain and the datastore.** `commands.CheckQuery.Execute` (`pkg/server/commands/check_command.go`) does the following before calling `checkResolver.ResolveCheck`:
1. It validates the request (`validateCheckRequest`).
2. It computes `LastCacheInvalidationTime`, unless the consistency is `HIGHER_CONSISTENCY`.
3. It builds a `graph.ResolveCheckRequest`.
4. It wraps the datastore with `storagewrappers.NewRequestStorageWrapperWithCache` (`pkg/storage/storagewrappers/request.go`) and puts the wrapped reader in the context.

That wrapper stack consists of storage decorators, not resolvers. From outermost to innermost:
1. `CombinedTupleReader`, which serves contextual tuples;
2. `sharediterator.IteratorDatastore`, if `SharedIteratorEnabled`;
3. `CachedDatastore`, if `ShouldCacheCheckIterators()`;
4. `BoundedTupleReader`;
5. the server's datastore. `NewServerWithOpts` wraps that datastore as `cachedOpenFGADatastore` around an optional `ContextTracerWrapper`, around the backend (`pkg/server/server.go`).

ListObjects builds a similar chain (`pkg/server/list_objects.go`), but without the shadow option.

## Q2

**Short answer.** The four storage backends and four storage wrappers implement `ReadUserTuple` with their own method; several other wrappers inherit it by embedding. In production code, only two functions call it, both in the Check resolvers. A signature change would ripple through the interface, all eight implementations, the two callers, the mocks and the tests.

**Where the method is declared.** It is on the `storage.RelationshipTupleReader` interface (`pkg/storage/storage.go`):

```go
ReadUserTuple(ctx context.Context, store string, filter ReadUserTupleFilter, options ReadUserTupleOptions) (*openfgav1.Tuple, error)
```

The contract is that it returns `ErrNotFound` when there is no match.
- `ReadUserTupleFilter` is a type alias: `type ReadUserTupleFilter = ReadFilter`.
- `ReadUserTupleOptions` holds a `Consistency` field.
- `RelationshipTupleReader` is embedded in `TupleBackend`, which is embedded in `OpenFGADatastore`.

**Types with their own implementation.**

| Type | File | Notes |
|---|---|---|
| `memory.MemoryBackend` | `pkg/storage/memory/memory.go` | Ignores the options |
| `postgres.Datastore` | `pkg/storage/postgres/postgres.go` | Uses `options.Consistency.Preference` to pick the pgx pool |
| `mysql.Datastore` | `pkg/storage/mysql/mysql.go` | Ignores the options |
| `sqlite.Datastore` | `pkg/storage/sqlite/sqlite.go` | Ignores the options |
| `storagewrappers.BoundedTupleReader` | `pkg/storage/storagewrappers/bounded_datastore.go` | Concurrency limit, read and item counting |
| `storagewrappers.CombinedTupleReader` | `pkg/storage/storagewrappers/combinedtuplereader.go` | Returns a matching contextual tuple first, then delegates |
| `storagewrappers.CachedTupleReader` | `pkg/storage/storagewrappers/cached_reader.go` | Plain pass-through, no caching (v2 Check path) |
| `storagewrappers.ContextTracerWrapper` | `pkg/storage/storagewrappers/context.go` | Runs the query on a detached context |

**Types that get the method only by embedding.** These have no method of their own:
- `CachedDatastore` (`cached_datastore.go`);
- `sharediterator.IteratorDatastore` (`sharediterator/shared_iterator_datastore.go`);
- `cachedOpenFGADatastore` (`model_caching.go`);
- `RequestStorageWrapper` (`request.go`).

**Mocks and test doubles.** These also implement it:
- `MockTupleBackend`, `MockRelationshipTupleReader` and `MockOpenFGADatastore` in `internal/mocks/mock_storage.go`, generated by mockgen from `pkg/storage/storage.go`;
- `slowDataStorage` in `internal/mocks/mock_slow_storage.go`.

**Production callers.** Outside the wrappers delegating to each other, two functions call it:
1. `LocalChecker.checkDirectUserTuple` (`internal/graph/check.go`), on the v1 Check path. It reads `object#relation@user` with the request's consistency, treats `ErrNotFound` as "not allowed", and then validates the tuple and evaluates its condition.
2. `Resolver.specificType` (`internal/check/check.go`), on the v2 weighted-graph Check path. It calls `ReadUserTuple` only when no contextual tuple matched.

Each of the wrappers listed above (Bounded, Combined, CachedTupleReader, ContextTracer) also calls the next layer's `ReadUserTuple`.

**What a signature change would touch.**
- The interface in `pkg/storage/storage.go`.
- The four backends: memory, postgres, mysql and sqlite.
- The four wrappers with their own method. Embedding-only wrappers adapt automatically.
- Both callers: `checkDirectUserTuple` and `specificType`.
- The regenerated mocks (`make generate-mocks`) and `slowDataStorage`.
- The datastore conformance suite in `pkg/storage/test/tuples.go`, which calls it many times.
- The tests that use it:
  - `internal/graph/check_test.go`;
  - `internal/check/check_test.go`;
  - `pkg/server/server_test.go`;
  - the postgres, mysql and sqlite `_test.go` files;
  - the storagewrappers tests (bounded, combined, cached_reader).
- The illustrative call traces in `docs/check/README.md`, if the call shape changes.
- **If the filter type changes:** `ReadUserTupleFilter` is only an alias of `ReadFilter`, so changing it in place also changes `Read` and `ReadPage`. You would want to make it a distinct type.
- **Outside this repo (inference):** `pkg/storage` is a public package, so any datastore implementation outside the repo would also break.

## Q3

**Short answer.** A cached Check result is stale when it was stored before the last write that the cache controller has seen in the store's changelog. The controller is off by default; without it, a cached result is only dropped when its TTL expires. With it, staleness is bounded by roughly the controller TTL plus one asynchronous refresh, it is per replica, and it has store-level granularity.

**Layer 1: entry TTL only.**
- `CachedCheckResolver.ResolveCheck` (`internal/graph/cached_resolver.go`) stores `CheckResponseCacheEntry{LastModified: time.Now(), ...}`.
- The TTL is `--check-query-cache-ttl` (default 10s), plus optional jitter (`CacheTTLJitterPercentage`).
- The cache key is built by `storage.CheckCacheKey`. It includes the store, object, relation and user, plus an invariant hash (`storage.InvariantCacheKey`) of the model id, request context and contextual tuples. A different model id therefore gives a different key.
- Without the cache controller, a write is only reflected when the entry expires.

**Layer 2: the cache controller.**
- **Type and location:** `cachecontroller.InMemoryCacheController` (`internal/cachecontroller/cache_controller.go`). It is created in `shared.NewSharedDatastoreResources` (`internal/shared/shared.go`) only if `ShouldCreateCacheController()` is true, i.e. some cache is enabled and `--cache-controller-enabled` is set (default `false`). Otherwise `NoopCacheController` is used, which always returns the zero time.
- **Reading the invalidation time:** `CheckQuery.Execute` (`pkg/server/commands/check_command.go`) calls `CacheController.DetermineInvalidationTime(ctx, storeID)`, unless the consistency is `HIGHER_CONSISTENCY`. The v2 path does the same in `Server.v2Check`.
- **The staleness test:** the result goes into `ResolveCheckRequest.LastCacheInvalidationTime`. A cached entry is used only if `res.LastModified.After(req.LastCacheInvalidationTime)`; otherwise it counts as an invalid hit (`checkCacheInvalidHit`) and is recomputed. The v2 resolver has the same test in `internal/check/check.go`: `!res.LastModified.After(r.lastCacheInvalidationTime)`.
- **`DetermineInvalidationTime`** reads a per-store `ChangelogCacheEntry` from the same in-memory cache.
  - If there is no entry, it calls `InvalidateIfNeeded` and returns the zero time, so every cached entry is treated as valid.
  - If `LastChecked` is older than `--cache-controller-ttl` (`minInvalidationInterval`, default 10s), it calls `InvalidateIfNeeded`.
  - In both cases it returns the currently known `LastModified`.
- **`InvalidateIfNeeded`**
  - It allows one refresh in flight per store, tracked in a `sync.Map`.
  - It runs `findChangesAndInvalidateIfNecessary` in a goroutine, on a 1s timeout.
  - That function calls `ReadChanges` in descending order, with one page of `storage.DefaultPageSize` (50).
  - It writes `ChangelogCacheEntry{LastModified: newest change timestamp, LastChecked: now}`, with TTL `queryCacheTTL`.
  - If there are newer changes, it also invalidates the iterator cache. That is either a full invalidation (`InvalidIteratorCacheKey`) or a partial one, keyed by object#relation and by user plus object type, and it only covers changes within the iterator-cache TTL.
- **Triggers:** only Check and ListObjects requests trigger invalidation; ListObjects does so via `InvalidateIfNeeded` in `pkg/server/commands/list_objects.go`. The Write path does not call the controller.

**Limits.**
- **Off by default.** Both the check query cache and the controller default to off (`pkg/server/config/config.go`).
- **Asynchronous.** The request that triggers a refresh still uses the old invalidation time. `docs/caching.md` ("First Check Staleness") says the first Check after a write, and any Checks until the job finishes, can be stale. Staleness is roughly the controller TTL plus the refresh time.
- **Store-level granularity for Check results.** Only `changes[0]`, the newest timestamp, is compared. Any write to the store invalidates every cached Check result for that store. The finer-grained invalidation applies only to iterator caches.
- **Per process.** The cache is an in-process LRU (`storage.NewInMemoryLRUCache` in `internal/shared/shared.go`). By inference, each replica keeps its own cache and polls the changelog independently.
- **Depends on a successful changelog read.**
  - If `ReadChanges` errors, the iterator cache is fully invalidated but no `ChangelogCacheEntry` is written. The code comment says that includes the case of no changes. In that state `DetermineInvalidationTime` keeps returning the zero time, so cached Check results remain trusted until their TTL.
  - If the 1s timeout is hit, nothing is updated.
- **Possible replica lag (inference).** Postgres `ReadChanges` uses the `MINIMIZE_LATENCY` pool (`pkg/storage/postgres/postgres.go`). If a read replica is configured, its lag could delay detection.
- **Two clocks (inference).** The changelog timestamp comes from the datastore, while the cache entry's `LastModified` comes from the server's `time.Now()`. Clock skew between them could misclassify entries. I did not verify any mitigation.
- **Bypass.** `HIGHER_CONSISTENCY` requests skip the Check cache entirely.

## Q4

**Short answer.** The tupleset relation `Y` must exist on the same type, must be a purely direct relation, must only allow plain object types (no usersets, no wildcards), and must lead to at least one type that defines `X` with an entrypoint. All of this is enforced in `typesystem.NewAndValidate` when a model is written.

**Write path.**
1. `Server.WriteAuthorizationModel` (`pkg/server/authorization_models.go`) calls `commands.WriteAuthorizationModelCommand.Execute` (`pkg/server/commands/write_authzmodel.go`).
2. `Execute` checks the type count and size limits, then calls `typesystem.NewAndValidate` (`pkg/typesystem/typesystem.go`). Any error is returned as `serverErrors.InvalidAuthorizationModelInput`.
3. `NewAndValidate` rejects unsupported schema versions; only 1.1 and 1.2 are accepted (`IsSchemaVersionSupported`). It then calls `validateRelation` for each type and relation, in sorted order.

**Constraints on the tupleset relation `Y` in `X from Y`.**

| # | Constraint | Function | Error |
|---|---|---|---|
| 1 | `Y` must be a relation defined on the **same object type** as the relation holding the rewrite (`t.GetRelation(objectType, tupleset)`) | `isUsersetRewriteValid` | `RelationUndefinedError` |
| 2 | `Y` must be a **direct relation only**: its rewrite must be exactly `this` (`Userset_This`), with no union, intersection, exclusion, computed or TTU rewrite | `isUsersetRewriteValid` | `"the '<type>#<Y>' relation is referenced in at least one tupleset and thus must be a direct relation"` |
| 3 | `X` must be defined on **at least one of `Y`'s directly related user types** | `isUsersetRewriteValid` (1.1+ branch) | `ErrRelationUndefined` ("... does not appear as a relation in any of the directly related user types ...") |
| 4 | `Y`'s type restrictions may only be plain object types. **No userset (`type#rel`) and no wildcard (`type:*`)** is allowed when `IsTuplesetRelation` is true. | `validateTypeRestrictions` | `InvalidRelationTypeError` |
| 5 | Because `Y` is direct, it must have **at least one** type restriction | `validateTypeRestrictions` (assignable ⇒ non-empty) | `AssignableRelationError` |
| 6 | At least one of `Y`'s directly related types must define `X` with an **entrypoint** | `hasEntrypoints`, `Userset_TupleToUserset` case, called from `validateRelation` | `InvalidRelationError` with `ErrNoEntrypoints` or `ErrNoEntryPointsLoop` |

**Notes.**
- `isUsersetRewriteValid` recurses into union, intersection and difference children. The checks therefore apply to a TTU nested anywhere in a rewrite.
- `IsTuplesetRelation` uses `ttuRelations`, which `typesystem.New` builds with `flattenUserset`, so nested TTUs are found there too.
- `isUsersetRewriteValid` has a branch for 1.0 models that only requires `X` to exist somewhere in the model. It cannot be reached through `NewAndValidate`, because 1.0 is rejected earlier with `ErrInvalidSchemaVersion`.
- `hasCycle` returns `false` for a TTU, so TTUs are not part of the computed-relation cycle check.
- Conditions on `Y`'s type restrictions (`[folder with cond]`) are not rejected by these checks. `validateTypeRestrictions` only verifies that the referenced condition exists.
- **Not verified here:** `typesystem.New` also builds a weighted graph with `graph.NewAuthorizationModelGraph` from the external `openfga/language` module, which is not in this checkout. I can't say whether it rejects further TTU shapes.
- **Related checks outside model writes:**
  - At runtime, `relationInvolves` (`RelationInvolvesIntersection` / `RelationInvolvesExclusion`) returns `ErrInvalidModel` if a tupleset relation has a `type#relation` restriction.
  - On tuple writes, `validateTuplesetRestrictions` (`internal/validation/validation.go`) rejects userset and wildcard users on tupleset relations.

## Q5

**Short answer.** Each Check gets its own `BoundedTupleReader`, which holds a counting semaphore. An operator sets its size with `maxConcurrentReadsForCheck`. The default is `math.MaxUint32`, i.e. effectively no limit.

**Mechanism.**
- **Where it is created.** On the v1 path, `CheckQuery.Execute` (`pkg/server/commands/check_command.go`) calls `storagewrappers.NewRequestStorageWrapperWithCache` with `Operation{Method: Check, Concurrency: c.maxConcurrentReads, ...}`. That wrapper builds the innermost layer: `NewBoundedTupleReader(ds, op)` (`pkg/storage/storagewrappers/bounded_datastore.go`). On the v2 path, `CheckQueryV2.Execute` (`pkg/server/commands/check.go`) builds it directly, from `datastoreOp.Concurrency`.
- **The semaphore.** `BoundedTupleReader` holds `limiter: make(chan struct{}, op.Concurrency)`. `Read`, `ReadUserTuple`, `ReadUsersetTuples` and `ReadStartingWithUser` each call `bound()`:
  - `waitForLimiter` sends into the channel and blocks while it is full, giving up if the context ends.
  - `defer b.done()` releases the slot with a non-blocking receive when the method returns.
  - `ReadPage` is not overridden, so it is not bounded.
- **Scope.** The limiter is created per Check request, so it limits one Check's reads, not reads across the server.
- **Reads that skip the limiter.** Contextual tuples (`CombinedTupleReader`) and iterator-cache hits (`CachedDatastore` / `sharediterator`) are served above the bounded layer, so they do not take a slot.
- **Delay metric.** Waits longer than 1ms are recorded in the `datastore_bounded_read_delay_ms` histogram.

**Configuration.**
- Config file key `maxConcurrentReadsForCheck`.
- Flag `--max-concurrent-reads-for-check` (`cmd/run/run.go`).
- Environment variable `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK` or `OPENFGA_MAXCONCURRENTREADSFORCHECK` (`cmd/run/flags.go`).
- The value goes to `Config.MaxConcurrentReadsForCheck` (`pkg/server/config/config.go`), then `server.WithMaxConcurrentReadsForCheck` (`pkg/server/server.go`), then `s.maxConcurrentReadsForCheck`. It is passed on through `commands.WithCheckCommandMaxConcurrentReads` (v1) or `commands.WithCheckQueryV2MaxConcurrentReads` (v2) in `pkg/server/check.go`.
- The default is `math.MaxUint32` (`DefaultMaxConcurrentReadsForCheck`; `.config-schema.json` default 4294967295).
- Embedders can call `server.WithMaxConcurrentReadsForCheck` directly.

**Limits and caveats.**
- **The slot covers the call, not the iteration.** The slot is released when `Read` (or a similar method) returns.
  - For Postgres, `Datastore.read` returns a `sqlcommon.SQLTupleIterator` built from `NewPgxTxnGetRows`, which only builds the SQL. The query runs later, in `GetRows`/`fetchBuffer`, when the iterator is consumed. So for iterator reads, the actual database round-trip appears to happen outside the bounded window.
  - `ReadUserTuple` runs `QueryRow` inside the window.
  - I did not verify MySQL or SQLite in the same depth.
- **BatchCheck does not apply the setting.** `BatchCheckQuery` builds each item's `NewCheckCommand` (`pkg/server/commands/batch_check_command.go`) without `WithCheckCommandMaxConcurrentReads`. Each item therefore uses the default `MaxUint32`, unless the setting is wired in somewhere I did not find.
- **Zero is not validated (inference).** I found no validation that rejects 0. With a capacity of 0, the channel is unbuffered and `done()` only does a non-blocking receive, so every bounded read would block until the context ends.
- **Related settings that are not concurrency limits on reads:**
  - The experimental `datastore_throttling` flag, with `--check-datastore-throttle-threshold` and `--check-datastore-throttle-duration`. Once a Check has made more than *threshold* reads, `bound()` adds a fixed delay to each further read and marks the request throttled.
  - `--resolve-node-breadth-limit`, which caps how many sub-problems at one level `LocalChecker` evaluates concurrently. It bounds the read fan-out only indirectly.
