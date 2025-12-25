<script setup lang="ts">
import { useMetaStore } from "@/stores/meta";
import type { BaseGenerateRequest } from "@/api/types";

const metaStore = useMetaStore();

const props = defineProps<{
  form: BaseGenerateRequest;
  presetMode?: "apply" | "manage";
  presetName: string;
  presetNames: string[];
  presetBusy: boolean;
  defaultPresetName: string;
  onSavePreset: () => void;
  onRenamePreset: () => void;
  onDeletePreset: () => void;
}>();

const emit = defineEmits<{
  (e: "update:presetName", v: string): void;
}>();
</script>

<template>
  <div class="grid grid-cols-1 gap-3 lg:grid-cols-3">
    <label class="form-control">
      <div class="label"><span class="label-text">模型</span></div>
      <select v-model="props.form.model" class="select select-bordered">
        <option v-for="m in metaStore.models" :key="m" :value="m">
          {{ m }}
        </option>
      </select>
    </label>

    <label class="form-control">
      <div class="label">
        <span class="label-text">预设（model + name）</span>
      </div>
      <div class="join w-full">
        <input
          :value="props.presetName"
          class="input input-bordered join-item w-full"
          :disabled="props.presetBusy"
          list="presetNames"
          placeholder="默认"
          @input="
            emit('update:presetName', ($event.target as HTMLInputElement).value)
          "
        />
        <template v-if="props.presetMode === 'manage'">
          <button
            class="btn join-item"
            type="button"
            :class="{ 'btn-disabled': props.presetBusy }"
            @click="props.onSavePreset"
          >
            保存
          </button>
          <button
            class="btn join-item"
            type="button"
            :class="{
              'btn-disabled':
                props.presetBusy ||
                props.presetName.trim() === props.defaultPresetName,
            }"
            @click="props.onRenamePreset"
          >
            重命名
          </button>
          <button
            class="btn btn-error join-item"
            type="button"
            :class="{
              'btn-disabled':
                props.presetBusy ||
                props.presetName.trim() === props.defaultPresetName,
            }"
            @click="props.onDeletePreset"
          >
            删除
          </button>
        </template>
      </div>
      <datalist id="presetNames">
        <option v-for="n in props.presetNames" :key="n" :value="n" />
      </datalist>
      <div class="label">
        <span class="label-text-alt">切换模型会强制应用对应预设</span>
        <span v-if="props.presetBusy" class="label-text-alt">加载中…</span>
      </div>
    </label>

    <label class="form-control">
      <div class="label"><span class="label-text">采样器</span></div>
      <select v-model="props.form.sampler" class="select select-bordered">
        <option v-for="s in metaStore.samplers" :key="s" :value="s">
          {{ s }}
        </option>
      </select>
    </label>
  </div>
</template>
