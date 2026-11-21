# Decisions

Each decision is one file in this directory. The filename is the id, and the filenames sort by the time the decision was made.

An id is `D` plus a ULID (26 Crockford characters). Mint one with `python3 scripts/decision-id.py` and add `docs/decisions/<id>.md`. Do not edit a decision that is already here; record a change as a new file and update the design doc it points at.

D1–D98 were backfilled. Those files have a `legacy` field, and a citation of the old number still resolves. A backfilled id ends in sixteen zeros: the random part is empty, and the time is the commit that added the row.

Open questions are [questions.md](../questions.md). The format, the comparison, and the backfill are [decision-files.md](../design/decision-files.md).
