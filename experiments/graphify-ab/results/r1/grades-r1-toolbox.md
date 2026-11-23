# Grades: round 1, Toolbox (blind)

The rubric at the top of `/tmp/graphify-ab/gold/toolbox.md` was applied to all ten answers in
`/tmp/graphify-ab/blind-r1/toolbox/`, for 21 key facts (T1a–T5d). Each cell is C (covered, 1), P
(partial, 0.5) or M (missed, 0). Coverage = (C + 0.5 × P) / 21. The frozen source at
`/tmp/graphify-ab/r1/toolbox-3/` was read only to check claims that might contradict a gold fact.

**Result:** every label covers every fact. No P or M was assigned. The facts did not separate these
answers: the scores hit the ceiling.

## Per-fact table

| Fact | 216 | 342 | 359 | 398 | 401 | 600 | 642 | 784 | 822 | 833 |
|---|---|---|---|---|---|---|---|---|---|---|
| T1a | C | C | C | C | C | C | C | C | C | C |
| T1b | C | C | C | C | C | C | C | C | C | C |
| T1c | C | C | C | C | C | C | C | C | C | C |
| T1d | C | C | C | C | C | C | C | C | C | C |
| T2a | C | C | C | C | C | C | C | C | C | C |
| T2b | C | C | C | C | C | C | C | C | C | C |
| T2c | C | C | C | C | C | C | C | C | C | C |
| T2d | C | C | C | C | C | C | C | C | C | C |
| T2e | C | C | C | C | C | C | C | C | C | C |
| T3a | C | C | C | C | C | C | C | C | C | C |
| T3b | C | C | C | C | C | C | C | C | C | C |
| T3c | C | C | C | C | C | C | C | C | C | C |
| T3d | C | C | C | C | C | C | C | C | C | C |
| T4a | C | C | C | C | C | C | C | C | C | C |
| T4b | C | C | C | C | C | C | C | C | C | C |
| T4c | C | C | C | C | C | C | C | C | C | C |
| T4d | C | C | C | C | C | C | C | C | C | C |
| T5a | C | C | C | C | C | C | C | C | C | C |
| T5b | C | C | C | C | C | C | C | C | C | C |
| T5c | C | C | C | C | C | C | C | C | C | C |
| T5d | C | C | C | C | C | C | C | C | C | C |

## Coverage per label

| Label | C | P | M | Coverage |
|---|---|---|---|---|
| 216 | 21 | 0 | 0 | 1.000 |
| 342 | 21 | 0 | 0 | 1.000 |
| 359 | 21 | 0 | 0 | 1.000 |
| 398 | 21 | 0 | 0 | 1.000 |
| 401 | 21 | 0 | 0 | 1.000 |
| 600 | 21 | 0 | 0 | 1.000 |
| 642 | 21 | 0 | 0 | 1.000 |
| 784 | 21 | 0 | 0 | 1.000 |
| 822 | 21 | 0 | 0 | 1.000 |
| 833 | 21 | 0 | 0 | 1.000 |

## Calibration decisions (applied to every label)

1. **Parenthetical asides in a gold fact are not required.** T5a's "(`toolbox mcp`, the stdio
   bridge, forwards to the same endpoint.)" appears in no answer. The core of T5a, that
   `client::tool` posts a JSON-RPC `tools/call` to `/mcp`, decides the call.
2. **A missing numeric parameter, when the names and mechanism are right, is not a material
   error.** For example, a backoff stated as "up to 30 s" without the 1 s start still counts. A
   wrong number would have been a material error (P); none was found.
3. **"ULID" implies its standard 26-character length.** Answers that say "`D` plus a ULID" with the
   48/80-bit split and Crockford base32 cover T3b without the literal "26".
4. **"String order is time order" is equivalent to "filename order is time order"** (T3b), because
   the filename is the id.
5. **A function name qualified by its module identifies the fact without its file path.** For
   example, `messages::validate_fixtures()` without `messages.rs`, or `handle` stated as being in
   `crates/toolbox-api/src/mcp.rs` instead of `mcp::handle`.
6. **T2d "only when records or deletes change" means the batch contains a `record` or `delete`.**
   The source (`changes_inputs` in `crates/toolbox-api/src/connectors.rs`) tests for presence, so
   answers saying "only if the batch contains a record or delete" are correct.
7. **Extra correct detail earns nothing. An extra claim is penalised only if it contradicts a gold
   fact.** Hedged claims are treated the same way: penalised only if they contradict the gold and
   are wrong.
8. **A fact spread across an answer's sections still counts** (for example, T5d's CLI output
   formats stated in the transport step rather than the `get_entity` step).

## Per label: P and M items, and close covered calls

No label has a P or an M item. The close covered calls are below.

### 216
- No close calls. All 21 facts are stated with their identifying names.

### 342
- No close calls.

### 359
- **T2a (close, C):** "If the child exits, it is restarted with exponential backoff, up to 30 s." The 1 s start is
  omitted. Covered under calibration 2.
- **T3b (close, C):** "The id is `D` plus a ULID: 48 bits of Unix milliseconds and 80 random bits, in Crockford
  base32." It doesn't say "26-character". Covered under calibration 3.

### 398
- No close calls.

### 401
- No close calls.

### 600
- **T3b (close, C):** "ULIDs need no coordinator, and string order is time order." Covered under
  calibration 4.

### 642
- No close calls on gold facts. A non-gold claim in the Q4 table, that `report` loads the portal
  through `validate_portal`, conflicts with the answer's own "Gaps" section. It touches no gold fact,
  so it was not penalised (calibration 7).

### 784
- **T5b (close, C):** "In `crates/toolbox-api/src/mcp.rs`, `handle` … first calls `state.caller(&headers)`, so every request needs an
  identity." The wording is not `mcp::handle`. Covered under calibration 5.

### 822
- **T1c (close, C):** the answer adds a hedged exception: "for a `compact` scorecard, `compare_with` treats an entity missing from the baseline as unclean when it has an `unknown` list. It then pushes a regression". This was checked against the source
  (`compare_with` in `crates/toolbox-cli/src/report.rs`, around lines 488–500). It is accurate, and it
  sits outside `compare_entity`, which is what T1c describes. No contradiction, so T1c stays covered.

### 833
- **T3b (close, C):** "the id is `D` plus a ULID: 48 bits of Unix milliseconds and 80 random bits in Crockford
  base32". It doesn't say "26-character". Covered under calibration 3.
- **T4c (close, C):** "`validate_portal()` (`toolbox-model/src/validate.rs`), which calls `messages::validate_fixtures()`".
  No `messages.rs` path. Covered under calibration 5.
