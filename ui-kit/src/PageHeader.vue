<script setup lang="ts">
withDefaults(defineProps<{ title: string; description?: string; heading?: "h1" | "h2" }>(), { heading: "h1" });
</script>

<template>
  <header class="tb-page-header">
    <div class="tb-page-heading">
      <component :is="heading"><slot name="title">{{ title }}</slot></component>
      <p v-if="description" class="tb-page-description">{{ description }}</p>
      <slot name="meta" />
    </div>
    <slot name="actions" />
  </header>
</template>
