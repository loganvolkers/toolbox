# deploy-ab prompts, recovered

These were not committed before the runs, which protocol 2 now requires
([D01M83RDWQ5EVKY5SFHDWAD5HFH](../../../docs/decisions/D01M83RDWQ5EVKY5SFHDWAD5HFH.md), P1). They
were recovered on 2026-11-23 from the first user message in each cloud agent's record. The prompt copies use the current project name. The agent records preserve the exact text used in the runs.

| File | Arm | Agent records | sha256 |
|---|---|---|---|
| [seikika.md](./seikika.md) | Seikika | `bc-51601c2d-7316-5e2f-a3ef-0888992f45fb` (first run), `bc-514b615a-26fc-5388-bd3d-937048f62413` (rerun) | `591a0db541cc501b87e14fdee70c4ac41516dfa935e700020939299b3a4eb524` |
| [backstage.md](./backstage.md) | Backstage | `bc-09dec1a5-e698-524a-9cbf-298556b324c8` (first run), `bc-49ebdbc7-fe3e-5819-9104-e159c3b1708b` and `bc-c748ae14-63a1-5622-a739-20abe221c457` (rerun) | `1d1928083d067e2c77a33c59709770da048c3188e4ca0b86e080f98b117be0b6` |

Check the files against the table from the repository root:

```bash
python3 -c "import hashlib,sys; [print(hashlib.sha256(open(f,'rb').read()).hexdigest(), f) for f in sys.argv[1:]]" experiments/deploy-ab/prompts/seikika.md experiments/deploy-ab/prompts/backstage.md
```

Every launch of an arm got the same prompt. What the prompts give each arm differently is set out in
[ADJUDICATION.md](../ADJUDICATION.md).
