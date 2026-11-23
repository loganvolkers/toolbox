# Answers

## Q1

**Short answer.** The default (v1) Check path builds a chain of `graph.CheckResolver`s for each request, in this order:

1. `CachedCheckResolver`, if enabled.
2. `DispatchThrottlingCheckResolver`, if enabled.
3. Either `ShadowResolver`, which wraps two `LocalChecker`s, or a plain `LocalChecker`.

Only the `LocalChecker` reads tuples. It reads them through a per-request stack of storage wrappers that sits between it and the datastore.

**Entry point and which path runs.** `Server.Check` in `pkg/server/check.go`:

- If the `weighted_graph_check` experimental flag is on (`serverconfig.ExperimentalWeightedGraphCheck`, `pkg/server/config/config.go`), it first calls `Server.v2Check`. That path builds a `commands.CheckQueryV2` (`pkg/server/commands/check.go`), which runs `check.Resolver` from `internal/check/check.go`. It uses strategies (default, weight2, recursive) and an edge-level cache, not the `CheckResolver` chain below. If v2 fails with an error that `isV2TerminalError` does not treat as terminal, Check falls back to v1.
- The v1 path calls `s.getCheckResolverBuilder(storeID).Build()` (`pkg/server/check.go`), then `commands.NewCheckCommand(...).Execute` (`pkg/server/commands/check_command.go`), which calls `checkResolver.ResolveCheck`.

**Code that builds the chain.**

- `Server.getCheckResolverBuilder` (`pkg/server/check.go`) calls `graph.NewOrderedCheckResolvers` with the builder options.
- `Server.getCheckResolverOptions` (`pkg/server/server.go`) supplies the cache options (the shared `CheckCache`, TTL and jitter) and the dispatch-throttling options (thresholds and a constant-rate throttler).
- `CheckResolverOrderedBuilder.Build` (`internal/graph/builder.go`) appends the resolvers in a fixed order and links them with `SetDelegate`. Each resolver delegates to the next, and the last one delegates back to the first, so the list is circular. `LocalChecker.dispatch` (`internal/graph/check.go`) sends each sub-problem to `c.delegate`, which means sub-problems go back through the cache and throttler.

**What decides whether each resolver is present:**

| Position | Type (file) | Present when |
|---|---|---|
| 1 | `CachedCheckResolver` (`internal/graph/cached_resolver.go`) | `CacheSettings.ShouldCacheCheckQueries()` is true (`pkg/server/config/cache_settings.go`). That requires `CheckCacheLimit > 0` and `CheckQueryCacheEnabled` (flags `--check-cache-limit` and `--check-query-cache-enabled`; caching is off by default). The server passes this in through `WithCachedCheckResolverOpts`. |
| 2 | `DispatchThrottlingCheckResolver` (`internal/graph/dispatch_throttling_check_resolver.go`) | `s.checkDispatchThrottlingEnabled`, set by `--check-dispatch-throttling-enabled` through `server.WithDispatchThrottlingCheckResolverEnabled` in `cmd/run/run.go` (off by default). The server passes this in through `WithDispatchThrottlingCheckResolverOpts`. |
| 3a | `ShadowResolver` (`internal/graph/shadow_resolver.go`), built by `NewShadowChecker(main, shadow)` with two `LocalChecker`s | The feature flag `shadow_check` (`ExperimentalShadowCheck`) is true for the store (`WithShadowResolverEnabled`). The default feature-flag client (`pkg/featureflags/client.go`) ignores the store id and only checks whether the flag is in `--experimentals`. |
| 3b | `LocalChecker` (`internal/graph/check.go`) | Always present, either inside the `ShadowResolver` (as `main`) or on its own. It is configured through `WithLocalCheckerOpts`: breadth limit, depth limit, planner, upstream timeout, and optimizations (the `enable-check-optimizations` flag). |

**Effect of each resolver:**

- `CachedCheckResolver.ResolveCheck` returns a cached result unless the request's consistency is `HIGHER_CONSISTENCY`.
- `DispatchThrottlingCheckResolver.ResolveCheck` throttles a request once its dispatch counter exceeds the threshold.
- `ShadowResolver.SetDelegate` only sets the delegate of the `main` checker. The shadow `LocalChecker` keeps itself as its delegate (`NewLocalChecker` sets `checker.delegate = checker`), so it runs asynchronously outside the cache and throttler.

