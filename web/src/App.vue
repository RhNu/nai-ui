<script setup lang="ts">
import { computed, onMounted } from "vue";
import { RouterLink, RouterView, useRoute } from "vue-router";
import { useThemeStore } from "@/stores/theme";

const route = useRoute();
const theme = useThemeStore();

const navLinks = [
  {
    label: "生成",
    to: "/generate/t2i",
    match: "/generate",
    icon: "fa-wand-magic-sparkles",
  },
  {
    label: "预设",
    to: "/presets/generate",
    match: "/presets",
    icon: "fa-layer-group",
  },
  { label: "导演", to: "/director", icon: "fa-clapperboard" },
  { label: "任务", to: "/jobs", icon: "fa-list-check" },
  { label: "输出", to: "/outputs", icon: "fa-images" },
];

const themeText = computed(() => (theme.theme === "dark" ? "暗" : "亮"));

function isActive(link: { to: string; match?: string }) {
  const key = link.match ?? link.to;
  return route.path.startsWith(key);
}

onMounted(() => {
  theme.applyTheme();
});
</script>

<template>
  <div class="min-h-screen bg-base-200">
    <div class="flex min-h-screen flex-col">
      <header
        class="sticky top-0 z-30 border-b border-base-300/60 bg-base-100/90 backdrop-blur"
      >
        <div
          class="mx-auto flex h-16 max-w-6xl items-center justify-between gap-3 px-4 sm:px-6 lg:px-8"
        >
          <RouterLink
            class="flex items-center gap-3 text-lg font-semibold"
            to="/"
          >
            <span
              class="grid h-10 w-10 place-items-center rounded-xl bg-primary text-primary-content shadow-lg"
            >
              NAI
            </span>
            <div class="leading-tight">
              <div>NovelAI UI</div>
              <div class="text-xs opacity-70">frontend</div>
            </div>
          </RouterLink>

          <nav
            class="hidden items-center gap-1 rounded-full bg-base-200/80 p-1 shadow-sm md:flex"
          >
            <RouterLink
              v-for="link in navLinks"
              :key="link.to"
              class="btn btn-sm btn-ghost rounded-full px-4"
              :class="{ 'btn-active bg-base-100 shadow': isActive(link) }"
              :to="link.to"
            >
              <i v-if="link.icon" class="fa-solid" :class="link.icon"></i>
              <span class="ml-2">{{ link.label }}</span>
            </RouterLink>
            <button
              class="btn btn-sm btn-ghost rounded-full px-3"
              type="button"
              @click="theme.toggle()"
            >
              模式：{{ themeText }}
            </button>
          </nav>

          <div class="md:hidden">
            <div class="dropdown dropdown-end">
              <div
                tabindex="0"
                role="button"
                class="btn btn-ghost btn-sm rounded-full"
              >
                菜单
              </div>
              <ul
                tabindex="0"
                class="menu dropdown-content mt-3 w-52 rounded-box bg-base-100 p-2 shadow"
              >
                <li v-for="link in navLinks" :key="link.to">
                  <RouterLink :class="{ active: isActive(link) }" :to="link.to">
                    <i
                      v-if="link.icon"
                      class="fa-solid mr-2"
                      :class="link.icon"
                    ></i>
                    {{ link.label }}
                  </RouterLink>
                </li>
                <li class="mt-1 border-t border-base-200" />
                <li>
                  <button type="button" @click="theme.toggle()">
                    切换模式（{{ themeText }}）
                  </button>
                </li>
              </ul>
            </div>
          </div>
        </div>
      </header>

      <main class="flex-1">
        <RouterView />
      </main>
    </div>
  </div>
</template>
