<script setup lang="ts">
defineProps<{ title: string; heading?: "h2" | "h3" | "h4" }>();
</script>

<template>
  <section class="seikika-section">
    <component :is="heading ?? 'h2'"><slot name="title">{{ title }}</slot></component>
    <slot />
  </section>
</template>
