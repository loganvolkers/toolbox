import type { Link, ReportData, Rubric, Score, ScorecardDef } from "../types";

const verdictLabels = { pass: "Pass", fail: "Fail", unsure: "Unsure", unknown: "Result unavailable" };
export type Verdict = keyof typeof verdictLabels;
export const verdictLabel = (value: Verdict) => verdictLabels[value];
const verdict = (value: unknown): Verdict =>
  value === "pass" || value === "fail" || value === "unsure" ? value : "unknown";

export interface AssessmentFilters {
  q?: string;
  rubric?: string;
  verdict?: string;
}

const text = (value: unknown) => typeof value === "string" && value.trim() ? value.trim() : undefined;
const strings = (value: unknown) => Array.isArray(value) ? value.filter((item): item is string => typeof item === "string") : [];

export function rubricTitle(name: string) {
  return name.replace(/[-_]+/g, " ").replace(/^./, letter => letter.toUpperCase());
}

export function publicLink(url: string) {
  try {
    const parsed = new URL(url);
    if (["https:", "http:"].includes(parsed.protocol)) return parsed.href;
  } catch {
    // A repository path is displayed as text when no URL is available.
  }
}

export function uniqueLinks(links: Link[]) {
  const urls = new Set<string>();
  return links.flatMap(link => {
    const url = publicLink(link.url);
    if (!url || urls.has(url)) return [];
    urls.add(url);
    return [{ title: link.title || url, url }];
  });
}

export interface AssessmentScore {
  name: string;
  definition: ScorecardDef;
  score: Score;
}
export function ruleProgress(score: Score) {
  const results = Object.values(score.rules ?? {});
  if (results.length) return results.filter(result => result === "pass").length + " of " + results.length + " rules passed";
}

export function nextRequirement({ definition, score }: AssessmentScore) {
  const index = definition.levels.indexOf(score.level);
  if (index < 0 && score.level !== "none") return;
  const level = definition.levels[index + 1];
  const rule = Object.entries(definition.rules).find(([name, rule]) => rule.level === level && score.rules?.[name] === "fail");
  if (rule) return { level, name: rule[0], title: rule[1].title || rubricTitle(rule[0]) };
}
export interface CriterionResult {
  name: string;
  verdict: Verdict;
  reason: string;
  suggestion?: string;
}
export interface AssessmentRun {
  id: string;
  rubricName?: string;
  rubricId?: string;
  subject?: string;
  subjectId?: string;
  subjectTitle?: string;
  source?: string;
  at?: string;
  verdict: Verdict;
  assessor?: string;
  summary?: string;
  criteria: CriterionResult[];
  path?: string;
  citations: string[];
  scores: AssessmentScore[];
  reviewState?: string;
  reviewVerdict?: Verdict;
  reviewId?: string;
  sameAssessor: boolean;
  isReview: boolean;
  reviewProvenance?: { stale: boolean; sameAssessor: boolean };
}
export function reviewQualification(run: AssessmentRun) {
  const provenance = run.reviewProvenance;
  if (provenance?.stale) return provenance.sameAssessor ? "Stale · same assessor" : "Stale review";
  if (provenance?.sameAssessor) return "Same assessor";
}
export interface AssessmentRubric {
  id: string;
  definition: Rubric;
  source?: string;
  runs: AssessmentRun[];
  scores: AssessmentScore[];
}

const dateOnly = /^\d{4}-\d{2}-\d{2}$/;
const zone = /(?:Z|[+-]\d{2}:?\d{2})$/i;
export const hasRunTime = (at: string) => !dateOnly.test(at);

function dateOrder(at?: string) {
  const parsed = Date.parse(at && at.includes("T") && !zone.test(at) ? at + "Z" : at ?? "");
  return Number.isNaN(parsed) ? Number.NEGATIVE_INFINITY : parsed;
}

const dayFormat = new Intl.DateTimeFormat("en", { dateStyle: "medium", timeZone: "UTC" });
const timeFormat = new Intl.DateTimeFormat("en", { hour: "2-digit", minute: "2-digit", hourCycle: "h23", timeZone: "UTC" });
export function runDate(at: string) {
  const stamp = dateOrder(at);
  if (!Number.isFinite(stamp)) return at;
  if (dateOnly.test(at)) return dayFormat.format(stamp);
  if (zone.test(at)) return dayFormat.format(stamp) + ", " + timeFormat.format(stamp) + " UTC";
  const [day, time] = at.split("T");
  return day && time ? dayFormat.format(Date.parse(day)) + ", " + time.slice(0, 5) + " (zone unspecified)" : at;
}

