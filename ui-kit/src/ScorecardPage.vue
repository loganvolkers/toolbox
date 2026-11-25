<script setup lang="ts">
import PageHeader from "./PageHeader.vue";
defineProps<{ title: string; description?: string; heading?: "h1" | "h2" }>();
</script>

<template>
  <article class="tb-scorecard">
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
