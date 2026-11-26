---
id: D01M857ZB3FZJZWX8G3BF5W1RNR
decided: 2026-11-23T22:40:00Z
---

# A server off loopback trusts no header by default, and connectors inherit no server credentials

**With `SEIKIKA_AUTH_MODE` unset, `seikika serve` runs `dev` auth (the `x-seikika-user` header is
trusted) only when it binds a loopback address. On any other address the default is the new
`token` mode, which accepts personal access tokens only. In-image connectors start with a cleared
environment plus an allowlist of process basics, so `DATABASE_URL`, PATs and `SEIKIKA_SECRET_*`
never reach them.**

1. **Why.** The Backstage comparison
   ([backstage-comparison.md](../design/backstage-comparison.md), factor *secure defaults*) found
   the shipped image worse than Backstage's production template:
   - The image sets `PORT=8080`, so it binds `0.0.0.0`. With no mode set, the default was `dev`,
     so any client that could reach the port could claim any identity, including an operator's.
   - The Backstage template refuses its guest provider outside development, unless
     `dangerouslyAllowOutsideDevelopment` is set, and answers unauthenticated reads with 401.
   - An in-image connector inherited the server's whole environment, database credentials
     included. That contradicts connector-control.md §3 ("no network, no credentials"), and makes
     the process boundary isolate memory but not credentials.
2. **What changes.**
   - `token` joins the auth modes.
   - The unset default depends on the bind address. An explicit `SEIKIKA_AUTH_MODE=dev` still
     works anywhere, as the `acme-live` deploys set it, and logs a warning off loopback.
   - Without OpenFGA, `token` mode refuses to start, like every non-dev mode. The error names
     `SEIKIKA_AUTH_MODE=dev` as the explicit way out.
   - Connectors get `PATH`, `HOME`, `USER`, locale, `TZ`, `TMPDIR`, CA-bundle and proxy variables,
     `RUST_LOG`, `RUST_BACKTRACE`, `GITHUB_API_URL` (an endpoint, the local simulation's switch)
     and `SEIKIKA_CONNECTOR_CONFIG`. A connector that read `GITHUB_TOKEN` from the server's
     environment now gets it through its config (`token: ${env:GITHUB_TOKEN}`), as `acme-live`
     already does.
   - `seikika report` still runs connectors with the caller's environment: there it is the
     caller's own machine, with no server credentials to protect.
3. **Uses outside the benchmark.** Every deployment of the image, and every `serve --listen
   0.0.0.0:…`. The acceptance scripts bind `127.0.0.1` and are unchanged.
