<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import { endpoints } from "@/api/endpoints";
import { useMetaStore } from "@/stores/meta";
import type { CharacterPrompt, PromptPreset } from "@/api/types";
import PromptFields from "../form/PromptFields.vue";
import CharacterCollapse from "../form/CharacterCollapse.vue";

const DEFAULT_NAME = "默认";
const metaStore = useMetaStore();

const presetName = ref<string>(DEFAULT_NAME);
const presetNames = ref<string[]>([DEFAULT_NAME]);
const busy = ref(false);

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
  try {
    const r = await endpoints.promptPresetsList();
    const names = (r.names ?? []).slice();
    if (!names.includes(DEFAULT_NAME)) names.unshift(DEFAULT_NAME);
    presetNames.value = names.length ? names : [DEFAULT_NAME];
  } catch {
    presetNames.value = [DEFAULT_NAME];
  }
}

async function loadSelected() {
  const name = presetName.value.trim() || DEFAULT_NAME;
  busy.value = true;
  try {
    await refreshNames();
    const r = await endpoints.promptPresetGet(name);
    if (r.preset) fromPreset(r.preset);
  } finally {
    busy.value = false;
  }
}

async function save() {
  const name = presetName.value.trim() || DEFAULT_NAME;
  busy.value = true;
  try {
    await endpoints.promptPresetPut({ name, preset: toPreset() });
    presetName.value = name;
    await refreshNames();
  } finally {
    busy.value = false;
  }
}

async function rename() {
  const from = presetName.value.trim() || DEFAULT_NAME;
  if (from === DEFAULT_NAME) return;
  const to = prompt("新预设名：", from)?.trim();
  if (!to || to === from) return;
  busy.value = true;
  try {
    await endpoints.promptPresetRename({ from, to });
    presetName.value = to;
    await refreshNames();
  } finally {
    busy.value = false;
  }
}

async function remove() {
  const name = presetName.value.trim() || DEFAULT_NAME;
  if (name === DEFAULT_NAME) return;
  if (!confirm(`确定删除提示词预设：${name}？`)) return;
  busy.value = true;
  try {
    await endpoints.promptPresetDelete(name);
    presetName.value = DEFAULT_NAME;
    await refreshNames();
  } finally {
    busy.value = false;
  }
}

const ucOptions = computed(() => ["inherit", ...(metaStore.ucPresets ?? [])]);

async function ensureMeta() {
  await metaStore.ensureLoaded();
}

void ensureMeta();
void refreshNames();
void loadSelected();
</script>

<template>
  <div class="grid gap-4">
    <div class="grid grid-cols-1 gap-3 lg:grid-cols-3">
      <label class="form-control items-start lg:col-span-2">
        <div class="label"><span class="label-text">预设名</span></div>
        <div class="join w-full">
          <input
            v-model="presetName"
            class="input input-bordered join-item w-full"
            :disabled="busy"
            list="promptPresetNames"
            placeholder="默认"
          />
          <button
            class="btn join-item"
            type="button"
            :class="{ 'btn-disabled': busy }"
            @click="save"
          >
            保存/更新
          </button>
          <button
            class="btn join-item"
            type="button"
            :class="{
              'btn-disabled': busy || presetName.trim() === DEFAULT_NAME,
            }"
            @click="rename"
          >
            重命名
          </button>
          <button
            class="btn btn-error join-item"
            type="button"
            :class="{
              'btn-disabled': busy || presetName.trim() === DEFAULT_NAME,
            }"
            @click="remove"
          >
            删除
          </button>
        </div>
        <datalist id="promptPresetNames">
          <option v-for="n in presetNames" :key="n" :value="n" />
        </datalist>
        <div class="label">
          <span class="label-text-alt">此处只编辑提示词预设字段子集</span>
          <span v-if="busy" class="label-text-alt">加载中…</span>
        </div>
      </label>

      <div class="form-control items-start gap-2">
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
          @click="loadSelected"
        >
          重载当前
        </button>
      </div>
    </div>

    <div class="divider">字段</div>

    <PromptFields :form="(form as any)" />

    <div class="grid grid-cols-1 gap-3 lg:grid-cols-3">
      <label class="form-control items-start">
        <div class="label">
          <span class="label-text">add_quality_tags</span>
        </div>
        <select v-model="addQualityMode" class="select select-bordered w-full">
          <option value="inherit">继承（null）</option>
          <option value="true">true</option>
          <option value="false">false</option>
        </select>
      </label>

      <label class="form-control items-start lg:col-span-2">
        <div class="label">
          <span class="label-text">undesired_content_preset</span>
        </div>
        <select v-model="ucPresetMode" class="select select-bordered w-full">
          <option v-for="u in ucOptions" :key="u" :value="u">
            {{ u === "inherit" ? "继承（null）" : u }}
          </option>
        </select>
      </label>
    </div>

    <CharacterCollapse
      :form="(form as any)"
      :position-options="[
        'A1',
        'A2',
        'A3',
        'A4',
        'A5',
        'B1',
        'B2',
        'B3',
        'B4',
        'B5',
        'C1',
        'C2',
        'C3',
        'C4',
        'C5',
        'D1',
        'D2',
        'D3',
        'D4',
        'D5',
        'E1',
        'E2',
        'E3',
        'E4',
        'E5',
      ]"
      :center-to-position="(c) => {
        const offset = 0.1;
        const grid = [0, 1, 2, 3, 4].map((i) => Math.round((i * 0.2 + offset) * 10) / 10);
        const nearestIndex = (v: number) => {
          let bestIdx = 0;
          let bestDist = Number.POSITIVE_INFINITY;
          for (let i = 0; i < grid.length; i++) {
            const d = Math.abs(grid[i] - v);
            if (d < bestDist) {
              bestDist = d;
              bestIdx = i;
            }
          }
          return bestIdx;
        };
        const li = nearestIndex(c.x);
        const ni = nearestIndex(c.y);
        return `${String.fromCharCode(65 + li)}${String(ni + 1)}`;
      }"
      :position-to-center="
        (p) => {
          const offset = 0.1;
          const letter = p[0] ?? 'C';
          const number = p[1] ?? '3';
          const li = Math.min(
            4,
            Math.max(0, letter.toUpperCase().charCodeAt(0) - 65)
          );
          const ni = Math.min(4, Math.max(0, parseInt(number, 10) - 1));
          return {
            x: Math.round((li * 0.2 + offset) * 10) / 10,
            y: Math.round((ni * 0.2 + offset) * 10) / 10,
          };
        }
      "
    />
  </div>
</template>
