// The report document `seikika report` inlines into the page (report_json in
// crates/seikika-cli/src/report.rs). Fields the Rust side writes as null are optional here.

export type Level = string;
export type Outcome = "pass" | "fail" | "unknown" | string;
export type Standing = "ahead" | "parity" | "behind" | "unknown";

export interface Link {
  title?: string | null;
  url: string;
}

export interface Rubric {
  name: string;
  kind: string;
  description?: string | null;
  subject?: string | null;
  criteria: { name: string; question: string; guidance?: string | null }[];
  links?: Link[];
}

export interface RuleDef {
  level: Level;
  fix?: string | null;
  check: string;
  basis: "judged" | "measured";
  title?: string;
  rationale?: string;
  links?: Link[];
  value?: string | null;
  better?: "lower" | "higher" | string | null;
  unit?: string | null;
}

export interface ScorecardDef {
  title?: string | null;
  description?: string | null;
  rationale?: string | null;
  links?: Link[];
  kind: string;
  levels: Level[];
  owner?: string | null;
  track?: string | null;
  rules: Record<string, RuleDef>;
  compare?: unknown;
  appliesTo?: string;
}

export interface Score {
  level: Level;
  rules: Record<string, Outcome>;
  values?: Record<string, unknown>;
  measures?: Record<string, number | null>;
  evidence?: Record<string, string | string[]>;
}

export interface Comparison {
  subject: string;
  peers: string[];
  verdict: Record<string, Standing>;
  verdictRule: "anyBehind" | "majority";
  rules: Record<string, Record<string, Standing>>;
  titles?: Record<string, string | null>;
  facts?: Record<string, Record<string, unknown>>;
  reads?: Record<string, string[]>;
  fields?: Record<string, string | null>;
  scores?: Record<string, Score>;
  [extra: string]: unknown;
}

export interface CampaignInclude {
  scorecard: string;
  atLeast?: Level | null;
  rules?: string[] | null;
}

export interface Campaign {
  title?: string | null;
  track?: string | null;
  owner?: string | null;
  description?: string | null;
  start?: string | null;
  end?: string | null;
  targetPercent?: number | null;
  draft?: boolean;
  includes: CampaignInclude[];
  status?: string | null;
  inScope?: number | null;
  meeting?: number | null;
  percent?: number | null;
  onTarget?: boolean | null;
  measure?: number | null;
  sinceStart?: SinceStart | null;
  milestones?: Milestone[] | null;
  entities?: Record<string, { meeting: boolean; gaps?: string[] }> | null;
}

export interface Milestone {
  name: string;
  targetPercent: number;
  met: boolean;
}

export interface Paydown {
  from: number;
  now: number;
  percentDone: number;
}

export interface SinceStart {
  start?: string | null;
  recorded?: string | null;
  entitiesShort?: Paydown;
  measure?: Paydown;
}

export interface Track {
  title?: string | null;
  owner?: string | null;
  description?: string | null;
  links?: Link[];
}

export interface Kind {
  title?: string | null;
  description?: string | null;
}

export interface KindGroup {
  name: string;
  title?: string | null;
  description?: string | null;
  kinds: string[];
}

export interface ReportData {
  portal: string;
  organization?: string;
  viewer?: string;
  actions?: ReportAction[];
  children?: ReportChild[];
  entities?: Record<string, ReportEntity>;
  assessmentKinds?: string[];
  rubrics?: Record<string, Rubric>;
  sources?: Record<string, { title: string; local?: boolean; repository?: boolean; commit?: string; error?: string }>;
  entitySources?: Record<string, { source: string; original: string }>;
  snapshot?: string | number | null;
  catalog: Record<string, string[]>;
  entityOverview?: Record<string, Record<string, unknown>>;
  kinds?: Record<string, Kind>;
  kindGroups?: KindGroup[];
  tracks?: Record<string, Track>;
  campaigns?: Record<string, Campaign>;
  scorecards: Record<string, ScorecardDef>;
  scores: Record<string, Record<string, Score>>;
  unassessed?: Record<string, string[]>;
  comparisons?: Record<string, Comparison>;
  repoRollup?: string[] | null;
}

export interface ActionInput {
  title?: string;
  description?: string;
  type?: string;
  enum?: (string | number | boolean)[];
  default?: unknown;
  items?: ActionInput;
}

export interface ReportAction {
  name: string;
  description?: string;
  kind: string;
  entities: string[];
  inputs?: { properties?: Record<string, ActionInput>; required?: string[] } | null;
  choices?: Record<string, Record<string, string[]>>;
  executor: string;
  scope?: string;
  approval?: string | null;
  permission?: string;
  verified?: boolean;
  portal?: { key: string; name?: string };
  source?: string;
  target?: string;
  previewOnly?: boolean;
}

export interface ReportChild {
  key: string;
  title: string;
  commit?: string;
  error?: string;
  report?: ReportData;
}

export interface ReportEntity {
  fields: Record<string, unknown>;
  provenance?: Record<string, string>;
  relations?: {
    in: { relation: string; from: string }[];
    out: { relation: string; to: string }[];
  };
}
