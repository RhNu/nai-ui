import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { endpoints } from "@/api/endpoints";
import type { Meta } from "@/api/types";

export const useMetaStore = defineStore("meta", () => {
  const meta = ref<Meta | null>(null);
  const loading = ref(false);
  const error = ref<string>("");

  const models = computed(() => meta.value?.models ?? []);
  const samplers = computed(() => meta.value?.samplers ?? []);
  const noiseSchedules = computed(() => meta.value?.noise_schedules ?? []);
  const ucPresets = computed(() => meta.value?.uc_presets ?? []);

  async function ensureLoaded() {
    if (meta.value) return;
    loading.value = true;
    error.value = "";
    try {
      meta.value = await endpoints.meta();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  return {
    meta,
    loading,
    error,
    models,
    samplers,
    noiseSchedules,
    ucPresets,
    ensureLoaded,
  };
});
