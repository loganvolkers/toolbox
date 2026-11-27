<script setup lang="ts">
import { computed, ref, watch } from "vue";
import LevelBadge from "@seikika/ui/components/LevelBadge.vue";
import PageHeader from "@seikika/ui/components/PageHeader.vue";
import SectionCard from "@seikika/ui/components/SectionCard.vue";
import AssessmentScores from "./AssessmentScores.vue";
import { assessmentIndex, hasRunTime, rubricTitle, runDate, uniqueLinks, verdictLabel } from "./lib/assessments";
import type { ReportData } from "./types";
import "./assessments.css";

const props = defineProps<{
  data: ReportData;
  mode: "assessments" | "assessment" | "rubrics" | "rubric";
  id?: string;
  filter?: string;
  portal?: boolean;
}>();

const index = computed(() => assessmentIndex(props.data));
const rubrics = computed(() => index.value.rubrics);
const runs = computed(() => index.value.runs);
const run = computed(() => runs.value.find(item => item.id === props.id));
const review = computed(() => runs.value.find(item => item.id === run.value?.reviewId));
const metaScore = computed(() => run.value?.scores.find(item => item.name.split("/").at(-1) === "meta-assessment")
  ?? run.value?.scores.find(item => item.name.split("/").at(-1) !== "review-assessment"));
const reviewScore = computed(() => run.value?.scores.find(item => item.name.split("/").at(-1) === "review-assessment"));
const reviewState = computed(() => {
  if (run.value?.rubricName === "assessment-quality") return "No further review";
  if (run.value?.reviewState === "stale") return "Stale review";
  if (run.value?.reviewVerdict) return "Current independent review";
  if (run.value?.sameAssessor) return "Same assessor";
  if (review.value) return "Independence unverified";
  return run.value?.reviewState === "missing" ? "No review recorded" : "Review status unavailable";
});
const rubric = computed(() => rubrics.value.find(item => item.id === (props.mode === "assessment" ? run.value?.rubricId : props.id)));
const heading = computed(() => props.portal ? "h1" : "h2");
const sectionHeading = computed(() => props.portal ? "h2" : "h3");

const encode = encodeURIComponent;
function view(mode: "assessments" | "assessment" | "rubrics" | "rubric", id?: string) {
  if (props.portal) {
    const root = mode.startsWith("rubric") ? "/assessments/rubrics" : "/assessments";
    return id ? root + (mode === "assessment" ? "/run/" : "/") + encode(id) : root;
  }
  return "#" + mode + (id ? "/" + encode(id) : "");
}
function href(path: string) {
  return props.portal ? "/report/#" + path : "#" + path;
}
const cardHref = (id: string) => href("scorecard/" + encode(id));
const ruleHref = (card: string, rule: string) => href("rule/" + encode(card) + "/" + encode(rule));
const subjectHref = (id: string) => runs.value.some(item => item.id === id)
  ? view("assessment", id)
  : props.portal && !props.data.entitySources?.[id]?.source.startsWith("child:")
    ? "/e/" + encode(id) : href("entity/" + encode(id));
const filteredHref = (id: string) => view("assessments") + "?rubric=" + encode(id);

