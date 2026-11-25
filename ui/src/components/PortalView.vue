<script setup lang="ts">
import { computed, ref, watch } from "vue";
import DataValue from "@toolbox-dev/ui/components/DataValue.vue";
import EntityPage from "@toolbox-dev/ui/components/EntityPage.vue";
import LevelBadge from "@toolbox-dev/ui/components/LevelBadge.vue";
import PageHeader from "@toolbox-dev/ui/components/PageHeader.vue";
import ScorecardPage from "@toolbox-dev/ui/components/ScorecardPage.vue";
import SectionCard from "@toolbox-dev/ui/components/SectionCard.vue";
import { entityHref, entityOverview } from "@toolbox-dev/ui/entity";
import type { ApiResult, Json } from "../api";
import type { PortalRoute } from "../routes";

const props = defineProps<{
  route: PortalRoute;
  result: ApiResult | null;
  ownership?: { status: string; explanation?: string; fix?: string } | null;
  refresh?: { running: boolean; failed: boolean; message: string };
}>();
const emit = defineEmits<{ navigate: [url: string]; refresh: []; retry: [] }>();
const encode = encodeURIComponent;
const body = computed(() => props.result?.body);
const draft = ref("");
watch(() => props.route, route => { draft.value = route.page === "catalog" ? route.filter : ""; }, { immediate: true });

const overview = computed(() => {
  if (props.route.page !== "entity" || !body.value?.entity) return null;
  const row = body.value.entity;
  return entityOverview(row.id ?? props.route.id, row.kind, row.fields);
});
const campaign = computed(() => body.value?.campaign ?? body.value);
const gaps = computed(() => (campaign.value?.entities ?? []).filter((entity: Json) => !entity.meeting));
const entries = (record: Record<string, Json> | undefined) => Object.entries(record ?? {});
function filter() {
  if (props.route.page !== "catalog") return;
  emit("navigate", "/c/" + encode(props.route.id) + (draft.value ? "?filter=" + encode(draft.value) : ""));
}
</script>

