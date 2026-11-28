# Assessment queue validation

Maintainer SDLC. These checks exercise the improvements from the [campaign retrospective](../../assessment-improvement-2026-11-28.md). Run the [two-loop replay](../../../../experiments/assessment-replay/README.md#assessment-work-queue-replay) with the CLI, workspace connector and runner built from the same revision.

| Loop | Starting evidence | Change and result |
| --- | --- | --- |
| Refresh a changed subject | A passing assessment had an obsolete subject hash and lived at a renamed YAML path. The queue reported the changed hash and the existing path. | The first trial surfaced inconsistent paths between the queue row and embedded brief. After fixing the brief, the replay updated the existing file, verified a fresh result, and checked that the assessment digest changed while the manifest digest held. |
| Verify an independent review | The original assessor was proposed as reviewer using different capitalization. The queue routed it to a handoff and withheld the brief. | The replay wrote a simulated self-review and confirmed the campaign still needed evidence. A grounded review from a distinct fixture assessor cleared the scoped campaign item. |
| Collector failure | The replay removed the facts file used by the fixture connector. | The run exited with code 2 and recorded the failed connector. The successful steps matched each run manifest's output digest to the emitted JSON. |

## Measured friction

On the disposable portal, successful check-and-record steps initially took 6.1–6.3 seconds. Hashing the 166 MB debug executable dominated those runs. Using the host's accelerated SHA-256 tool when available, with an internal SHA-256 fallback, reduced those observed steps to 0.8–1.0 seconds. The run continues to record the executable's content digest.

On the live self-portal, one paired observation used the same stale observability assessment, checkout, and connector exclusions. The baseline executable performed an unfiltered check and a separate brief. The updated executable selected that item and returned its verdict, failure reasons and brief from one collection, including a run manifest. The brief's facts, subject hash and rubric hash matched.

| Workflow | Collector runs | Elapsed | JSON on stdout |
| --- | ---: | ---: | ---: |
| Original check and separate brief | 2 | 38.9 s | 201,171 bytes |
| Scoped queue with brief and run record | 1 | 20.1 s | 8,327 bytes |

This example measures the work needed to read and hand off one assessment. The queue bounds the result and shares collector work across briefs. It still derives the portal snapshot and validates all local assessment files. The fixture people test the routing rules. A real reviewer must have independent authority to attest to the evidence. These observations are engineering smoke tests, with no comparative agent-performance or token-use claim. [Comparative benchmark design](../../assessment-improvement-benchmark.md)