**Storage layer below the `LocalChecker`.**

`CheckQuery.Execute` (`pkg/server/commands/check_command.go`) builds `storagewrappers.NewRequestStorageWrapperWithCache` (`pkg/storage/storagewrappers/request.go`) and puts it in the context with `storage.ContextWithRelationshipTupleReader`. `LocalChecker` gets it back with `storage.RelationshipTupleReaderFromContext`.

Going from the outermost wrapper inward, a read passes through:

1. `CombinedTupleReader`, which serves contextual tuples.
2. `sharediterator.IteratorDatastore`, only if `SharedIteratorEnabled`.
3. `CachedDatastore` (the iterator cache), only if `ShouldCacheCheckIterators()`.
4. `BoundedTupleReader`, which applies the concurrency limit and datastore throttle.
5. `s.datastore`. In `server.go` this is `cachedOpenFGADatastore` (model cache), wrapping `ContextTracerWrapper` (only when context propagation to the datastore is disabled), wrapping the backend.

## Q2

**The interface.** `ReadUserTuple(ctx, store string, filter ReadUserTupleFilter, options ReadUserTupleOptions) (*openfgav1.Tuple, error)` is declared on `storage.RelationshipTupleReader` in `pkg/storage/storage.go`. That interface is embedded in `TupleBackend`, which is in turn embedded in `OpenFGADatastore`. `ReadUserTupleFilter` is a type alias for `ReadFilter`, and `ReadUserTupleOptions` holds a `Consistency` field (both in `pkg/storage/storage.go`).

**Types that define the method.**

Datastore backends:

- `memory.MemoryBackend` (`pkg/storage/memory/memory.go`)
- `mysql.Datastore` (`pkg/storage/mysql/mysql.go`)
- `postgres.Datastore` (`pkg/storage/postgres/postgres.go`)
- `sqlite.Datastore` (`pkg/storage/sqlite/sqlite.go`)

Wrappers that define it explicitly:

- `storagewrappers.BoundedTupleReader` (`bounded_datastore.go`): concurrency bound and counting.
- `storagewrappers.CombinedTupleReader` (`combinedtuplereader.go`): checks contextual tuples first, then delegates.
- `storagewrappers.CachedTupleReader` (`cached_reader.go`): delegates directly with no caching.
- `storagewrappers.ContextTracerWrapper` (`context.go`): runs the call on a detached context.

Types that only get the method by embedding the interface: `RequestStorageWrapper`, `CachedDatastore`, `cachedOpenFGADatastore` (`model_caching.go`), and `sharediterator.IteratorDatastore`.

Mocks and test doubles: the gomock types `MockTupleBackend`, `MockRelationshipTupleReader` and `MockOpenFGADatastore` in `internal/mocks/mock_storage.go`, and `slowDataStorage` in `internal/mocks/mock_slow_storage.go`.

**Production callers.** Leaving aside the wrappers that just pass the call through, only two places call it:

1. `LocalChecker.checkDirectUserTuple` in `internal/graph/check.go` (the v1 Check path). It reads the datastore from the context and calls `ds.ReadUserTuple` with the request's object, relation and user and its consistency preference.
2. `Resolver.specificType` in `internal/check/check.go` (the v2 weighted-graph Check path). It calls `r.datastore.ReadUserTuple` only when no matching contextual tuple was found.

The pass-through calls are in `BoundedTupleReader`, `CombinedTupleReader`, `CachedTupleReader` and `ContextTracerWrapper`. `docs/check/README.md` also refers to `storage#ReadUserTuple` in its traces.

**What a signature change would affect:**

- The interface declaration in `pkg/storage/storage.go`.
- The four backends: memory, mysql, postgres and sqlite.
- The four wrappers that define the method: bounded, combined, cached reader and context.
- The two production callers above.
- `internal/mocks/mock_storage.go`, which should be regenerated. Its header gives the command: `mockgen -source pkg/storage/storage.go -destination internal/mocks/mock_storage.go -package mocks OpenFGADatastore`.
- `internal/mocks/mock_slow_storage.go`.
- The shared datastore conformance suite `pkg/storage/test/tuples.go`, plus tests that call the method or set `EXPECT().ReadUserTuple(...)`. These include `internal/graph/check_test.go`, `internal/check/check_test.go`, `pkg/server/server_test.go`, the `storagewrappers/*_test.go` files and the per-backend `*_test.go` files.
- Any out-of-tree storage adapter that implements `OpenFGADatastore`. I couldn't verify these from this repository.

