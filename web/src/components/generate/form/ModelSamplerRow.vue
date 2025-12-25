<script setup lang="ts">
import { useMetaStore } from "@/stores/meta";
import type { BaseGenerateRequest } from "@/api/types";

const metaStore = useMetaStore();

const props = defineProps<{
  form: BaseGenerateRequest;
}>();
</script>

<template>
  <div class="grid grid-cols-1 gap-3 lg:grid-cols-3">
    <fieldset class="fieldset">
      <span>模型</span>
      <select v-model="props.form.model" class="select select-bordered w-full">
        <option v-for="m in metaStore.models" :key="m" :value="m">
          {{ m }}
        </option>
      </select>
    </fieldset>

    <div class="hidden lg:block"></div>

    <fieldset class="fieldset">
      <span>采样器</span>
      <select
        v-model="props.form.sampler"
        class="select select-bordered w-full"
      >
        <option v-for="s in metaStore.samplers" :key="s" :value="s">
          {{ s }}
        </option>
      </select>
    </fieldset>
  </div>
</template>
