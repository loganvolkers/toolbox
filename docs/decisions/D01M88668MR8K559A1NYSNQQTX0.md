---
id: D01M88668MR8K559A1NYSNQQTX0
decided: 2026-11-25T01:49:12Z
---

# Connector confinement is a setting: `best-effort` by default, `required` to refuse unconfined connectors

**`SEIKIKA_CONNECTOR_CONFINEMENT` says what `seikika serve` does when it cannot put an in-image
connector in its own Landlock domain. `best-effort`, the default, starts the connector unconfined
and makes that visible. `required` refuses to start it.** Where Landlock is available, nothing
changes: every in-image connector is confined either way.

1. **Why a setting, and why not fail closed by default.** Landlock is a kernel feature, and the
   host chooses it. A sandboxed container runtime such as gVisor, or a seccomp profile without the
   Landlock calls, has none, and the image cannot change that. If missing Landlock refused every
   in-image connector, the one-container deployment (charter goal 1) would break on exactly those
   hosts. Until now the server started the connector anyway and warned once in a log nobody
   reads. An operator who needs the isolation guarantee had no way to make it hold.
2. **What changes.**
   - The setting accepts `best-effort` or `required`. Unset or empty means `best-effort`. Any other
     value makes `serve` exit 2 and name the two values.
   - Under `required`, a connector that can't be confined is not started, and its supervisor
     stops instead of retrying, because the kernel won't gain Landlock while the server runs.
     The server logs an error naming the setting.
   - Every in-image connector's last start is recorded as `landlock`, `unconfined` or `refused`.
     `GET /v1/connectors` (admins) gives that per connector, plus the setting and the names of
     the unconfined and refused connectors. `/readyz` is unauthenticated, so it gives only the
     setting and the two counts. Readiness doesn't change, because a refused connector leaves
     the portal serving its other sources.
   - A field's explain adds `sourceConfinement` when the field came from an in-image connector.
   - `seikika doctor` prints a `connector confinement` line for a portal with in-image
     connectors: `ok` where the host has Landlock, and `UNCONFINED` where it doesn't, as a warning
     that keeps exit 0. It prints `REFUSED` where it doesn't and `required` is set, and `WRONG` for
     a bad value. Both of those exit 1. Doctor checks the host it runs on, which is the server's
     host only when it runs in the image.
3. **What it does not cover.** Landlock limits what a connector can read of other processes'
   memory and `/proc`. Its files and network are unchanged (connector-control.md §3). A connector
   that must not share the server's host at all runs remotely.
4. **Uses outside this repository.** Deployments that set nothing behave as before, except that
   they now show any unconfined connectors. Deployments whose host has Landlock can set
   `required`, so that a later change of host can't drop the isolation without saying so.

**Status:** ✅

**Where:** `crates/seikika-api/src/confine.rs` (`Confinement`, `spawn`),
`crates/seikika-api/src/connectors.rs` (`supervise`, `confinement_summary`),
`crates/seikika-cli/src/install.rs` (`doctor`),
[connector-control.md](../design/connector-control.md) §3,
[backstage-comparison.md](../design/backstage-comparison.md) §4.