Changing `ReadUserTupleFilter` itself has a wider effect, because it is the same type as `ReadFilter`, which `Read` and `ReadPage` also use.

## Q3

**Short answer.** OpenFGA does not invalidate the cache when tuples are written. The Write path (`pkg/server/commands/write.go`) never touches the cache. Instead, a cache controller polls the store's changelog on later reads and records a "last write" time per store. A cached Check result is treated as stale if it was stored at or before that time. Without the controller, entries simply expire after their TTL.

**Where it lives.** `internal/cachecontroller/cache_controller.go`:

- The `CacheController` interface.
- `InMemoryCacheController`, with `DetermineInvalidationTime`, `InvalidateIfNeeded` and `findChangesAndInvalidateIfNecessary`.

It is created in `internal/shared/shared.go` only when `CacheSettings.ShouldCreateCacheController()` is true: some cache must be enabled and `--cache-controller-enabled` must be set (default off). Otherwise `NoopCacheController` is used, and it always returns the zero time.

**The mechanism:**

1. At the start of each Check, `CheckQuery.Execute` (`pkg/server/commands/check_command.go`), or `Server.v2Check` (`pkg/server/check.go`) on the v2 path, calls `CacheController.DetermineInvalidationTime(ctx, storeID)`. It skips this when consistency is `HIGHER_CONSISTENCY`. The result goes into `ResolveCheckRequest.LastCacheInvalidationTime`, or `check.Config.LastCacheInvalidationTime` for v2.
2. `DetermineInvalidationTime` looks up the store's `ChangelogCacheEntry`, stored under `storage.ChangelogCacheKey`.
   - **On a miss**, it triggers an async `InvalidateIfNeeded` and returns the zero time, so every cached entry is accepted for now.
   - **On a hit**, it returns `entry.LastModified`, the timestamp of the most recent change. It also triggers an async refresh, but only if `LastChecked` is older than `minInvalidationInterval` (`--cache-controller-ttl`, default 10s).
3. `findChangesAndInvalidateIfNecessary` runs in a goroutine, with at most one in flight per store (tracked in the `inflightInvalidations` `sync.Map`).
   - It calls `ReadChanges` in descending order (`findChangesDescending`, one page of `storage.DefaultPageSize`) with a 1-second timeout.
   - It stores a `ChangelogCacheEntry{LastModified: newest change timestamp, LastChecked: now}`, with a TTL of the query-cache TTL.
   - For the iterator cache, it writes invalidation markers. If every change in the page falls inside the iterator TTL window, it writes a store-wide marker (`InvalidIteratorCacheKey`). Otherwise it writes per-change markers: `InvalidIteratorByObjectRelationCacheKey` and `InvalidIteratorByUserObjectTypeCacheKey`.
   - If `ReadChanges` fails, it invalidates the whole store's iterator cache.
4. Validity is then checked in three places:
   - **v1 query cache:** `CachedCheckResolver.ResolveCheck` (`internal/graph/cached_resolver.go`) accepts an entry only if `res.LastModified.After(req.LastCacheInvalidationTime)`. Otherwise it increments `check_cache_invalid_hit_count` and recomputes.
   - **v2 query cache:** `Resolver.isCached` (`internal/check/check.go`) applies the same rule.
   - **Iterator cache:** `findInCache` and `isInvalidAt` (`pkg/storage/storagewrappers/cached_datastore.go`) and `CachedTupleReader` (`cached_reader.go`) compare an entry's timestamp with the invalidation markers and drop entries that are stale.

**Limits:**

