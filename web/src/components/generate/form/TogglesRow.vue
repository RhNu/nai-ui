<script setup lang="ts">
import type { BaseGenerateRequest } from "@/api/types";

const props = defineProps<{
  form: BaseGenerateRequest;
  isV3Model: boolean;
}>();
</script>

<template>
  <div class="grid grid-cols-1 gap-3 lg:grid-cols-2">
    <fieldset class="fieldset">
      <span>添加质量提示词(add_quality_tags)</span>
      <label
        class="flex items-center gap-3 rounded-lg border border-base-300/70 bg-base-100/70 px-3 py-2"
      >
        <input
          v-model="props.form.add_quality_tags"
          type="checkbox"
          class="toggle"
        />
        <span class="text-sm">{{
          props.form.add_quality_tags ? "开启" : "关闭"
        }}</span>
      </label>
    </fieldset>

    <div class="grid grid-cols-2 gap-3">
      <fieldset class="fieldset" :class="{ 'opacity-60': !props.isV3Model }">
        <span>sm (v3)</span>
        <label
          class="flex items-center gap-3 rounded-lg border border-base-300/70 bg-base-100/70 px-3 py-2"
        >
          <input
            v-model="props.form.sm"
            type="checkbox"
            class="toggle"
            :disabled="!props.isV3Model"
          />
          <span class="text-sm">{{ props.form.sm ? "开启" : "关闭" }}</span>
        </label>
      </fieldset>

      <fieldset class="fieldset" :class="{ 'opacity-60': !props.isV3Model }">
        <span>sm_dyn (v3)</span>
        <label
          class="flex items-center gap-3 rounded-lg border border-base-300/70 bg-base-100/70 px-3 py-2"
        >
          <input
            v-model="props.form.sm_dyn"
            type="checkbox"
            class="toggle"
            :disabled="!props.isV3Model"
          />
          <span class="text-sm">{{ props.form.sm_dyn ? "开启" : "关闭" }}</span>
        </label>
      </fieldset>

      <fieldset class="fieldset" :class="{ 'opacity-60': props.isV3Model }">
        <span>use_coords (v4)</span>
        <label
          class="flex items-center gap-3 rounded-lg border border-base-300/70 bg-base-100/70 px-3 py-2"
        >
          <input
            v-model="props.form.use_coords"
            type="checkbox"
            class="toggle"
            :disabled="props.isV3Model"
          />
          <span class="text-sm">{{
            props.form.use_coords ? "开启" : "关闭"
          }}</span>
        </label>
      </fieldset>

      <fieldset class="fieldset" :class="{ 'opacity-60': props.isV3Model }">
        <span>legacy_uc (v4)</span>
        <label
          class="flex items-center gap-3 rounded-lg border border-base-300/70 bg-base-100/70 px-3 py-2"
        >
          <input
            v-model="props.form.legacy_uc"
            type="checkbox"
            class="toggle"
            :disabled="props.isV3Model"
          />
          <span class="text-sm">{{
            props.form.legacy_uc ? "开启" : "关闭"
          }}</span>
        </label>
      </fieldset>
    </div>
  </div>
</template>
