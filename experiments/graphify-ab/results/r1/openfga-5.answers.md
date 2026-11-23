# Answers

## Q1

**Short answer.** On the default (v1) Check path, the request goes through a chain of `graph.CheckResolver`s in this order:

1. `CachedCheckResolver`, if enabled
2. `DispatchThrottlingCheckResolver`, if enabled
3. `LocalChecker`, or a `ShadowResolver` that wraps a main and a shadow `LocalChecker`

`LocalChecker` is the only resolver that reads tuples. The last resolver's delegate points back to the first one, so the chain is circular and every recursive sub-problem passes through the whole chain again.

**Where the chain is built.**
- `pkg/server/check.go`, `(*Server).Check`: for v1 it calls `s.getCheckResolverBuilder(storeID).Build()`. It passes the returned resolver to `commands.NewCheckCommand`, and `(*CheckQuery).Execute` in `pkg/server/commands/check_command.go` calls `c.checkResolver.ResolveCheck(...)`.
- `pkg/server/check.go`, `(*Server).getCheckResolverBuilder`: calls `graph.NewOrderedCheckResolvers(...)` with these options:
  - `WithLocalCheckerOpts`, which sets the breadth limit, optimizations, max resolution depth, planner, upstream timeout and logger;
  - `WithLocalShadowCheckerOpts`;
  - `WithShadowResolverEnabled(featureFlag ExperimentalShadowCheck)`;
  - `WithShadowResolverOpts`;
  - `WithCachedCheckResolverOpts(s.cacheSettings.ShouldCacheCheckQueries(), ...)`;
  - `WithDispatchThrottlingCheckResolverOpts(s.checkDispatchThrottlingEnabled, ...)`.

  The cache and throttling options themselves come from `(*Server).getCheckResolverOptions` in `pkg/server/server.go`. `BatchCheck` uses the same builder (`pkg/server/batch_check.go`).
- `internal/graph/builder.go`, `CheckResolverOrderedBuilder.Build`: this function fixes the order, whatever order the options were passed in.
  1. It appends `NewCachedCheckResolver(...)` if `cachedCheckResolverEnabled` is set.
  2. It appends `NewDispatchThrottlingCheckResolver(...)` if `dispatchThrottlingCheckResolverEnabled` is set.
  3. If `shadowResolverEnabled` is set, it appends `NewShadowChecker(NewLocalChecker(main opts), NewLocalChecker(shadow opts))`. Otherwise it appends `NewLocalChecker(...)`.
  4. It wires each resolver's delegate with `SetDelegate(resolvers[i+1])`. The last one gets `SetDelegate(resolvers[0])`.
  5. It returns `resolvers[0]` and a closer.

  `ShadowResolver.SetDelegate` (`internal/graph/shadow_resolver.go`) sets the delegate only on `main`. `LocalChecker.dispatch` (`internal/graph/check.go`) increments depth and calls `c.delegate.ResolveCheck`, which closes the loop.

**What decides whether each resolver is present.**

| Resolver | Present when | Source |
|---|---|---|
| `CachedCheckResolver` | `CacheSettings.ShouldCacheCheckQueries()` is true, i.e. `CheckCacheLimit > 0 && CheckQueryCacheEnabled` (flags `check-query-cache-enabled` and `check-cache-limit`). If present, a request with `HIGHER_CONSISTENCY` skips the cache lookup inside `ResolveCheck`. | `pkg/server/config/cache_settings.go` |
| `DispatchThrottlingCheckResolver` | `s.checkDispatchThrottlingEnabled` is true (flag `check-dispatch-throttling-enabled`). It throttles once `DispatchCounter` passes the threshold, then delegates. | `internal/graph/dispatch_throttling_check_resolver.go`, `ResolveCheck` |
| `LocalChecker` / `ShadowResolver` | Always. The shadow variant is used when the per-store feature flag `shadow_check` (`serverconfig.ExperimentalShadowCheck`) is on. | `builder.go`, `Build` |

