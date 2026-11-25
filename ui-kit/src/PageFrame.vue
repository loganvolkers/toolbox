<script setup lang="ts">
defineProps<{ label?: string }>();
</script>

<template>
  <div class="toolbox-ui">
    <slot name="navigation" />
    <main class="tb-frame" :aria-label="label">
      <slot />
    </main>
    <slot name="footer" />
  </div>
</template>
