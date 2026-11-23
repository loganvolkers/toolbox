# Answers

## Q1

**Short answer.** On the default (v1) path, the chain is `[CachedCheckResolver]? → [DispatchThrottlingCheckResolver]? → (ShadowResolver | LocalChecker)`. It is built by `CheckResolverOrderedBuilder.Build()` in `internal/graph/builder.go`. The `LocalChecker` (or the main `LocalChecker` inside `ShadowResolver`) is the only resolver that reads tuples. It reads them through a per-request storage-wrapper stack.

**Where the chain is built**
- `Server.Check` (`pkg/server/check.go`) calls `s.getCheckResolverBuilder(storeID).Build()` for each request.
- `getCheckResolverBuilder` (same file) calls `graph.NewOrderedCheckResolvers(...)` with options. The options come from `Server.getCheckResolverOptions` (`pkg/server/server.go`).
- `Build()` appends the resolvers in a fixed order, independent of the order of the options. It then links each resolver to the next with `SetDelegate`. The last resolver delegates back to the first, so the chain is a circular list. `Build()` returns `c.resolvers[0]` and a closer that closes every resolver.

**The resolvers, in order, and what decides whether each one is present**

| # | Type (file) | Present when |
|---|---|---|
| 1 | `CachedCheckResolver` (`internal/graph/cached_resolver.go`) | `WithCachedCheckResolverOpts(s.cacheSettings.ShouldCacheCheckQueries(), ...)` is true. `CacheSettings.ShouldCacheCheckQueries()` (`pkg/server/config/cache_settings.go`) returns `CheckCacheLimit > 0 && CheckQueryCacheEnabled`. The flags are `--check-query-cache-enabled` (default false) and `--check-cache-limit`. The resolver uses the shared `CheckCache` through `WithExistingCache`. |
| 2 | `DispatchThrottlingCheckResolver` (`internal/graph/dispatch_throttling_check_resolver.go`) | `s.checkDispatchThrottlingEnabled`, set by `WithDispatchThrottlingCheckResolverEnabled` (`pkg/server/server.go`). The flag is `--check-dispatch-throttling-enabled` (default `false`, `DefaultCheckDispatchThrottlingEnabled` in `pkg/server/config/config.go`). Its `ResolveCheck` throttles when the request's `DispatchCounter` exceeds the threshold, then delegates. |
| 3a | `ShadowResolver` (`internal/graph/shadow_resolver.go`), wrapping two `LocalChecker`s | The per-store feature flag `shadow_check` (`ExperimentalShadowCheck`) is on, via `WithShadowResolverEnabled(s.featureFlagClient.Boolean(...))`. The main checker runs synchronously and its result is returned. The shadow checker runs afterwards in a goroutine, only to compare and log. `SetDelegate` is applied only to `main`. |
| 3b | `LocalChecker` (`internal/graph/check.go`) | Always present. It is used directly when shadow mode is off. |

With the default feature-flag client, `featureflags.NewDefaultClient` (`pkg/featureflags/client.go`), a flag is on for every store when its name is listed in `--experimentals`.

**What happens before and between the resolvers**
- `CheckQuery.Execute` (`pkg/server/commands/check_command.go`) runs first:
  1. It validates the request (`validateCheckRequest`).
  2. It gets the cache-invalidation time from the cache controller.
  3. It builds the `ResolveCheckRequest`.
  4. It wraps the datastore with `storagewrappers.NewRequestStorageWrapperWithCache` (`pkg/storage/storagewrappers/request.go`) and stores the wrapper in the context with `storage.ContextWithRelationshipTupleReader`.
  5. It calls `checkResolver.ResolveCheck`.
- `LocalChecker.ResolveCheck` checks depth, cycles and the typesystem first. Its handlers, such as `checkDirectUserTuple`, then read tuples through `storage.RelationshipTupleReaderFromContext(ctx)`.
- Sub-problems are dispatched to the delegate, which is the head of the chain. So recursive checks pass through the cache and the throttler again.

**The reader stack under the resolvers** (outermost first, from `NewRequestStorageWrapperWithCache`)

| Layer | Present when |
|---|---|
| `CombinedTupleReader` (contextual tuples) | Always |
| `sharediterator.IteratorDatastore` | `CacheSettings.SharedIteratorEnabled` |
| `CachedDatastore` | `ShouldCacheCheckIterators()` |
| `BoundedTupleReader` (concurrency limit and datastore throttling) | Always |
| `cachedOpenFGADatastore` | Always. Added at server start in `pkg/server/server.go`. |
| `ContextTracerWrapper` | Unless `contextPropagationToDatastore` is set. Added at server start in `pkg/server/server.go`. |

