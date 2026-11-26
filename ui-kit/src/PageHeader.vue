<script setup lang="ts">
/** Displays a page heading and optional description, metadata, and actions. */
interface Props {
  /** Heading text used when the `title` slot is empty. */
  title: string;
  /** Text displayed below the heading when provided. */
  description?: string;
  /** Heading level. Defaults to `h1`. Use `h2` inside a page with its own main heading. */
  heading?: "h1" | "h2";
}
withDefaults(defineProps<Props>(), { heading: "h1" });
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
