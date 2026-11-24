# Answers

## Q1

**Short answer.** On the default ("v1") Check path the request goes through a chain built by
`graph.CheckResolverOrderedBuilder.Build` (`internal/graph/builder.go`). The order is always:

1. `CachedCheckResolver` (`internal/graph/cached_resolver.go`), if it is enabled;
2. `DispatchThrottlingCheckResolver` (`internal/graph/dispatch_throttling_check_resolver.go`), if it is enabled;
3. either a `ShadowResolver` (`internal/graph/shadow_resolver.go`) that wraps two `LocalChecker`s
   (main and shadow), or a plain `LocalChecker` (`internal/graph/check.go`).

Only `LocalChecker` reads relationship tuples. The chain is circular: `Build` sets each resolver's
delegate to the next one, and the last resolver's delegate back to the first
(`resolver.SetDelegate(c.resolvers[0])`). When `LocalChecker` dispatches a sub-problem
(`LocalChecker.dispatch`, which calls `c.delegate.ResolveCheck`), that sub-problem starts again at
the head of the chain, so it can hit the cache and be throttled too. `AGENTS.md` ("Resolver Chain")
describes the same structure:
`[CachedCheckResolver]? → [DispatchThrottlingCheckResolver]? → [ShadowResolver | LocalChecker] ⟲`.

**What builds the chain and decides each link.** For each Check request, `Server.Check`
(`pkg/server/check.go`) calls `s.getCheckResolverBuilder(storeID).Build()`. The chain is built per
request because the shadow flag is looked up per store. `getCheckResolverBuilder` passes these options:

| Resolver | Option in `getCheckResolverBuilder` | Present when |
|---|---|---|
| `CachedCheckResolver` | `graph.WithCachedCheckResolverOpts(s.cacheSettings.ShouldCacheCheckQueries(), …)` | `CacheSettings.ShouldCacheCheckQueries()` (`pkg/server/config/cache_settings.go`) is true, which means `CheckCacheLimit > 0 && CheckQueryCacheEnabled` (flags `check-cache-limit` and `check-query-cache-enabled`; the latter defaults to `false` in `pkg/server/config/config.go`). Its options come from `Server.getCheckResolverOptions` (`pkg/server/server.go`): the shared `CheckCache`, `CheckQueryCacheTTL` and jitter. |
| `DispatchThrottlingCheckResolver` | `graph.WithDispatchThrottlingCheckResolverOpts(s.checkDispatchThrottlingEnabled, …)` | `checkDispatchThrottlingEnabled`, set from `CheckDispatchThrottling.Enabled` (flag `check-dispatch-throttling-enabled`, through `server.WithDispatchThrottlingCheckResolverEnabled` in `cmd/run/run.go`). Its threshold, max threshold and frequency come from `getCheckResolverOptions`. |
| `ShadowResolver` instead of `LocalChecker` | `graph.WithShadowResolverEnabled(s.featureFlagClient.Boolean(serverconfig.ExperimentalShadowCheck, storeID))` | The `shadow_check` experimental flag. The default feature-flag client (`pkg/featureflags/client.go`, `defaultClient.Boolean`) returns true if the name appears in `--experimentals`. Otherwise `Build` appends `NewLocalChecker(c.localCheckerOptions...)`. |

Details:

- `CachedCheckResolver.ResolveCheck` returns a cached result if the consistency preference is not
  `HIGHER_CONSISTENCY` and the entry is still valid. Otherwise it calls its delegate and caches the
  result, unless a cycle was detected.
- `DispatchThrottlingCheckResolver.ResolveCheck` calls `threshold.ShouldThrottle` on the request's
  `DispatchCounter`. If that returns true, it marks the request as throttled and waits on the
  throttler before it delegates.
- `ShadowResolver.ResolveCheck` runs `main` synchronously and runs `shadow` in a goroutine with
  `shadowTimeout`, only to compare the results. `ShadowResolver.SetDelegate` sets the delegate only on
  `main`. The shadow `LocalChecker` always gets `WithOptimizations(true)`.
- `LocalChecker.ResolveCheck` checks depth (`ErrResolutionDepthExceeded`), cycles, self-defining
  tuples and `PathExists` before it builds handlers. Tuples are then read from the
  `RelationshipTupleReader` in the context, for example in `checkDirect`, `checkDirectUserTuple`
  and `checkPublicAssignable`.

**Before the chain runs.** These steps happen first, and none of them is a `CheckResolver`:

- `Server.Check` validates the request, runs `checkAuthz`, and resolves the typesystem.
- `commands.CheckQuery.Execute` (`pkg/server/commands/check_command.go`) does the following:
  - runs `validateCheckRequest`;
  - gets `LastCacheInvalidationTime` from the cache controller;
  - builds the `ResolveCheckRequest`;
  - wraps the datastore with `storagewrappers.NewRequestStorageWrapperWithCache` and puts it in the
    context.

That wrapper is not a resolver, but every tuple read passes through it. From outermost to innermost
(`pkg/storage/storagewrappers/request.go`):

1. `CombinedTupleReader` (contextual tuples);
2. the optional shared-iterator datastore;
3. the optional `CachedDatastore` (Check iterator cache);
4. `BoundedTupleReader`;
5. the server's datastore. In `NewServerWithOpts` (`pkg/server/server.go`) that datastore is wrapped
   by `NewCachedOpenFGADatastore`, then, unless context propagation is enabled, by
   `ContextTracerWrapper`, and then the backend.

**Exception.** If the `weighted_graph_check` experimental flag is on for the store, `Server.Check`
first tries `s.v2Check`. That path uses `commands.CheckQueryV2` and `internal/check.Resolver`, and
does not use this resolver chain. It falls back to the v1 chain on errors that are not terminal
(`isV2TerminalError`).

## Q2

**Types with their own `ReadUserTuple` method.** The method is declared on
`storage.RelationshipTupleReader` in `pkg/storage/storage.go`. That interface is embedded in
`TupleBackend`, which is embedded in `OpenFGADatastore`. The signature is
`ReadUserTuple(ctx, store string, filter ReadUserTupleFilter, options ReadUserTupleOptions) (*openfgav1.Tuple, error)`.
`ReadUserTupleFilter` is a type alias for `ReadFilter`.

- Backends:
  - `memory.MemoryBackend` (`pkg/storage/memory/memory.go`);
  - `postgres.Datastore` (`pkg/storage/postgres/postgres.go`);
  - `mysql.Datastore` (`pkg/storage/mysql/mysql.go`);
  - `sqlite.Datastore` (`pkg/storage/sqlite/sqlite.go`).
- Wrappers that override the method:
  - `storagewrappers.BoundedTupleReader` (`bounded_datastore.go`): applies the concurrency limit, then delegates;
  - `storagewrappers.CombinedTupleReader` (`combinedtuplereader.go`): checks contextual tuples first, then delegates;
  - `storagewrappers.CachedTupleReader` (`cached_reader.go`): delegates without caching;
  - `storagewrappers.ContextTracerWrapper` (`context.go`): swaps in the query context, then delegates.
- Wrappers that get the method by embedding the interface:
  - `storagewrappers.CachedDatastore`;
  - `storagewrappers.RequestStorageWrapper`;
  - `storagewrappers.cachedOpenFGADatastore` (`model_caching.go`);
  - `sharediterator.IteratorDatastore`.
- Generated mocks and test doubles (not production):
  - `MockTupleBackend`, `MockRelationshipTupleReader` and `MockOpenFGADatastore` in
    `internal/mocks/mock_storage.go`, generated by mockgen from `pkg/storage/storage.go`;
  - `slowDataStorage` in `internal/mocks/mock_slow_storage.go`.

**Production callers.** These are the only non-test, non-mock call sites:

- `LocalChecker.checkDirectUserTuple` in `internal/graph/check.go` (v1 Check). It is used by
  `LocalChecker.checkDirect` when `shouldCheckDirectTuple` is true. It treats `storage.ErrNotFound`
  as "not allowed".
- `Resolver.specificType` in `internal/check/check.go` (the weighted-graph "v2" Check). It is
  reached from the direct-edge `SpecificType` case and from `specificTypeAndRelation`. It also maps
  `ErrNotFound` to not allowed.
- The forwarding wrappers listed above (`BoundedTupleReader`, `CombinedTupleReader`,
  `CachedTupleReader`, `ContextTracerWrapper`). They call the inner reader's `ReadUserTuple`.

`LocalChecker.checkPublicAssignable` deliberately uses `ReadUsersetTuples` instead of
`ReadUserTuple`, so that it can use the iterator cache (see the comment there).

**If the signature changed, these would have to change:**

- The interface in `pkg/storage/storage.go`. If the change involves `ReadUserTupleFilter`, note that
  it is an alias of `ReadFilter`, so changing that alias would also affect `Read` and `ReadPage`.