**Alternative path.** If the per-store flag `weighted_graph_check` (`ExperimentalWeightedGraphCheck`) is on, `Server.Check` calls `s.v2Check` first. This path does not use the resolver chain. It uses `commands.CheckQueryV2.Execute` (`pkg/server/commands/check.go`), which builds a `check.Resolver` (`internal/check/check.go`). That resolver caches per edge (`isCached`) and reads through `NewBoundedTupleReader` and an optional `NewCachedTupleReader`. If v2 returns an error that `isV2TerminalError` does not treat as terminal, `Server.Check` falls back to the v1 chain.

## Q2

**Short answer.** Four datastores and four wrappers implement `ReadUserTuple` explicitly. Two production functions call it: `LocalChecker.checkDirectUserTuple` (v1) and `Resolver.specificType` (v2). A signature change would touch the interface, every explicit implementer, both callers, the generated mocks and the storage test suite.

**The interface.** `ReadUserTuple` is declared on `storage.RelationshipTupleReader` (`pkg/storage/storage.go`). That interface is embedded in `TupleBackend`, which is embedded in `OpenFGADatastore`. The declaration is:

```go
ReadUserTuple(ctx context.Context, store string, filter ReadUserTupleFilter, options ReadUserTupleOptions) (*openfgav1.Tuple, error)
```

It must return `storage.ErrNotFound` when no tuple matches. `ReadUserTupleFilter` is a type alias for `ReadFilter`. `ReadUserTupleOptions` holds only `Consistency`.

**Types with their own `ReadUserTuple` method**

| Type | File | Notes |
|---|---|---|
| `MemoryBackend` | `pkg/storage/memory/memory.go` | Datastore |
| `postgres.Datastore` | `pkg/storage/postgres/postgres.go` | Datastore. Uses `options`. |
| `mysql.Datastore` | `pkg/storage/mysql/mysql.go` | Datastore. Ignores `options`. |
| `sqlite.Datastore` | `pkg/storage/sqlite/sqlite.go` | Datastore. Ignores `options`. |
| `BoundedTupleReader` | `pkg/storage/storagewrappers/bounded_datastore.go` | Concurrency bound and item counting |
| `CombinedTupleReader` | `pkg/storage/storagewrappers/combinedtuplereader.go` | Answers from contextual tuples first |
| `CachedTupleReader` | `pkg/storage/storagewrappers/cached_reader.go` | Plain pass-through (v2 path) |
| `ContextTracerWrapper` | `pkg/storage/storagewrappers/context.go` | Calls the inner store with `context.WithoutCancel` |

Mocks also implement it: the generated `MockTupleBackend`, `MockRelationshipTupleReader` and `MockOpenFGADatastore` in `internal/mocks/mock_storage.go`, and the hand-written `slowDataStorage` in `internal/mocks/mock_slow_storage.go`.

These types get the method by embedding, with no method of their own:
- `CachedDatastore` (`cached_datastore.go`). It does not cache `ReadUserTuple`.
- `sharediterator.IteratorDatastore`
- `RequestStorageWrapper` (`request.go`)
- `cachedOpenFGADatastore` (`model_caching.go`)

**Production callers**
- `LocalChecker.checkDirectUserTuple` (`internal/graph/check.go`). This is the v1 direct-tuple lookup, selected in `checkDirect` when `shouldCheckDirectTuple` is true.
- `Resolver.specificType` (`internal/check/check.go`). This is the v2 weighted-graph Check, used when contextual tuples do not already answer.
- The wrappers forward the call to their inner reader: `BoundedTupleReader`, `CombinedTupleReader`, `CachedTupleReader` and `ContextTracerWrapper`.

There are no other non-test callers. `pkg/storage/test/tuples.go` calls it, but that file is the shared datastore test suite.

**What would have to change if the signature changed**
1. The interface method in `pkg/storage/storage.go`.
   - If the filter type changed, `ReadUserTupleFilter` is an alias of `ReadFilter`, so `Read` would change too unless the alias were split.
2. All four datastore implementations: memory, postgres, mysql and sqlite.
3. The four wrappers with explicit methods: `BoundedTupleReader`, `CombinedTupleReader`, `CachedTupleReader` and `ContextTracerWrapper`.
   - The embedding wrappers pick up the change automatically.
4. Both callers: `internal/graph/check.go` and `internal/check/check.go`.
5. The mocks:
   - Regenerate `internal/mocks/mock_storage.go`. It is generated by `mockgen -source pkg/storage/storage.go`; use `make generate-mocks`.
   - Hand-edit `internal/mocks/mock_slow_storage.go`.
