<script setup lang="ts">
defineProps<{
  title: string
  tabs?: string[]
  activeTab?: string
}>()
</script>

<template>
  <div class="section-title">
    <h2>{{ title }}</h2>
    <div v-if="tabs?.length" class="tab-row">
      <button
        v-for="tab in tabs"
        :key="tab"
        type="button"
        :class="['tab-chip', { active: tab === activeTab }]"
      >
        {{ tab }}
      </button>
    </div>
  </div>
</template>
