<script setup lang="ts">
import { computed } from "vue";
import DataValue from "@seikika/ui/components/DataValue.vue";
import PageHeader from "@seikika/ui/components/PageHeader.vue";
import SectionCard from "@seikika/ui/components/SectionCard.vue";
import type { ReportData } from "@seikika/astro-report/types";
import type { Json } from "../api";
import { usePortalRequest } from "../requests";
import PortalStatus from "./PortalStatus.vue";

const props = defineProps<{ mode: string; query: string; guestReport?: ReportData }>();
const search = computed(() => props.mode === "search");
const guestSearch = computed(() => search.value ? props.guestReport : undefined);
const { result, reload } = usePortalRequest(() => {
  if (!search.value) return "/v1/views/ops";
  return guestSearch.value ? null : "/v1/entities?" + new URLSearchParams({ q: props.query, limit: "50" });
});
const body = computed(() => result.value?.body);
const guest = computed(() => Boolean(guestSearch.value) || body.value?.viewer?.method === "guest");
const items = computed(() => {
  const data = guestSearch.value;
  if (!data) return body.value?.items ?? [];
  const term = props.query.trim().toLowerCase();
  return Object.values(data.catalog).flat().map(id => ({
    id,
    label: data.entitySources?.[id]?.original ?? id,
    fields: data.entityOverview?.[id] ?? {},
  })).filter(({ label, fields }) => [label, fields.title, fields.displayName, fields.name]
    .some(value => typeof value === "string" && value.toLowerCase().includes(term))).slice(0, 50);
});
const entityLink = (id: string) => "#entity/" + encodeURIComponent(id);
const entries = (record: Record<string, Json> | undefined) => Object.entries(record ?? {});
</script>

<template>
  <div class="view">
    <PageHeader heading="h2" :title="search ? (query ? 'Search · ' + query : 'Search') : 'Operations'">
      <template v-if="result?.status === 200 && !search" #meta>
        <p class="sub">Snapshot <code>{{ body.snapshot }}</code> · input version {{ body.input_version }} · last recompute {{ body.recompute_ms }} ms</p>
      </template>
    </PageHeader>
    <PortalStatus v-if="!guestSearch && result?.status !== 200" :result="result"
      :loading="search ? 'Searching entities…' : 'Loading operations…'" @retry="reload" />
    <SectionCard v-else-if="search" title="Results" heading="h3">
      <p v-if="!items.length" class="sub">No entities match.</p>
      <table v-else class="portal-table">
        <thead><tr><th>Entity</th><th>Title</th><th v-if="!guest">Owner</th></tr></thead>
        <tbody>
          <tr v-for="entity in items" :key="entity.id">
            <td><a :href="entityLink(entity.id)">{{ entity.label ?? entity.id }}</a></td>
            <td><DataValue :value="entity.fields.title ?? entity.fields.displayName" /></td>
            <td v-if="!guest">
              <DataValue :value="entity.fields.ownership?.owner ?? entity.fields.owner" :entity-href="entityLink" />
              <span v-if="entity.fields.ownership?.status" class="sub"> ({{ entity.fields.ownership.status }})</span>
            </td>
          </tr>
        </tbody>
      </table>
    </SectionCard>
    <template v-else>
      <SectionCard title="Connectors" heading="h3">
        <table class="portal-table">
          <thead><tr><th>Connector</th><th>Runtime</th><th>Registered</th><th>Last full read</th></tr></thead>
          <tbody>
            <tr v-for="connector in body.connectors" :key="connector.name">
              <td>{{ connector.name }}</td><td>{{ connector.runtime }}</td><td>{{ connector.registered ?? 'no' }}</td>
              <td><div v-for="[kind, time] in entries(connector.last_read)" :key="kind">{{ kind }} · {{ time ?? 'never' }}</div></td>
            </tr>
          </tbody>
        </table>
      </SectionCard>
      <SectionCard :title="'Pending commands (' + body.pending_commands.length + ')'" heading="h3">
        <ul v-if="body.pending_commands.length">
          <li v-for="command in body.pending_commands" :key="command.id">{{ command.id }} · {{ command.command }} {{ command.kind ?? command.kinds?.join(', ') }} via {{ command.connector }} ({{ command.status }})</li>
        </ul>
      </SectionCard>
      <SectionCard title="Diagnostics" heading="h3">
        <ul>
          <li v-for="id in body.unverified_ownership" :key="id">Unverified ownership claim · <a :href="entityLink(id)">{{ id }}</a></li>
          <li v-for="id in body.orphans" :key="id">Orphan (no identity source) · <code>{{ id }}</code></li>
          <li v-for="(diagnostic, index) in body.diagnostics" :key="index">{{ diagnostic.code }} · {{ diagnostic.location }} — {{ diagnostic.message }}</li>
        </ul>
      </SectionCard>
    </template>
  </div>
</template>