- The four backend implementations: memory, postgres, mysql and sqlite.
- The four overriding wrappers. The embedding wrappers pick up the new signature automatically.
- The two production callers: `checkDirectUserTuple` and `Resolver.specificType`.
- The generated mocks. Regenerate them with `make generate-mocks`; don't edit them by hand
  (`AGENTS.md`). `slowDataStorage` must be edited by hand.
- Tests that call it or set expectations on it:
  - the shared datastore conformance suite `pkg/storage/test/tuples.go`;
  - `pkg/storage/{postgres,mysql,sqlite}/*_test.go`;
  - `pkg/storage/storagewrappers/*_test.go`;
  - `internal/graph/check_test.go`;
  - `internal/check/check_test.go`;
  - `pkg/server/server_test.go`.
- Any external datastore implementation, because the interface is public in `pkg/`. I can't see
  those from this repository.

## Q3

**Mechanism.** A cached Check result is compared against the time of the store's last write. That
time comes from the database changelog and is maintained by a cache controller.

- **Entry timestamps.** When `CachedCheckResolver.ResolveCheck` (`internal/graph/cached_resolver.go`)
  stores a result, it saves a `CheckResponseCacheEntry{LastModified: time.Now(), …}`. When it reads
  one back, it uses it only if `res.LastModified.After(req.LastCacheInvalidationTime)`. Otherwise it
  increments `check_cache_invalid_hit_count` and recomputes. The v2 path does the same in
  `internal/check/check.go`, `Resolver.isCached`, using `!res.LastModified.After(r.lastCacheInvalidationTime)`.
- **Where `LastCacheInvalidationTime` comes from.** `CheckQuery.Execute`
  (`pkg/server/commands/check_command.go`) sets it, and so does `Server.v2Check` (`pkg/server/check.go`).
  Both call `CacheController.DetermineInvalidationTime(ctx, storeID)`, but only when the consistency
  preference is not `HIGHER_CONSISTENCY`. `HIGHER_CONSISTENCY` bypasses the cache entirely.
- **The cache controller.** It is `InMemoryCacheController` in `internal/cachecontroller/cache_controller.go`.
  - `DetermineInvalidationTime` reads a `ChangelogCacheEntry` (`LastModified`, `LastChecked`) under
    `storage.ChangelogCacheKey(storeID)` and returns its `LastModified`, the time of the last known
    write.
  - If the entry is missing, it starts an asynchronous invalidation and returns the zero time, so
    every cached entry counts as valid.
  - If `time.Since(entry.LastChecked) > minInvalidationInterval`, it starts an asynchronous refresh.
    `minInvalidationInterval` is the cache-controller TTL.
- **The asynchronous refresh.** `InvalidateIfNeeded` allows one in-flight run per store, tracked in
  `inflightInvalidations`. `findChangesAndInvalidateIfNecessary` then does the following:
  - It reads the newest changes with `ReadChanges` in descending order. The page size is
    `storage.DefaultPageSize`, which is 50, and the call has a 1-second timeout.
  - It writes a new `ChangelogCacheEntry`, where `LastModified` is the newest change's timestamp.
    That entry's TTL is the Check query cache TTL.
  - If there are new changes, it also invalidates iterator-cache entries:
    - if every fetched change is within the iterator TTL, it invalidates the whole store with
      `invalidateIteratorCache`;
    - otherwise it invalidates per object#relation and per user/objectType, with
      `invalidateIteratorCacheByObjectRelation` and `invalidateIteratorCacheByUserAndObjectType`.
    - These write `InvalidEntityCacheEntry` markers. `CachedDatastore` checks them through
      `findInCache` and `isInvalidAt` in `pkg/storage/storagewrappers/cached_datastore.go`.
- **The write path does no invalidation itself.** `pkg/server/commands/write.go` doesn't touch the
  cache. Staleness is detected only by reading the changelog later.
- **Wiring.** `shared.NewSharedDatastoreResources` (`internal/shared/shared.go`) creates the
  controller only when `CacheSettings.ShouldCreateCacheController()` is true. That requires a cache
  to be enabled and `cacheController.enabled` (flag `cache-controller-enabled`), which defaults to
  false. In every other case `NoopCacheController` is used. Its `DetermineInvalidationTime` always
  returns zero time, so cached results expire only by TTL (`check-query-cache-ttl`, default 10s).

**Limits.**

- **It is eventually consistent.** After a write, a cached result can be served until the next
  refresh, which happens at most every `cache-controller-ttl` (default 10s), plus however long the
  asynchronous job takes. `docs/caching.md` ("Cache Controller", "First Check Staleness") documents
  that the request which triggers the refresh, and any that arrive before it finishes, can still get
  stale data. This is also what happens whenever the changelog entry is missing (zero time).
