<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import { endpoints } from "@/api/endpoints";
import type { Center, CharacterSlotPreset } from "@/api/types";

const busy = ref(false);
const presetNames = ref<string[]>([]);
const selected = ref<string>("");

const working = reactive<CharacterSlotPreset>({
  prompt: "",
  uc: "",
  center: { x: 0.5, y: 0.5 },
});

const saveAsName = ref<string>("");

const POSITION_OPTIONS = (() => {
  const opts: string[] = [];
  for (let li = 0; li < 5; li++) {
    const letter = String.fromCharCode(65 + li);
    for (let ni = 1; ni <= 5; ni++) opts.push(`${letter}${ni}`);
  }
  return opts;
})();

function positionToCenter(position: string): Center {
  const offset = 0.1;
  const letter = position[0] ?? "C";
  const number = position[1] ?? "3";
  const li = Math.min(4, Math.max(0, letter.toUpperCase().charCodeAt(0) - 65));
  const ni = Math.min(4, Math.max(0, parseInt(number, 10) - 1));
  return {
    x: Math.round((li * 0.2 + offset) * 10) / 10,
    y: Math.round((ni * 0.2 + offset) * 10) / 10,
  };
}

function centerToPosition(center: Center): string {
  const offset = 0.1;
  const grid = [0, 1, 2, 3, 4].map(
    (i) => Math.round((i * 0.2 + offset) * 10) / 10
  );
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
  const li = nearestIndex(center.x);
  const ni = nearestIndex(center.y);
  return `${String.fromCharCode(65 + li)}${String(ni + 1)}`;
}

const position = computed({
  get() {
    return centerToPosition(working.center);
  },
  set(v: string) {
    working.center = positionToCenter(v);
  },
});

async function refreshNames() {
  try {
    const r = await endpoints.characterPresetsList();
    presetNames.value = (r.names ?? []).slice();
  } catch {
    presetNames.value = [];
  }
}

async function loadSelected() {
  const name = selected.value.trim();
  if (!name) return;
  busy.value = true;
  try {
    const r = await endpoints.characterPresetGet(name);
    if (!r.preset) return;
    working.prompt = r.preset.prompt;
    working.uc = r.preset.uc;
    working.center = r.preset.center;
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
