# Federation prototype (F0.5) as an experiment; production pause unchanged

**Ship a docs-as-code federation vertical slice under `experiments/federation-prototype/`
without lifting the production federation pause.** The experiment proves push (package
includes + monotonic tightening), leaf/org extend, weaken rejection, bottom-up report
aggregation, multi-entity catalog merge (Repository/Service/Component/Library), and an
observe-only mobile leaf. It does not enable runtime portal coupling, OpenFGA subtree
auth, or T4T ratchet changes that pretend federation is production-ready.

**Context:** D68 designed federation and paused implementation; D01M7Y46VDDBK2SW6H96ZSG89MP
keeps effort on repo-level value. The prototype answers the model questions with fixtures
and `compose.py`, aligned with packages (D89) and campaigns, so later F1 starts from a
tested shape rather than a greenfield design.

**Where:** [design/federation.md](../design/federation.md) §8 F0.5,
[experiments/federation-prototype/](../../experiments/federation-prototype/)
