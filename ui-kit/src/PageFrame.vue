<script setup lang="ts">
/** Wraps themed page content in a main landmark, with optional navigation and footer slots. */
interface Props {
  /** Accessible label for the main landmark when the page needs one. */
  label?: string;
}
defineProps<Props>();
</script>

<template>
  <div class="seikika-ui">
    <slot name="navigation" />
    <main class="seikika-frame" :aria-label="label">
      <slot />
    </main>
    <slot name="footer" />
  </div>
</template>
