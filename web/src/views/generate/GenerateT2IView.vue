<script setup lang="ts">
import { ref } from "vue";
import BaseGenerateForm from "@/components/BaseGenerateForm.vue";
import GenerateNav from "@/components/GenerateNav.vue";
import PageShell from "@/components/layout/PageShell.vue";
import { endpoints } from "@/api/endpoints";
import type { BaseGenerateRequest } from "@/api/types";
import { useJobsStore } from "@/stores/jobs";

const jobs = useJobsStore();

const loading = ref(false);
const errorText = ref("");
const lastJobId = ref("");

async function runJob(req: BaseGenerateRequest) {
  loading.value = true;
  errorText.value = "";
  lastJobId.value = "";
  try {
    const r = await endpoints.jobT2i(req);
    jobs.track(r.job_id, "t2i");
    lastJobId.value = r.job_id;
  } catch (e) {
    errorText.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <PageShell title="生成" subtitle="文本到图像（t2i）提交队列" max-width="2xl">
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
              Job ID：<span class="font-mono break-all">{{ lastJobId }}</span>
            </div>
          </div>
        </div>
      </BaseGenerateForm>
    </div>
  </PageShell>
</template>
