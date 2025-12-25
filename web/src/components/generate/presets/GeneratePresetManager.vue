<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { endpoints } from "@/api/endpoints";
import { useMetaStore } from "@/stores/meta";
import { usePresetStore } from "@/stores/presets";
import type { GeneratePreset } from "@/api/types";

const metaStore = useMetaStore();
const presetStore = usePresetStore();

const model = ref<string>("nai-diffusion-4-5-full");
const presetName = ref<string>("");
const busy = ref(false);

const presetNames = computed(
  () => presetStore.generateNames[model.value] ?? []
);

const isV3Model = computed(
  () =>
    model.value === "nai-diffusion-3" || model.value === "nai-diffusion-furry-3"
);

const preset = reactive<GeneratePreset>({
  quantity: 1,
  width: 832,
  height: 1216,
  steps: 27,
  scale: 5,
  sampler: "k_euler_ancestral",
  noise_schedule: "karras",
  cfg_rescale: null as any,
  seed: -1,
  add_quality_tags: true,
  undesired_content_preset: "None" as any,
  sm: false,
  sm_dyn: false,
  use_coords: true,
  legacy_uc: false,
});

function applyDefaultsForModel(m: string) {
  const isV3 = m === "nai-diffusion-3" || m === "nai-diffusion-furry-3";

  preset.quantity = 1;
  preset.width = 832;
  preset.height = 1216;
  preset.steps = 27;
  preset.scale = 5;
  preset.sampler = "k_euler_ancestral";
  preset.noise_schedule = null as any;
  preset.cfg_rescale = null as any;
  preset.seed = -1;
  preset.add_quality_tags = true;
  preset.undesired_content_preset = "None" as any;

  if (isV3) {
    preset.sm = false;
    preset.sm_dyn = false;
  } else {
    preset.use_coords = true;
    preset.legacy_uc = false;
  }
}

async function refreshNames() {
  const names = await presetStore.refreshGenerateNames(model.value);
  if (!names.includes(presetName.value)) {
    presetName.value = names[0] ?? "";
  }
}

async function loadSelected() {
  const name = presetName.value.trim() || presetNames.value[0] || "";
  if (!name) {
    applyDefaultsForModel(model.value);
    return;
  }
  busy.value = true;
  try {
    const p = await presetStore.fetchGeneratePreset(model.value, name);
    if (!p) {
      applyDefaultsForModel(model.value);
      return;
    }

    // copy fields
    Object.assign(preset, p);
  } catch {
    applyDefaultsForModel(model.value);
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
    await endpoints.presetPut({
      model: model.value,
      name,
      preset: { ...preset },
    });
    presetStore.rememberGeneratePreset(model.value, name, { ...preset });
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
    await endpoints.presetRename({ model: model.value, from, to });
    presetStore.evictGeneratePreset(model.value, from);
    presetStore.rememberGeneratePreset(model.value, to, { ...preset });
    presetName.value = to;
    await refreshNames();
  } finally {
    busy.value = false;
  }
}

async function remove() {
  const name = presetName.value.trim();
  if (!name) return;
  if (!confirm(`确定删除预设：${name}？`)) return;
  busy.value = true;
  try {
    await endpoints.presetDelete(model.value, name);
    presetStore.evictGeneratePreset(model.value, name);
    await refreshNames();
    presetName.value = presetNames.value[0] ?? "";
    await loadSelected();
  } finally {
    busy.value = false;
  }
}

watch(
  () => model.value,
  () => {
    presetName.value = "";
    applyDefaultsForModel(model.value);
    void refreshNames();
  },
  { immediate: true }
);

watch(
  () => presetName.value,
  (v, prev) => {
    if (v === prev) return;
    void loadSelected();
  }
);

async function ensureMeta() {
  await metaStore.ensureLoaded();
  if (metaStore.models.length && !metaStore.models.includes(model.value)) {
    model.value = metaStore.models[0];
  }
}

void ensureMeta();
</script>

