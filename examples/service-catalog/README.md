# Service catalog integration kit

The kit exercises the box from the maintainer and operator SDLCs. It serves the real demo inventory with GitHub and Cloudflare API emulators and Grafana LGTM in a local Docker container. It checks the connector protocol, a failed Cloudflare read, an 80/20 rollout, the GitHub repository join, and real Grafana responses for a finite value, zero, and no data. It then checks the signed-out catalog, the Service page, and its deep link in Chromium. [Service catalog design](../../docs/design/service-catalog.md)

From the repository root, build the UI and binaries and install Chromium for the existing report Playwright dependency.

```sh
pnpm install --frozen-lockfile
pnpm build
cargo build --locked --workspace
pnpm --filter @seikika/astro-report exec playwright install chromium
scripts/ci-tools.sh target/debug openfga
SEIKIKA_OPENFGA_BIN="$PWD/target/debug/openfga" scripts/service-catalog-acceptance.sh
```

The kit needs Docker, Python 3, Node, curl and jq. Set `SEIKIKA_BIN_DIR` to use another build and `SEIKIKA_CHROMIUM` to use an installed Chromium executable. It uses an isolated temporary state directory, dynamic loopback ports, and synthetic telemetry. It removes the emulators and Grafana container on exit. No platform credentials are needed.
