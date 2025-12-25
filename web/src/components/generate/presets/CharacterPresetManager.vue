<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import { endpoints } from "@/api/endpoints";
import { usePresetStore } from "@/stores/presets";
import {
  POSITION_OPTIONS,
  centerToPosition,
  positionToCenter,
} from "@/composables/useCharacterPosition";
import type { CharacterSlotPreset } from "@/api/types";

const presetStore = usePresetStore();
const busy = ref(false);
const selected = ref<string>("");

const presetNames = computed(() => presetStore.characterNames);

const working = reactive<CharacterSlotPreset>({
  prompt: "",
  uc: "",
  center: { x: 0.5, y: 0.5 },
});

const saveAsName = ref<string>("");

const position = computed({
  get() {
    return centerToPosition(working.center);
  },
  set(v: string) {
    working.center = positionToCenter(v);
  },
});

async function refreshNames() {
  const names = await presetStore.refreshCharacterNames();
  if (!names.includes(selected.value)) {
    selected.value = names[0] ?? "";
  }
}

async function loadSelected() {
  const name = selected.value.trim();
  if (!name) return;
  busy.value = true;
  try {
    const preset = await presetStore.fetchCharacterPreset(name);
    if (!preset) return;
    working.prompt = preset.prompt;
    working.uc = preset.uc;
    working.center = preset.center;
  } finally {
    busy.value = false;
  }
}

async function save() {
  const name = saveAsName.value.trim() || selected.value.trim();
  if (!name) return;
  busy.value = true;
  try {
    await endpoints.characterPresetPut({ name, preset: { ...working } });
    presetStore.rememberCharacterPreset(name, { ...working });
    selected.value = name;
    saveAsName.value = "";
    await refreshNames();
  } finally {
    busy.value = false;
  }
}

async function rename() {
  const from = selected.value.trim();
  if (!from) return;
  const to = prompt("新预设名：", from)?.trim();
  if (!to || to === from) return;
  busy.value = true;
  try {
    await endpoints.characterPresetRename({ from, to });
    presetStore.evictCharacterPreset(from);
    presetStore.rememberCharacterPreset(to, { ...working });
    selected.value = to;
    await refreshNames();
  } finally {
    busy.value = false;
  }
}

async function remove() {
  const name = selected.value.trim();
  if (!name) return;
  if (!confirm(`确定删除角色预设：${name}？`)) return;
  busy.value = true;
  try {
    await endpoints.characterPresetDelete(name);
    presetStore.evictCharacterPreset(name);
    selected.value = "";
    await refreshNames();
  } finally {
    busy.value = false;
  }
}

void refreshNames();
</script>

<template>
  <div class="grid gap-4">
    <div class="grid grid-cols-1 gap-3 lg:grid-cols-3">
      <label class="form-control items-start lg:col-span-2">
        <div class="label"><span class="label-text">选择预设</span></div>
        <div class="join w-full">
          <input
            v-model="selected"
            class="input input-bordered join-item w-full"
            list="characterPresetNames"
            placeholder="选择后加载编辑"
            :disabled="busy"
          />
          <datalist id="characterPresetNames">
            <option v-for="n in presetNames" :key="n" :value="n" />
          </datalist>
          <button
            class="btn join-item"
            type="button"
            :class="{ 'btn-disabled': busy }"
            @click="loadSelected"
          >
            加载
          </button>
          <button
            class="btn join-item"
            type="button"
            :class="{ 'btn-disabled': busy }"
            @click="refreshNames"
          >
            刷新
          </button>
          <button
            class="btn"
            type="button"
            :class="{ 'btn-disabled': busy || !selected.trim() }"
            @click="rename"
          >
            重命名
          </button>
          <button
            class="btn btn-error"
            type="button"
            :class="{ 'btn-disabled': busy || !selected.trim() }"
            @click="remove"
          >
            删除
          </button>
        </div>
      </label>
    </div>

    <div class="divider">字段</div>

    <div class="grid grid-cols-1 gap-3 lg:grid-cols-3">
      <label class="form-control items-start">
        <div class="label"><span class="label-text">位置</span></div>
        <select
          v-model="position"
          class="select select-bordered w-full"
          :disabled="busy"
        >
          <option v-for="p in POSITION_OPTIONS" :key="p" :value="p">
            {{ p }}
          </option>
        </select>
      </label>

      <label class="form-control items-start lg:col-span-2">
        <div class="label"><span class="label-text">保存为</span></div>
        <div class="join w-full">
          <input
            v-model="saveAsName"
            class="input input-bordered join-item w-full"
            placeholder="新预设名（留空则覆盖当前选择）"
            :disabled="busy"
          />
          <button
            class="btn btn-primary join-item"
            type="button"
            :class="{ 'btn-disabled': busy }"
            @click="save"
          >
            保存
          </button>
        </div>
      </label>

      <fieldset class="fieldset lg:col-span-3">
        <legend class="fieldset-legend">正向（角色）</legend>
        <textarea
          v-model="working.prompt"
          class="textarea textarea-bordered h-28 w-full"
          :disabled="busy"
        />
      </fieldset>

      <fieldset class="fieldset lg:col-span-3">
        <legend class="fieldset-legend">反向（角色）</legend>
        <textarea
          v-model="working.uc"
          class="textarea textarea-bordered h-28 w-full"
          :disabled="busy"
        />
      </fieldset>
    </div>

    <div class="text-xs opacity-70">
      只保存/编辑单个角色槽位（prompt/uc/位置）。
    </div>
  </div>
</template>
