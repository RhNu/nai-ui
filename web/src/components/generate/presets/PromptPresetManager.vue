<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import { endpoints } from "@/api/endpoints";
import { useMetaStore } from "@/stores/meta";
import { usePresetStore } from "@/stores/presets";
import type { CharacterPrompt, PromptPreset } from "@/api/types";
import PromptFields from "../form/PromptFields.vue";
import CharacterCollapse from "../form/CharacterCollapse.vue";
import {
  POSITION_OPTIONS,
  centerToPosition,
  positionToCenter,
} from "@/composables/useCharacterPosition";

const metaStore = useMetaStore();
const presetStore = usePresetStore();

const presetName = ref<string>("");
const busy = ref(false);

const presetNames = computed(() => presetStore.promptNames);

const form = reactive({
  positive: "",
  negative: "",
  character_prompts: [] as CharacterPrompt[],
});

const addQualityMode = ref<"inherit" | "true" | "false">("inherit");
const ucPresetMode = ref<string>("inherit");

function inflateCharacterSlots(enabled: CharacterPrompt[] | null | undefined) {
  const slots = 6;
  const empty = (): CharacterPrompt => ({
    prompt: "",
    uc: "",
    center: { x: 0.5, y: 0.5 },
    enabled: false,
  });
  const out: CharacterPrompt[] = Array.from({ length: slots }, empty);
  const src = Array.isArray(enabled) ? enabled : [];
  for (let i = 0; i < Math.min(slots, src.length); i++) out[i] = { ...src[i] };
  return out;
}

function toPreset(): PromptPreset {
  return {
    positive: form.positive ?? "",
    negative: form.negative ?? "",
    add_quality_tags:
      addQualityMode.value === "inherit"
        ? null
        : addQualityMode.value === "true",
    undesired_content_preset:
      ucPresetMode.value === "inherit" ? null : ucPresetMode.value,
    character_prompts: inflateCharacterSlots(form.character_prompts),
  };
}

function fromPreset(p: PromptPreset) {
  form.positive = p.positive ?? "";
  form.negative = p.negative ?? "";
  form.character_prompts = inflateCharacterSlots(p.character_prompts ?? []);

  addQualityMode.value =
    p.add_quality_tags == null
      ? "inherit"
      : p.add_quality_tags
      ? "true"
      : "false";
  ucPresetMode.value = p.undesired_content_preset ?? "inherit";
}

async function refreshNames() {
  const names = await presetStore.refreshPromptNames();

  // If current选项无效，回退到第一个有效名称。
  if (!names.includes(presetName.value)) {
    presetName.value = names[0] ?? "";
  }
}

async function loadSelected(skipRefresh = false) {
  const name = presetName.value.trim();
  if (!name) return;
  busy.value = true;
  try {
    if (!skipRefresh) {
      await refreshNames();
    }
    const preset = await presetStore.fetchPromptPreset(name);
    if (preset) fromPreset(preset);
  } finally {
    busy.value = false;
  }
}

async function save() {
  const name = presetName.value.trim();
  if (!name) {
    alert("请输入预设名");
    return;
  }
  busy.value = true;
  try {
    const preset = toPreset();
    await endpoints.promptPresetPut({ name, preset });
    presetStore.rememberPromptPreset(name, preset);
    presetName.value = name;
    await refreshNames();
  } finally {
    busy.value = false;
  }
}

async function rename() {
  const from = presetName.value.trim();
  if (!from) return;
  const to = prompt("新预设名：", from)?.trim();
  if (!to || to === from) return;
  busy.value = true;
  try {
    await endpoints.promptPresetRename({ from, to });
    presetStore.evictPromptPreset(from);
    presetStore.rememberPromptPreset(to, toPreset());
    presetName.value = to;
    await refreshNames();
  } finally {
    busy.value = false;
  }
}

async function remove() {
  const name = presetName.value.trim();
  if (!name) return;
  if (!confirm(`确定删除提示词预设：${name}？`)) return;
  busy.value = true;
  try {
    await endpoints.promptPresetDelete(name);
    presetStore.evictPromptPreset(name);
    await refreshNames();
    presetName.value = presetNames.value[0] ?? "";
    await loadSelected(true);
  } finally {
    busy.value = false;
  }
}

const ucOptions = computed(() => ["inherit", ...(metaStore.ucPresets ?? [])]);

async function ensureMeta() {
  await metaStore.ensureLoaded();
}

void ensureMeta();
void refreshNames().then(() => loadSelected(true));
</script>

<template>
  <div class="grid gap-4">
    <div class="grid grid-cols-1 gap-3 lg:grid-cols-3">
      <fieldset class="fieldset lg:col-span-2">
        <legend class="fieldset-legend">预设名</legend>
        <div class="join w-full">
          <input
            v-model="presetName"
            class="input input-bordered join-item w-full"
            list="promptPresetNames"
            placeholder="输入或选择预设名"
            :disabled="busy"
          />

          <datalist id="promptPresetNames">
            <option v-for="n in presetNames" :key="n" :value="n" />
          </datalist>
          <button
            class="btn join-item"
            type="button"
            :class="{ 'btn-disabled': busy || !presetName.trim() }"
            @click="save"
          >
            保存/更新
          </button>
          <button
            class="btn join-item"
            type="button"
            :class="{
              'btn-disabled': busy || !presetName.trim(),
            }"
            @click="rename"
          >
            重命名
          </button>
          <button
            class="btn btn-error join-item"
            type="button"
            :class="{
              'btn-disabled': busy || !presetName.trim(),
            }"
            @click="remove"
          >
            删除
          </button>
          <button
            class="btn"
            type="button"
            :class="{ 'btn-disabled': busy }"
            @click="refreshNames"
          >
            刷新列表
          </button>
          <button
            class="btn"
            type="button"
            :class="{ 'btn-disabled': busy }"
            @click="loadSelected()"
          >
            重载当前
          </button>
        </div>
        <div class="label">
          <span class="label-text-alt">此处只编辑提示词预设字段子集</span>
          <span v-if="busy" class="label-text-alt">加载中…</span>
        </div>
      </fieldset>
    </div>

    <div class="divider">字段</div>

    <PromptFields :form="(form as any)" />

    <div class="grid grid-cols-1 gap-3 lg:grid-cols-3">
      <fieldset class="fieldset">
        <legend class="fieldset-legend">
          添加质量提示词(add_quality_tags)
        </legend>
        <select v-model="addQualityMode" class="select select-bordered w-full">
          <option value="inherit">继承（null）</option>
          <option value="true">是</option>
          <option value="false">否</option>
        </select>
      </fieldset>

      <fieldset class="fieldset lg:col-span-2">
        <legend class="fieldset-legend">
          负面提示词预设(undesired_content_preset)
        </legend>
        <select v-model="ucPresetMode" class="select select-bordered w-full">
          <option v-for="u in ucOptions" :key="u" :value="u">
            {{ u === "inherit" ? "继承（null）" : u }}
          </option>
        </select>
      </fieldset>
    </div>

    <CharacterCollapse
      :form="(form as any)"
      :position-options="POSITION_OPTIONS"
      :center-to-position="centerToPosition"
      :position-to-center="positionToCenter"
    />
  </div>
</template>
