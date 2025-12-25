<script setup lang="ts">
const props = defineProps<{
  presetMode?: "apply" | "manage";
  promptPresetName: string;
  promptPresetNames: string[];
  onApplyOnce: () => void;
  onSave: () => void;
  onRename: () => void;
  onDelete: () => void;
}>();

const emit = defineEmits<{
  (e: "update:promptPresetName", v: string): void;
}>();
</script>

<template>
  <div class="card bg-base-200">
    <div class="card-body gap-3">
      <div class="font-medium">提示词预设（apply 一次）</div>
      <div class="grid grid-cols-1 gap-3 lg:grid-cols-3">
        <label class="form-control">
          <div class="label"><span class="label-text">预设名</span></div>
          <input
            :value="props.promptPresetName"
            class="input input-bordered"
            list="promptPresetNames"
            placeholder="例如：构图-风格-人物"
            @input="
              emit(
                'update:promptPresetName',
                ($event.target as HTMLInputElement).value
              )
            "
          />
          <datalist id="promptPresetNames">
            <option v-for="n in props.promptPresetNames" :key="n" :value="n" />
          </datalist>
          <div class="label">
            <span class="label-text-alt"
              >应用只覆盖一次；生成后会融合到上次参数</span
            >
          </div>
        </label>

        <div class="flex flex-wrap items-end gap-2">
          <button
            class="btn btn-primary"
            type="button"
            @click="props.onApplyOnce"
          >
            应用（一次）
          </button>
          <template v-if="props.presetMode === 'manage'">
            <button class="btn" type="button" @click="props.onSave">
              保存/更新
            </button>
            <button class="btn" type="button" @click="props.onRename">
              重命名
            </button>
            <button
              class="btn btn-outline"
              type="button"
              @click="props.onDelete"
            >
              删除
            </button>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>
