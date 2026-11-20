// Prototype of `@toolbox/manifest`: a typed builder whose only output is the v1 manifest JSON
// (docs/design/typescript-builder.md). No runtime, no I/O except printing; the server never runs it.

// ------------------------------------------------------------------------------------------- CEL
/** A CEL expression. Kept as a string (CEL is checked by `toolbox validate`, not by tsc). */
export type Cel = string & { readonly __cel?: true };
/** Tagged template for CEL, so editors and the LSP can find expressions. Interpolations must be
 *  `FieldRef`s (typed field paths) or other CEL fragments. */
export function cel(strings: TemplateStringsArray, ...parts: (FieldRef | Cel)[]): Cel {
  return strings.reduce((acc, s, i) => acc + s + (i < parts.length ? String(parts[i]) : ""), "");
}
export type FieldRef = string & { readonly __field?: true };

// ------------------------------------------------------------------------------------ connectors
export type SourceId = `connector:${string}` | `git:${string}` | "native";
type Duration = `${number}${"s" | "m" | "h" | "d"}`;
type RefreshScope = "external_ids" | "selector" | "since";

export interface ConnectorSpec<R extends string> {
  package: string;
  runtime: "in-image" | "remote";
  config?: Record<string, unknown>;
  records: readonly R[];
  refresh?: Partial<Record<R, RefreshScope[]>>;
  freshness?: Partial<Record<R, Duration>>;
}
export interface Connector<N extends string, R extends string> {
  name: N;
  spec: ConnectorSpec<R>;
  /** `<connector>/<record kind>`, only for declared record kinds. */
  record<K extends R>(kind: K): `${N}/${K}`;
  /** Default source label for fragments from this connector. */
  source: `connector:${N}`;
}
export function connector<const N extends string, const R extends string>(
  name: N, spec: ConnectorSpec<R>,
): Connector<N, R> {
  return { name, spec, record: (k) => `${name}/${k}` as const, source: `connector:${name}` };
}

// ------------------------------------------------------------------------------------------ kinds
export type Field =
  | { type: "string" | "boolean" | "integer" | "number" | "object"; facet?: string; computed?: Cel }
  | { type: "enum"; values: readonly string[]; facet?: string }
  | { type: "ref"; to: string; facet?: string }
  | { type: "list"; items: string; facet?: string; computed?: Cel }
  | { type: "object"; authority: CodeownersAuthority };

export interface CodeownersAuthority {
  type: "codeowners"; claim: string; rules: string; path: string; teamField: `${string}.${string}`;
}

export const field = {
  string: (o: { facet?: string } = {}): Field => ({ type: "string", ...o }),
  boolean: (): Field => ({ type: "boolean" }),
  object: (): Field => ({ type: "object" }),
  enum: (values: readonly string[]): Field => ({ type: "enum", values }),
  ref: (to: { name: string }): Field => ({ type: "ref", to: to.name }),
  list: (items: string, o: { computed?: Cel } = {}): Field => ({ type: "list", items, ...o }),
  /** CODEOWNERS authority (D17): `claim` and `rules` must name fields of the same kind. */
  codeowners: <F extends string>(a: { claim: F; rules: F; path: string; teamField: `${string}.${string}` }): Field =>
    ({ type: "object", authority: { type: "codeowners", ...a } }),
};

export interface MapRule<Fs extends string> {
  from: `${string}/${string}`;
  source?: SourceId;
  id: Cel;
  set: Cel;
  relations?: Record<string, Cel>;
  /** Not emitted: lets tsc check that `set` only targets declared fields when written as an object. */
  _fields?: Fs;
}

export interface KindSpec<F extends Record<string, Field>, Rel extends string = never> {
  identityFrom: readonly SourceId[];
  fields: F;
  relations?: Record<Rel, { to: string; many?: boolean }>;
  sources: { default: readonly SourceId[]; fields?: Partial<Record<keyof F & string, readonly SourceId[]>> };
  writeBack?: Partial<Record<string, string>>;
  map?: readonly MapRule<keyof F & string>[];
}

export interface Kind<N extends string, F extends Record<string, Field>> {
  name: N;
  spec: KindSpec<F, string>;
  /** Typed field path for CEL: `entity.<field>`. */
  f: { [K in keyof F]: FieldRef };
}
export function kind<const N extends string, const F extends Record<string, Field>, const Rel extends string = never>(
  name: N, spec: KindSpec<F, Rel>,
): Kind<N, F> {
  const f = Object.fromEntries(Object.keys(spec.fields).map((k) => [k, `entity.${k}`])) as Kind<N, F>["f"];
  return { name, spec: spec as KindSpec<F, string>, f };
}

