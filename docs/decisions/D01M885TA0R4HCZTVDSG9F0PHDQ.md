---
id: D01M885TA0R4HCZTVDSG9F0PHDQ
decided: 2026-11-25T01:41:54Z
---

# Dev auth off loopback needs an explicit acknowledgement, or the server refuses to start

**`toolbox serve` with `TOOLBOX_AUTH_MODE=dev` on a non-loopback address refuses to start unless
`TOOLBOX_ALLOW_DEV_AUTH_OFF_LOOPBACK=1` is also set. With it, the server starts and logs the
warning it logged before.** This amends point 2 of D01M857ZB3FZJZWX8G3BF5W1RNR, under which an
explicit `dev` worked anywhere and only logged a warning. The defaults don't change: `dev` on
loopback, `token` elsewhere.

1. **Why.** Dev auth trusts the `x-toolbox-user` header, so any client that can reach the port
   can claim any identity, an operator's included. One variable set by mistake is enough. It can
   be copied from a local setup into a deploy, or left in an image's environment. A log line
   that nobody reads doesn't stop it. Backstage's production template refuses its guest provider
   outside development unless `dangerouslyAllowOutsideDevelopment` is set. This is the same rule:
   two settings, each one saying what it does.
2. **What changes.**
   - `TOOLBOX_AUTH_MODE=dev` on an address other than `127.0.0.0/8`, `::1` or `localhost` exits 2.
     The error names the real modes, binding loopback, and the acknowledgement.
   - `TOOLBOX_ALLOW_DEV_AUTH_OFF_LOOPBACK` accepts `1`, `true` or `yes`.
   - When `token` mode refuses to start without OpenFGA, its error names both settings as the way
     out off loopback.
   - The `acme-live` deploys (compose, Score, Helm chart) set the acknowledgement, because dev
     auth is their stand-in for an identity-aware proxy.
3. **Why not stop the image binding `0.0.0.0`.** `PORT` binding every interface is the contract
   with a container host such as Cloud Run (charter goal 1). The binary already binds
   `127.0.0.1:8080` when neither `PORT` nor `--listen` is set. Off loopback, what protects the
   portal is the auth mode, so that is what this decision constrains.
4. **Uses outside this repository.** Any deployment that sets `TOOLBOX_AUTH_MODE=dev` off loopback
   must add the acknowledgement or switch to a real mode. The error message says so. Loopback
   servers, the acceptance scripts and the DX Benchmarks' harnesses bind `127.0.0.1` and are
   unchanged.

**Status:** ✅

**Where:** `crates/toolbox-cli/src/server.rs` (`auth_mode`),
[backstage-comparison.md](../design/backstage-comparison.md) §4,
[examples/acme-live](../../examples/acme-live/README.md).
