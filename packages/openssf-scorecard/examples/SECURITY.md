# Security Policy

This is a **sample** `SECURITY.md` for Scorecard's Security-Policy check (~10). Copy it to
your repository root (or `.github/` / `docs/`), replace the reporting URL and email, and keep the
vulnerability / disclosure / timeline wording. Scorecard awards points for a contact link or email,
free-form policy text, and those three themes together — see
[Security-Policy](https://github.com/ossf/scorecard/blob/main/docs/checks.md#security-policy).

## Supported versions

Security fixes apply to the latest release on the default branch. Older tags are not backported
unless a release notes exception says otherwise.

## What counts as a vulnerability

A **vulnerability** is a weakness that could allow unauthorized access, data exfiltration,
privilege escalation, supply-chain compromise of released artifacts, or denial of service against a
default or documented deployment. Speculative issues without a plausible exploit path are treated as
hardening suggestions, not vulnerabilities.

## How to report

Prefer **private vulnerability reporting** on this repository: open the Security tab and choose
*Report a vulnerability*
(https://github.com/OWNER/REPO/security/advisories/new).

If private reporting is unavailable, email security@example.com with subject line `[SECURITY]`.

Do **not** open a public issue or discussion for an undisclosed vulnerability.

## Disclosure process and timelines

1. We acknowledge receipt within **3 business days**.
2. We aim to confirm whether the report is a vulnerability within **14 days**.
3. We target a fix or mitigation within **90 days** of confirmation, then coordinate public
   **disclosure** (Security Advisory and/or release notes). Critical remotely exploitable issues may
   be disclosed sooner once a fix is available.
4. We credit reporters who wish to be named, unless they prefer anonymity.

## Safe harbor

Good-faith security research that follows this policy and avoids privacy violations, destruction of
data, and service disruption is welcome.
