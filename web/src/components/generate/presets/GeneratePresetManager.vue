<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { endpoints } from "@/api/endpoints";
import { useMetaStore } from "@/stores/meta";
import type { GeneratePreset } from "@/api/types";

const DEFAULT_PRESET_NAME = "默认";
const metaStore = useMetaStore();

const model = ref<string>("nai-diffusion-4-5-full");
const presetName = ref<string>(DEFAULT_PRESET_NAME);
const presetNames = ref<string[]>([DEFAULT_PRESET_NAME]);
const busy = ref(false);

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
  noise_schedule: null as any,
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
  try {
    const r = await endpoints.presetsList(model.value);
    const names = (r.names ?? []).slice();
    if (!names.includes(DEFAULT_PRESET_NAME))
      names.unshift(DEFAULT_PRESET_NAME);
    presetNames.value = names.length ? names : [DEFAULT_PRESET_NAME];
  } catch {
    presetNames.value = [DEFAULT_PRESET_NAME];
  }
}

async function loadSelected() {
  const name = presetName.value.trim() || DEFAULT_PRESET_NAME;
  busy.value = true;
  try {
    await refreshNames();
    const r = await endpoints.presetGet(model.value, name);
    if (!r.preset) {
      applyDefaultsForModel(model.value);
      return;
    }

    // copy fields
    Object.assign(preset, r.preset);
  } catch {
    applyDefaultsForModel(model.value);
  } finally {
    busy.value = false;
  }
}

async function save() {
  const name = presetName.value.trim() || DEFAULT_PRESET_NAME;
  busy.value = true;
  try {
    await endpoints.presetPut({
      model: model.value,
      name,
      preset: { ...preset },
    });
    presetName.value = name;
    await refreshNames();
  } finally {
    busy.value = false;
  }
}

async function rename() {
  const from = presetName.value.trim() || DEFAULT_PRESET_NAME;
  if (from === DEFAULT_PRESET_NAME) return;
  const to = prompt("新预设名：", from)?.trim();
  if (!to || to === from) return;
  busy.value = true;
  try {
    await endpoints.presetRename({ model: model.value, from, to });
    presetName.value = to;
    await refreshNames();
  } finally {
    busy.value = false;
  }
}

async function remove() {
  const name = presetName.value.trim() || DEFAULT_PRESET_NAME;
  if (name === DEFAULT_PRESET_NAME) return;
  if (!confirm(`确定删除预设：${name}？`)) return;
  busy.value = true;
  try {
    await endpoints.presetDelete(model.value, name);
    presetName.value = DEFAULT_PRESET_NAME;
    await refreshNames();
    await loadSelected();
  } finally {
    busy.value = false;
  }
}

