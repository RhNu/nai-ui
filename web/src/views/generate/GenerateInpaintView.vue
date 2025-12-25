<script setup lang="ts">
import { ref } from "vue";
import { endpoints } from "@/api/endpoints";
import type { InpaintRequest } from "@/api/types";
import BaseGenerateForm from "@/components/BaseGenerateForm.vue";
import GenerateNav from "@/components/GenerateNav.vue";
import PageShell from "@/components/layout/PageShell.vue";
import { fileToBase64Payload } from "@/components/fileBase64";
import { useJobsStore } from "@/stores/jobs";

const jobs = useJobsStore();

const loading = ref(false);
const errorText = ref("");
const lastJobId = ref("");

const imageFile = ref<File | null>(null);
const maskFile = ref<File | null>(null);
const strength = ref(0.5);
const noise = ref(0.2);
const extraNoiseSeed = ref<number | null>(null);
const colorCorrect = ref(false);

function onPickImage(ev: Event) {
  const input = ev.target as HTMLInputElement;
  imageFile.value = input.files?.[0] ?? null;
}

function onPickMask(ev: Event) {
  const input = ev.target as HTMLInputElement;
  maskFile.value = input.files?.[0] ?? null;
}

async function runJob(base: any) {
  if (!imageFile.value || !maskFile.value) {
    errorText.value = "请选择原图与 mask 图片";
    return;
  }
  loading.value = true;
  errorText.value = "";
  lastJobId.value = "";
  try {
    const image_base64 = await fileToBase64Payload(imageFile.value);
    const mask_base64 = await fileToBase64Payload(maskFile.value);
    const req: InpaintRequest = {
      ...base,
      image_base64,
      mask_base64,
      strength: strength.value,
      noise: noise.value,
      extra_noise_seed: extraNoiseSeed.value,
      color_correct: colorCorrect.value,
    };
    const r = await endpoints.jobInpaint(req);
    jobs.track(r.job_id, "inpaint");
    lastJobId.value = r.job_id;
  } catch (e) {
    errorText.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <PageShell
    title="生成"
    subtitle="修复 / 补全（inpaint）提交队列"
    max-width="2xl"
  >
    <template #actions>
      <RouterLink class="btn btn-outline btn-sm" to="/jobs"
        >查看任务</RouterLink
      >
    </template>
    <template #tabs>
      <GenerateNav />
    </template>

    <div v-if="errorText" class="alert alert-error">
      <span>{{ errorText }}</span>
    </div>

    <div class="grid gap-4">
      <div
        class="grid gap-3 rounded-xl border border-base-300/70 bg-base-200/60 p-4"
      >
        <div class="grid grid-cols-1 gap-3 lg:grid-cols-2">
          <fieldset class="fieldset">
            <legend class="fieldset-legend">原图</legend>
            <input
              class="file-input file-input-bordered w-full"
              type="file"
              accept="image/*"
              @change="onPickImage"
            />
          </fieldset>
          <fieldset class="fieldset">
            <legend class="fieldset-legend">遮罩(mask)</legend>
            <input
              class="file-input file-input-bordered w-full"
              type="file"
              accept="image/*"
              @change="onPickMask"
            />
          </fieldset>
        </div>

        <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
          <fieldset class="fieldset">
            <legend class="fieldset-legend">强度(strength)</legend>
            <input
              v-model.number="strength"
              class="input input-bordered w-full"
              type="number"
              step="0.01"
              min="0"
              max="1"
            />
          </fieldset>
          <fieldset class="fieldset">
            <legend class="fieldset-legend">噪声(noise)</legend>
            <input
              v-model.number="noise"
              class="input input-bordered w-full"
              type="number"
              step="0.01"
              min="0"
              max="1"
            />
          </fieldset>
        </div>

        <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
          <fieldset class="fieldset">
            <legend class="fieldset-legend">
              额外噪声种子(extra_noise_seed)
            </legend>
            <input
              v-model.number="extraNoiseSeed"
              class="input input-bordered w-full"
              type="number"
              placeholder="(可选)"
            />
          </fieldset>
          <fieldset class="fieldset">
            <legend class="fieldset-legend">颜色修正(color_correct)</legend>
            <label
              class="flex items-center gap-3 rounded-lg border border-base-300/70 bg-base-100/70 px-3 py-2"
            >
              <input v-model="colorCorrect" type="checkbox" class="toggle" />
              <span class="text-sm">{{ colorCorrect ? "开启" : "关闭" }}</span>
            </label>
          </fieldset>
        </div>
      </div>

      <BaseGenerateForm hide-submit @submit="runJob" v-slot:actions="{ build }">
        <div class="grid gap-3">
          <div class="flex flex-wrap items-center gap-3">
            <button
              class="btn btn-primary"
              :class="{ 'btn-disabled': loading }"
              type="button"
              @click="() => runJob(build())"
            >
              提交任务
            </button>
            <div v-if="loading" class="loading loading-spinner" />
            <div
              v-if="lastJobId"
              class="rounded-full bg-base-200 px-3 py-1 text-xs font-medium"
            >
              Job ID: <span class="font-mono break-all">{{ lastJobId }}</span>
            </div>
          </div>
        </div>
      </BaseGenerateForm>
    </div>
  </PageShell>
</template>
