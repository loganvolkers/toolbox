You are answering questions about the Toolbox repository, checked out at /tmp/graphify-ab/r1/toolbox-10. The project root is /tmp/graphify-ab/r1/toolbox-10.

Rules:
- Work offline. Don't use the web or any network tool.
- Read only files under /tmp/graphify-ab/r1/toolbox-10. Don't read or search anything elsewhere on this machine.
- Don't change any file under /tmp/graphify-ab/r1/toolbox-10, except to write /tmp/graphify-ab/r1/toolbox-10/ANSWERS.md.
- Don't start subagents.

Answer the five questions below in /tmp/graphify-ab/r1/toolbox-10/ANSWERS.md, with one `## Qn` heading per question. In each answer, state the facts and cite where they come from: the file, and the function, type or section. Answer from the code and docs as they are in /tmp/graphify-ab/r1/toolbox-10. Where you are not sure, say so instead of guessing. Don't describe how you searched.

- **Q1.** When a scorecard rule's connector data is missing, what outcome does the rule get, how does that affect the entity's level, and how does `toolbox report --baseline` (the ratchet) treat it compared with a rule that fails? Name the decision and the functions involved.
- **Q2.** When `toolbox serve` runs an in-image connector, how do the connector's records get into the store and trigger a recompute? Name the functions, the transport, and the checks applied to the records on the way.
- **Q3.** The oldest decision files have ids ending in a long run of zeros and a `legacy:` field, and newer ones don't. Why do decisions have these ids, how were the old ids' timestamps chosen, and what checks that a decision id cited in a document exists?
- **Q4.** If `schema/connector-messages.schema.json` changed, which code would enforce the new schema, and at which points (runtime and check time) would a message that doesn't conform be rejected?
- **Q5.** Trace `toolbox get <id> --url <server>` from the CLI to the code that reads the entity. How is the request carried, and where are identity and visibility applied?

When ANSWERS.md is written, reply with one line: DONE.
