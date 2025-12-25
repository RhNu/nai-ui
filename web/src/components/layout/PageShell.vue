<script setup lang="ts">
import { computed } from "vue";

type MaxWidth = "lg" | "xl" | "2xl";

const props = defineProps<{
  title: string;
  subtitle?: string;
  maxWidth?: MaxWidth;
  loading?: boolean;
  loadingText?: string;
}>();

const widthClass = computed(() => {
  const map: Record<MaxWidth, string> = {
    lg: "max-w-5xl",
    xl: "max-w-6xl",
    "2xl": "max-w-7xl",
  };
  return map[props.maxWidth ?? "xl"];
});
</script>

<template>
  <section class="page-shell py-6 sm:py-8">
    <div :class="['mx-auto w-full px-4 sm:px-6 lg:px-8', widthClass]">
      <div class="relative flex flex-col gap-4">
        <header class="flex flex-wrap items-start justify-between gap-3">
          <div class="space-y-1">
            <p
              class="text-[11px] font-semibold uppercase tracking-[0.32em] text-primary"
            >
              NovelAI UI
            </p>
            <div class="flex flex-wrap items-center gap-2">
              <h1 class="text-2xl font-semibold leading-tight">{{ title }}</h1>
              <slot name="label" />
            </div>
            <p v-if="subtitle" class="max-w-3xl text-sm opacity-70">
              {{ subtitle }}
            </p>
          </div>
          <div v-if="$slots.actions" class="flex flex-wrap items-center gap-2">
            <slot name="actions" />
          </div>
        </header>

        <div
          v-if="$slots.tabs"
          class="rounded-box border border-base-300/60 bg-base-100/70 px-3 py-2 shadow-sm backdrop-blur"
        >
          <slot name="tabs" />
        </div>

        <div
          class="relative rounded-2xl border border-base-300/70 bg-base-100/90 shadow-2xl backdrop-blur"
        >
          <div class="p-4 sm:p-6 lg:p-8">
            <slot />
          </div>
        </div>

        <div
          v-if="props.loading"
          class="absolute inset-0 z-30 flex items-center justify-center rounded-2xl bg-base-100/80 backdrop-blur-sm"
        >
          <div class="flex items-center gap-3 text-sm text-base-content/80">
            <span class="loading loading-spinner" />
            <span>{{ props.loadingText || "加载中，请稍候…" }}</span>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
