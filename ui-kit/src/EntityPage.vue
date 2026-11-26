<script setup lang="ts">
import PageHeader from "./PageHeader.vue";
import type { EntityOverview } from "./entity";

defineProps<{ entity: EntityOverview; ownerHref?: string; heading?: "h1" | "h2"; snapshot?: string | number | null }>();
</script>

<template>
  <article class="seikika-entity">
    <slot name="breadcrumbs" />
    <slot name="header">
      <PageHeader :title="entity.title" :description="entity.description" :heading="heading">
        <template #meta>
          <slot name="meta"><p class="seikika-meta">
            <code v-if="entity.id !== entity.title">{{ entity.id }} · </code>{{ entity.kind }}
            <template v-if="snapshot != null"> · <span class="asof">as of snapshot <code>{{ snapshot }}</code></span></template>
          </p></slot>
        </template>
        <template #actions><slot name="header-actions" /></template>
      </PageHeader>
    </slot>
    <div v-if="entity.owner || entity.ownershipStatus || entity.links.length" class="seikika-entity-meta">
      <p v-if="entity.owner || entity.ownershipStatus">
        <strong>Owner</strong>{{ " " }}
        <template v-if="entity.owner">
          <a v-if="ownerHref" :href="ownerHref">{{ entity.owner }}</a>
          <template v-else>{{ entity.owner }}</template>
        </template>
        <template v-else>Unassigned</template>
        <span v-if="entity.ownershipStatus" class="seikika-meta"> ({{ entity.ownershipStatus }})</span>
      </p>
      <ul v-if="entity.links.length" aria-label="Entity links">
        <li v-for="link in entity.links" :key="link.href"><a :href="link.href">{{ link.label }}</a></li>
      </ul>
    </div>
    <slot />
    <slot name="scorecards" />
    <slot name="actions" />
  </article>
</template>
