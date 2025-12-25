<script setup lang="ts">
import PresetsNav from "@/components/PresetsNav.vue";
import PromptPresetManager from "@/components/generate/presets/PromptPresetManager.vue";
import PageShell from "@/components/layout/PageShell.vue";
</script>

<template>
  <PageShell title="预设" subtitle="提示词预设管理" max-width="2xl">
    <template #tabs>
      <PresetsNav />
    </template>

    <div class="grid gap-4">
      <PromptPresetManager />
    </div>
  </PageShell>
</template>
