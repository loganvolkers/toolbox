<script setup lang="ts">
import PageHeader from "./PageHeader.vue";
/** Wraps scorecard content with a header and slots for context and actions. */
interface Props {
  /** Scorecard title used by the built-in header. */
  title: string;
  /** Text displayed below the title in the built-in header when provided. */
  description?: string;
  /** Heading level for the built-in header. Defaults to `h1`. */
  heading?: "h1" | "h2";
}
defineProps<Props>();
</script>

<template>
  <article class="seikika-scorecard">
    <slot name="breadcrumbs" />
    <slot name="header">
      <PageHeader :title="title" :description="description" :heading="heading">
        <template #meta><slot name="meta" /></template>
        <template #actions><slot name="header-actions" /></template>
      </PageHeader>
    </slot>
    <slot />
  </article>
</template>