watch(
  () => model.value,
  () => {
    presetName.value = DEFAULT_PRESET_NAME;
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
      <label class="form-control items-start">
        <div class="label"><span class="label-text">模型</span></div>
        <select v-model="model" class="select select-bordered w-full">
          <option v-for="m in metaStore.models" :key="m" :value="m">
            {{ m }}
          </option>
        </select>
      </label>

      <label class="form-control items-start lg:col-span-2">
        <div class="label"><span class="label-text">预设名</span></div>
        <div class="join w-full">
          <input
            v-model="presetName"
            class="input input-bordered join-item w-full"
            :disabled="busy"
            list="managePresetNames"
            placeholder="默认"
          />
          <button
            class="btn join-item"
            type="button"
            :class="{ 'btn-disabled': busy }"
            @click="save"
          >
            保存
          </button>
          <button
            class="btn join-item"
            type="button"
            :class="{
              'btn-disabled': busy || presetName.trim() === DEFAULT_PRESET_NAME,
            }"
            @click="rename"
          >
            重命名
          </button>
          <button
            class="btn btn-error join-item"
            type="button"
            :class="{
              'btn-disabled': busy || presetName.trim() === DEFAULT_PRESET_NAME,
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
          <span class="label-text-alt">此处只编辑“生成预设”字段子集</span>
          <span v-if="busy" class="label-text-alt">加载中…</span>
        </div>
      </label>
    </div>

    <div class="divider">预设字段</div>

    <div class="grid grid-cols-1 gap-3 lg:grid-cols-3">
      <label class="form-control items-start">
        <div class="label"><span class="label-text">张数</span></div>
        <input
          v-model.number="preset.quantity"
          type="number"
          min="1"
          class="input input-bordered w-full"
        />
      </label>

      <label class="form-control items-start">
        <div class="label"><span class="label-text">宽</span></div>
        <input
          v-model.number="preset.width"
          type="number"
          min="64"
          step="64"
          class="input input-bordered w-full"
        />
      </label>

      <label class="form-control items-start">
        <div class="label"><span class="label-text">高</span></div>
        <input
          v-model.number="preset.height"
          type="number"
          min="64"
          step="64"
          class="input input-bordered w-full"
        />
      </label>

      <label class="form-control items-start">
        <div class="label"><span class="label-text">Steps</span></div>
        <input
          v-model.number="preset.steps"
          type="number"
          min="1"
          class="input input-bordered w-full"
        />
      </label>

      <label class="form-control items-start">
        <div class="label"><span class="label-text">Scale</span></div>
        <input
          v-model.number="preset.scale"
          type="number"
          min="0"
          step="0.1"
          class="input input-bordered w-full"
        />
      </label>

      <label class="form-control items-start">
        <div class="label"><span class="label-text">Sampler</span></div>
        <select v-model="preset.sampler" class="select select-bordered w-full">
          <option v-for="s in metaStore.samplers" :key="s" :value="s">
            {{ s }}
          </option>
        </select>
      </label>

      <label class="form-control items-start">
        <div class="label"><span class="label-text">Noise schedule</span></div>
        <input
          v-model="preset.noise_schedule"
          class="input input-bordered w-full"
          placeholder="null / value"
        />
      </label>

      <label class="form-control items-start">
        <div class="label"><span class="label-text">CFG rescale</span></div>
        <input
          v-model="preset.cfg_rescale"
          class="input input-bordered w-full"
          placeholder="null / value"
        />
      </label>

      <label class="form-control items-start">
        <div class="label"><span class="label-text">Seed</span></div>
        <input
          v-model.number="preset.seed"
          type="number"
          class="input input-bordered w-full"
        />
      </label>

      <label class="form-control items-start">
        <div class="label">
          <span class="label-text">Add quality tags</span>
        </div>
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
      </label>

      <label class="form-control items-start">
        <div class="label">
          <span class="label-text">Undesired content</span>
        </div>
        <input
          v-model="preset.undesired_content_preset"
          class="input input-bordered"
        />
      </label>

      <template v-if="isV3Model">
        <label class="form-control items-start">
          <div class="label"><span class="label-text">SM</span></div>
          <label
            class="flex items-center gap-3 rounded-lg border border-base-300/70 bg-base-100/70 px-3 py-2"
          >
            <input v-model="preset.sm" type="checkbox" class="toggle" />
            <span class="text-sm">{{ preset.sm ? "开启" : "关闭" }}</span>
          </label>
        </label>
        <label class="form-control items-start">
          <div class="label"><span class="label-text">SM dyn</span></div>
          <label
            class="flex items-center gap-3 rounded-lg border border-base-300/70 bg-base-100/70 px-3 py-2"
          >
            <input v-model="preset.sm_dyn" type="checkbox" class="toggle" />
            <span class="text-sm">{{ preset.sm_dyn ? "开启" : "关闭" }}</span>
          </label>
        </label>
      </template>

      <template v-else>
        <label class="form-control items-start">
          <div class="label"><span class="label-text">Use coords</span></div>
          <label
            class="flex items-center gap-3 rounded-lg border border-base-300/70 bg-base-100/70 px-3 py-2"
          >
            <input v-model="preset.use_coords" type="checkbox" class="toggle" />
            <span class="text-sm">{{
              preset.use_coords ? "开启" : "关闭"
            }}</span>
          </label>
        </label>
        <label class="form-control items-start">
          <div class="label"><span class="label-text">Legacy UC</span></div>
          <label
            class="flex items-center gap-3 rounded-lg border border-base-300/70 bg-base-100/70 px-3 py-2"
          >
            <input v-model="preset.legacy_uc" type="checkbox" class="toggle" />
            <span class="text-sm">{{
              preset.legacy_uc ? "开启" : "关闭"
            }}</span>
          </label>
        </label>
      </template>
    </div>
  </div>
</template>
