<script setup lang="ts">
import type { BaseGenerateRequest } from "@/api/types";

const props = defineProps<{
  form: BaseGenerateRequest;
}>();
</script>

<template>
  <fieldset class="fieldset">
    <legend class="fieldset-legend">正向提示词</legend>
    <textarea
      v-model="props.form.positive"
      class="textarea textarea-bordered h-28 w-full"
      placeholder="positive prompt"
    />
  </fieldset>

  <fieldset class="fieldset">
    <legend class="fieldset-legend">反向提示词</legend>
    <textarea
      v-model="props.form.negative"
      class="textarea textarea-bordered h-28 w-full"
      placeholder="negative prompt"
    />
  </fieldset>
</template>
