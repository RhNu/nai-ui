<script setup lang="ts">
import { ref } from "vue";
import { endpoints } from "@/api/endpoints";
import type { CharacterRequest } from "@/api/types";
import BaseGenerateForm from "@/components/BaseGenerateForm.vue";
import GenerateNav from "@/components/GenerateNav.vue";
import PageShell from "@/components/layout/PageShell.vue";
import { fileToBase64Payload } from "@/components/fileBase64";
import { useJobsStore } from "@/stores/jobs";

const jobs = useJobsStore();

const loading = ref(false);
const errorText = ref("");
const lastJobId = ref("");

const refFile = ref<File | null>(null);
const styleAware = ref(true);
const fidelity = ref(0.35);

function onPick(ev: Event) {
  const input = ev.target as HTMLInputElement;
  refFile.value = input.files?.[0] ?? null;
}

async function runJob(base: any) {
  if (!refFile.value) {
    errorText.value = "请选择角色参考图";
    return;
  }
  loading.value = true;
  errorText.value = "";
  lastJobId.value = "";
  try {
    const character_reference_image_base64 = await fileToBase64Payload(
      refFile.value
    );
    const req: CharacterRequest = {
      ...base,
      character_reference_image_base64,
      style_aware: styleAware.value,
      fidelity: fidelity.value,
    };
    const r = await endpoints.jobCharacter(req);
    jobs.track(r.job_id, "character");
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
    subtitle="角色定制（character）提交队列"
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
        <label class="form-control">
          <div class="label">
            <span class="label-text">角色参考图</span>
          </div>
          <input
            class="file-input file-input-bordered"
            type="file"
            accept="image/*"
            @change="onPick"
          />
        </label>

        <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
          <label class="form-control">
            <div class="label"><span class="label-text">style_aware</span></div>
            <label
              class="flex items-center gap-3 rounded-lg border border-base-300/70 bg-base-100/70 px-3 py-2"
            >
              <input v-model="styleAware" type="checkbox" class="toggle" />
              <span class="text-sm">{{ styleAware ? "开启" : "关闭" }}</span>
            </label>
          </label>
          <label class="form-control">
            <div class="label">
              <span class="label-text">fidelity</span>
            </div>
            <input
              v-model.number="fidelity"
              class="range"
              type="range"
              min="0"
              max="1"
              step="0.01"
            />
            <div class="text-xs opacity-70">{{ fidelity.toFixed(2) }}</div>
          </label>
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
              job id：<span class="font-mono break-all">{{ lastJobId }}</span>
            </div>
          </div>
        </div>
      </BaseGenerateForm>
    </div>
  </PageShell>
</template>
