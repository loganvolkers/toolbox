<script setup lang="ts">
import { computed } from "vue";
import DataValue from "@seikika/ui/components/DataValue.vue";
import SectionCard from "@seikika/ui/components/SectionCard.vue";
import type { CatalogReport } from "./types";
import type { Summary } from "./lib/summary";
import { catalogEntityPath } from "./lib/routes";
import { sourceLabel } from "./lib/federation";

const props = defineProps<{ data: CatalogReport; summary: Summary; id: string }>();
const details = computed(() => [
  { original: props.id, source: undefined, detail: props.data.entities?.[props.id] },
  ...(props.data.entityObservations?.[props.id] ?? []),
]);
const entityLink = (id: string) => props.summary.kindOf[id] ? "#" + catalogEntityPath(props.data, id) : "";
</script>

<template>
  <div v-for="{ detail, original, source } in details" :key="source ?? original" class="report-entity-details">
    <SectionCard v-if="detail && Object.keys(detail.fields).length || source"
      :title="source ? 'Fields from ' + data.sources?.[source]?.title : details.length > 1 ? 'Inventory fields' : 'Fields'">
      <p v-if="source" class="sub">{{ sourceLabel(data, source) }} · <code>{{ original }}</code></p>
      <dl>
        <template v-for="(value, name) in detail?.fields" :key="name">
          <dt>{{ name }}</dt><dd><DataValue :value="value" :entity-href="entityLink" />
            <small v-if="detail?.provenance?.[name]" class="sub"> · {{ detail.provenance[name] }}</small></dd>
        </template>
      </dl>
    </SectionCard>
    <SectionCard v-if="detail?.relations?.in.length || detail?.relations?.out.length" title="Related entities">
      <ul>
        <li v-for="edge in detail.relations.in" :key="'in:' + edge.relation + ':' + edge.from">
          {{ edge.relation }} ← <a :href="entityLink(edge.from)">{{ data.entitySources?.[edge.from]?.original ?? edge.from }}</a>
        </li>
        <li v-for="edge in detail.relations.out" :key="'out:' + edge.relation + ':' + edge.to">
          {{ edge.relation }} → <a :href="entityLink(edge.to)">{{ data.entitySources?.[edge.to]?.original ?? edge.to }}</a>
        </li>
      </ul>
    </SectionCard>
  </div>
</template>
