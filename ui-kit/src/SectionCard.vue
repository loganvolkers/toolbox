<script setup lang="ts">
/** Groups related content under a section heading. */
interface Props {
  /** Heading text used when the `title` slot is empty. */
  title: string;
  /** Heading level. Defaults to `h2`. Match the surrounding page outline. */
  heading?: "h2" | "h3" | "h4";
}
defineProps<Props>();
</script>

<template>
  <section class="seikika-section">
    <component :is="heading ?? 'h2'"><slot name="title">{{ title }}</slot></component>
    <slot />
  </section>
</template>