const selected = ref("");
const selectedVerdict = ref("");
const query = ref("");
const pageSize = 12;
const limit = ref(pageSize);
watch(() => [props.mode, props.filter], () => {
  selected.value = props.filter ?? "";
  selectedVerdict.value = "";
  query.value = "";
}, { immediate: true });
watch([selected, selectedVerdict, query], () => { limit.value = pageSize; });
const filtered = computed(() => {
  const term = query.value.trim().toLowerCase();
  return runs.value.filter(item =>
    (!selected.value || item.rubricId === selected.value) &&
    (!selectedVerdict.value || item.verdict === selectedVerdict.value) &&
    (!term || [item.subjectTitle, item.subject, item.rubricName, item.assessor, item.source]
      .some(value => value?.toLowerCase().includes(term))));
});
function clear() {
  selected.value = selectedVerdict.value = query.value = "";
}
const verdictFilter = ["pass", "fail", "unsure"] as const;
const order = { fail: 0, unsure: 1, pass: 2, unknown: 3 };
const results = computed(() => [...(run.value?.criteria ?? [])].sort((a, b) => order[a.verdict] - order[b.verdict]));
const passed = computed(() => results.value.filter(result => result.verdict === "pass").length);
const questions = computed(() => new Map(rubric.value?.definition.criteria.map(criterion => [criterion.name, criterion.question])));
const links = computed(() => uniqueLinks(rubric.value?.definition.links ?? []));
const evidenceLinks = computed(() => uniqueLinks([
  ...links.value, ...results.value.flatMap(result =>
    [...result.reason.matchAll(/https?:\/\/[^\s`<>"']+/g)].map(([url]) => ({ url: url.replace(/[.,);\]]+$/, "") }))),
]));
const paths = computed(() => [...new Set([run.value?.path, ...(run.value?.citations ?? [])].filter(Boolean))]);
const runTitle = computed(() => run.value?.subjectTitle ?? "Assessment");
const runRubric = computed(() => run.value?.rubricName ? rubricTitle(run.value.rubricName) : "Rubric unavailable");
</script>

<template>
  <section class="assessment-view">
    <nav v-if="mode === 'assessment' || mode === 'rubric'" aria-label="Assessment breadcrumb" class="assessment-breadcrumb">
      <a :href="view('assessments')">Assessments</a>
      <template v-if="mode === 'rubric'"><span aria-hidden="true">/</span><a :href="view('rubrics')">Rubrics</a></template>
    </nav>

    <template v-if="mode === 'assessments' || mode === 'rubrics'">
      <PageHeader title="Assessments" :heading="heading" />
      <nav class="assessment-nav" aria-label="Assessment views">
        <a :href="view('assessments')" :aria-current="mode === 'assessments' ? 'page' : undefined">Recent runs <span>{{ runs.length }}</span></a>
        <a :href="view('rubrics')" :aria-current="mode === 'rubrics' ? 'page' : undefined">Rubrics <span>{{ rubrics.length }}</span></a>
      </nav>
    </template>

    <template v-if="mode === 'assessments'">
      <SectionCard v-if="!runs.length" title="No assessment runs yet" :heading="sectionHeading">
        <p v-if="rubrics.length">Browse the rubrics to see what this portal is ready to assess.</p>
        <p v-else>This portal has no published assessment results or rubrics in this snapshot.</p>
        <a v-if="rubrics.length" :href="view('rubrics')">Browse rubrics</a>
      </SectionCard>
      <SectionCard v-else title="Recent runs" :heading="sectionHeading">
        <p class="assessment-muted">Latest published result for each subject and rubric, newest first.</p>
        <div class="assessment-filters">
          <label>Search runs <input v-model="query" type="search" placeholder="Subject, rubric or assessor"></label>
          <label v-if="rubrics.length">Rubric
            <select v-model="selected">
              <option value="">All rubrics</option>
              <option v-for="item in rubrics" :key="item.id" :value="item.id">{{ rubricTitle(item.definition.name) }}{{ item.source ? ' · ' + item.source : '' }}</option>
            </select>
          </label>
          <label>Result
            <select v-model="selectedVerdict">
              <option value="">All results</option>
              <option v-for="item in verdictFilter" :key="item" :value="item">{{ verdictLabel(item) }}</option>
            </select>
          </label>
        </div>
        <p class="assessment-muted assessment-count" role="status">{{ Math.min(limit, filtered.length) }} of {{ filtered.length }} matching runs</p>
        <ul v-if="filtered.length" class="assessment-list assessment-runs">
          <li v-for="item in filtered.slice(0, limit)" :key="item.id" class="assessment-row">
            <div class="assessment-row-main">
              <a class="assessment-primary" :href="view('assessment', item.id)">{{ item.subjectTitle || item.id }}</a>
              <p class="assessment-muted">{{ item.rubricName ? rubricTitle(item.rubricName) : 'Rubric unavailable' }}<template v-if="item.source"> · {{ item.source }}</template></p>
            </div>
            <div class="assessment-run-status">
              <span class="assessment-verdict" :data-verdict="item.verdict">{{ verdictLabel(item.verdict) }}</span>
              <time v-if="item.at" :datetime="item.at">{{ runDate(item.at) }}</time>
              <span v-else class="assessment-muted">Date unavailable</span>
            </div>
          </li>
        </ul>
        <p v-else>No runs match your filters. <button type="button" class="assessment-button" @click="clear">Clear filters</button></p>
        <button v-if="limit < filtered.length" type="button" class="assessment-button" @click="limit += pageSize">Show more runs</button>
      </SectionCard>
    </template>

    <template v-else-if="mode === 'rubrics'">
      <SectionCard title="Rubrics" :heading="sectionHeading">
        <p v-if="!rubrics.length">This snapshot has no rubric definitions.</p>
        <ul v-else class="assessment-list assessment-rubrics">
          <li v-for="item in rubrics" :key="item.id">
            <div class="assessment-row">
              <div class="assessment-row-main">
                <a class="assessment-primary" :href="view('rubric', item.id)">{{ rubricTitle(item.definition.name) }}</a>
                <p class="assessment-muted">{{ item.definition.kind }}<template v-if="item.source"> · {{ item.source }}</template></p>
              </div>
              <div class="assessment-muted">{{ item.definition.criteria.length }} criteria · {{ item.runs.length }} {{ item.runs.length === 1 ? 'run' : 'runs' }}</div>
            </div>
            <p v-if="item.definition.description">{{ item.definition.description }}</p>
          </li>
        </ul>
      </SectionCard>
    </template>

    <template v-else-if="mode === 'rubric' && rubric">
      <PageHeader :title="rubricTitle(rubric.definition.name)" :description="rubric.definition.description ?? undefined" :heading="heading">
        <template #meta>
          <p class="assessment-muted">Subject kind · {{ rubric.definition.kind }}<template v-if="rubric.source"> · {{ rubric.source }}</template></p>
          <ul v-if="links.length" class="assessment-list assessment-doc-links assessment-meta" aria-label="Rubric documentation">
            <li v-for="link in links" :key="link.url"><a :href="link.url">{{ link.title }}</a></li>
          </ul>
        </template>
      </PageHeader>
      <div class="assessment-detail-grid">
        <SectionCard title="Rubric criteria" :heading="sectionHeading">
          <ol class="assessment-criteria">
            <li v-for="criterion in rubric.definition.criteria" :key="criterion.name">
              <strong>{{ criterion.question }}</strong>
              <p v-if="criterion.guidance">{{ criterion.guidance }}</p>
            </li>
          </ol>
        </SectionCard>
        <aside>
          <SectionCard title="Rubric quality" :heading="sectionHeading">
            <AssessmentScores :scores="rubric.scores" :card-href="cardHref" :rule-href="ruleHref" />
          </SectionCard>
          <SectionCard title="Runs for this rubric" :heading="sectionHeading">
            <a v-if="rubric.runs.length" :href="filteredHref(rubric.id)">View {{ rubric.runs.length }} {{ rubric.runs.length === 1 ? 'run' : 'runs' }}</a>
            <p v-else class="assessment-muted">No published runs yet.</p>
          </SectionCard>
        </aside>
      </div>
    </template>

    <template v-else-if="mode === 'assessment' && run">
      <PageHeader :title="runTitle" :heading="heading">
        <template #meta>
          <div class="assessment-meta">
            <a v-if="run.rubricId" :href="view('rubric', run.rubricId)">{{ runRubric }} rubric</a>
            <span v-else>{{ runRubric }}</span>
            <a v-if="run.subjectId" :href="subjectHref(run.subjectId)" :title="run.subject">Open assessed subject</a>
            <span v-if="run.source">{{ run.source }}</span>
          </div>
        </template>
      </PageHeader>
      <div class="assessment-result" aria-label="Assessment outcome summary" role="group">
        <div>
          <span class="assessment-muted">Assessment verdict</span>
          <strong class="assessment-verdict" :data-verdict="run.verdict">{{ verdictLabel(run.verdict) }}</strong>
          <p v-if="results.length">{{ passed }} of {{ results.length }} criteria passed</p>
          <p v-if="run.at" class="assessment-muted">
            <time :datetime="run.at">{{ runDate(run.at) }}</time><span v-if="!hasRunTime(run.at)"> · date only</span>
          </p>
          <p v-else class="assessment-muted">Date unavailable</p>
        </div>
        <div>
          <span class="assessment-muted">Scorecard checks</span>
          <p v-if="metaScore"><LevelBadge :level="metaScore.score.level" /> <span class="assessment-muted">· {{ metaScore.definition.title || rubricTitle(metaScore.name) }}</span></p>
          <p v-if="reviewScore"><LevelBadge :level="reviewScore.score.level" /> <span class="assessment-muted">· Review checks</span></p>
          <p v-if="!metaScore && !reviewScore">No scores available</p>
        </div>
        <div class="assessment-review-summary">
          <span class="assessment-muted">Independent review</span>
          <strong>{{ reviewState }}</strong>
          <p v-if="review || run.reviewVerdict">
            <span v-if="review && review.verdict !== 'unknown'">Recorded verdict <strong>{{ verdictLabel(review.verdict) }}</strong></span>
            <span v-else-if="run.reviewVerdict">Recorded verdict <strong>{{ verdictLabel(run.reviewVerdict) }}</strong></span>
            <template v-if="review?.at"> · <time :datetime="review.at">{{ runDate(review.at) }}</time></template>
          </p>
          <p v-if="run.reviewState === 'stale' || run.sameAssessor" class="assessment-muted">
            <template v-if="run.reviewState === 'stale'">The assessment changed after the review.</template>
            <template v-if="run.sameAssessor"> Same assessor.</template>
          </p>
          <a v-if="review" :href="view('assessment', review.id)">Read the review</a>
        </div>
      </div>
      <details v-if="run.summary" class="assessment-summary">
        <summary>{{ run.assessor ? 'Summary by ' + run.assessor : 'Assessment summary' }}</summary>
        <p>{{ run.summary }}</p>
      </details>
      <div class="assessment-detail-grid">
        <SectionCard :key="run.id" title="Criterion results" :heading="sectionHeading">
          <p v-if="!results.length" class="assessment-muted">Criterion results are unavailable in this snapshot.</p>
          <ul v-else class="assessment-list assessment-criteria">
            <li v-for="(result, i) in results" :key="i">
              <details :open="i === 0 && result.verdict === 'fail'">
                <summary class="assessment-row">
                  <strong>{{ result.name ? rubricTitle(result.name) : 'Criterion' }}</strong>
                  <span class="assessment-verdict" :data-verdict="result.verdict">{{ verdictLabel(result.verdict) }}</span>
                </summary>
                <p v-if="questions.has(result.name)" class="assessment-muted">{{ questions.get(result.name) }}</p>
                <p>{{ result.reason }}</p>
                <p v-if="result.suggestion"><strong>Suggested change</strong> {{ result.suggestion }}</p>
              </details>
            </li>
          </ul>
        </SectionCard>
        <aside>
          <SectionCard title="Assessment quality scorecards" :heading="sectionHeading">
            <AssessmentScores :key="run.id" :scores="run.scores" :card-href="cardHref" :rule-href="ruleHref" />
          </SectionCard>
          <SectionCard v-if="paths.length || evidenceLinks.length" title="Sources" :heading="sectionHeading">
            <ul class="assessment-list assessment-doc-links">
              <li v-for="link in evidenceLinks" :key="link.url"><a :href="link.url">{{ link.title }}</a></li>
              <li v-for="path in paths" :key="path"><code>{{ path }}</code></li>
            </ul>
          </SectionCard>
        </aside>
      </div>
    </template>

    <PageHeader v-else title="Assessment page not found" description="This link has no match in the current snapshot." :heading="heading" />
  </section>
</template>
