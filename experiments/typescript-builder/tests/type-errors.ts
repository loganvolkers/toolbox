// Negative type tests: every `@ts-expect-error` must correspond to a real compile error, or
// `tsc --noEmit` fails with "Unused '@ts-expect-error' directive". This measures what the builder's
// types catch before `seikika validate` runs.
import { action, cel, connector, field, kind, scorecard } from "../src/index.js";

const gh = connector("github", { package: "builtin:github", runtime: "in-image", records: ["repository"] });

// 1. Mapping from a record kind the connector doesn't declare.
// @ts-expect-error "installation" is not a declared record kind of github
gh.record("installation");

// 2. Freshness for an undeclared record kind.
// @ts-expect-error freshness keys must be declared record kinds
connector("x", { package: "p", runtime: "remote", records: ["a"], freshness: { b: "1h" } });

// 3. A precedence list for a field the kind doesn't have.
kind("Repo", {
  identityFrom: [gh.source],
  fields: { tier: field.enum(["0", "1"]) },
  // @ts-expect-error "owner" is not a field of Repo
  sources: { default: [gh.source], fields: { owner: [gh.source] } },
});

// 4. A malformed source id.
kind("Repo2", {
  // @ts-expect-error source ids are connector:<name>, git:<name> or native
  identityFrom: ["github"],
  fields: { tier: field.string() },
  sources: { default: [gh.source] },
});

const Repo = kind("Repo3", { identityFrom: [gh.source], fields: { archived: field.boolean() }, sources: { default: [gh.source] } });

// 5. A rule at a level the scorecard doesn't define.
scorecard("s", {
  kind: Repo, owner: "group:x", levels: ["bronze"],
  // @ts-expect-error "gold" is not one of the scorecard's levels
  rules: [{ name: "r", level: "gold", check: cel`true` }],
});

// 6. A typed field path that doesn't exist.
// @ts-expect-error Repo3 has no field "tier"
Repo.f.tier;

// 7. An action with neither `expect` nor `unverified` (action-runs §5).
// @ts-expect-error actions must declare a postcondition or explicitly accept unverified runs
action("a", { kind: Repo, scope: "day2", description: "does something", inputs: {}, permission: "p", executor: { type: "webhook", url: "https://x" } });

// 8. A bad duration.
action("b", {
  kind: Repo, scope: "day2", description: "does something", inputs: {}, permission: "p", executor: { type: "webhook" },
  // @ts-expect-error durations are <number>s|m|h|d
  expect: { postcondition: cel`true`, within: "two minutes" },
});