- **Eventually consistent.** Invalidation is asynchronous and rate-limited by `--cache-controller-ttl`. On a changelog-entry miss it returns the zero time and explicitly accepts stale hits. The code comment calls this "an acceptable trade-off". The `--check-query-cache-enabled` help text (`cmd/run/run.go`) says caching makes Check and ListObjects eventually consistent.
- **Coarse for Check results.** Any write to the store invalidates every cached Check result older than that write, not only the affected ones. Only the iterator cache is invalidated at object#relation or user+object-type granularity.
- **Only one page of changes is read.** For partial invalidation, the code considers only one page of the changelog plus the iterator-TTL window.
- **Per process, in memory.** The cache and controller live in each server's memory (`storage.NewInMemoryLRUCache` in `internal/shared/shared.go`), so replicas do not share invalidation state.
- **Controller off by default.** Without it, stale results last until the TTL expires (`--check-query-cache-ttl`, default 10s, plus optional jitter from `CacheTTLJitterPercentage`).
- **`HIGHER_CONSISTENCY` bypasses the cache entirely.** See `tryCache` in `CachedCheckResolver.ResolveCheck` and the consistency check in `isCached`.
- **ListObjects behaves differently.** It calls `InvalidateIfNeeded` directly when its iterator cache is enabled (`pkg/server/commands/list_objects.go`).

## Q4

Constraints are enforced when a model is written. `WriteAuthorizationModelCommand.Execute` (`pkg/server/commands/write_authzmodel.go`) calls `typesystem.NewAndValidate`. Any error is returned as `serverErrors.InvalidAuthorizationModelInput`. `NewAndValidate` (`pkg/typesystem/typesystem.go`) calls `validateRelation` for every type and relation, in sorted order. `validateRelation` applies the following rules to the tupleset relation `Y` in `X from Y`:

1. **`Y` must be defined on the same object type** as the relation that contains the TTU. `isUsersetRewriteValid`, in its `Userset_TupleToUserset` case, calls `t.GetRelation(objectType, tupleset)` and returns `RelationUndefinedError` otherwise. TTUs nested in a union, intersection or difference are checked recursively.
2. **`Y` must be a pure direct relation**, meaning its rewrite is exactly `this` (just `[types]`), with no union, computed relation or other rewrite. This is also in `isUsersetRewriteValid`. It returns the error "the '`type#Y`' relation is referenced in at least one tupleset and thus must be a direct relation".
3. **`X` must exist on at least one of `Y`'s directly related types** (schema 1.1 and 1.2, per `IsSchemaVersionSupported`). This is also in `isUsersetRewriteValid`. It returns `ErrRelationUndefined` with "… does not appear as a relation in any of the directly related user types …".
4. **`Y`'s type restrictions cannot be usersets (`type#rel`) or wildcards (`type:*`).** `validateTypeRestrictions` handles this as its rule 4. Whenever a restriction has `GetRelationOrWildcard() != nil` and `IsTuplesetRelation(objectType, relationName)` is true, it returns `InvalidRelationTypeError`. `IsTuplesetRelation` uses the `ttuRelations` map built in `typesystem.New` by `flattenUserset`.
5. **The general rules for direct relations also apply to `Y`.** It must have at least one type restriction (`AssignableRelationError`). Referenced types must exist. Any condition on a restriction must be defined in the model (`RelationConditionError`). All of these are checked in `validateTypeRestrictions`.
6. **Entrypoints.** In `hasEntrypoints`, `Userset_TupleToUserset` case, at least one of `Y`'s directly related types must give `X` an entrypoint. Otherwise `validateRelation` returns `InvalidRelationError` with `ErrNoEntrypoints` or `ErrNoEntryPointsLoop`. `Y` itself must be defined (`"undefined type definition for 'type#Y'"`).

The cycle check `hasCycle` treats a TTU as a leaf, so `X from Y` edges never count as cycles. Before validation, `Execute` also enforces model-wide limits: the maximum number of type definitions and the maximum model size in bytes.

**Related checks that are not part of writing a model:**

- `TypeSystem.relationInvolves` (`pkg/typesystem/typesystem.go`) returns `ErrInvalidModel` at evaluation time if a tupleset's restriction carries a relation. This is a defensive duplicate of rule 4.
- When tuples are written or read, `validateTuplesetRestrictions` in `internal/validation/validation.go` (called from `ValidateTupleForRead` and `ValidateTupleForWrite`) rejects tuples on a tupleset relation if the relation has a non-`this` rewrite, or if the user is a wildcard or not a plain object (for example a userset).

