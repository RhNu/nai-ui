<script setup lang="ts">
import { computed, ref } from "vue";
import { usePresetStore } from "@/stores/presets";
import type {
  BaseGenerateRequest,
  CharacterPrompt,
  PromptPreset,
} from "@/api/types";

const props = defineProps<{
  form: BaseGenerateRequest;
  inflateCharacterSlots: (
    enabled: CharacterPrompt[] | null | undefined
  ) => CharacterPrompt[];
}>();

const presetStore = usePresetStore();
const presetName = ref<string>("");
const busy = ref(false);

const presetNames = computed(() => presetStore.promptNames);

async function refreshNames() {
  const names = await presetStore.refreshPromptNames();
  if (!names.includes(presetName.value)) {
    presetName.value = names[0] ?? "";
  }
}

function applyToForm(p: PromptPreset) {
  props.form.positive = p.positive ?? "";
  props.form.negative = p.negative ?? "";
  props.form.add_quality_tags = (p.add_quality_tags ??
    props.form.add_quality_tags) as any;
  props.form.undesired_content_preset = (p.undesired_content_preset ??
    props.form.undesired_content_preset) as any;
  props.form.character_prompts = props.inflateCharacterSlots(
    p.character_prompts ?? []
  );
}

async function applyOnce() {
  const name = presetName.value.trim();
  if (!name) return;
  busy.value = true;
  try {
    const preset = await presetStore.fetchPromptPreset(name);
    if (preset) applyToForm(preset);
  } catch {
    // ignore
  } finally {
    busy.value = false;
  }
}

void refreshNames();
</script>

<template>
  <div class="rounded-xl border border-base-300 bg-base-100/80 p-4 shadow-sm">
    <div class="flex flex-wrap items-start justify-between gap-3">
      <div>
        <div class="text-xs uppercase tracking-[0.2em] text-primary">Apply</div>
        <div class="font-semibold">提示词预设</div>
        <div class="text-xs opacity-70">
          一次性覆盖正/反/可选继承项/角色槽位
        </div>
      </div>
      <div class="flex items-center gap-2">
        <button
          class="btn btn-primary btn-sm"
          type="button"
          :class="{ 'btn-disabled': busy || !presetName.trim() }"
          @click="applyOnce"
        >
          应用
        </button>
        <button
          class="btn btn-sm"
          type="button"
          :class="{ 'btn-disabled': busy }"
          @click="refreshNames"
        >
          刷新
        </button>
        <div v-if="busy" class="loading loading-spinner loading-xs" />
      </div>
    </div>

    <div class="mt-3 grid gap-3 lg:grid-cols-[1.4fr,auto]">
      <label class="floating-label mt-4">
        <span class="label-text">预设名</span>
        <input
          v-model="presetName"
          class="input input-bordered"
          list="promptPresetNames"
          placeholder="输入或选择预设名"
          :disabled="busy"
        />
        <datalist id="promptPresetNames">
          <option v-for="n in presetNames" :key="n" :value="n" />
        </datalist>
      </label>
    </div>
  </div>
</template>
