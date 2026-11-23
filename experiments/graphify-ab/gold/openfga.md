# Gold key facts: OpenFGA at 7d89a566

Graded per fact: covered (1), partial (0.5) or missed (0). A fact is covered when the answer states
it, in any words, with the names that identify it (a type, function, file or flag). Partial: the
gist without the identifying names, or right with one material error. A wrong claim that
contradicts a fact makes it missed.

## Q1. Check resolver chain (5 facts)

- **O1a.** `Server.Check` (pkg/server/check.go) runs the v1 path through `commands.NewCheckCommand(...).Execute`, unless the experimental weighted-graph check (`ExperimentalWeightedGraphCheck`) is enabled for the store, in which case it runs the v2 `commands.NewCheckQuery` and falls back to v1 on a non-terminal error.
- **O1b.** The v1 chain is built by `graph.CheckResolverOrderedBuilder.Build` (internal/graph/builder.go), configured by the server (`Server.getCheckResolverBuilder`) from its options.
- **O1c.** Order: `CachedCheckResolver` (present only when the check query cache is enabled), then `DispatchThrottlingCheckResolver` (only when dispatch throttling is enabled), then `LocalChecker`.
- **O1d.** The chain is circular: the last resolver's delegate is set to the first, so recursive sub-problems dispatched by `LocalChecker` go back through the cache and throttler.
- **O1e.** When the experimental shadow check is enabled, a `ShadowResolver` (`NewShadowChecker`) wraps a main and a shadow `LocalChecker` in the last position.

## Q2. ReadUserTuple implementations and callers (4 facts)

- **O2a.** Datastore implementations: `MemoryBackend` (pkg/storage/memory) and the MySQL, Postgres and SQLite `Datastore`s (pkg/storage/mysql, postgres, sqlite).
- **O2b.** Storage wrappers in pkg/storage/storagewrappers also implement it: `BoundedTupleReader`, `CachedTupleReader`, `CombinedTupleReader` and `ContextTracerWrapper`.
- **O2c.** Production callers: `LocalChecker.checkDirectUserTuple` (internal/graph/check.go) and the v2 check's `Resolver.specificType` (internal/check/check.go).
- **O2d.** It is declared on the storage reader interface (`RelationshipTupleReader`, pkg/storage/storage.go), so a signature change also means the generated mocks are regenerated and every implementation and wrapper is updated.

## Q3. Check cache staleness (5 facts)

- **O3a.** A cache controller decides it: `InMemoryCacheController` in internal/cachecontroller (a no-op controller when disabled). `DetermineInvalidationTime` reads the store's changelog cache entry and, when it is missing or older than the controller's TTL, triggers an asynchronous `InvalidateIfNeeded`.
- **O3b.** `findChangesAndInvalidateIfNecessary` reads the store's changelog (`ReadChanges`, newest first). If there are changes newer than the cached ones, it invalidates iterator caches: the whole store when all changes are within the iterator TTL window, otherwise per object#relation and per user and object type. It writes invalidation entries rather than deleting cache entries.
- **O3c.** Writes do not invalidate the cache directly; invalidation is lazy and discovered by later Checks, so a result can be stale until the next asynchronous check completes.
- **O3d.** A cached Check result is used only if its last-modified time is after the request's `LastCacheInvalidationTime` (internal/graph/cached_resolver.go; internal/check/check.go), which the check command sets from the cache controller.
- **O3e.** Requests with `HIGHER_CONSISTENCY` bypass the cache.

## Q4. Tuple-to-userset tupleset constraints (4 facts)

- **O4a.** They are enforced by type-system validation, `typesystem.NewAndValidate` (pkg/typesystem/typesystem.go), which `WriteAuthorizationModel` (pkg/server/commands/write_authzmodel.go) calls, returning an invalid-model input error.
- **O4b.** The tupleset relation must be a direct relation: its rewrite must be direct assignment only (`this`), not a computed, union or other rewrite (`isUsersetRewriteValid`: "is referenced in at least one tupleset and thus must be a direct relation").
- **O4c.** Its type restrictions must be plain object types: no `type#relation` userset and no `type:*` wildcard (`validateTypeRestrictions`, `InvalidRelationTypeError`).
- **O4d.** The same validation applies when a model is loaded to serve requests (pkg/typesystem/resolver.go wraps the error as `ErrInvalidModel`).

## Q5. Concurrent datastore reads in a Check (4 facts)

- **O5a.** `storagewrappers.BoundedTupleReader` (bounded_datastore.go) limits them with a buffered-channel semaphore sized to the limit, around `Read`, `ReadUserTuple`, `ReadUsersetTuples` and `ReadStartingWithUser`.
- **O5b.** It is created per request, by the request storage wrapper (`NewRequestStorageWrapper` / `NewRequestStorageWrapperWithCache`, request.go) and in the v2 check command.
- **O5c.** The limit is `MaxConcurrentReadsForCheck`: flag `--max-concurrent-reads-for-check`, environment variable `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK`, default `math.MaxUint32` (effectively unbounded).
- **O5d.** Optional, experimental datastore throttling adds a delay once a request passes a threshold of reads, and marks the request as throttled.