Parsing of the DSL `from` syntax happens in the external `openfga/language` package, not in this repository, so I haven't checked what it enforces.

## Q5

**Mechanism.** The limit is per request. It lives in `storagewrappers.BoundedTupleReader` (`pkg/storage/storagewrappers/bounded_datastore.go`). `NewBoundedTupleReader` creates a buffered channel, `limiter := make(chan struct{}, op.Concurrency)`. The methods `Read`, `ReadUserTuple`, `ReadUsersetTuples` and `ReadStartingWithUser` each work like this:

1. They call `bound()`. Inside it, `waitForLimiter` blocks sending to the channel until a slot is free or the context is cancelled. Waits longer than a threshold are recorded in `concurrentReadDelayMsHistogram`.
2. They delegate to the wrapped reader.
3. A deferred `done()` releases the slot.

For iterator-returning methods, the slot is released when the call returns the iterator, not after the iterator has been consumed. So the limit covers in-flight read calls. Whether the backend's query work happens entirely inside that call, or partly while the iterator is consumed, depends on each backend. I haven't verified that.

`bound()` can also throttle. If datastore throttling is enabled, `threshold > 0` and the request's read count exceeds the threshold, it sleeps for `throttleTime` and marks the request as throttled.

**Where it is wired into Check:**

- **v1:** `CheckQuery.Execute` (`pkg/server/commands/check_command.go`) builds `NewRequestStorageWrapperWithCache` with `Operation{Method: Check, Concurrency: c.maxConcurrentReads, ...}`. The bounded reader sits beneath the iterator cache and shared-iterator wrappers (`request.go`), so cache hits don't use a slot. A new wrapper, and therefore a new limiter, is built for each Check.
- **v2 (weighted graph):** `CheckQueryV2.Execute` (`pkg/server/commands/check.go`) wraps the datastore in `NewBoundedTupleReader(q.datastore, &q.datastoreOp)`, with `datastoreOp.Concurrency` set from `WithCheckQueryV2MaxConcurrentReads`.
- Both values come from `s.maxConcurrentReadsForCheck` in `Server.Check` and `Server.v2Check` (`pkg/server/check.go`).

**Configuration for operators:**

- **Setting:** `maxConcurrentReadsForCheck` (`Config.MaxConcurrentReadsForCheck`, `pkg/server/config/config.go`). The default is `DefaultMaxConcurrentReadsForCheck = math.MaxUint32`, which is effectively unlimited.
- **How to set it:**
  - CLI flag `--max-concurrent-reads-for-check` (`cmd/run/run.go`).
  - Environment variable `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK` or `OPENFGA_MAXCONCURRENTREADSFORCHECK` (`cmd/run/flags.go`).
  - Config key `maxConcurrentReadsForCheck` (`.config-schema.json`).
- **Path into the server:** `run.go` passes it to `server.WithMaxConcurrentReadsForCheck`.
- **Sizing:** the doc comment on that option in `pkg/server/server.go` advises sizing it from expected RPS, the replica count and `Datastore.MaxOpenConns`. For example, 100 connections, one replica and 1 RPS gives 100.

**Related settings:**

- **Datastore throttling:**
  - It needs the `datastore_throttling` experimental flag.
  - It is configured with `--check-datastore-throttle-threshold` and `--check-datastore-throttle-duration`, which go through `server.WithCheckDatabaseThrottle`.
  - The server passes them on through `WithCheckDatastoreThrottler` or `WithCheckQueryV2DatastoreThrottling`.
- **`--resolve-node-breadth-limit`** (default 10) limits concurrent sub-problem evaluation rather than reads.
- **The datastore connection pool** is the global cap across all requests.

**Caveats:**

- **Zero is not rejected for Check.** `Config.VerifyServerSettings` only rejects zero for `MaxConcurrentReadsForListUsers`. A Check value of 0 would create an unbuffered channel, which as far as I can tell from the code would block every read until the context is cancelled. I haven't tested this.
- **BatchCheck does not appear to apply the limit.** `BatchCheckQuery` (`pkg/server/commands/batch_check_command.go`) builds each check with `NewCheckCommand` but never passes `WithCheckCommandMaxConcurrentReads`. Those checks therefore seem to use the `MaxUint32` default, inferred from reading the code and not tested.
