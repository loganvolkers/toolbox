<script setup lang="ts">
defineProps<{ packages: Array<{ name: string; registry: string; description: string; source: string }> }>();
</script>

<template>
  <table>
    <thead><tr><th scope="col">Package</th><th scope="col">Purpose</th></tr></thead>
    <tbody>
      <tr v-for="pkg in packages" :key="pkg.name">
        <th scope="row">
          <a :href="pkg.registry"><code>{{ pkg.name }}</code></a>
          <a class="source" :href="'https://github.com/seikika/seikika/tree/main/' + pkg.source"
            :aria-label="'GitHub source for ' + pkg.name">GitHub</a>
        </th>
        <td>{{ pkg.description }}</td>
      </tr>
    </tbody>
  </table>
</template>

<style scoped>
table { width: 100%; border-collapse: collapse; }
.source { display: block; font-size: .85em; font-weight: normal; margin-block-start: .3rem; }
th, td { padding: .4rem; text-align: left; vertical-align: top; overflow-wrap: anywhere; border-bottom: 1px solid var(--sl-color-gray-5, var(--line)); }
</style>