6. The tests:
   - `pkg/storage/test/tuples.go`.
   - About 86 call or `EXPECT()` sites across 9 `_test.go` files: `internal/graph/check_test.go`, `internal/check/check_test.go`, `pkg/server/server_test.go`, the postgres, mysql and sqlite tests, and the bounded, combined and cached-reader wrapper tests.
7. Optionally, the link in `docs/check/README.md`.

## Q3

**Short answer.** OpenFGA does not invalidate the Check cache when tuples are written. A cached result is treated as stale when it was stored before the store's last recorded change. The time of that change comes from the changelog, which the cache controller reads asynchronously and at most once every `cache-controller-ttl` per store. Without the cache controller, a cached result is used until its TTL expires.

**The Write path.** `pkg/server/write.go` and `pkg/server/commands/write.go` contain no cache logic.

**The staleness test**
- Each cached Check result is a `CheckResponseCacheEntry{LastModified: time.Now(), ...}`, set in `CachedCheckResolver.ResolveCheck` (`internal/graph/cached_resolver.go`). The entry is stored with TTL `storage.JitteredTTL(cacheTTL, jitter)`.
- When `CachedCheckResolver.ResolveCheck` finds an entry, it uses it only if `res.LastModified.After(req.LastCacheInvalidationTime)`. Otherwise it counts an invalid hit (`check_cache_invalid_hit_count`) and recomputes.
- The v2 path does the same in `Resolver.isCached` (`internal/check/check.go`).
- `HIGHER_CONSISTENCY` requests skip the cache entirely.

**Where `LastCacheInvalidationTime` comes from**
- `CheckQuery.Execute` (`pkg/server/commands/check_command.go`), and `Server.v2Check`, call `CacheController.DetermineInvalidationTime(ctx, storeID)` once per request. They do not call it for `HIGHER_CONSISTENCY` requests.
- `InMemoryCacheController` (`internal/cachecontroller/cache_controller.go`) is created in `internal/shared/shared.go` only when `ShouldCreateCacheController()` is true: a cache exists and `--cache-controller-enabled` is set (default `false`). Otherwise `NoopCacheController` returns the zero time, and every entry counts as valid until its TTL expires (`--check-query-cache-ttl`, default 10s).

**How `InMemoryCacheController` works**
1. `DetermineInvalidationTime` reads the `ChangelogCacheEntry` under `storage.ChangelogCacheKey(storeID)` and returns its `LastModified`.
   - If the entry is missing, it starts `InvalidateIfNeeded` and returns the zero time.
   - If `LastChecked` is older than `minInvalidationInterval` (`--cache-controller-ttl`, default 10s), it also starts `InvalidateIfNeeded`.
2. `InvalidateIfNeeded` allows only one invalidation in flight per store (`inflightInvalidations`). It runs `findChangesAndInvalidateIfNecessary` in a goroutine.
3. `findChangesAndInvalidateIfNecessary`:
   - Calls `ReadChanges` in descending order, one page of `storage.DefaultPageSize` = 50 changes, with a 1s timeout.
   - Stores `ChangelogCacheEntry{LastModified: newest change timestamp, LastChecked: now}` with TTL `queryCacheTTL`. Because Check-result entries are compared against this one timestamp, a write anywhere in the store invalidates every older cached Check result for that store.
   - Also invalidates iterator-cache entries:
     - If all 50 changes fall within the iterator TTL, it invalidates the whole store's iterator cache (`InvalidIteratorCacheKey`).
     - Otherwise, for each change within the TTL, it invalidates by object and relation (`InvalidIteratorByObjectRelationCacheKey`) and by user and object type (`InvalidIteratorByUserObjectTypeCacheKey`).
     - `findInCache` and `isInvalidAt` (`pkg/storage/storagewrappers/cached_datastore.go`) check these keys before using an iterator entry.
4. ListObjects requests also call `InvalidateIfNeeded` when the ListObjects iterator cache is enabled (`pkg/server/commands/list_objects.go`). They do this regardless of the controller TTL.