- **Granularity is the whole store.** Any write to the store invalidates every cached Check result
  for that store. Only the iterator cache gets targeted invalidation.
- **Only 50 changes are read.** If there are more recent changes, the controller falls back to
  invalidating the whole iterator cache.
- **Failures.** If the changelog read times out after 1s, nothing is updated and a later request
  retries. If it returns an error, only the iterator cache is invalidated (`invalidateIteratorCache`)
  and the changelog entry is not refreshed.
- **Per replica.** Caches and the controller live in each replica's memory. Writes made through any
  replica are still seen, because each one reads the shared database changelog.
- **Shadow cache.** The shadow/v2 path has a separate `ShadowCacheController`.
- **BatchCheck.** `BatchCheckCommand` defaults to `NoopCacheController`, but `Server.BatchCheck`
  passes the shared resources through `WithBatchCheckCacheOptions`.
- **Unverified.** I did not verify how a changelog with zero entries behaves. The comment on
  `findChangesDescending` says `ReadChanges` returns an error in that case, which takes the error
  branch above.

## Q4

For a tuple-to-userset rewrite `X from Y`, the tupleset relation is `Y`. Every
`WriteAuthorizationModel` request enforces the constraints below in
`typesystem.NewAndValidate` (`pkg/typesystem/typesystem.go`). The call comes from
`WriteAuthorizationModelCommand.Execute` (`pkg/server/commands/write_authzmodel.go`), which wraps any
error as `InvalidAuthorizationModelInput`. `NewAndValidate` calls `validateRelation` for every
type/relation in sorted order. `validateRelation` runs `isUsersetRewriteValid`, then
`validateTypeRestrictions`, then `hasEntrypoints`, then `HasCycle`.

| Constraint on `Y` | Where it is enforced |
|---|---|
| `Y` must be a relation defined on the same object type as the rewrite. | `isUsersetRewriteValid`, case `*openfgav1.Userset_TupleToUserset`: `t.GetRelation(objectType, tupleset)`, returning `RelationUndefinedError` / `ErrRelationUndefined` |
| `Y` must be purely direct: its rewrite must be exactly `this` (`Userset_This`), with no union, computed userset or other rewrite. | Same case: `reflect.TypeOf(tuplesetRewrite.GetUserset()) != reflect.TypeOf(&openfgav1.Userset_This{})`, with the error "the '%s#%s' relation is referenced in at least one tupleset and thus must be a direct relation" |
| `X` (the computed relation) must be defined on at least one of `Y`'s directly related user types. | Same case, schema 1.1 branch: loop over `tuplesetRelation.GetTypeInfo().GetDirectlyRelatedUserTypes()`, otherwise `ErrRelationUndefined` "… does not appear as a relation in any of the directly related user types". The schema 1.0 branch only requires `X` somewhere in the model, but `NewAndValidate` rejects schemas other than 1.1/1.2 earlier (`IsSchemaVersionSupported`), so that branch looks unreachable from this path. |
| `Y`'s type restrictions must be plain object types: no userset (`type#relation`) and no wildcard (`type:*`). | `validateTypeRestrictions` (rule 4 in its doc comment): when `related.GetRelationOrWildcard() != nil` and `t.IsTuplesetRelation(objectType, relationName)` is true, it returns `InvalidRelationTypeError`. `IsTuplesetRelation` looks up `t.ttuRelations`, which is filled in `typesystem.New`. |
| The general type-restriction rules also apply to `Y`: as a direct relation it must have at least one type restriction, every referenced type must exist, and every referenced condition must exist. | `validateTypeRestrictions` (`AssignableRelationError`, `InvalidRelationTypeError`, `RelationConditionError`) |
| At least one of `Y`'s directly related types must define `X` with an entrypoint. | `hasEntrypoints`, case `*openfgav1.Userset_TupleToUserset`; otherwise `ErrNoEntrypoints` / `ErrNoEntryPointsLoop` |

Two related points:

- `relationInvolves` also returns `ErrInvalidModel` if a tupleset relation has a `type#relation`
  restriction. It is a query-time helper, not part of `NewAndValidate`, so the
  `validateTypeRestrictions` rule above is what enforces this on write.
- `typesystem.New` also builds an authorization-model graph with `graph.NewAuthorizationModelGraph`
  from the external `openfga/language` module, and can return an error. I did not inspect that
  module, so I can't say whether it adds further tupleset checks.

## Q5

