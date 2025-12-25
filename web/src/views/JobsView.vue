<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { RouterLink } from "vue-router";
import PageShell from "@/components/layout/PageShell.vue";
import { useJobsStore } from "@/stores/jobs";

const jobs = useJobsStore();

const manualId = ref("");
const intervalSeconds = ref(jobs.pollingIntervalMs / 1000);
const pollingProgress = ref(0);

let pollTimer: number | undefined;
let progressRaf: number | undefined;
let nextRefreshAt: number | null = null;

const pollingEnabled = computed({
  get: () => jobs.pollingEnabled,
  set: (value: boolean) => jobs.setPollingEnabled(value),
});

async function refreshAll() {
  await jobs.refreshAll();
}

function trackManual() {
  const id = manualId.value.trim();
  if (!id) return;
  jobs.track(id, "manual");
  manualId.value = "";
}

function stopProgressLoop() {
  if (progressRaf) {
    window.cancelAnimationFrame(progressRaf);
    progressRaf = undefined;
  }
  pollingProgress.value = 0;
}

function startProgressLoop() {
  stopProgressLoop();
  const loop = () => {
    if (!jobs.pollingEnabled || nextRefreshAt === null) {
      pollingProgress.value = 0;
      progressRaf = undefined;
      return;
    }

    const total = jobs.pollingIntervalMs;
    const remaining = Math.max(0, nextRefreshAt - performance.now());
    const elapsed = total - remaining;
    const percent =
      total <= 0 ? 100 : Math.min(100, Math.max(0, (elapsed / total) * 100));
    pollingProgress.value = percent;
    progressRaf = window.requestAnimationFrame(loop);
  };

  progressRaf = window.requestAnimationFrame(loop);
}

function stopPollingTimers() {
  if (pollTimer) {
    window.clearTimeout(pollTimer);
    pollTimer = undefined;
  }

  nextRefreshAt = null;
  stopProgressLoop();
}

function schedulePolling(runImmediateRefresh: boolean) {
  stopPollingTimers();
  if (!jobs.pollingEnabled) return;

  const interval = jobs.pollingIntervalMs;

  const startNextCycle = () => {
    nextRefreshAt = performance.now() + interval;
    startProgressLoop();
    pollTimer = window.setTimeout(async () => {
      await refreshAll();
      startNextCycle();
    }, interval);
  };

  if (runImmediateRefresh) {
    void refreshAll().finally(startNextCycle);
  } else {
    startNextCycle();
  }
}

async function refreshNow() {
  stopPollingTimers();
  await refreshAll();
  if (jobs.pollingEnabled) {
    schedulePolling(false);
  }
}

function applyInterval() {
  const ms = Math.max(500, Math.round(intervalSeconds.value * 1000));
  jobs.setPollingIntervalMs(ms);
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
  void refreshAll();
  if (jobs.pollingEnabled) {
    schedulePolling(false);
  }
});

onUnmounted(() => {
  stopPollingTimers();
});
</script>

<template>
  <PageShell
    title="任务"
    subtitle="可配置自动/手动轮询生成队列状态"
    max-width="2xl"
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

        <div class="join">
          <input
            v-model.number="intervalSeconds"
            type="number"
            min="0.5"
            step="0.5"
            class="input input-bordered join-item w-24"
            aria-label="轮询间隔（秒）"
          />
          <span class="btn btn-ghost btn-sm join-item no-animation">秒</span>
          <button
            class="btn btn-outline btn-sm join-item"
            @click="applyInterval"
          >
            应用
          </button>
        </div>

        <button class="btn btn-primary btn-sm" @click="refreshNow">
          {{ pollingEnabled ? "立即刷新" : "手动查询" }}
        </button>
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

      <div class="join">
        <input
          v-model="manualId"
          class="input input-bordered join-item w-full"
          placeholder="输入 job id 以追踪"
        />
        <button class="btn join-item" @click="trackManual">添加</button>
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

      <div class="alert alert-info">
        <span>根据上方设置自动轮询，或关闭后手动查询任务状态。</span>
      </div>
    </div>
  </PageShell>
</template>
