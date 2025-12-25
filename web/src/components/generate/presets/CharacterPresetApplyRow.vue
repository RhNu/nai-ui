<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { usePresetStore } from "@/stores/presets";
import type { BaseGenerateRequest, CharacterPrompt } from "@/api/types";

const props = defineProps<{
  form: BaseGenerateRequest;
  slots?: number;
}>();

const presetStore = usePresetStore();
const slots = Math.max(1, Math.floor(props.slots ?? 6));

const slot = ref(1);
const selected = ref<string>("");
const busy = ref(false);

const names = computed(() => presetStore.characterNames);

async function refreshNames() {
  const list = await presetStore.refreshCharacterNames();
  if (!list.includes(selected.value)) {
    selected.value = list[0] ?? "";
  }
}

function currentSlotIndex(): number {
  return Math.min(slots, Math.max(1, slot.value)) - 1;
}

async function applyToSlot() {
  const name = selected.value.trim();
  if (!name) return;

  busy.value = true;
  try {
    const preset = await presetStore.fetchCharacterPreset(name);
    if (!preset) return;

    const idx = currentSlotIndex();
    const slotObj = (props.form.character_prompts?.[idx] ??
      null) as CharacterPrompt | null;
    if (!slotObj) return;

    slotObj.enabled = true;
    slotObj.prompt = preset.prompt;
    slotObj.uc = preset.uc;
    slotObj.center = preset.center;
  } catch (error) {
    console.warn("Failed to apply character preset", error);
  } finally {
    busy.value = false;
  }
}

onMounted(() => void refreshNames());
</script>

<template>
  <div class="rounded-xl border border-base-300 bg-base-100/80 p-4 shadow-sm">
    <div class="flex flex-wrap items-start justify-between gap-3">
      <div>
        <div class="text-xs uppercase tracking-[0.2em] text-primary">Apply</div>
        <div class="font-semibold">角色预设</div>
        <div class="text-xs opacity-70">单槽位覆盖 prompt / uc / 位置</div>
      </div>
      <div class="flex items-center gap-2">
        <button
          class="btn btn-primary btn-sm"
          type="button"
          :class="{ 'btn-disabled': busy }"
          @click="applyToSlot"
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

    <div class="mt-3 grid grid-cols-1 gap-3 lg:grid-cols-3">
      <label class="floating-label mt-4">
        <span>槽位</span>
        <select
          v-model.number="slot"
          class="select select-bordered w-full"
          :disabled="busy"
        >
          <option v-for="i in slots" :key="i" :value="i">{{ i }}</option>
        </select>
      </label>

      <label class="floating-label mt-4">
        <span>选择预设</span>
        <select
          v-model="selected"
          class="select select-bordered w-full"
          :disabled="busy"
        >
          <option v-for="n in names" :key="n" :value="n">{{ n }}</option>
        </select>
      </label>
    </div>

    <div class="mt-2 text-xs opacity-70">
      应用到目标槽位 {{ slot }} ，不存在时不做任何更改。
    </div>
  </div>
</template>
