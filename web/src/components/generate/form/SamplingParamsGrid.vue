<script setup lang="ts">
import { useMetaStore } from "@/stores/meta";
import type { BaseGenerateRequest } from "@/api/types";

const metaStore = useMetaStore();

const props = defineProps<{
  form: BaseGenerateRequest;
}>();
</script>

<template>
  <div class="grid grid-cols-2 gap-3 lg:grid-cols-4">
    <fieldset class="fieldset">
      <span>种子(seed)</span>
      <input
        v-model.number="props.form.seed"
        class="input input-bordered w-full"
        type="number"
      />
      <div class="label">-1 为随机</div>
    </fieldset>

    <fieldset class="fieldset">
      <span>数量</span>
      <input
        v-model.number="props.form.quantity"
        class="input input-bordered w-full"
        type="number"
        min="1"
        step="1"
      />
      <div class="label">>1 将作为单个 job 批量生成</div>
    </fieldset>

    <fieldset class="fieldset">
      <span>噪声调度器(noise_schedule)</span>
      <select
        v-model="props.form.noise_schedule"
        class="select select-bordered w-full"
      >
        <option :value="null">(默认)</option>
        <option v-for="n in metaStore.noiseSchedules" :key="n" :value="n">
          {{ n }}
        </option>
      </select>
    </fieldset>

    <fieldset class="fieldset">
      <span>负面提示词预设(uc preset)</span>
      <select
        v-model="props.form.undesired_content_preset"
        class="select select-bordered w-full"
      >
        <option v-for="u in metaStore.ucPresets" :key="u" :value="u">
          {{ u }}
        </option>
      </select>
    </fieldset>

    <fieldset class="fieldset">
      <span>cfg_rescale</span>
      <input
        v-model.number="props.form.cfg_rescale"
        class="input input-bordered w-full"
        type="number"
        step="0.01"
      />
    </fieldset>
  </div>
</template>
