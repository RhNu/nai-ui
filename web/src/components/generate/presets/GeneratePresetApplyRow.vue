<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { usePresetStore } from "@/stores/presets";
import type { GeneratePreset } from "@/api/types";

const props = defineProps<{
  model: string;
  onApplyPresetToForm: (model: string, p: GeneratePreset) => void;
  onApplyDefaultsForModel: (model: string) => void;
}>();

const presetStore = usePresetStore();

const presetName = ref<string>("");
const presetBusy = ref(false);

const presetNames = computed(
  () => presetStore.generateNames[props.model] ?? []
);

async function refreshPresetNames() {
  const names = await presetStore.refreshGenerateNames(props.model);
  if (!names.includes(presetName.value)) {
    presetName.value = names[0] ?? "";
  }
}

async function applySelectedPreset() {
  const name = (presetName.value || "").trim() || presetNames.value[0] || "";
  if (!name) return;
  presetBusy.value = true;
  try {
    const preset = await presetStore.fetchGeneratePreset(props.model, name);
    if (preset) {
      props.onApplyPresetToForm(props.model, preset);
    } else {
      props.onApplyDefaultsForModel(props.model);
    }
  } catch {
    props.onApplyDefaultsForModel(props.model);
  } finally {
    presetBusy.value = false;
  }
}

watch(
  () => props.model,
  () => {
    presetName.value = "";
    void refreshPresetNames();
  },
  { immediate: true }
);
</script>

<template>
  <div class="rounded-xl border border-base-300 bg-base-100/80 p-4 shadow-sm">
    <div class="flex flex-wrap items-start justify-between gap-3">
      <div>
        <div class="text-xs uppercase tracking-[0.2em] text-primary">Apply</div>
        <div class="font-semibold">生成预设</div>
        <div class="text-xs opacity-70">按模型+名称覆盖当前表单</div>
      </div>
      <div class="flex items-center gap-2">
        <button
          class="btn btn-primary btn-sm"
          type="button"
          :class="{ 'btn-disabled': presetBusy || !presetName.trim() }"
          @click="applySelectedPreset"
        >
          应用
        </button>
        <button
          class="btn btn-sm"
          type="button"
          :class="{ 'btn-disabled': presetBusy }"
          @click="refreshPresetNames"
        >
          刷新
        </button>
        <div v-if="presetBusy" class="loading loading-spinner loading-xs" />
      </div>
    </div>

    <div class="mt-3 grid gap-3 lg:grid-cols-[1.4fr,auto]">
      <label class="floating-label mt-4">
        <span>预设（{{ props.model }}）</span>
        <select
          v-model="presetName"
          class="select select-bordered w-full"
          :disabled="presetBusy"
        >
          <option v-for="n in presetNames" :key="n" :value="n">{{ n }}</option>
        </select>
      </label>
    </div>
  </div>
</template>
