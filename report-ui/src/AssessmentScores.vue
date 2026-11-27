<script setup lang="ts">
import LevelBadge from "@seikika/ui/components/LevelBadge.vue";
import { rubricTitle, uniqueLinks, type AssessmentScore } from "./lib/assessments";

defineProps<{
  scores: AssessmentScore[];
  cardHref: (name: string) => string;
  ruleHref: (card: string, rule: string) => string;
}>();
</script>

<template>
  <p v-if="!scores.length" class="assessment-muted">No scorecard results are available.</p>
  <ul v-else class="assessment-list assessment-scores">
    <li v-for="{ name, definition, score } in scores" :key="name">
      <div class="assessment-row">
        <div>
          <a :href="cardHref(name)" class="assessment-primary">{{ definition.title || rubricTitle(name) }}</a>
        </div>
        <LevelBadge :level="score.level" />
      </div>
      <details v-if="score.rules && Object.keys(score.rules).length">
        <summary>{{ Object.values(score.rules).filter(result => result === 'pass').length }} of {{ Object.keys(score.rules).length }} rules passed</summary>
        <ul class="assessment-list assessment-rule-list">
          <li v-for="[rule, outcome] in Object.entries(score.rules)" :key="rule" class="assessment-row">
            <a :href="ruleHref(name, rule)">{{ definition.rules[rule]?.title || rubricTitle(rule) }}</a>
            <span class="assessment-verdict" :data-verdict="outcome">{{ outcome }}</span>
          </li>
        </ul>
      </details>
      <p v-else-if="!score.rules" class="assessment-muted">Rule details are unavailable.</p>
      <ul v-if="definition.links?.length" class="assessment-list assessment-doc-links">
        <li v-for="link in uniqueLinks(definition.links)" :key="link.url"><a :href="link.url">{{ link.title }}</a></li>
      </ul>
    </li>
  </ul>
</template>
