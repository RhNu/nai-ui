<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import PageShell from "@/components/layout/PageShell.vue";
import { endpoints } from "@/api/endpoints";
import { outputsUrl } from "@/components/urls";
import type { OutputItem } from "@/api/types";

const loading = ref(false);
const errorText = ref("");
const items = ref<OutputItem[]>([]);
const selected = ref<Record<string, boolean>>({});

const selectedPaths = computed(() =>
  Object.entries(selected.value)
    .filter(([, v]) => !!v)
    .map(([k]) => k)
);

const groups = computed(() => {
  const out: Array<{ key: string; title: string; items: OutputItem[] }> = [];
  const idx = new Map<string, number>();
  for (const it of items.value) {
    const key = `${it.op_type}__${it.date}`;
    const title = `${it.op_type} - ${it.date}`;
    const at = idx.get(key);
    if (at === undefined) {
      idx.set(key, out.length);
      out.push({ key, title, items: [it] });
    } else {
      out[at].items.push(it);
    }
  }
  return out;
});

async function refresh() {
  loading.value = true;
  errorText.value = "";
  try {
    const r = await endpoints.outputs();
    items.value = r.items;
  } catch (e) {
    errorText.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

function clearSelection() {
  selected.value = {};
}

async function deleteSelected() {
  const paths = selectedPaths.value;
  if (!paths.length) return;
  loading.value = true;
  errorText.value = "";
  try {
    await endpoints.outputsDelete({ items: paths });
    clearSelection();
    await refresh();
  } catch (e) {
    errorText.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  void refresh();
});
</script>

<template>
  <PageShell
    title="输出列表"
    subtitle="按操作类型与日期聚合的生成结果"
    max-width="2xl"
  >
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div class="flex items-center gap-2">
        <div class="text-sm opacity-70" v-if="selectedPaths.length">
          已选 {{ selectedPaths.length }}
        </div>
        <button class="btn" @click="clearSelection">清空选择</button>
        <button
          class="btn btn-error"
          :class="{ 'btn-disabled': !selectedPaths.length || loading }"
          @click="deleteSelected"
        >
          删除所选
        </button>
      </div>
      <button
        class="btn btn-primary"
        :class="{ 'btn-disabled': loading }"
        @click="refresh"
      >
        刷新
      </button>
    </div>

    <div v-if="errorText" class="alert alert-error">
      <span>{{ errorText }}</span>
    </div>

    <div v-if="loading" class="flex justify-center py-10">
      <div class="loading loading-spinner loading-lg" />
    </div>

    <div v-else-if="items.length" class="grid gap-6">
      <div v-for="g in groups" :key="g.key" class="grid gap-3">
        <div class="font-medium">{{ g.title }}</div>
        <div class="grid grid-cols-2 gap-3 md:grid-cols-3 lg:grid-cols-4">
          <div v-for="it in g.items" :key="it.path" class="relative">
            <input
              class="checkbox checkbox-sm absolute left-2 top-2 z-10"
              type="checkbox"
              v-model="selected[it.path]"
              @click.stop
            />
            <a
              class="block"
              :href="outputsUrl(it.path)"
              target="_blank"
              rel="noreferrer"
            >
              <img
                class="rounded bg-base-200"
                :src="outputsUrl(it.path)"
                loading="lazy"
              />
              <div class="mt-1 text-xs opacity-70 break-all">
                {{ it.filename }}
              </div>
            </a>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="text-sm opacity-70">暂无输出</div>
  </PageShell>
</template>
