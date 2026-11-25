<script setup lang="ts">
import { ref } from "vue";
defineProps<{ user?: string | null; newer?: boolean }>();
const emit = defineEmits<{ navigate: [to: string]; reload: [] }>();
const query = ref("");
function search() {
  if (query.value.trim()) emit("navigate", "/search?q=" + encodeURIComponent(query.value.trim()));
}
</script>

<template>
  <nav class="top" aria-label="Portal">
    <a href="/"><strong>Toolbox</strong></a>
    <form role="search" @submit.prevent="search">
      <input v-model="query" aria-label="Search" placeholder="Search entities" />
    </form>
    <a href="/ops">Operations</a>
    <button v-if="newer" @click="$emit('reload')">Newer data — update</button>
    <span v-if="user" class="muted">dev: {{ user }}</span>
  </nav>
</template>
