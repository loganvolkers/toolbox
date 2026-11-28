import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { copyFile, cp, mkdir, mkdtemp, readFile, rm, symlink } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const binary = resolve(process.env.SEIKIKA_BIN ?? join(root, "target/debug/seikika"));
const scratch = await mkdtemp(join(tmpdir(), "seikika-federation-parity-"));
const key = 'child:["seikika/seikika"]';
const prefix = "@seikika%2Fseikika/";
const canonical = "repository:seikika/seikika";
const original = "repo:seikika/seikika";
const read = async path => JSON.parse(await readFile(path, "utf8"));
const run = (...args) => execFileSync(binary, args, { cwd: root, stdio: "inherit" });
const output = process.env.SEIKIKA_FEDERATION_OUT ? resolve(process.env.SEIKIKA_FEDERATION_OUT) : join(scratch, "federated");
let vite;
try {
  const standalone = join(scratch, "standalone");
  const publication = process.env.SEIKIKA_SELF_REPORT ?? join(standalone, "report.json");
  if (!process.env.SEIKIKA_SELF_REPORT) {
    run("report", ".seikika", "--skip", "runner,github-actions", "-o", "web", "--out", standalone);
  }
  const example = join(scratch, "examples/github-org-federation");
  await mkdir(join(scratch, "examples"));
  await cp(join(root, "examples/github-org-federation"), example, {
    recursive: true, filter: path => !path.endsWith("/publications"),
  });
  await symlink(join(root, "packages"), join(scratch, "packages"), "dir");
  await mkdir(join(example, "publications"));
  await copyFile(publication, join(example, "publications/seikika.json"));
  run("report", join(example, "public.yaml"), "--fixtures", "-o", "web", "--out", output);

  vite = await createServer({ root, configFile: false, server: { middlewareMode: true, ws: false }, appType: "custom" });
  const { aggregateReport, entitySourceKeys } = await vite.ssrLoadModule("/npm/report-ui/src/lib/federation.ts");
  const { assessmentIndex } = await vite.ssrLoadModule("/npm/report-ui/src/lib/assessments.ts");
  const { campaignStanding } = await vite.ssrLoadModule("/npm/report-ui/src/lib/campaigns.ts");
  const source = await read(publication);
  const snapshot = await read(join(output, "report.json"));
  const published = snapshot.children.find(child => child.key === "seikika/seikika");
  assert.match(published.digest, /^[a-f0-9]{64}$/);
  const { actions, children, ...publicSource } = source;
  assert.deepEqual(published.report, { ...publicSource, children });
  const parent = aggregateReport(snapshot);
  const local = aggregateReport(source);
  assert.equal(parent.sources[key].title, "Seikika for Seikika");
  assert.equal(parent.sources[key].error, undefined);
  const unmeasured = Object.values(parent.campaigns).find(campaign => campaign.title.startsWith("Review the organization portal"));
  assert.equal(campaignStanding(parent, unmeasured), "empty", "an unscored campaign cannot count as on target");
  const counts = {};
  for (const section of ["tracks", "campaigns", "scorecards", "comparisons", "rubrics"]) {
    const ids = Object.keys(local[section] ?? {}).sort();
    assert(ids.length > 0, section);
    counts[section] = ids.length;
    assert.deepEqual(Object.keys(parent[section]).filter(id => id.startsWith(prefix)).sort(), ids.map(id => prefix + id), section);
  }
  counts.scorecards -= counts.comparisons;
  let total = 0;
  for (const [kind, ids] of Object.entries(local.catalog)) {
    for (const id of ids) {
      if (kind === "Action" && !source.catalog.Action?.includes(id)) continue;
      total++;
      const target = id === original ? canonical : prefix + id;
      assert(parent.catalog[kind]?.includes(target), `${kind} ${target} is missing from the catalog`);
      assert(entitySourceKeys(parent, target).includes(key), `${kind} ${target} lost its source`);
    }
  }
  counts.catalog = total;
  const childAssessments = assessmentIndex(parent).runs.filter(run => run.source === "Seikika for Seikika");
  const localAssessments = assessmentIndex(local).runs;
  counts.assessments = localAssessments.length;
  assert(counts.assessments > 0);
  assert.deepEqual(childAssessments.map(run => run.id).sort(), localAssessments.map(run => prefix + run.id).sort());
  assert.equal(parent.catalog.Repository.filter(id => id === canonical || id.includes(original)).length, 1);
  assert.deepEqual(parent.entities[canonical], snapshot.entities[canonical], "child data cannot overwrite parent facts");
  assert.deepEqual(parent.entityObservations[canonical][0].detail.fields, source.entities[original].fields);
  assert.deepEqual(parent.scores["repo-hygiene"][canonical], snapshot.scores["repo-hygiene"][canonical]);
  assert.equal(parent.scores["repository-identity"][canonical].level, "silver");
  assert.equal(parent.scores[prefix + "maintainer-repo"][canonical].level, source.scores["maintainer-repo"][original].level);

  const sibling = { ...published, key: "other/project", title: "Other project" };
  const multiple = aggregateReport({ ...snapshot, children: [published, sibling] });
  assert(multiple.catalog.Repository.includes("@other%2Fproject/" + original), "equal unapproved identities stay distinct");
  assert.equal(multiple.comparisonSources["@other%2Fproject/" + Object.keys(local.comparisons)[0]], 'child:["other/project"]');
  const ambiguous = aggregateReport({ ...snapshot, catalogAliases: [...snapshot.catalogAliases,
    { ...snapshot.catalogAliases[0], to: "repository:seikika/ready" }] });
  assert(ambiguous.catalog.Repository.includes(prefix + original), "conflicting aliases keep both records visible");
  console.log("Seikika for Seikika federation parity", counts);
} finally {
  await vite?.close();
  await rm(scratch, { recursive: true, force: true });
}
