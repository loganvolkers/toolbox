---
id: D01M87T16JRH53YV6VVWM8PH169
decided: 2026-11-24T18:00:00Z
---

# CI hosts are declared apart from forges

**Revises D01M87AXG095Q0Z0DGGTKYYAM7Y, which made dispatch, track and collect capabilities of a
forge.** A forge hosts code: read, write, propose, and who people are. A CI host runs it:
dispatch, track, collect, cancel. A manifest declares them in two lists, `forges:` and `ci:`, and
a forge's entry says which CI its projects build on.

1. **Why:** an organisation's code and its CI are often on different systems: GitHub repositories
   built on Buildkite, Jenkins or a GitLab that mirrors them, a GitLab project whose pipelines run
   elsewhere. With CI inside the forge driver, those children could only run through the
   parent's checkout, and adding a CI meant adding a forge. The executor review
   (`action-executor-review.md`, step 3) found the same conflation in executors:
   `gitlab-pipeline` resolved its endpoint through `forges:`, and `github-workflow` took a raw
   `api` and `token`.
2. **The manifest:** `ci:` lists named hosts, each with a `type` (`github-actions`, `gitlab-ci`,
   or `http`), a `url`, an `api` and a `token` variable. It may also map a forge project to its
   project on the host (`project: mirrors/{project}`) and limit which projects a parent may start
   runs in (`projects`). `forges[].ci` names the host that runs that forge's `runner: {type: ci}`
   children, or `none`. Without it, a forge's projects build on the forge's own CI, so every
   existing manifest means what it meant. `github-workflow` and `gitlab-pipeline` executors may
   name `ci:` instead of `forge`, `api` and `token`.
3. **`http` bridges any CI:** an `http` host is a service that speaks the executor protocol
   (`run`, `status`, `cancel`; D01M86BY3YQX3SA1W99QXF1QA1H's executor contract). It answers `run`
   with `accepted` and an `executor_run_id`, answers `status` with the run's `result` once the
   run is done, and puts the child's run record in that result's `outputs.record`. Seikika gains
   no Buildkite or Jenkins driver; the bridge is a tool that works without Seikika, and it is not
   an in-process plugin.
4. **Trust stays with the parent (T1, revised):** a child names neither `ci` nor `via` in its
   runner, and is refused if it does. Which CI host runs it, and with whose token, follows from
   the parent's entry for the forge the child was cloned from. Both the forge's `projects` and the
   CI host's `projects` must admit the project.
5. **In code:** `seikika_model::ci` resolves a forge project to a CI target. `ForgeDriver` loses
   its `ci` method, and `seikika_exec::ci::driver` builds a `CiDriver` from the target. Adding a
   CI is one `CiType` and one driver; adding a forge no longer touches CI.
6. **Deferred:** a forge with no CI of its own but a declared host (plain git or Origin with
   Buildkite) needs a forge type without a driver. Today `forges[].ci` applies only to `github`
   and `gitlab` forges.

**Status:** ✅

**Where:** [docs/design/forges.md](../design/forges.md) §1, §4.1, §4.3, §4.7, §5, §7,
[docs/design/federated-actions.md](../design/federated-actions.md) §5.3, §6.2,
[docs/v1-spec.md](../v1-spec.md) §7.5,
[examples/federated-actions/hybrid-org](../../examples/federated-actions/hybrid-org/seikika.yaml)
