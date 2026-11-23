## What changed
- Component sync on by default in `ForgeEntityProvider` (`includeComponents`).
- `OUT/entities.ndjson`: 8 rows (3 components, 3 issues, 2 projects).

## Could not do
- Nothing blocking.

## DX feedback
1. Phase gate was an env flag, not an API change — easy to miss.
2. Components are a bare array vs paged objects for projects/search.
3. Component catalog id `{key}:{name}` vs `data.id` Forge numeric id.
4. Reading GithubEntityProvider mostly confirmatory under tight budget.

**Tool calls (transcript):** 23
**Agent:** [Connector A/B v2 phase3 C](bc-b25d4935-630a-54c9-8423-45c9d59e8804)
