<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { entityOverview, safeWebHref } from "@seikika/ui/entity";
import SectionCard from "@seikika/ui/components/SectionCard.vue";
import type { CatalogReport, ReportEntity } from "./types";
import { catalogEntityPath } from "./lib/routes";

interface Deployment {
  id: string;
  createdOn: string;
  versions: { versionId: string; percentage: number }[];
}
interface Observation {
  observedAt: number;
}
interface Metrics extends Observation {
  dashboardUrl?: string;
  metrics: { id: string; label: string; unit: string; status: string; value: number | null }[];
}

const props = defineProps<{ data: CatalogReport; id: string; detail: ReportEntity }>();
const now = ref(Date.now());
let clock: ReturnType<typeof setInterval>;
onMounted(() => { clock = setInterval(() => { now.value = Date.now(); }, 15_000); });
onUnmounted(() => clearInterval(clock));

const freshness = (observation: Observation, maxAgeSeconds: number) =>
  now.value <= (observation.observedAt + maxAgeSeconds) * 1000;
const utc = (value: string | number) => {
  const date = new Date(value);
  return Number.isNaN(date.valueOf()) ? String(value) : date.toISOString();
};
const format = new Intl.NumberFormat(undefined, { maximumFractionDigits: 2 });
const environments = computed(() => (props.detail.relations?.in ?? []).flatMap(edge => {
  if (edge.relation !== "environmentOf") return [];
  const fields = props.data.entities?.[edge.from]?.fields;
  if (!fields || typeof fields.environment !== "string") return [];
  const deployments = fields.deployments as (Observation & { recent?: Deployment[] }) | null;
  const metrics = fields.metrics as Metrics | null;
  return [{
    ...entityOverview(edge.from, "Service environment", fields),
    worker: fields.worker,
    serviceName: fields.serviceName,
    dashboardUid: fields.dashboardUid,
    deployments: Array.isArray(deployments?.recent) ? deployments : null,
    telemetry: Array.isArray(metrics?.metrics) ? metrics : null,
    dashboardHref: safeWebHref(metrics?.dashboardUrl),
  }];
}));
const repositoryHref = computed(() => {
  const repository = props.detail.fields.repository;
  return typeof repository === "string" && props.data.catalog.Repository?.includes(repository)
    ? "#" + catalogEntityPath(props.data, repository)
    : undefined;
});
</script>

<template>
  <SectionCard title="Service identity">
    <dl class="service-identities">
      <div>
        <dt>Catalog ID</dt>
        <dd><code>{{ data.entitySources?.[id]?.original ?? id }}</code></dd>
      </div>
      <div>
        <dt>Source repository</dt>
        <dd><a v-if="repositoryHref" :href="repositoryHref">{{ detail.fields.code }}</a><code v-else>{{ detail.fields.code }}</code></dd>
      </div>
      <div v-if="detail.fields.lifecycle">
        <dt>Lifecycle</dt>
        <dd>{{ detail.fields.lifecycle }}</dd>
      </div>
    </dl>
  </SectionCard>

  <SectionCard v-if="!environments.length" title="Environments">
    <p class="sub">No environments are listed for this service.</p>
  </SectionCard>
  <SectionCard v-for="env in environments" :key="env.id" :title="env.title">
    <ul v-if="env.links.length" class="service-links" aria-label="Environment links">
      <li v-for="link in env.links" :key="link.href"><a :href="link.href">{{ link.label }}</a></li>
    </ul>
    <dl class="service-identities">
      <div v-if="env.worker">
        <dt>Cloudflare Worker</dt>
        <dd><code>{{ env.worker }}</code></dd>
      </div>
      <div v-if="env.serviceName">
        <dt>OpenTelemetry service.name</dt>
        <dd><code>{{ env.serviceName }}</code></dd>
      </div>
      <div v-if="env.dashboardUid">
        <dt>Grafana dashboard UID</dt>
        <dd><code>{{ env.dashboardUid }}</code></dd>
      </div>
    </dl>
    <section v-if="env.worker">
      <h4>Recent deployments</h4>
      <p v-if="!env.deployments" class="sub">Deployment history is unavailable.</p>
      <template v-else>
        <p class="sub">
          <strong v-if="!freshness(env.deployments, 360)">Stale · </strong>Checked
          <time :datetime="utc(env.deployments.observedAt * 1000)">{{ utc(env.deployments.observedAt * 1000) }}</time>
        </p>
        <p v-if="!env.deployments.recent?.length" class="sub">Cloudflare returned no deployments.</p>
        <div v-else class="tablewrap">
          <table class="cmp">
            <thead><tr><th scope="col">Deployment</th><th scope="col">Created (UTC)</th><th scope="col">Version and traffic</th></tr></thead>
            <tbody>
              <tr v-for="(deployment, index) in env.deployments.recent" :key="deployment.id">
                <td><strong v-if="index === 0">Active at last check</strong><div><code>{{ deployment.id }}</code></div></td>
                <td><time :datetime="deployment.createdOn">{{ utc(deployment.createdOn) }}</time></td>
                <td>
                  <div v-for="version in deployment.versions" :key="version.versionId">
                    <code>{{ version.versionId }}</code> · {{ format.format(version.percentage) }}%
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </template>
    </section>
    <section v-if="env.serviceName">
      <h4>Grafana metrics</h4>
      <p v-if="env.dashboardHref"><a :href="env.dashboardHref">Open Grafana dashboard</a></p>
      <p v-if="!env.telemetry" class="sub">Metric observations are unavailable.</p>
      <template v-else>
        <p class="sub">Queried <time :datetime="utc(env.telemetry.observedAt * 1000)">{{ utc(env.telemetry.observedAt * 1000) }}</time></p>
        <p v-if="!freshness(env.telemetry, 180)" class="sub">Metrics are stale. Open Grafana for current values.</p>
        <dl v-else class="service-identities">
          <div v-for="metric in env.telemetry.metrics" :key="metric.id">
            <dt>{{ metric.label }}</dt>
            <dd v-if="metric.status === 'ok' && typeof metric.value === 'number'">
              {{ format.format(metric.value) }} {{ metric.unit }}
            </dd>
            <dd v-else class="sub">{{ metric.status === 'no-data' ? 'No data' : 'Unavailable' }}</dd>
          </div>
        </dl>
      </template>
    </section>
  </SectionCard>
</template>
