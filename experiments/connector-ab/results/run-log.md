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

## Phase 4 scores (2026-11-23)

| Arm | Truth | Shape | Glue |
|---|---|---|---|
| A | pass (4, tenant B) | pass | 550 |
| B | pass (4) | pass | 342 |
| C | pass (4) | pass | 483 |

Agents still finishing replies: bc-c9a3934b, bc-7898db12, bc-cad824e0.

## Phase 5 scores

All arms 5/5 on read-key via score_read.py.

## Final bar

| Hypothesis | Result |
|---|---|
| C1 | Fail (A65 B63 C46) |
| C2 | Fail (A15 B15 C16) |
| C3 | Hold |


## Rerun v2 (learning-tax fixes, 2026-11-23)

Product fixes on branch `cursor/connector-learning-dx-530d` (D01M83FFM2K6XS4EAVHVP92TBDZ). Same bar C1–C3. Fresh agents for phase 1.

### Phase 1 v2 agent ids
- A Toolbox: bc-5133c227-6bc1-5efa-b07f-9c22a1bb6e30
- B Ocean: bc-0c51db1a-e909-51c7-9492-4a8c35124dd9
- C Backstage: bc-22217758-3292-54f0-b08c-b3f6e652cce4

### Phase 1 v2 scores (partial; A agent may still be wrapping up)
| Arm | Truth | Shape | Glue | Calls (transcript) | Agent |
|---|---|---|---|---|---|
| B Ocean | pass (5) | pass | 302 | 37 | bc-0c51db1a |
| C Backstage | pass (5) | pass | 358 | 38 | bc-22217758 |
| A Toolbox | pass (5) | pass | 342 | 36 | bc-5133c227 |

### Phase 3 v2 agent ids
- A: bc-a2dd38c6-ec52-59ec-bd76-7b3bd57611a8
- B: bc-4b572909-11d8-5971-8b55-c918a5b57b0d
- C: bc-b25d4935-630a-54c9-8423-45c9d59e8804

### Phase 3 v2 scores (partial)
| Arm | Truth | Shape | Calls | Agent |
|---|---|---|---|---|
| A | pass (8) | pass | *pending wrap-up* | bc-a2dd38c6 |
| B | pass (8) | pass | *pending wrap-up* | bc-4b572909 |
| C | pass (8) | pass | 23 | bc-b25d4935 |

### Phase 3 v2 final + C1
| Arm | Calls 1 | Calls 3 | C1 (1+3) |
|---|---|---|---|
| A Toolbox | 36 | 23 | **59** |
| B Ocean | 37 | 26 | 63 |
| C Backstage | 38 | 23 | 61 |

**C1 holds** (A fewer than B and C). v1 failed with A65 / B63 / C46.