**Limits** (from the code and `docs/caching.md`, sections "Cache Controller" and "Caveats")
- **Eventually consistent.** After a write, Checks can return stale results for up to `cache-controller-ttl`.
- **First-Check staleness.** Invalidation is asynchronous. The request that triggers it, and any request before it finishes, still uses the old timestamp, or the zero time when there is no entry. Infrequent requests can therefore see stale data long after a write.
- **The time is fixed per request.** It is read once at the start, so a write during a Check is not seen by that Check.
- **Per-replica.** Caches are in memory per replica, and each replica polls the changelog on its own.
- **Coarse for Check results.** Invalidation is store-wide, while iterator invalidation is targeted.
- **Failures.**
  - If `ReadChanges` errors, the controller invalidates the whole iterator cache but does not update the changelog entry. Check results keep being compared against the previous timestamp until that entry's TTL expires.
  - The code comment says `ReadChanges` returns an error when there are no changes, so a store with an empty changelog takes this error path. I have not verified what that means for the query cache in that case.
  - A timeout simply returns, and a later request retries.
- **Clock comparison (inference, not verified).** Entry times come from the server's clock (`time.Now()`). Change times come from the datastore's changelog timestamps. Clock skew between the two could make the comparison wrong.
- **Cycles are not cached.** Results with `CycleDetected` are never stored.

## Q4

**Short answer.** In `X from Y`, `Y` must be a relation on the same type, defined only as direct type restrictions (`[type, ...]`). Those restrictions may not include usersets or wildcards. `X` must exist on at least one of `Y`'s directly related types. These rules are enforced at model write by `typesystem.NewAndValidate`.

**The path at model write**
1. `Server.WriteAuthorizationModel` (`pkg/server/authorization_models.go`) validates the protobuf (`req.Validate()`).
2. `WriteAuthorizationModelCommand.Execute` (`pkg/server/commands/write_authzmodel.go`) checks the type count and model size. It then calls `typesystem.NewAndValidate(ctx, model)` and maps any error to `InvalidAuthorizationModelInput`.
3. `NewAndValidate` (`pkg/typesystem/typesystem.go`):
   - Calls `New`, which records each type's TTU rewrites in `ttuRelations` through `flattenUserset`.
   - Checks the schema version. Only 1.1 and 1.2 pass `IsSchemaVersionSupported`.
   - Checks for duplicate types and names.
   - Calls `validateRelation` for each relation, in sorted order.

**Constraints on the tupleset relation `Y`**

| Constraint | Enforced in | Error |
|---|---|---|
| `Y` must be defined on the same object type | `TypeSystem.isUsersetRewriteValid`, case `*openfgav1.Userset_TupleToUserset` | `RelationUndefinedError` |
| `Y` must be purely direct: its rewrite must be exactly `Userset_This`, with no union, computed userset or TTU | Same function, `reflect.TypeOf(...) != reflect.TypeOf(&openfgav1.Userset_This{})` | "…is referenced in at least one tupleset and thus must be a direct relation" |
| `X` must be defined on at least one of `Y`'s directly related user types | Same function, loop over `GetDirectlyRelatedUserTypes()` | `ErrRelationUndefined` "…does not appear as a relation in any of the directly related user types…" |
| `Y`'s type restrictions may not be usersets (`type#rel`) or wildcards (`type:*`) | `TypeSystem.validateTypeRestrictions`, rule 4 (`related.GetRelationOrWildcard() != nil && IsTuplesetRelation(...)`) | `InvalidRelationTypeError` |
| At least one path through `Y`'s types to `X` must have an entrypoint | `hasEntrypoints`, case `Userset_TupleToUserset` | `ErrNoEntrypoints` / `ErrNoEntryPointsLoop` |

**Other details**
- `isUsersetRewriteValid` recurses into union, intersection and difference, so the rules apply to TTUs nested anywhere in a rewrite.
- `isUsersetRewriteValid` also has a branch for schema 1.0 models, where `X` only has to exist somewhere in the model. `NewAndValidate` rejects 1.0 models before this point, so that branch does not run on this path.
- Conditions on `Y`'s type restrictions are not rejected. `validateTypeRestrictions` only checks that the named condition exists.
- `relationInvolves` (`pkg/typesystem/typesystem.go`) also returns `ErrInvalidModel` for a `type#relation` restriction on a tupleset. It is a query helper rather than part of `NewAndValidate`.
- The same rules apply to tuples at write time:
  - `validateTuplesetRestrictions` (`internal/validation/validation.go`) rejects userset users, wildcards and non-object users on a tupleset relation.
  - It also rejects a tupleset relation that has a rewrite.
- The DSL parser lives in the external `github.com/openfga/language` module, which is not in this checkout, so any checks it makes are not covered here.

## Q5

**Short answer.** Each Check gets its own `BoundedTupleReader`, a channel semaphore sized by `maxConcurrentReadsForCheck`. The default is effectively unlimited. Operators set it with `--max-concurrent-reads-for-check`, `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK` or `maxConcurrentReadsForCheck` in `config.yaml`.

