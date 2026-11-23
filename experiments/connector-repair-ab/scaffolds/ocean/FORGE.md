# Forge (mock Jira Cloud)

Tenant A: http://127.0.0.1:9743  token tenant-a-token
Tenant B: http://127.0.0.1:9744  token tenant-b-token

Endpoints:
- GET /rest/api/3/project?startAt=&maxResults=   (paged object: values, startAt, isLast)
- GET /rest/api/3/search?startAt=&maxResults=      (paged object: issues, startAt, total)
- GET /rest/api/3/project/{key}/components         (required from phase 3; bare JSON array, not paged)

Catalog mapping for components: entity id is `{project}:{name}` (e.g. ALPHA:api).
Forge's numeric component id belongs in data.id only. Do not use it as the
catalog id or as the sole externalId suffix. Catalog data fields for components
are only `{id,name,project}` — omit extras like description.

Header: Authorization: Bearer <token>