// ------------------------------------------------------------------------------------- scorecards
export interface ScorecardSpec<L extends string> {
  kind: Kind<string, any>;
  owner: string;
  levels: readonly L[];
  appliesTo?: Cel;
  rules: readonly { name: string; level: NoInfer<L>; check: Cel; fix?: string }[];
  facets?: { summary?: Facet; details?: Facet };
}
type Facet = "public" | "owner" | "operator";
export function scorecard<const L extends string>(name: string, spec: ScorecardSpec<L>) {
  return { name, spec };
}

// ---------------------------------------------------------------------------------------- actions
export interface ActionSpec {
  kind: Kind<string, any>;
  scope: "create" | "day2" | "delete" | "global";
  description: string;
  inputs: Record<string, unknown>;
  permission: string;
  approval?: Cel;
  context?: Record<string, { kind: Kind<string, any>; where: Cel }>;
  preconditions?: readonly Cel[];
  executor: { type: "in-image" | "http" | "github-workflow" | "webhook"; command?: readonly string[];
              url?: string; workflow?: string; identity?: "user" | "service"; secrets?: readonly string[] };
  expect?: { postcondition: Cel; within: Duration;
             refresh?: readonly { connector: Connector<string, string>; kind: string; selector?: Cel; ids?: Cel }[] };
  unverified?: true;
  annotations?: { destructive?: boolean; idempotent?: boolean; openWorld?: boolean };
  mcp?: { promote: string };
  ui?: { bundle: UiBundle };
}
/** Exactly one of `expect` or `unverified` (action-runs §5) is enforced by this overload type. */
type VerifiedAction = ActionSpec & ({ expect: NonNullable<ActionSpec["expect"]>; unverified?: never }
                                  | { unverified: true; expect?: never });
export function action(name: string, spec: VerifiedAction) { return { name, spec }; }

export interface UiBundle { name: string; path: string; entry: string; framework?: string; mounts: { slot: string }[] }
export function uiBundle(b: UiBundle): UiBundle { return b; }

// ----------------------------------------------------------------------------------------- portal
export interface PortalSpec {
  portal: { name: string; title?: string; owner: string; toolbox: string };
  policies?: Record<string, unknown>;
  connectors: Connector<string, string>[];
  kinds: Kind<string, any>[];
  scorecards?: ReturnType<typeof scorecard>[];
  authz?: { model: string; tests?: string[];
            tuples: ({ static: { user: string; relation: string; object: string }[] } | { kind: Kind<string, any>; emit: Cel })[] };
  actions?: ReturnType<typeof action>[];
  ui?: { bundles: UiBundle[] };
  mcp?: { instructions?: string };
  tests?: string[];
}

/** Lower the typed objects to the manifest JSON (apiVersion toolbox.dev/v1alpha1). */
export function toManifest(p: PortalSpec): Record<string, unknown> {
  const clean = (o: any): any =>
    Array.isArray(o) ? o.map(clean)
    : o && typeof o === "object"
      ? Object.fromEntries(Object.entries(o).filter(([k, v]) => v !== undefined && !k.startsWith("_")).map(([k, v]) => [k, clean(v)]))
      : o;
  return clean({
    apiVersion: "toolbox.dev/v1alpha1",
    portal: p.portal,
    policies: p.policies,
    connectors: p.connectors.map((c) => ({ name: c.name, ...c.spec })),
    kinds: p.kinds.map((k) => ({ name: k.name, ...k.spec })),
    scorecards: p.scorecards?.map((s) => ({ name: s.name, ...s.spec, kind: s.spec.kind.name })),
    authz: p.authz && {
      ...p.authz,
      tuples: p.authz.tuples.map((t) => ("kind" in t ? { kind: t.kind.name, emit: t.emit } : t)),
    },
    actions: p.actions?.map((a) => ({
      name: a.name, ...a.spec, kind: a.spec.kind.name,
      context: a.spec.context && Object.fromEntries(Object.entries(a.spec.context).map(([n, c]) => [n, { kind: c.kind.name, where: c.where }])),
      expect: a.spec.expect && { ...a.spec.expect,
        refresh: a.spec.expect.refresh?.map((r) => ({ ...r, connector: r.connector.name })) },
      ui: a.spec.ui && { bundle: a.spec.ui.bundle.name },
    })),
    ui: p.ui,
    mcp: p.mcp,
    tests: p.tests,
  });
}

/** Canonical JSON: keys sorted, two-space indent. What `toolbox build` would embed and hash. */
export function canonical(v: unknown): string {
  const sort = (o: any): any =>
    Array.isArray(o) ? o.map(sort)
    : o && typeof o === "object" ? Object.fromEntries(Object.keys(o).sort().map((k) => [k, sort(o[k])])) : o;
  return JSON.stringify(sort(v), null, 2) + "\n";
}
