// Keep the assessment stories focused on published runs and the rubrics and scores they use.
import { readFileSync } from "node:fs";

const data = JSON.parse(readFileSync(process.argv[2], "utf8"));
const allRuns = data.assessmentKinds.flatMap(kind => data.catalog[kind] ?? []);
const runIds = new Set(allRuns);
const crate = "assessment:crate-design/crate:seikika-expr";
const review = "assessment:assessment-quality/" + crate;
const latest = allRuns.toSorted((a, b) => String(data.entities[b].fields.at).localeCompare(String(data.entities[a].fields.at))).slice(0, 12);
const runs = new Set([...latest, crate, review]);
const items = new Set(Object.keys(data.rubrics));
for (const id of runs) {
  const subject = data.entities[id].fields.entity;
  items.add(id);
  items.add(subject);
  if (runIds.has(subject)) runs.add(subject);
}
const project = (entries, keep) => Object.fromEntries(Object.entries(entries).filter(([key]) => keep(key)));
const scores = Object.fromEntries(Object.entries(data.scores).flatMap(([card, rows]) => {
  if (data.comparisons?.[card]) return [];
  const selected = project(rows, id => runs.has(id) || Object.hasOwn(data.rubrics, id));
  return Object.keys(selected).length ? [[card, selected]] : [];
}));
const fixture = {
  portal: data.portal,
  snapshot: data.snapshot,
  assessmentKinds: data.assessmentKinds,
  rubrics: data.rubrics,
  catalog: Object.fromEntries(Object.entries(data.catalog).flatMap(([kind, ids]) => {
    const selected = ids.filter(id => items.has(id));
    return selected.length ? [[kind, selected]] : [];
  })),
  entities: project(data.entities, id => runs.has(id)),
  entityOverview: project(data.entityOverview, id => items.has(id)),
  scorecards: project(data.scorecards, card => Object.hasOwn(scores, card)),
  scores,
};
process.stdout.write(JSON.stringify(fixture) + "\n");
