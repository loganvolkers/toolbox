# Run log

Phase 1 agents launched 2026-11-23 (fresh general-purpose, inherit model):

| Phase | Arm | Agent id |
|---|---|---|
| 1 | A Toolbox | bc-62446a50-de08-5f9a-9cad-f4b578d1e809 |
| 1 | B Ocean | bc-803dd0c6-ebe7-55b1-b2ba-69b4ca143797 |
| 1 | C Backstage | bc-9fe61657-8a07-5349-885f-709eaed6fe09 |

Discards and reruns are appended here.

## Phase 1 scores (2026-11-23)

| Arm | Truth | Shape | Glue lines |
|---|---|---|---|
| A Toolbox | pass (5) | pass | 440 |
| B Ocean | pass (5) | pass | 295 |
| C Backstage | pass (5) | pass | 427 |

Harness fix during phase 1: copy `rust-toolchain.toml` into arm A (edition2024). Recorded as harness friction, not counted against C1.

### Phase 1 agent ids and deviations
- A: bc-62446a50 (53 calls > 50 budget)
- B: bc-803dd0c6 (41 calls)
- C: bc-9fe61657 (29 calls)
- Harness fix mid-run: rust-toolchain.toml into arm A / scaffold (edition2024). Does not change phase-1 correctness.

### Phase 3 launch
- Discarded truncated prompt launch bc-1f45cf68 (incomplete brief); relaunched A as bc-3da53731
- B: bc-eb6a791a
- C: bc-6a3bfcc2

## Phase 3 scores (2026-11-23)

| Arm | Truth | Shape | Glue | Calls | Agent |
|---|---|---|---|---|---|
| A | pass (8) | pass | see 3-A-glue | 12 | [Connector A/B phase3 Toolbox v2](bc-3da53731-56c4-5a88-99d0-d41f8f7a041e) |
| B | pass (8) | pass | see 3-B-glue | 22 | [Connector A/B phase3 Ocean](bc-eb6a791a-d811-5717-8ff3-4387ffdf261c) |
| C | pass (8) | pass | see 3-C-glue | 17 | [Connector A/B phase3 Backstage](bc-6a3bfcc2-5b23-5ef0-a18c-29395631e296) |

Discarded: [Connector A/B phase3 Toolbox](bc-1f45cf68-6549-53fc-980d-bee57bdba06c) (truncated prompt); work overlapped with v2, scored from final tree.