**Caveats.**
- **v2 path.** If the per-store feature flag `weighted_graph_check` (`ExperimentalWeightedGraphCheck`) is on, `Server.Check` first tries `s.v2Check`. That path uses `commands.NewCheckQuery` (`pkg/server/commands/check.go`) and `internal/check.Resolver`, and does not use this resolver chain at all. It falls back to the v1 chain on non-terminal errors (`isV2TerminalError`).
- **Storage wrappers between `LocalChecker` and the datastore.** These are not resolvers. `CheckQuery.Execute` builds them with `storagewrappers.NewRequestStorageWrapperWithCache` (`pkg/storage/storagewrappers/request.go`) and puts them in the context. From the datastore up, the stack is:
  1. `BoundedTupleReader`
  2. optional `CachedDatastore`, the check iterator cache
  3. optional shared-iterator `IteratorDatastore`
  4. `CombinedTupleReader`, which serves contextual tuples first

## Q2

**Types that implement `ReadUserTuple`.** The method is declared on the `storage.RelationshipTupleReader` interface (`pkg/storage/storage.go`). That interface is embedded in `TupleBackend`, which `OpenFGADatastore` embeds. The signature is:

```go
ReadUserTuple(ctx, store string, filter ReadUserTupleFilter, options ReadUserTupleOptions) (*openfgav1.Tuple, error)
```

`ReadUserTupleFilter` is an alias for `ReadFilter`, and `ReadUserTupleOptions` is `{Consistency ConsistencyOptions}`.

| Type | File | How |
|---|---|---|
| `postgres.Datastore` | `pkg/storage/postgres/postgres.go` | own method (SQL, uses `options.Consistency` to choose the pool) |
| `mysql.Datastore` | `pkg/storage/mysql/mysql.go` | own method (ignores options) |
| `sqlite.Datastore` | `pkg/storage/sqlite/sqlite.go` | own method (ignores options) |
| `memory.MemoryBackend` | `pkg/storage/memory/memory.go` | own method |
| `storagewrappers.BoundedTupleReader` | `bounded_datastore.go` | own method: concurrency bound, read and item counting, then delegates |
| `storagewrappers.CombinedTupleReader` | `combinedtuplereader.go` | own method: matches contextual tuples first, then delegates |
| `storagewrappers.CachedTupleReader` | `cached_reader.go` | own method: pure delegation |
| `storagewrappers.ContextTracerWrapper` | `context.go` | own method: detaches the query context, then delegates |
| `CachedDatastore`, `IteratorDatastore` (sharediterator), `cachedOpenFGADatastore` (`model_caching.go`), `RequestStorageWrapper` (`request.go`) | storagewrappers | promoted from an embedded `RelationshipTupleReader` or `OpenFGADatastore`; no method of their own |
| Mocks: `MockRelationshipTupleReader`, `MockTupleBackend`, `MockOpenFGADatastore` (+ recorders), `slowDataStorage` | `internal/mocks/mock_storage.go` (generated by mockgen from `pkg/storage/storage.go`), `internal/mocks/mock_slow_storage.go` | test only |

**Production callers.** These are the call sites outside wrappers that only delegate, tests and mocks:

1. `internal/graph/check.go`, `(*LocalChecker).checkDirectUserTuple`. This is the v1 Check direct-tuple lookup. It reads the datastore from context with `storage.RelationshipTupleReaderFromContext`.
2. `internal/check/check.go`, `(*Resolver).specificType`. This is the v2 weighted-graph Check, reached through `r.datastore`.

The four wrappers with their own method (`BoundedTupleReader`, `CombinedTupleReader`, `CachedTupleReader`, `ContextTracerWrapper`) also call `ReadUserTuple` on their inner reader. `pkg/storage/test/tuples.go` calls it too, but that is the shared datastore test suite, not production.

