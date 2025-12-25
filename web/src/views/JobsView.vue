<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useIntervalFn, useRafFn, useTimeoutFn } from "@vueuse/core";
import { RouterLink } from "vue-router";
import PageShell from "@/components/layout/PageShell.vue";
import { useJobsStore } from "@/stores/jobs";

type JobTiming = {
  queueMs: number;
  runMs: number;
  totalMs: number;
  finished: boolean;
};

const jobs = useJobsStore();
const loading = ref(false);

const intervalSeconds = ref(jobs.pollingIntervalMs / 1000);
const pollingProgress = ref(0);
const nowMs = ref(Date.now());
const nextRefreshAt = ref<number | null>(null);
const pollDelay = computed(() =>
  Math.max(500, Math.round(intervalSeconds.value * 1000))
);

const pollingEnabled = computed({
  get: () => jobs.pollingEnabled,
  set: (value: boolean) => jobs.setPollingEnabled(value),
});

const jobTimings = computed<Map<string, JobTiming>>(() => {
  const map = new Map<string, JobTiming>();
  for (const j of jobs.sorted) {
    const created = j.created_at_ms;
    if (created == null) continue;
    const start = j.started_at_ms ?? created;
    const end = j.finished_at_ms ?? nowMs.value;
    const queueMs = Math.max(0, start - created);
    const runMs = Math.max(0, end - start);
    const totalMs = Math.max(0, end - created);
    map.set(j.id, { queueMs, runMs, totalMs, finished: !!j.finished_at_ms });
  }
  return map;
});

async function refreshAll() {
  loading.value = true;
  try {
    await jobs.refreshAll();
  } finally {
    loading.value = false;
  }
}
const { pause: pauseNowTicker, resume: resumeNowTicker } = useIntervalFn(
  () => {
    nowMs.value = Date.now();
  },
  1000,
  { immediate: false }
);

const { start: startPollTimeout, stop: stopPollTimeout } = useTimeoutFn(
  async () => {
    await refreshAll();
    schedulePolling(false);
  },
  pollDelay,
  { immediate: false }
);

const { pause: pauseProgress, resume: resumeProgress } = useRafFn(
  () => {
    if (!jobs.pollingEnabled || nextRefreshAt.value === null) {
      pollingProgress.value = 0;
      return;
    }

    const total = pollDelay.value;
    const remaining = Math.max(0, nextRefreshAt.value - performance.now());
    const elapsed = total - remaining;
    const percent =
      total <= 0 ? 100 : Math.min(100, Math.max(0, (elapsed / total) * 100));
    pollingProgress.value = percent;
  },
  { immediate: true }
);

function schedulePolling(runImmediateRefresh: boolean) {
  stopPollTimeout();
  if (!jobs.pollingEnabled) {
    nextRefreshAt.value = null;
    pollingProgress.value = 0;
    pauseProgress();
    return;
  }

  nextRefreshAt.value = performance.now() + pollDelay.value;
  resumeProgress();

  if (runImmediateRefresh) {
    void refreshAll().finally(() => {
      nextRefreshAt.value = performance.now() + pollDelay.value;
      startPollTimeout();
    });
  } else {
    startPollTimeout();
  }
}

function stopPollingTimers() {
  stopPollTimeout();
  nextRefreshAt.value = null;
  pollingProgress.value = 0;
  pauseProgress();
}

function startNowTicker() {
  resumeNowTicker();
}

function stopNowTicker() {
  pauseNowTicker();
}

async function refreshNow() {
  stopPollTimeout();
  nextRefreshAt.value = null;
  await refreshAll();
  if (jobs.pollingEnabled) {
    schedulePolling(false);
  }
}

function applyInterval() {
  const ms = Math.max(500, Math.round(intervalSeconds.value * 1000));
  jobs.setPollingIntervalMs(ms);
}

