<script setup lang="ts">
withDefaults(defineProps<{ title: string; description?: string; heading?: "h1" | "h2" }>(), { heading: "h1" });
</script>

<template>
  <header class="seikika-page-header">
    <div class="seikika-page-heading">
      <component :is="heading"><slot name="title">{{ title }}</slot></component>
      <p v-if="description" class="seikika-page-description">{{ description }}</p>
      <slot name="meta" />
    </div>
    <slot name="actions" />
  </header>
</template>
