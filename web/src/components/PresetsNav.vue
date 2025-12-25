<script setup lang="ts">
import { RouterLink, useRoute } from "vue-router";

const route = useRoute();

function isActive(path: string) {
  return route.path === path;
}
</script>

<template>
  <div class="overflow-x-auto">
    <div
      role="tablist"
      class="tabs tabs-boxed min-w-max rounded-full bg-base-200/70 p-1"
    >
      <RouterLink
        role="tab"
        class="tab rounded-full px-4 text-sm"
        :class="{ 'tab-active shadow': isActive('/presets/generate') }"
        to="/presets/generate"
      >
        生成预设
      </RouterLink>
      <RouterLink
        role="tab"
        class="tab rounded-full px-4 text-sm"
        :class="{ 'tab-active shadow': isActive('/presets/prompt') }"
        to="/presets/prompt"
      >
        提示词预设
      </RouterLink>
      <RouterLink
        role="tab"
        class="tab rounded-full px-4 text-sm"
        :class="{ 'tab-active shadow': isActive('/presets/character') }"
        to="/presets/character"
      >
        角色预设
      </RouterLink>
    </div>
  </div>
</template>
