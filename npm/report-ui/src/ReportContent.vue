<script setup lang="ts">
import { ref, watchPostEffect } from "vue";

const props = defineProps<{ build: () => Node | Node[] | null; as?: "div" | "span" | "section" }>();
const element = ref<HTMLElement>();
watchPostEffect(() => {
  if (!element.value) return;
  const content = props.build();
  element.value.replaceChildren(...(content ? [content].flat() : []));
});
</script>

<template><component :is="as ?? 'div'" ref="element"></component></template>
