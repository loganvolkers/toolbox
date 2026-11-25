<script setup lang="ts">
import { computed, ref } from "vue";
import type { CliCatalog } from "../lib/cli-catalog";

const props = withDefaults(defineProps<{ catalog: CliCatalog; base?: string }>(), { base: "/toolbox" });
const query = ref("");
const filtered = computed(() => {
  const search = query.value.trim().toLowerCase();
  return props.catalog.commands.filter(command =>
    [command.name, command.about, command.mcpTool].join(" ").toLowerCase().includes(search));
});
</script>

<template>
  <section class="cli-ref">
    <label class="cli-ref__search">
      <span class="sr-only">Filter commands</span>
      <input v-model="query" type="search" placeholder="Filter commands…" autocomplete="off" />
    </label>
    <p class="cli-ref__count" aria-live="polite">
      <template v-if="query.trim()">{{ filtered.length }} of </template>{{ catalog.commands.length }} commands
    </p>
    <ul class="cli-ref__list" role="list">
      <li v-for="command in filtered" :key="command.slug" class="cli-ref__item">
        <a class="cli-ref__link" :href="'#' + command.slug">
          <code>toolbox {{ command.name }}</code>
          <span v-if="command.mcpTool" class="cli-ref__badge">MCP {{ command.mcpTool }}</span>
        </a>
        <p v-if="command.about" class="cli-ref__about">{{ command.about }}</p>
      </li>
    </ul>
    <h2 id="mcp-tools">MCP tools</h2>
    <p>Built-in MCP tools share the same authorization path as the CLI.
      <a :href="base + '/agents/reads/'">Read the catalog from an agent</a>.
    </p>
    <div class="cli-ref__table-wrap">
      <table>
        <thead><tr><th>MCP tool</th><th>CLI</th><th>Description</th></tr></thead>
        <tbody>
          <tr v-for="tool in catalog.mcpTools" :key="tool.name">
            <td><code>{{ tool.name }}</code></td>
            <td>
              <code v-for="name in tool.cliCommands" :key="name" class="cli-ref__cli">{{ name }}</code>
              <template v-if="!tool.cliCommands.length">—</template>
            </td>
            <td>{{ tool.description ?? '—' }}</td>
          </tr>
        </tbody>
      </table>
    </div>
    <h2 id="command-details">Command details</h2>
    <article v-for="command in catalog.commands" :id="command.slug" :key="command.slug" class="cli-ref__detail">
      <h3><code>toolbox {{ command.name }}</code></h3>
      <p v-if="command.about">{{ command.about }}</p>
      <p v-if="command.usage" class="cli-ref__usage"><span class="cli-ref__label">Usage</span> <code>{{ command.usage }}</code></p>
      <p v-if="command.mcpTool"><span class="cli-ref__label">MCP</span> <code>{{ command.mcpTool }}</code></p>
      <div v-if="command.args.length" class="cli-ref__table-wrap">
        <table>
          <thead><tr><th>Argument</th><th>Kind</th></tr></thead>
          <tbody>
            <tr v-for="argument in command.args" :key="argument.name">
              <td><code>{{ argument.long ? '--' + argument.long : argument.name }}</code></td>
              <td>{{ argument.positional ? 'positional' : 'flag' }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <details v-if="command.helpText" class="cli-ref__raw">
        <summary>Raw <code>--help</code></summary>
        <pre><code>{{ command.helpText }}</code></pre>
      </details>
    </article>
  </section>
</template>

<style scoped>
  .cli-ref {
    --cli-border: color-mix(in srgb, var(--sl-color-gray-5, var(--muted)) 80%, transparent);
    display: grid;
    gap: 1.25rem;
  }
  .cli-ref__search input {
    width: 100%;
    padding: 0.65rem 0.85rem;
    border: 1px solid var(--cli-border);
    border-radius: 0.4rem;
    background: var(--sl-color-gray-7, var(--card));
    color: inherit;
    font: inherit;
  }
  .cli-ref__count {
    margin: 0;
    opacity: 0.75;
    font-size: 0.9rem;
  }
  .cli-ref__list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 0.65rem;
  }
  .cli-ref__item {
    padding: 0.75rem 0;
    border-bottom: 1px solid var(--cli-border);
  }
  .cli-ref__link {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: baseline;
    text-decoration: none;
  }
  .cli-ref__badge {
    font-size: 0.75rem;
    color: var(--sl-color-text, var(--fg));
    border: 1px solid var(--cli-border);
    border-radius: 0.25rem;
    padding: 0.1rem 0.4rem;
  }
  .cli-ref__about {
    margin: 0.35rem 0 0;
    opacity: 0.85;
  }
  .cli-ref__table-wrap {
    overflow-x: auto;
  }
  .cli-ref__cli + .cli-ref__cli::before {
    content: ', ';
  }
  .cli-ref__detail {
    padding: 1.25rem 0;
    border-top: 1px solid var(--cli-border);
  }
  .cli-ref__label {
    display: inline-block;
    min-width: 4.5rem;
    opacity: 0.7;
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .cli-ref__usage code {
    white-space: pre-wrap;
  }
  .cli-ref__raw {
    margin-top: 0.75rem;
  }
  .cli-ref__raw pre {
    overflow-x: auto;
    padding: 0.75rem;
    border-radius: 0.35rem;
    background: var(--sl-color-gray-6, var(--card));
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    border: 0;
  }
  .cli-ref__item[hidden] {
    display: none;
  }
</style>
