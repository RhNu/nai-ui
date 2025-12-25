<script setup lang="ts">
import PresetsNav from "@/components/PresetsNav.vue";
import PromptSnippetManager from "@/components/generate/presets/PromptSnippetManager.vue";
import PageShell from "@/components/layout/PageShell.vue";
</script>

<template>
  <PageShell title="预设" subtitle="提示词片段管理" max-width="2xl">
    <template #tabs>
      <PresetsNav />
    </template>

    <div class="grid gap-4">
      <PromptSnippetManager />
    </div>
  </PageShell>
</template>