<template>
  <div v-if="route.page === 'missing'">
    <PageHeader title="Page not found" />
    <a href="/">Home</a>
  </div>
  <template v-else>
    <PageHeader v-if="route.page === 'search'" :title="route.query ? 'Search · ' + route.query : 'Search'" />
    <template v-if="route.page === 'catalog'">
      <PageHeader :title="route.id" />
      <form class="filter" @submit.prevent="filter">
        <label for="filter">CEL filter over <code>entity</code></label>
        <input id="filter" v-model="draft" placeholder="entity.ownership.status != 'verified'" />
        <button type="submit">Apply</button>
      </form>
    </template>

    <div v-if="!result" class="skeleton" role="status" aria-busy="true" aria-label="Loading"></div>
    <div v-else-if="result.status !== 200" class="problem" role="alert">
      <strong>{{ body?.title ?? "HTTP " + result.status }}</strong> {{ body?.detail }}
      <p v-if="result.status === 401">Sign in through your identity-aware proxy, or on a dev server add <code>?as=&lt;user&gt;</code> to the URL.</p>
      <button v-if="result.status === 0" @click="emit('retry')">Retry</button>
    </div>

    <template v-else-if="route.page === 'home'">
      <PageHeader :title="body.portal?.title ?? 'Toolbox'">
        <template #meta>
          <p class="tb-meta">Signed in as {{ body.viewer.user }}{{ body.viewer.admin ? ' (portal admin)' : '' }}.
            <template v-if="result.snapshot">As of snapshot <code>{{ result.snapshot }}</code>.</template>
          </p>
        </template>
      </PageHeader>
      <SectionCard title="Your teams’ entities">
        <p v-if="!body.teams.length" class="muted">Your teams have no owned entities yet.</p>
        <table v-if="body.owned.length">
          <thead><tr><th>Entity</th><th>Levels</th><th>Failing rules</th></tr></thead>
          <tbody>
            <tr v-for="entity in body.owned" :key="entity.id">
              <td><a :href="entityHref(entity.id)">{{ entity.id }}</a></td>
              <td><span v-for="[name, level] in entries(entity.levels)" :key="name" class="chip">{{ name }} <LevelBadge :level="level" /></span></td>
              <td>{{ entity.failing.join(', ') || 'none' }}</td>
            </tr>
          </tbody>
        </table>
      </SectionCard>
      <div class="grid">
        <SectionCard title="Browse">
          <ul><li v-for="[kind, count] in entries(body.kinds)" :key="kind"><a :href="'/c/' + encode(kind)">{{ kind }}</a> <span class="muted">({{ count }})</span></li></ul>
        </SectionCard>
        <SectionCard title="Scorecards">
          <ul><li v-for="card in body.scorecards" :key="card.name"><a :href="'/s/' + encode(card.name)">{{ card.name }}</a> <span class="muted">on {{ card.kind }}</span></li></ul>
        </SectionCard>
        <SectionCard v-if="body.campaigns?.length" title="Campaigns">
          <ul><li v-for="item in body.campaigns" :key="item.name"><a :href="'/cgn/' + encode(item.name)">{{ item.title ?? item.name }}</a> <span class="muted">{{ item.percent }}% / {{ item.targetPercent }}%</span></li></ul>
        </SectionCard>
      </div>
    </template>

    <template v-else-if="route.page === 'search' || route.page === 'catalog'">
      <p v-if="!body.items.length" class="muted">{{ route.page === 'search' ? 'Nothing matches.' : 'No entities match.' }}</p>
      <table v-else>
        <thead><tr><th>Entity</th><th>Title</th><th>Owner</th></tr></thead>
        <tbody>
          <tr v-for="entity in body.items" :key="entity.id">
            <td><a :href="entityHref(entity.id)">{{ entity.id }}</a></td>
            <td><DataValue :value="entity.fields.title ?? entity.fields.displayName" /></td>
            <td>
              <DataValue :value="entity.fields.ownership?.owner ?? entity.fields.owner" :entity-href="entityHref" />
              <span v-if="entity.fields.ownership?.status" class="status" :class="'status-' + entity.fields.ownership.status">{{ entity.fields.ownership.status }}</span>
            </td>
          </tr>
        </tbody>
      </table>
    </template>

    <EntityPage v-else-if="route.page === 'entity' && overview" :entity="overview"
      :owner-href="overview.owner && entityHref(overview.owner)" :snapshot="body.snapshot ?? result.snapshot">
      <template #header-actions>
        <span class="refresh">
          <button type="button" :disabled="refresh?.running" :aria-busy="refresh?.running" @click="emit('refresh')">Refresh</button>
          <span role="status" class="muted" :class="{ error: refresh?.failed, done: !refresh?.running && !refresh?.failed }">{{ refresh?.message }}</span>
        </span>
      </template>
      <div v-if="ownership" class="callout" :class="'status-' + ownership.status">
        <strong>Ownership · {{ ownership.status }}</strong> {{ ownership.explanation }}
        <em v-if="ownership.fix">{{ ownership.fix }}</em>
      </div>
      <div class="grid">
        <SectionCard title="Fields">
          <dl class="fields">
            <div v-for="[key, value] in entries(body.entity.fields).filter(([name]) => name !== 'ownership')" :key="key">
              <dt :title="'From ' + (body.entity.provenance?.[key] ?? 'unknown')">{{ key }}</dt>
              <dd><DataValue :value="value" :entity-href="entityHref" /> <span class="source">{{ body.entity.provenance?.[key] }}</span></dd>
            </div>
          </dl>
        </SectionCard>
        <SectionCard title="Related entities">
          <ul v-if="body.relations.out.length || body.relations.in.length">
            <li v-for="(relation, i) in body.relations.out" :key="'out-' + i">{{ relation.relation }} → <a :href="entityHref(relation.to)">{{ relation.to }}</a></li>
            <li v-for="(relation, i) in body.relations.in" :key="'in-' + i"><a :href="entityHref(relation.from)">{{ relation.from }}</a> → {{ relation.relation }}</li>
          </ul>
          <p v-else class="muted">None visible.</p>
        </SectionCard>
      </div>
      <template #scorecards>
        <SectionCard title="Scorecards">
          <ScorecardPage v-for="[name, card] in entries(body.scorecards)" :key="name" :title="name" heading="h2">
            <template #header>
              <h3><a :href="'/s/' + encode(name)">{{ name }}</a> <LevelBadge :level="card.level" /></h3>
            </template>
            <DataValue v-if="card.rules?.redacted" :value="card.rules" />
            <ul v-else-if="card.rules" class="rules">
              <li v-for="[name, rule] in entries(card.rules)" :key="name" :class="'rule-' + rule.outcome">
                <span class="outcome">{{ rule.outcome }}</span> {{ name }}
                <div v-if="rule.fix" class="fix">{{ rule.fix }}</div>
              </li>
            </ul>
          </ScorecardPage>
          <p v-if="!entries(body.scorecards).length" class="muted">No scorecards apply.</p>
        </SectionCard>
      </template>
      <template #actions>
        <div class="grid">
          <SectionCard title="Actions">
            <ul v-if="body.actions.length">
              <li v-for="action in body.actions" :key="action.name">
                <strong>{{ action.name }}</strong> {{ action.description }}
                <span v-if="action.allowed" class="ok">available</span>
                <span v-else class="muted">(needs {{ action.permission }})</span>
              </li>
            </ul>
            <p v-else class="muted">No actions.</p>
          </SectionCard>
          <SectionCard title="Freshness">
            <ul><li v-for="source in body.freshness" :key="source.connector + '/' + source.kind">{{ source.connector }}/{{ source.kind }} · {{ source.last_read ?? 'never read by a connector' }}</li></ul>
          </SectionCard>
        </div>
      </template>
    </EntityPage>

    <ScorecardPage v-else-if="route.page === 'scorecard'" :title="body.scorecard.title ?? route.id" :description="body.scorecard.description">
      <template #meta>
        <p class="tb-meta">On {{ body.scorecard.kind }}, owned by <DataValue :value="body.scorecard.owner" :entity-href="entityHref" />.
          <template v-if="result.snapshot">As of snapshot <code>{{ result.snapshot }}</code>.</template>
        </p>
      </template>
      <div class="grid">
        <SectionCard title="By level">
          <div v-for="[level, ids] in entries(body.by_level)" :key="level">
            <h3><LevelBadge :level="level" /> <span class="muted">({{ ids.length }})</span></h3>
            <ul><li v-for="id in ids" :key="id"><a :href="entityHref(id)">{{ id }}</a></li></ul>
          </div>
        </SectionCard>
        <SectionCard title="Failing rules">
          <ul v-if="entries(body.failing_rules).length"><li v-for="[rule, count] in entries(body.failing_rules)" :key="rule">{{ rule }} · {{ count }}</li></ul>
          <p v-else class="muted">None you can see.</p>
        </SectionCard>
      </div>
    </ScorecardPage>

    <template v-else-if="route.page === 'campaign'">
      <PageHeader :title="campaign.title ?? campaign.name" :description="campaign.description">
        <template #meta>
          <p class="tb-meta">{{ campaign.status }} · {{ campaign.percent }}% of {{ campaign.targetPercent }}% target ({{ campaign.meeting }}/{{ campaign.inScope }}).
            <template v-if="result.snapshot">As of snapshot <code>{{ result.snapshot }}</code>.</template>
          </p>
        </template>
      </PageHeader>
      <SectionCard :title="'Not meeting (' + gaps.length + ')'">
        <ul><li v-for="entity in gaps" :key="entity.id"><a :href="entityHref(entity.id)">{{ entity.id }}</a> <span class="muted">{{ entity.gaps?.join(', ') }}</span></li></ul>
        <p v-if="!gaps.length" class="muted">Every visible in-scope entity meets the goal.</p>
      </SectionCard>
    </template>

    <template v-else-if="route.page === 'ops'">
      <PageHeader title="Operations">
        <template #meta><p class="tb-meta">Snapshot <code>{{ body.snapshot }}</code>, input version {{ body.input_version }}, last recompute {{ body.recompute_ms }} ms.</p></template>
      </PageHeader>
      <SectionCard title="Connectors">
        <table>
          <thead><tr><th>Connector</th><th>Runtime</th><th>Registered</th><th>Last full read</th></tr></thead>
          <tbody>
            <tr v-for="connector in body.connectors" :key="connector.name">
              <td>{{ connector.name }}</td><td>{{ connector.runtime }}</td><td>{{ connector.registered ?? 'no' }}</td>
              <td><div v-for="[kind, time] in entries(connector.last_read)" :key="kind">{{ kind }} · {{ time ?? 'never' }}</div></td>
            </tr>
          </tbody>
        </table>
      </SectionCard>
      <SectionCard :title="'Pending commands (' + body.pending_commands.length + ')'">
        <ul><li v-for="command in body.pending_commands" :key="command.id">{{ command.id }} · {{ command.command }} {{ command.kind ?? command.kinds?.join(', ') }} via {{ command.connector }} ({{ command.status }})</li></ul>
      </SectionCard>
      <SectionCard title="Diagnostics">
        <ul>
          <li v-for="id in body.unverified_ownership" :key="id">Unverified ownership claim · <a :href="entityHref(id)">{{ id }}</a></li>
          <li v-for="id in body.orphans" :key="id">Orphan (no identity source) · <code>{{ id }}</code></li>
          <li v-for="(diagnostic, index) in body.diagnostics" :key="index">{{ diagnostic.code }} · {{ diagnostic.location }} — {{ diagnostic.message }}</li>
        </ul>
      </SectionCard>
    </template>
  </template>
</template>
