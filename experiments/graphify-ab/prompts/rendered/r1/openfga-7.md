You are answering questions about the OpenFGA repository, checked out at /tmp/graphify-ab/r1/openfga-7. The project root is /tmp/graphify-ab/r1/openfga-7.

Rules:
- Work offline. Don't use the web or any network tool.
- Read only files under /tmp/graphify-ab/r1/openfga-7. Don't read or search anything elsewhere on this machine.
- Don't change any file under /tmp/graphify-ab/r1/openfga-7, except to write /tmp/graphify-ab/r1/openfga-7/ANSWERS.md.
- Don't start subagents.

Answer the five questions below in /tmp/graphify-ab/r1/openfga-7/ANSWERS.md, with one `## Qn` heading per question. In each answer, state the facts and cite where they come from: the file, and the function, type or section. Answer from the code and docs as they are in /tmp/graphify-ab/r1/openfga-7. Where you are not sure, say so instead of guessing. Don't describe how you searched.

- **Q1.** When a client calls the Check API, which resolvers does the request pass through before relationship tuples are read, in what order, and what decides whether each one is present? Name the types and the code that builds the chain.
- **Q2.** Which types implement the storage method `ReadUserTuple`, and which production code (not tests or mocks) calls it? If its signature changed, what else would have to change?
- **Q3.** After tuples are written, how does OpenFGA decide that a cached Check result is stale? Describe the mechanism, where it lives, and its limits.
- **Q4.** What constraints does OpenFGA place on the tupleset relation of a tuple-to-userset rewrite (`X from Y`), and where in the code are they enforced when a model is written?
- **Q5.** How does OpenFGA limit concurrent datastore reads during a single Check, and how does an operator configure that limit?

When ANSWERS.md is written, reply with one line: DONE.