**If the signature changed, these would also have to change:**
- The interface in `pkg/storage/storage.go`. If the parameter types changed, `ReadUserTupleFilter`, `ReadUserTupleOptions` and possibly `ReadFilter` would change too; `ReadFilter` is shared with `Read`/`ReadPage`, because of the alias.
- The 4 backend implementations: postgres, mysql, sqlite and memory.
- The 4 wrappers with their own method. Embedding wrappers pick up the change automatically.
- Both production callers: `checkDirectUserTuple` and `Resolver.specificType`.
- The generated mocks. Regenerate `internal/mocks/mock_storage.go` with `make generate-mocks`, and hand-edit `mock_slow_storage.go`.
- `pkg/storage/test/tuples.go` and any `_test.go` files that call it or set `EXPECT().ReadUserTuple(...)`.
- Docs: `docs/check/README.md` names `storage#ReadUserTuple` in its traces.
- Any out-of-tree datastore that implements `OpenFGADatastore`, since this is a public interface in `pkg/storage`.

## Q3

**Mechanism.** OpenFGA compares timestamps.

- **When an entry is stored.** Each Check query-cache entry is a `CheckResponseCacheEntry{LastModified: time.Now(), CheckResponse}`. `CachedCheckResolver.ResolveCheck` (`internal/graph/cached_resolver.go`) stores it with a jittered TTL. Results that report a cycle are never cached.
- **When an entry is read.** `CachedCheckResolver.ResolveCheck` treats the entry as valid only if `res.LastModified.After(req.LastCacheInvalidationTime)`. Otherwise it increments `check_cache_invalid_hit_count` and recomputes through its delegate.
- **Where `LastCacheInvalidationTime` comes from.** `CheckQuery.Execute` (`pkg/server/commands/check_command.go`) sets it, unless consistency is `HIGHER_CONSISTENCY`, by calling `CacheController.DetermineInvalidationTime(ctx, storeID)`. The v2 path does the same in `Server.v2Check` (`pkg/server/check.go`).
- **The cache controller.** `InMemoryCacheController` in `internal/cachecontroller/cache_controller.go` works as follows:
  - `DetermineInvalidationTime` reads a per-store `ChangelogCacheEntry` (key `storage.ChangelogCacheKey(storeID)`, `cc.<storeID>`) from the shared check cache and returns its `LastModified`, the time of the store's last write.
    - If that entry is missing, it returns the zero time and starts `InvalidateIfNeeded`.
    - If `LastChecked` is older than `minInvalidationInterval` (`cache-controller-ttl`), it also starts `InvalidateIfNeeded`.
  - `InvalidateIfNeeded` runs one async invalidation per store at a time (`inflightInvalidations`). That job, `findChangesAndInvalidateIfNecessary`, reads the newest changelog page with `ReadChanges(SortDesc)` under a 1 s timeout. It then stores `ChangelogCacheEntry{LastModified: newest change timestamp, LastChecked: now}` with TTL `queryCacheTTL`.
  - Any query-cache entry set before that write time is then stale.
  - The same job also invalidates iterator-cache entries:
    - If every change on the page falls inside the iterator TTL, it invalidates everything for the store (`InvalidIteratorCacheKey`).
    - Otherwise it invalidates per object#relation and per user plus object type (`invalidateIteratorCacheByObjectRelation`, `invalidateIteratorCacheByUserAndObjectType`).
    - If the changelog read fails, it invalidates the whole store's iterator cache.
- **The cache key.** It is `storage.CheckCacheKey(store, object, relation, user, req.GetInvariantCacheKey())`, which includes the model ID, contextual tuples and context (`docs/caching.md`, "Check Query Cache"). A new model therefore never reuses old entries.

**When it is active.** The controller exists only if `CacheSettings.ShouldCreateCacheController()` is true: a cache is enabled and `CacheControllerEnabled` is set (flag `cache-controller-enabled`, default `false`). See `internal/shared/shared.go`, `NewSharedDatastoreResources`, and `pkg/server/config/config.go`. Otherwise `NoopCacheController` always returns the zero time, so a cached result is trusted until its TTL expires (`check-query-cache-ttl`, default 10s).

