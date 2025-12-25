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
        :class="{ 'tab-active shadow': isActive('/generate/t2i') }"
        to="/generate/t2i"
      >
        Text to Image
      </RouterLink>
      <RouterLink
        role="tab"
        class="tab rounded-full px-4 text-sm"
        :class="{ 'tab-active shadow': isActive('/generate/i2i') }"
        to="/generate/i2i"
      >
        Image to Image
      </RouterLink>
      <RouterLink
        role="tab"
        class="tab rounded-full px-4 text-sm"
        :class="{ 'tab-active shadow': isActive('/generate/inpaint') }"
        to="/generate/inpaint"
      >
        Inpaint
      </RouterLink>
      <RouterLink
        role="tab"
        class="tab rounded-full px-4 text-sm"
        :class="{ 'tab-active shadow': isActive('/generate/character') }"
        to="/generate/character"
      >
        Character
      </RouterLink>
    </div>
  </div>
</template>