**The mechanism**
- `BoundedTupleReader` is in `pkg/storage/storagewrappers/bounded_datastore.go`.
- `NewBoundedTupleReader` creates `limiter: make(chan struct{}, op.Concurrency)`.
- `Read`, `ReadUserTuple`, `ReadUsersetTuples` and `ReadStartingWithUser` each call `bound()` first:
  - `waitForLimiter` blocks until a slot is free or the context ends.
  - Waits longer than 1ms are recorded in `datastore_bounded_read_delay_ms`.
  - The slot is released with `defer b.done()`.
- If datastore throttling is enabled, `bound()` also sleeps for `ThrottleDuration` once the request's read count exceeds `ThrottleThreshold`, and marks the request as throttled.

**How it is wired per request**
- **v1.** `CheckQuery.Execute` (`pkg/server/commands/check_command.go`) passes `Operation{Concurrency: c.maxConcurrentReads, ...}` to `NewRequestStorageWrapperWithCache`, which builds the `BoundedTupleReader` as the innermost request wrapper. The wrapper goes into the context, so every resolver and dispatched sub-check in that request shares the same limiter.
- **v2.** `CheckQueryV2.Execute` (`pkg/server/commands/check.go`) builds its own `BoundedTupleReader` from `datastoreOp`, set by `WithCheckQueryV2MaxConcurrentReads`.
- **Where the value comes from.** `Server.Check` passes `s.maxConcurrentReadsForCheck` through `commands.WithCheckCommandMaxConcurrentReads` (v1) and `WithCheckQueryV2MaxConcurrentReads` (v2) in `pkg/server/check.go`.

**Operator configuration**
- Set it in any of these ways:
  - The flag `--max-concurrent-reads-for-check` (`cmd/run/run.go`).
  - The environment variable `OPENFGA_MAX_CONCURRENT_READS_FOR_CHECK` or `OPENFGA_MAXCONCURRENTREADSFORCHECK` (`cmd/run/flags.go`).
  - The key `maxConcurrentReadsForCheck` in `config.yaml`. That file is loaded from `/etc/openfga`, `$HOME/.openfga` or `.` (`cmd/root.go`, `NewRootCommand`).
- `Config.MaxConcurrentReadsForCheck` (`pkg/server/config/config.go`) is passed in `ServerContext.Run` as `server.WithMaxConcurrentReadsForCheck(...)` (`cmd/run/run.go`). For embedded use, set it with the `server.WithMaxConcurrentReadsForCheck` option.
- The default is `DefaultMaxConcurrentReadsForCheck = math.MaxUint32`, which is effectively unlimited (`.config-schema.json`: 4294967295).
- The doc comment on `WithMaxConcurrentReadsForCheck` (`pkg/server/server.go`) advises sizing it from `Datastore.MaxOpenConns`, the number of replicas and the expected RPS. Its example: with 100 connections, 1 replica and 100 RPS, set it to 1.
- Related settings that are not this limit:
  - `--resolve-node-breadth-limit` (default 10) caps how many sub-problems on one level are evaluated at once. It feeds `WithResolveNodeBreadthLimit` and the v2 `ConcurrencyLimit`.
  - `--check-datastore-throttle-threshold` and `--check-datastore-throttle-duration`, with the `datastore_throttling` experimental flag, control the throttling described above.

**Limits observed in the code**
- **The limit is per request.** It does not cap total datastore concurrency across requests.
- **It may not cover the SQL query itself (from reading the code, not run).** The slot is released when the read call returns, not when the returned iterator is drained.
  - SQL datastores return a lazy `sqlcommon.SQLTupleIterator`, whose query runs on the first `Next` or `Head` call (`fetchBuffer`).
  - So for `Read`, `ReadUsersetTuples` and `ReadStartingWithUser` on SQL datastores, the bound covers opening iterators rather than the SQL query.
  - `ReadUserTuple` runs its query inside the call, so it is fully bounded.
- **Zero is not rejected.** `Config.Verify` rejects 0 for `maxConcurrentReadsForListUsers` but not for Check. With 0 the channel is unbuffered and `done()` uses a non-blocking receive. From reading the code, every read would then block until the context ends; I have not run this.
- **BatchCheck does not pass the setting.** `BatchCheckQuery` (`pkg/server/commands/batch_check_command.go`) builds each item's `NewCheckCommand` without `WithCheckCommandMaxConcurrentReads`, so batch items fall back to `math.MaxUint32`. I cannot tell from the code whether this is intended.
