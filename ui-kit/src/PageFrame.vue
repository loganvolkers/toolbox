<script setup lang="ts">
defineProps<{ label?: string }>();
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
