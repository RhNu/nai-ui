<script setup lang="ts">
import type { BaseGenerateRequest } from "@/api/types";

const props = defineProps<{
  form: BaseGenerateRequest;
  sizePreset: string;
  sizePresets: readonly string[];
}>();

const emit = defineEmits<{
  (e: "update:sizePreset", v: string): void;
}>();
</script>

<template>
  <div class="grid grid-cols-2 gap-3 lg:grid-cols-5">
    <fieldset class="fieldset">
      <span>尺寸预设</span>
      <select
        :value="props.sizePreset"
        class="select select-bordered w-full"
        @change="
          emit('update:sizePreset', ($event.target as HTMLSelectElement).value)
        "
      >
        <option value="自定义">自定义</option>
        <option v-for="s in props.sizePresets" :key="s" :value="s">
          {{ s }}
        </option>
      </select>
    </fieldset>
    <fieldset class="fieldset">
      <span>宽</span>
      <input
        v-model.number="props.form.width"
        class="input input-bordered w-full"
        type="number"
        min="64"
        step="64"
      />
    </fieldset>
    <fieldset class="fieldset">
      <span>高</span>
      <input
        v-model.number="props.form.height"
        class="input input-bordered w-full"
        type="number"
        min="64"
        step="64"
      />
    </fieldset>
    <fieldset class="fieldset">
      <span>步数(steps)</span>
      <input
        v-model.number="props.form.steps"
        class="input input-bordered w-full"
        type="number"
        min="1"
        max="200"
      />
    </fieldset>
    <fieldset class="fieldset">
      <span>CFG(Scale)</span>
      <input
        v-model.number="props.form.scale"
        class="input input-bordered w-full"
        type="number"
        min="0"
        step="0.1"
      />
    </fieldset>
  </div>
</template>
