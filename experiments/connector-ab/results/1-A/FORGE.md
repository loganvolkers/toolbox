# Forge (mock Jira Cloud)

Tenant A: http://127.0.0.1:9743  token tenant-a-token
Tenant B: http://127.0.0.1:9744  token tenant-b-token

Endpoints:
- GET /rest/api/3/project?startAt=&maxResults=
- GET /rest/api/3/search?startAt=&maxResults=
- GET /rest/api/3/project/{key}/components   (required from phase 3)

Header: Authorization: Bearer <token>
