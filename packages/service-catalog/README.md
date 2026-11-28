# Service catalog

This operator SDLC package is the box for a version-controlled software service catalog. It supplies the default `Service` and `ServiceEnvironment` kinds and optional Cloudflare and Grafana collectors. [Design and portal research](../../docs/design/service-catalog.md)

Include the package once. In the checkout the running portal uses, add a list of declarations to `catalog/services.yaml` and their environments to `catalog/environments.yaml`. Change `root` if Seikika runs from another directory.

- A service has a stable lowercase `id`, `title`, `description`, team `owner`, `lifecycle` (`experimental`, `production`, or `deprecated`), and source project `code`. Its optional `repository` points to the existing Repository in your catalog. Add links using `{title, url}` with absolute HTTP(S) URLs.
- An environment has a `service` slug, `environment` slug, `title`, and an `id` of `<service>@<environment>`. Add `worker`, `serviceName`, and `dashboardUid` for the collectors you enable. Copy those identities into the matching collector targets under `with`. The Service page shows deployment and metric evidence alongside the declared identities and links.

Use [the demo inventory](../../examples/cloudflare-containers/seikika-forks/catalog/services.yaml) and [include configuration](../../examples/cloudflare-containers/seikika-forks/seikika.yaml) as working examples. Both provider configs default to no targets. The portal needs the workspace, Cloudflare, and Grafana connector binaries beside `seikika` or on `PATH`. Validate your manifest and use the [local kit](../../examples/service-catalog/README.md) to check a three-provider join.