<template>
  <div class="grid gap-4">
    <div class="grid grid-cols-1 gap-3 lg:grid-cols-3">
      <fieldset class="fieldset">
        <legend class="fieldset-legend">模型</legend>
        <select v-model="model" class="select select-bordered w-full">
          <option v-for="m in metaStore.models" :key="m" :value="m">
            {{ m }}
          </option>
        </select>
      </fieldset>

      <fieldset class="fieldset lg:col-span-2">
        <legend class="fieldset-legend">预设名</legend>
        <div class="join w-full">
          <input
            v-model="presetName"
            class="input input-bordered join-item w-full"
            :disabled="busy"
            list="managePresetNames"
            placeholder="输入或选择预设名"
          />
          <button
            class="btn join-item"
            type="button"
            :class="{ 'btn-disabled': busy || !presetName.trim() }"
            @click="save"
          >
            保存
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
        </div>
        <datalist id="managePresetNames">
          <option v-for="n in presetNames" :key="n" :value="n" />
        </datalist>
        <div class="label">
          <span class="label-text-alt">此处编辑“生成预设”字段子集</span>
          <span v-if="busy" class="label-text-alt">加载中…</span>
        </div>
      </fieldset>
    </div>

    <div class="divider">预设字段</div>

    <div class="grid grid-cols-1 gap-3 lg:grid-cols-3">
      <fieldset class="fieldset">
        <legend class="fieldset-legend">张数</legend>
        <input
          v-model.number="preset.quantity"
          type="number"
          min="1"
          class="input input-bordered w-full"
        />
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">宽</legend>
        <input
          v-model.number="preset.width"
          type="number"
          min="64"
          step="64"
          class="input input-bordered w-full"
        />
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">高</legend>
        <input
          v-model.number="preset.height"
          type="number"
          min="64"
          step="64"
          class="input input-bordered w-full"
        />
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Steps</legend>
        <input
          v-model.number="preset.steps"
          type="number"
          min="1"
          class="input input-bordered w-full"
        />
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Scale</legend>
        <input
          v-model.number="preset.scale"
          type="number"
          min="0"
          step="0.1"
          class="input input-bordered w-full"
        />
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Sampler</legend>
        <select v-model="preset.sampler" class="select select-bordered w-full">
          <option v-for="s in metaStore.samplers" :key="s" :value="s">
            {{ s }}
          </option>
        </select>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Noise schedule</legend>
        <select
          v-model="preset.noise_schedule"
          class="select select-bordered w-full"
        >
          <option v-for="u in metaStore.noiseSchedules" :key="u" :value="u">
            {{ u }}
          </option>
        </select>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">CFG rescale</legend>
        <input
          v-model="preset.cfg_rescale"
          class="input input-bordered w-full"
          placeholder="null / value"
        />
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Seed</legend>
        <input
          v-model.number="preset.seed"
          type="number"
          class="input input-bordered w-full"
        />
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Add quality tags</legend>
        <label
          class="flex items-center gap-3 rounded-lg border border-base-300/70 bg-base-100/70 px-3 py-2"
        >
          <input
            v-model="preset.add_quality_tags"
            type="checkbox"
            class="toggle"
          />
          <span class="text-sm">{{
            preset.add_quality_tags ? "开启" : "关闭"
          }}</span>
        </label>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Undesired content</legend>
        <select
          v-model="preset.undesired_content_preset"
          class="select select-bordered w-full"
        >
          <option v-for="u in metaStore.ucPresets" :key="u" :value="u">
            {{ u }}
          </option>
        </select>
      </fieldset>

      <template v-if="isV3Model">
        <fieldset class="fieldset">
          <legend class="fieldset-legend">SM</legend>
          <label
            class="flex items-center gap-3 rounded-lg border border-base-300/70 bg-base-100/70 px-3 py-2"
          >
            <input v-model="preset.sm" type="checkbox" class="toggle" />
            <span class="text-sm">{{ preset.sm ? "开启" : "关闭" }}</span>
          </label>
        </fieldset>
        <fieldset class="fieldset">
          <legend class="fieldset-legend">SM dyn</legend>
          <label
            class="flex items-center gap-3 rounded-lg border border-base-300/70 bg-base-100/70 px-3 py-2"
          >
            <input v-model="preset.sm_dyn" type="checkbox" class="toggle" />
            <span class="text-sm">{{ preset.sm_dyn ? "开启" : "关闭" }}</span>
          </label>
        </fieldset>
      </template>

      <template v-else>
        <fieldset class="fieldset">
          <legend class="fieldset-legend">Use coords</legend>
          <label
            class="flex items-center gap-3 rounded-lg border border-base-300/70 bg-base-100/70 px-3 py-2"
          >
            <input v-model="preset.use_coords" type="checkbox" class="toggle" />
            <span class="text-sm">{{
              preset.use_coords ? "开启" : "关闭"
            }}</span>
          </label>
        </fieldset>
        <fieldset class="fieldset">
          <legend class="fieldset-legend">Legacy UC</legend>
          <label
            class="flex items-center gap-3 rounded-lg border border-base-300/70 bg-base-100/70 px-3 py-2"
          >
            <input v-model="preset.legacy_uc" type="checkbox" class="toggle" />
            <span class="text-sm">{{
              preset.legacy_uc ? "开启" : "关闭"
            }}</span>
          </label>
        </fieldset>
      </template>
    </div>
  </div>
</template>