**Mechanism.** Each Check request gets its own `storagewrappers.BoundedTupleReader`
(`pkg/storage/storagewrappers/bounded_datastore.go`). Its `limiter` is a buffered channel of
capacity `Operation.Concurrency`.

- `Read`, `ReadUserTuple`, `ReadUsersetTuples` and `ReadStartingWithUser` each call `bound()` first.
- `bound()` calls `waitForLimiter`, which sends into the channel. It blocks until a slot is free or
  the context is cancelled, and records `datastore_bounded_read_delay_ms` when the wait is longer
  than 1ms.
- Each method then releases the slot with `defer b.done()` when the inner call returns.
- Other methods, such as `ReadPage`, are promoted from the embedded reader and are not bounded.

**Wiring.**

- v1 Check: `CheckQuery.Execute` (`pkg/server/commands/check_command.go`) builds the reader through
  `storagewrappers.NewRequestStorageWrapperWithCache`, passing
  `&storagewrappers.Operation{Method: apimethod.Check, Concurrency: c.maxConcurrentReads, …}`.
  `NewRequestStorageWrapperWithCache` (`request.go`) puts `NewBoundedTupleReader` innermost, under
  the iterator cache and the contextual-tuple reader, so iterator-cache hits don't use a slot.
  `c.maxConcurrentReads` is set by `WithCheckCommandMaxConcurrentReads(s.maxConcurrentReadsForCheck)`
  in `Server.Check` (`pkg/server/check.go`). The default is
  `defaultMaxConcurrentReadsForCheck = math.MaxUint32`, which is effectively unlimited.
- Weighted-graph v2 Check: `Server.v2Check` passes the same value through
  `WithCheckQueryV2MaxConcurrentReads`, and `CheckQueryV2.Execute` (`pkg/server/commands/check.go`)
  calls `NewBoundedTupleReader` directly.
- Shadow checks: the shadow `LocalChecker` uses the same context values (`context.WithoutCancel`),
  so as far as I can tell its reads count against the same per-request limiter.

**Operator configuration.** Use any of these:

- the `--max-concurrent-reads-for-check` flag (`cmd/run/run.go`);
- the `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK` environment variable (also
  `OPENFGA_MAXCONCURRENTREADSFORCHECK`; see `cmd/run/flags.go`);
- the `maxConcurrentReadsForCheck` key in `config.yaml`, which is searched in `/etc/openfga`,
  `$HOME/.openfga` and `.` (`cmd/root.go`).

The value is `Config.MaxConcurrentReadsForCheck` (`pkg/server/config/config.go`, default
`DefaultMaxConcurrentReadsForCheck = math.MaxUint32`; also `.config-schema.json`). It is passed to
`server.WithMaxConcurrentReadsForCheck` in `cmd/run/run.go`. That option's doc comment in
`pkg/server/server.go` explains how to size it from the datastore's `MaxOpenConns`, the replica count
and the RPS, for example 100 connections with 1 replica at 100 RPS gives 1.

**Related knobs and limits.**

- **Per-request read throttling.** This is separate from the concurrency cap. It is on only when the
  `datastore_throttling` experimental flag is set and `check-datastore-throttle-threshold > 0`. After
  that many reads in one request, each further read is delayed by `check-datastore-throttle-duration`.
  It is implemented in `BoundedTupleReader.bound` and wired by `WithCheckDatastoreThrottler`.
- **Breadth of evaluation.** `--resolve-node-breadth-limit` (default 10) limits how many sub-problems
  at one level are evaluated concurrently (`LocalChecker` `concurrencyLimit`). It is not a read limit.
- **BatchCheck is not capped.** `BatchCheckQuery.Execute` (`pkg/server/commands/batch_check_command.go`)
  calls `NewCheckCommand` without `WithCheckCommandMaxConcurrentReads`. Each check in a batch
  therefore uses the `MaxUint32` default. Only `max-concurrent-checks-per-batch-check` bounds how
  many checks in a batch run at once.
- **The slot covers the call, not the iteration.** The slot is held only until the reader method
  returns. For Postgres and MySQL, `Read`/`ReadUsersetTuples`/`ReadStartingWithUser` return a lazy
  `sqlcommon.SQLTupleIterator`, which runs the query on the first `Next` (`fetchBuffer`). So for those
  methods the slot can be released before the SQL query runs. `ReadUserTuple` runs its query eagerly
  (`QueryRow`). I did not check SQLite's iterator in the same detail.
- **Zero.** I found no validation that rejects a value of 0. With 0 the channel would be unbuffered,
  and reads would block until the context ends. I haven't verified this at runtime.
