<script setup lang="ts">
import PresetsNav from "@/components/PresetsNav.vue";
import CharacterPresetManager from "@/components/generate/presets/CharacterPresetManager.vue";
import PageShell from "@/components/layout/PageShell.vue";
</script>

<template>
  <PageShell title="预设" subtitle="角色预设管理" max-width="2xl">
    <template #tabs>
      <PresetsNav />
    </template>

    <div class="grid gap-4">
      <CharacterPresetManager />
    </div>
  </PageShell>
</template>