**Limits.**
- **Writes don't invalidate.** Invalidation is pull-based: only Check requests trigger it, at most once per `cache-controller-ttl` per store, and ListObjects requests trigger it when the ListObjects iterator cache is enabled. Stale results can be served for up to the controller TTL after a write (`docs/caching.md`, "Cache Controller").
- **First Check after a write can be stale.** Invalidation is asynchronous and `DetermineInvalidationTime` returns the old value (or the zero time) right away. So the first Check after a write, and any Checks before the job finishes, can return stale data even if more than the controller TTL has passed (`docs/caching.md`, "First Check Staleness").
- **Per replica only.** Caches and controllers are in memory, so each replica detects writes on its own (`docs/caching.md`, overview note).
- **Store-wide granularity for query results.** Any write to the store invalidates all earlier query-cache entries for that store, even unrelated ones.
- **Iterator invalidation sees one changelog page.** It looks only at the newest page, which `docs/caching.md` says is currently 50 changes.
- **If the changelog entry is lost** (eviction from the LRU, or expiry after `queryCacheTTL`), the zero time is returned until the async refresh completes. During that window, cached entries are treated as valid. I inferred this from `DetermineInvalidationTime`; it is not stated in the docs.
- **Two clocks are compared.** Entries are stamped with the server's `time.Now()`, but the write time comes from the datastore's changelog timestamp, so clock skew between them could affect the comparison. This is my inference; I did not find this addressed in the code or docs.
- **`HIGHER_CONSISTENCY` bypasses the cache.** Such requests skip both the cache lookup and the invalidation-time lookup.

## Q4

**Constraints on the tupleset relation `Y` in `X from Y`.** All of these are enforced when a model is written. The path is:

`Server.WriteAuthorizationModel` (`pkg/server/authorization_models.go`, which first runs the proto `req.Validate()`) → `WriteAuthorizationModelCommand.Execute` (`pkg/server/commands/write_authzmodel.go`) → `typesystem.NewAndValidate`.

`NewAndValidate` builds `ttuRelations` in `New`, then calls `validateRelation` for each type#relation in sorted order (`pkg/typesystem/typesystem.go`). A failure is returned as `serverErrors.InvalidAuthorizationModelInput(err)`.

| Constraint | Where |
|---|---|
| `Y` must be a relation defined on the **same object type** as the rewrite; otherwise `RelationUndefinedError`. | `(*TypeSystem).isUsersetRewriteValid`, `case *openfgav1.Userset_TupleToUserset` |
| `Y` must be a **direct relation only**: its rewrite must be exactly `this`, with no union, intersection, difference or computed userset. Error: "the 'type#Y' relation is referenced in at least one tupleset and thus must be a direct relation". | same function (`reflect.TypeOf(tuplesetRewrite.GetUserset()) != Userset_This`) |
| `X` must exist on **at least one of the types directly related to `Y`**. For schema 1.1/1.2 this is checked against `Y`'s `DirectlyRelatedUserTypes`. A 1.0 branch would only need `X` to exist somewhere, but `NewAndValidate` rejects schema versions other than 1.1/1.2 before reaching it (`IsSchemaVersionSupported`). | same function |
| `Y`'s type restrictions must be **plain object types**, with no `type#relation` userset and no `type:*` wildcard. Any restriction with a relation or wildcard on a relation where `IsTuplesetRelation` is true returns `InvalidRelationTypeError`. Conditions on the restriction (`[folder with cond]`) are not rejected by this rule. | `(*TypeSystem).validateTypeRestrictions`, rule 4; `(*TypeSystem).IsTuplesetRelation` |
| General rules also apply to `Y`: being directly assignable, it must have at least one type restriction, and every referenced type must exist. | `validateTypeRestrictions`, rules 1–3 |
| The relation containing the TTU must have an entrypoint through at least one of `Y`'s directly related types, otherwise `ErrNoEntrypoints` or `ErrNoEntryPointsLoop`. | `hasEntrypoints`, `Userset_TupleToUserset` case, called from `validateRelation` |

**Related checks outside model writes.**
- At tuple-write time (and for contextual tuples), `validateTuplesetRestrictions` in `internal/validation/validation.go`, called by `ValidateTupleForWrite` and `ValidateTupleForRead`, rejects tuples on a tupleset relation whose user is a wildcard or a userset or not a valid object, and errors if the tupleset relation has a rewrite.
- A later helper in `typesystem.go` (around the `relationInvolves` logic) also errors with `ErrInvalidModel` if a tupleset restriction has a relation. That is a defensive check on an already-built typesystem, not the write-time enforcement.

