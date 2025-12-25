import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { endpoints } from "@/api/endpoints";
import type { JobStatus } from "@/api/types";

const DEFAULT_POLL_INTERVAL_MS = 2000;
const SETTINGS_STORAGE_KEY = "nai-ui-jobs-settings";

type StoredPollingSettings = {
  enabled: boolean;
  intervalMs: number;
};

function loadPollingSettings(): StoredPollingSettings {
  if (typeof window === "undefined") {
    return { enabled: true, intervalMs: DEFAULT_POLL_INTERVAL_MS };
  }

  const raw = window.localStorage.getItem(SETTINGS_STORAGE_KEY);
  if (!raw) {
    return { enabled: true, intervalMs: DEFAULT_POLL_INTERVAL_MS };
  }

  try {
    const parsed = JSON.parse(raw) as Partial<StoredPollingSettings>;
    if (
      typeof parsed.enabled === "boolean" &&
      typeof parsed.intervalMs === "number" &&
      parsed.intervalMs > 0
    ) {
      return parsed as StoredPollingSettings;
    }
  } catch (_e) {
    // Ignore corrupted storage and fall back to defaults.
  }

  return { enabled: true, intervalMs: DEFAULT_POLL_INTERVAL_MS };
}

function persistPollingSettings(settings: StoredPollingSettings) {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(settings));
}

export type TrackedJob = {
  id: string;
  kind: string;
  created_at_ms?: number;
  started_at_ms?: number | null;
  finished_at_ms?: number | null;
  updated_at_ms?: number;
  status?: JobStatus;
  lastError?: string;
};

export const useJobsStore = defineStore("jobs", () => {
  const jobs = ref<TrackedJob[]>([]);

  const initialSettings = loadPollingSettings();
  const pollingEnabled = ref(initialSettings.enabled);
  const pollingIntervalMs = ref(initialSettings.intervalMs);

  const sorted = computed(() => jobs.value.slice().reverse());

  function track(id: string, kind: string) {
    if (jobs.value.some((j) => j.id === id)) return;
    jobs.value.push({ id, kind });
  }

  async function refresh(id: string) {
    const job = jobs.value.find((j) => j.id === id);
    if (!job) return;
    try {
      job.status = await endpoints.jobStatus(id);
      job.lastError = undefined;
    } catch (e) {
      job.lastError = e instanceof Error ? e.message : String(e);
    }
  }

  async function refreshAll() {
    try {
      const list = await endpoints.jobsList();
      const prev = new Map(jobs.value.map((j) => [j.id, j] as const));
      jobs.value = list.items.map((it) => {
        const old = prev.get(it.id);
        return {
          id: it.id,
          kind: it.kind,
          created_at_ms: it.created_at_ms,
          started_at_ms: it.started_at_ms,
          finished_at_ms: it.finished_at_ms,
          updated_at_ms: it.updated_at_ms,
          status: it.status,
          lastError: old?.lastError,
        };
      });
    } catch (e) {
      // Fallback: refresh known ids one-by-one.
      await Promise.all(jobs.value.map((j) => refresh(j.id)));
    }
  }

  function setPollingEnabled(enabled: boolean) {
    pollingEnabled.value = enabled;
    persistPollingSettings({
      enabled: pollingEnabled.value,
      intervalMs: pollingIntervalMs.value,
    });
  }

  function setPollingIntervalMs(intervalMs: number) {
    const clamped = Math.max(500, Math.round(intervalMs));
    pollingIntervalMs.value = clamped;
    persistPollingSettings({
      enabled: pollingEnabled.value,
      intervalMs: pollingIntervalMs.value,
    });
  }

  return {
    jobs,
    sorted,
    track,
    refresh,
    refreshAll,
    pollingEnabled,
    pollingIntervalMs,
    setPollingEnabled,
    setPollingIntervalMs,
  };
});
