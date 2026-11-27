<script setup lang="ts">
import { computed } from "vue";
import LevelBadge from "@seikika/ui/components/LevelBadge.vue";
import { nextRequirement, rubricTitle, ruleProgress, uniqueLinks, type AssessmentScore } from "./lib/assessments";

const props = defineProps<{
  scores: AssessmentScore[];
  cardHref: (name: string) => string;
  ruleHref: (card: string, rule: string) => string;
}>();
const cards = computed(() => props.scores.map(item => ({
  ...item,
  next: nextRequirement(item),
  progress: ruleProgress(item.score),
  links: uniqueLinks(item.definition.links ?? []),
})));
</script>

<template>
  <p v-if="!scores.length" class="assessment-muted">No scorecard results are available.</p>
  <ul v-else class="assessment-list assessment-scores">
    <li v-for="item in cards" :key="item.name">
      <div class="assessment-row">
        <a :href="cardHref(item.name)" class="assessment-primary">{{ item.definition.title || rubricTitle(item.name) }}</a>
        <LevelBadge :level="item.score.level" />
      </div>
      <p v-if="item.next" class="assessment-muted">
        For <LevelBadge :level="item.next.level" />, address <a :href="ruleHref(item.name, item.next.name)">{{ item.next.title }}</a>.
      </p>
      <details v-if="item.progress">
        <summary>{{ item.progress }}</summary>
        <ul class="assessment-list assessment-rule-list">
          <li v-for="[rule, outcome] in Object.entries(item.score.rules ?? {})" :key="rule" class="assessment-row">
            <a :href="ruleHref(item.name, rule)">{{ item.definition.rules[rule]?.title || rubricTitle(rule) }}</a>
            <span class="assessment-verdict" :data-verdict="outcome">{{ outcome }}</span>
          </li>
        </ul>
      </details>
      <p v-else-if="item.score.level === 'skipped'" class="assessment-muted">This scorecard did not apply.</p>
      <p v-else-if="!item.score.rules" class="assessment-muted">Rule details are unavailable.</p>
      <ul v-if="item.links.length" class="assessment-list assessment-doc-links">
        <li v-for="link in item.links" :key="link.url"><a :href="link.url">{{ link.title }}</a></li>
      </ul>
    </li>
  </ul>
</template>