## Q5

**Mechanism.** Each Check request gets its own `storagewrappers.BoundedTupleReader` (`pkg/storage/storagewrappers/bounded_datastore.go`), which is a counting semaphore.

- **Construction.** `NewBoundedTupleReader` creates `limiter: make(chan struct{}, op.Concurrency)`.
- **Acquire.** Every `Read`, `ReadUserTuple`, `ReadUsersetTuples` and `ReadStartingWithUser` first calls `bound(ctx, op)`. That calls `waitForLimiter`, which blocks sending into the channel until a slot frees up or the context is cancelled.
- **Release.** `done()` drains a slot with `defer` when the method returns.
- **Metrics.** Waits longer than 1 ms are recorded in `openfga_datastore_bounded_read_delay_ms`. The wrapper also counts reads and items for `DatastoreQueryCount` and `DatastoreItemCount`.
- **Optional throttling.** In the same `bound`, if datastore throttling is enabled (feature flag `ExperimentalDatastoreThrottling` with `checkDatastoreThrottleThreshold` and `checkDatastoreThrottleDuration`), each read after the threshold count is delayed by the throttle duration and the request is marked throttled.

**How the limit reaches it.**
- **v1.** `Server.Check` passes `commands.WithCheckCommandMaxConcurrentReads(s.maxConcurrentReadsForCheck)` (`pkg/server/check.go`). `CheckQuery.Execute` passes it as `storagewrappers.Operation{Concurrency: c.maxConcurrentReads}` to `NewRequestStorageWrapperWithCache` (`pkg/storage/storagewrappers/request.go`), which wraps the datastore in `NewBoundedTupleReader` innermost. The iterator cache and shared-iterator layers sit above it, so cache hits do not take a slot.
- **v2.** `WithCheckQueryV2MaxConcurrentReads(s.maxConcurrentReadsForCheck)` feeds `CheckQueryV2.Execute`, which also calls `NewBoundedTupleReader` (`pkg/server/commands/check.go`).

**Operator configuration.** The setting is `maxConcurrentReadsForCheck`.

| Form | Value |
|---|---|
| Flag | `--max-concurrent-reads-for-check` |
| Config key | `maxConcurrentReadsForCheck` |
| Env | `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK` (also `OPENFGA_MAXCONCURRENTREADSFORCHECK`) |
| Default | `math.MaxUint32` (4294967295), i.e. effectively unbounded |

The wiring for these is in:
- `cmd/run/run.go`: the flag definition and `server.WithMaxConcurrentReadsForCheck(config.MaxConcurrentReadsForCheck)`;
- `cmd/run/flags.go`: the bindings;
- `pkg/server/config/config.go`: `DefaultMaxConcurrentReadsForCheck` and `Config.MaxConcurrentReadsForCheck`;
- `.config-schema.json`.

The doc comment on `WithMaxConcurrentReadsForCheck` (`pkg/server/server.go`) advises sizing it from expected Check RPS, the number of replicas and `Datastore.MaxOpenConns`. For example, with 100 connections, 1 replica and 1 RPS, set 100; with 2 replicas, set 50; at 100 RPS, set 1.

**Limits.**
- **Per request, not global.** The limit applies to each Check request's wrapper, not across the server.
- **BatchCheck does not use it.** `BatchCheckQuery` builds each `NewCheckCommand` without `WithCheckCommandMaxConcurrentReads` (`pkg/server/commands/batch_check_command.go`), so those checks keep the default `MaxUint32`.
- **Other APIs have their own settings.** ListObjects, for example, uses `max-concurrent-reads-for-list-objects`.
- **Bounds starting a read, not reading its rows.** The slot is released when the read method returns. For the iterator-returning methods, the remaining rows are consumed after the slot is freed.
- **A value of 0 is not rejected.** I found no validation against 0 in the files I read. A capacity of 0 would make every read block until the context ends. That is inferred from the channel semantics and not verified by a test.