export function assessmentIndex(data: ReportData) {
  const kinds = data.assessmentKinds ?? (data.catalog.Assessment ? ["Assessment"] : []);
  const runIds = new Set(data.assessmentIds ?? kinds.flatMap(kind => data.catalog[kind] ?? []));
  const visible = new Set(Object.values(data.catalog).flat());
  const original = (id: string) => data.entitySources?.[id]?.original ?? id;
  const source = (id: string) => data.entitySources?.[id]?.source ?? "local";
  const sourceTitle = (id: string) => {
    const item = data.sources?.[source(id)];
    return item && !item.local ? item.title : undefined;
  };
  const key = (scope: string, name: string) => JSON.stringify([scope, name]);
  const visibleBySource = new Map([...visible].map(id => [key(source(id), original(id)), id]));
  for (const [id, observations] of Object.entries(data.entityObservations ?? {})) {
    for (const item of observations) visibleBySource.set(key(item.source, item.original), id);
  }
  const rubricsByName = new Map(Object.entries(data.rubrics ?? {}).map(([id, rubric]) => [key(source(id), rubric.name), id]));
  const allCardScores = Object.entries(data.scorecards);
  const scoresFor = (id: string): AssessmentScore[] => allCardScores.flatMap(([name, definition]) => {
    const score = data.scores[name]?.[id];
    return score ? [{ name, definition, score }] : [];
  });
  const idTitle = (id: string) => id.includes(":") ? id.slice(id.indexOf(":") + 1) : id;
  function subjectTitleFor(id: string, subjectId?: string) {
    const overview = subjectId ? data.entityOverview?.[subjectId] : undefined;
    const title = text(overview?.title) ?? text(overview?.displayName);
    const assessed = /^assessment:([^/]+)\/(.+)$/.exec(id);
    return title ?? (assessed ? rubricTitle(assessed[1]) + " assessment of " + idTitle(assessed[2]) : idTitle(id));
  }
  const runs: AssessmentRun[] = [...runIds].map(id => {
    const fields = data.entities?.[id]?.fields ?? {};
    // Older snapshots have catalog IDs and scores but no projected assessment fields.
    const legacy = !data.entities?.[id] && /^assessment:([^/]+)\/(.+)$/.exec(original(id));
    const rubricName = text(fields.rubric) ?? (legacy ? legacy[1] : undefined);
    const subject = text(fields.entity) ?? (legacy ? legacy[2] : undefined);
    const subjectId = subject && visibleBySource.get(key(source(id), subject));
    const suggestions = new Map(strings(fields.suggestions).flatMap(line => {
      const match = /^([\w-]+):\s*([\s\S]+)$/.exec(line);
      return match ? [[match[1], match[2]] as const] : [];
    }));
    const criteria = strings(fields.reasons).map((line): CriterionResult => {
      const match = /^(pass|fail|unsure) ([\w-]+):\s*([\s\S]*)$/.exec(line);
      return match
        ? { name: match[2], verdict: verdict(match[1]), reason: match[3], suggestion: suggestions.get(match[2]) }
        : { name: "", verdict: "unknown", reason: line };
    });
    const review = verdict(fields.independentReview);
    return {
      id, rubricName, rubricId: rubricName && rubricsByName.get(key(source(id), rubricName)),
      subject, subjectId, subjectTitle: subject ? subjectTitleFor(subject, subjectId) : undefined,
      source: sourceTitle(id), at: text(fields.at), verdict: verdict(fields.verdict),
      assessor: text(fields.assessorName) ?? text(fields.assessorKind),
      summary: text(fields.summary), criteria, path: text(fields.path), citations: strings(fields.cited),
      scores: scoresFor(id), reviewState: text(fields.reviewState),
      reviewVerdict: review === "unknown" ? undefined : review,
      sameAssessor: fields.selfReviewed === true,
      isReview: Boolean(subjectId && runIds.has(subjectId) || subject?.startsWith("assessment:")),
    };
  }).sort((a, b) => dateOrder(b.at) - dateOrder(a.at) || a.id.localeCompare(b.id));
  const reviewsBySubject = new Map<string, AssessmentRun[]>();
  const runsByRubric = new Map<string, AssessmentRun[]>();
  for (const run of runs) {
    if (run.subjectId) {
      const reviews = reviewsBySubject.get(run.subjectId) ?? [];
      reviews.push(run);
      reviewsBySubject.set(run.subjectId, reviews);
    }
    if (run.rubricId) {
      const group = runsByRubric.get(run.rubricId) ?? [];
      group.push(run);
      runsByRubric.set(run.rubricId, group);
    }
  }
  for (const run of runs) {
    // Seikika's assessment-quality rubric records the independent review of an assessment.
    const review = reviewsBySubject.get(run.id)?.find(item => item.rubricName === "assessment-quality");
    if (review) {
      run.reviewId = review.id;
      review.reviewProvenance = { stale: run.reviewState === "stale", sameAssessor: run.sameAssessor };
    }
  }
  const rubrics: AssessmentRubric[] = Object.entries(data.rubrics ?? {}).map(([id, definition]) => ({
    id, definition, source: sourceTitle(id), runs: runsByRubric.get(id) ?? [], scores: scoresFor(id),
  })).sort((a, b) => a.definition.name.localeCompare(b.definition.name) || a.id.localeCompare(b.id));
  return { runs, rubrics };
}
