<script setup lang="ts">
import PresetsNav from "@/components/PresetsNav.vue";
import GeneratePresetManager from "@/components/generate/presets/GeneratePresetManager.vue";
import PageShell from "@/components/layout/PageShell.vue";
</script>

<template>
  <PageShell title="预设" subtitle="生成参数预设管理" max-width="2xl">
    <template #tabs>
      <PresetsNav />
    </template>

    <div class="grid gap-4">
      <GeneratePresetManager />
    </div>
  </PageShell>
</template>