function formatDuration(ms: number): string {
  const totalSeconds = Math.max(0, Math.floor(ms / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  const parts: string[] = [];
  if (hours) parts.push(`${hours}小时`);
  if (minutes) parts.push(`${minutes}分`);
  parts.push(`${seconds}秒`);
  return parts.join(" ");
}

watch(
  () => jobs.pollingIntervalMs,
  (ms) => {
    intervalSeconds.value = +(ms / 1000).toFixed(1);
  },
  { immediate: true }
);

watch(
  () => [jobs.pollingEnabled, jobs.pollingIntervalMs],
  ([enabled], [prevEnabled]) => {
    if (enabled) {
      schedulePolling(prevEnabled === false);
    } else {
      stopPollingTimers();
    }
  }
);

onMounted(() => {
  startNowTicker();
  void refreshAll();
  if (jobs.pollingEnabled) {
    schedulePolling(false);
  }
});
onUnmounted(() => {
  stopPollingTimers();
  stopNowTicker();
});
</script>

<template>
  <PageShell
    title="任务"
    subtitle="可配置自动/手动轮询生成队列状态"
    max-width="2xl"
    :loading="loading"
    loading-text="加载中"
  >
    <template #actions>
      <div class="flex flex-wrap items-center gap-3">
        <label class="label cursor-pointer gap-2">
          <span class="label-text text-sm">自动轮询</span>
          <input
            v-model="pollingEnabled"
            type="checkbox"
            class="toggle toggle-primary"
            aria-label="切换自动轮询"
          />
        </label>

        <fieldset class="fieldset w-auto flex items-center join">
          <label class="input h-8">
            <input
              v-model.number="intervalSeconds"
              type="number"
              min="0.5"
              step="0.5"
              class="w-12"
              aria-label="轮询间隔（秒）"
            />
            <span>秒</span>
          </label>
          <button
            class="btn btn-outline btn-sm join-item"
            @click="applyInterval"
          >
            应用
          </button>
          <button class="btn btn-primary btn-sm" @click="refreshNow">
            {{ pollingEnabled ? "立即刷新" : "手动查询" }}
          </button>
        </fieldset>
      </div>
    </template>

    <div class="grid gap-4">
      <div
        v-if="pollingEnabled"
        class="rounded-xl border border-primary/20 bg-primary/5 p-4"
      >
        <div class="flex items-center justify-between text-sm">
          <div class="flex items-center gap-2">
            <span class="font-medium">自动轮询中</span>
            <span class="opacity-70">
              每 {{ (jobs.pollingIntervalMs / 1000).toFixed(1) }} 秒
            </span>
          </div>
          <span class="text-xs opacity-70">
            下一次进度 {{ pollingProgress.toFixed(0) }}%
          </span>
        </div>
        <progress
          class="progress progress-primary mt-2 h-2"
          :value="pollingProgress"
          max="100"
        ></progress>
      </div>

      <div v-else class="alert alert-warning">
        <div>自动轮询已关闭。</div>
        <div class="text-xs opacity-70">
          需要时点击“手动查询”或重新打开开关。
        </div>
      </div>

      <div v-if="!jobs.sorted.length" class="text-sm opacity-70">
        暂无任务（在生成页面点“提交任务”）
      </div>

      <div
        v-for="j in jobs.sorted"
        :key="j.id"
        class="rounded-xl border border-base-300 bg-base-200/80 p-4"
      >
        <div class="flex flex-wrap items-center justify-between gap-2">
          <div class="font-mono text-sm break-all">{{ j.id }}</div>
          <div class="badge badge-neutral">{{ j.kind }}</div>
        </div>

        <div class="mt-2 text-sm">
          <span class="opacity-70">status：</span>
          <span>{{ j.status?.status ?? "unknown" }}</span>
        </div>

        <div
          v-if="jobTimings.get(j.id)"
          class="mt-2 flex flex-wrap gap-3 text-xs opacity-80"
        >
          <span>
            排队 {{ formatDuration(jobTimings.get(j.id)?.queueMs ?? 0) }}
          </span>
          <span>
            运行 {{ formatDuration(jobTimings.get(j.id)?.runMs ?? 0) }}
          </span>
          <span v-if="jobTimings.get(j.id)?.finished">
            总计 {{ formatDuration(jobTimings.get(j.id)?.totalMs ?? 0) }}
          </span>
          <span v-else>
            累计 {{ formatDuration(jobTimings.get(j.id)?.totalMs ?? 0) }}
          </span>
        </div>

        <div v-if="j.lastError" class="mt-2 alert alert-error">
          <span>{{ j.lastError }}</span>
        </div>

        <div
          v-if="j.status?.status === 'failed'"
          class="mt-2 alert alert-error"
        >
          <span>{{ j.status.error }}</span>
        </div>

        <div v-if="j.status?.status === 'succeeded'" class="mt-3 grid gap-2">
          <div class="text-sm opacity-80">
            outputs: {{ j.status.outputs.length }}（老图请到输出页查看）
          </div>
          <RouterLink class="btn btn-sm" to="/outputs">打开输出</RouterLink>
        </div>
      </div>
    </div>
  </PageShell>
</template>
