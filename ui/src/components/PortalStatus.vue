<script setup lang="ts">
import type { ApiResult } from "../api";

defineProps<{ result: ApiResult | null; loading: string }>();
defineEmits<{ retry: [] }>();
</script>

<template>
  <p v-if="!result" class="sub" role="status" aria-busy="true">{{ loading }}</p>
  <div v-else class="portal-problem" role="alert">
    <strong>{{ result.body?.title ?? "HTTP " + result.status }}</strong>
    <p v-if="result.body?.detail">{{ result.body.detail }}</p>
    <p v-if="result.status === 401">Sign in through your identity-aware proxy, or on a dev server add <code>?as=&lt;user&gt;</code> to the URL.</p>
    <button v-if="result.status !== 401 && result.status !== 403" type="button" @click="$emit('retry')">Retry</button>
  </div>
</template>
