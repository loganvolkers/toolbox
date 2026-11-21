# Decision files: one ULID each

*Status: decided and checked, 2026-11-20 (D01M7XKEXQH4HG5371629M50E2G). Amends the row check in
[decision-citations.md](./decision-citations.md) (D84). This is the maintainer SDLC: the log is how
this repository records a decision before the code lands.*

---

## 1. Conclusion

**A decision is a file whose name sorts by time, and two agents can mint ids without reading each other's branch.**

The log used to be one table, `docs/decisions.md`. The next id was "one more than the highest `D` number". Two pull requests that each added a decision both took that number. Git cannot merge two appends at the same line. A directory of files does not have that line.

| Question | What answers it |
|---|---|
| What is the id? | `D` plus a ULID. `python3 scripts/decision-id.py` prints one |
| Where does the text live? | `docs/decisions/<id>.md`, and the filename is the id |
| What happens to D1–D98? | Each file has `legacy: Dnn`. Citations of those numbers stay valid and were not rewritten |
| What happens to open questions? | They stay one list, [questions.md](../questions.md). They are not the collision |

`cited-decisions-exist` reads `id` and `legacy` from each file. A citation is a legacy `D` plus one to four digits, or `D` plus 26 Crockford characters. The directory itself is not scored: the entries are the log, and a later wording rule does not reach back into them.

## 2. Which id

The requirement is the one MongoDB ObjectIds were built for: a time prefix, then enough entropy that a process can mint an id alone, encoded so that byte order and string order agree. Several encodings do that. The one we cite has to be unambiguous in a repository that already contains git shas and content hashes.

| Scheme | Text | Time | Same-tick order | Why it lost or won |
|---|---|---|---|---|
| ObjectId | 24 lowercase hex | seconds, through 2106 | a 24-bit counter after 5 random bytes. Two processes in the same second sort by the random bytes, not by the counter | The right shape. Hex is what our hashes already look like, and a second is coarser than the backfill needs |
| xid | 20 base32hex | seconds | the same 12 bytes as an ObjectId | Shorter ObjectId. Uniqueness leans on a machine id, which two cloud agents can share |
| KSUID | 27 base62 | seconds since 2014 | none. Same-second order is the random payload | No counter to keep D1–D32 in order inside one commit |
| UUIDv7 | 36, with hyphens | milliseconds | an optional counter in the random bits (RFC 9562 §5.7) | The IETF form. The hyphens and the length make a poor citation and a poor filename |
| TypeID | a prefix, an underscore, then 26 | UUIDv7 | as UUIDv7 | The `D` prefix already says "decision". A second prefix only lengthens it |
| Snowflake | up to 20 digits | milliseconds | a per-worker sequence | A worker id is a shared counter. That is the collision we are removing |
| ULID | 26 Crockford base32 | milliseconds, through 10889 | 80 random bits; a monotonic increment is specified for one process | **Chosen.** No ambiguous `I`, `L`, `O`, `U`. No coordinator. String order is time order |

Crockford's alphabet is `0123456789ABCDEFGHJKMNPQRSTVWXYZ`. The canonical text is uppercase. The checker accepts either case.

## 3. The tool

`scripts/decision-id.py` implements the ULID spec and nothing else. It is not a product dependency: the id is a maintainer filename, and the sprawl budget (D45) is for the image. The script's `--self-test` checks the vector published by oklog/ulid (`TestAlizainCompatibility`): Unix milliseconds `1469918176385` and ten zero bytes encode as `01ARYZ6S410000000000000000`. CI runs that test. A new id reads 10 bytes from `secrets`.

## 4. Backfill

D1–D98 were copied out of the table, not rewritten.

| Field | Value |
|---|---|
| `legacy` | The original `Dnn` |
| `decided` | The commit that first added the row, from `git log --reverse -p` |
| id time | That commit's second, plus the row's place among the decisions added in the same second. The largest commit added 32, so every offset stays inside the second |
| random part | Zero. A backfilled id ends in sixteen `0`s, and the same inputs produce the same id |

The old day headings are not the timestamp. D1–D14 are headed 2026-11-18, and the commit that added them is 2026-11-19T17:09:39Z. The commit is when the row became shared state, and it orders every decision with no inversions against `D1`…`D98`.

Citations of `Dnn` were left in place. Rewriting them would be a second conflict on every open branch, and it would edit history the log says not to edit. A new decision has no `legacy` field. Cite its id.

## 5. What is not a decision file

Open questions stay in `docs/questions.md` with their `Q` numbers. The directory is the decision log only. `docs/decisions/README.md` explains the format and does not list the decisions: a list in one file is the table we just removed.

## Sources

- ULID: [github.com/ulid/spec](https://github.com/ulid/spec). The compatibility vector is `TestAlizainCompatibility` in [oklog/ulid](https://github.com/oklog/ulid) `ulid_test.go`.
- UUID version 7: RFC 9562 §5.7, May 2024.
- BSON ObjectId: [mongodb/specifications](https://github.com/mongodb/specifications) `source/bson-objectid/objectid.md` (4-byte seconds, 5-byte random, 3-byte counter).
- xid: [github.com/rs/xid](https://github.com/rs/xid), the ObjectId layout in 20 characters of base32hex.
- KSUID: [github.com/segmentio/ksuid](https://github.com/segmentio/ksuid).
- TypeID: [github.com/jetify-com/typeid](https://github.com/jetify-com/typeid).
- This repository's D1–D98, introduced across 30 commits, and D84, the citation check this amends.
