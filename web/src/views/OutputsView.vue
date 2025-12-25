<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useEventListener, useIntersectionObserver } from "@vueuse/core";
import PageShell from "@/components/layout/PageShell.vue";
import { endpoints } from "@/api/endpoints";
import { outputsUrl } from "@/components/urls";
import type { OutputItem } from "@/api/types";

const loading = ref(false);
const loadingMore = ref(false);
const errorText = ref("");
const items = ref<OutputItem[]>([]);
const selected = ref<Record<string, boolean>>({});
const offset = ref(0);
const hasMore = ref(true);
const loadMoreEl = ref<HTMLElement | null>(null);
const copyingPath = ref<string | null>(null);
const downloadingPath = ref<string | null>(null);
const pageSize = 60;
const openAction = ref<string | null>(null);
const activeTab = ref("");
const previewItem = ref<OutputItem | null>(null);

const selectedPaths = computed(() =>
  Object.entries(selected.value)
    .filter(([, v]) => !!v)
    .map(([k]) => k)
);

const categories = computed(() => {
  const set = new Set(items.value.map((it) => it.op_type));
  return Array.from(set).sort();
});

watch(categories, (cats) => {
  if (!cats.length) {
    activeTab.value = "";
    return;
  }
  if (!cats.includes(activeTab.value)) {
    activeTab.value = cats[0];
  }
});

function parseIndex(name: string): number {
  // 仅按文件名末尾数字排序，老格式无编号则置为 -1
  const match = name.match(/(\d+)(?=\.[^.]+$)/);
  if (!match) return -1;
  const n = Number.parseInt(match[1], 10);
  return Number.isNaN(n) ? -1 : n;
}

const tabItems = computed(() => {
  const filtered = items.value.filter(
    (it) => !activeTab.value || it.op_type === activeTab.value
  );
  return filtered
    .slice()
    .sort((a, b) => parseIndex(b.filename) - parseIndex(a.filename))
    .sort((a, b) => b.date.localeCompare(a.date));
});

const dateGroups = computed(() => {
  const grouped = new Map<string, OutputItem[]>();
  for (const it of tabItems.value) {
    const list = grouped.get(it.date) ?? [];
    list.push(it);
    grouped.set(it.date, list);
  }
  return Array.from(grouped.entries())
    .sort((a, b) => b[0].localeCompare(a[0]))
    .map(([date, list]) => ({ date, items: list }));
});

function clearSelection() {
  selected.value = {};
  openAction.value = null;
}

const { stop: stopObserver } = useIntersectionObserver(
  loadMoreEl,
  (entries) => {
    if (entries.some((e) => e.isIntersecting)) {
      void loadMore();
    }
  },
  { rootMargin: "300px" }
);

onUnmounted(() => {
  stopObserver();
});

async function loadMore(force = false) {
  if ((loading.value || loadingMore.value) && !force) return;
  if (!hasMore.value && !force) return;

  const isFirstPage = offset.value === 0;
  if (isFirstPage) {
    loading.value = true;
  } else {
    loadingMore.value = true;
  }
  errorText.value = "";

  try {
    const r = await endpoints.outputs({
      offset: offset.value,
      limit: pageSize,
    });
    const incoming = r.items;
    if (isFirstPage) {
      items.value = incoming;
    } else {
      const existing = new Set(items.value.map((it) => it.path));
      items.value.push(...incoming.filter((it) => !existing.has(it.path)));
    }
    offset.value = r.next_offset;
    hasMore.value = r.has_more;
  } catch (e) {
    errorText.value = e instanceof Error ? e.message : String(e);
    hasMore.value = false;
  } finally {
    if (isFirstPage) {
      loading.value = false;
    } else {
      loadingMore.value = false;
    }
  }
}

async function refresh() {
  clearSelection();
  offset.value = 0;
  hasMore.value = true;
  loading.value = false;
  loadingMore.value = false;
  await loadMore(true);
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

async function copyBlobToClipboard(blob: Blob) {
  if (!navigator.clipboard || !("write" in navigator.clipboard)) {
    throw new Error("当前浏览器不支持剪贴板复制");
  }
  await navigator.clipboard.write([
    new ClipboardItem({
      [blob.type]: blob,
    }),
  ]);
}

async function fetchImageBlob(path: string): Promise<Blob> {
  const resp = await fetch(outputsUrl(path));
  if (!resp.ok) {
    throw new Error(`获取图片失败：${resp.status}`);
  }
  return await resp.blob();
}

async function stripMetadata(blob: Blob): Promise<Blob> {
  const bitmap = await createImageBitmap(blob);
  const canvas = document.createElement("canvas");
  canvas.width = bitmap.width;
  canvas.height = bitmap.height;
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    bitmap.close();
    throw new Error("无法创建绘图上下文");
  }
  ctx.drawImage(bitmap, 0, 0);
  bitmap.close();

  const stripped = await new Promise<Blob>((resolve, reject) => {
    canvas.toBlob((b) => {
      if (b) resolve(b);
      else reject(new Error("重编码失败"));
    }, "image/png");
  });
  return stripped;
}

async function copyOriginal(path: string) {
  copyingPath.value = path;
  errorText.value = "";
  try {
    const blob = await fetchImageBlob(path);
    await copyBlobToClipboard(blob);
  } catch (e) {
    errorText.value = e instanceof Error ? e.message : String(e);
  } finally {
    copyingPath.value = null;
  }
}

async function copyStripped(path: string) {
  copyingPath.value = path;
  errorText.value = "";
  try {
    const blob = await fetchImageBlob(path);
    const clean = await stripMetadata(blob);
    await copyBlobToClipboard(clean);
  } catch (e) {
    errorText.value = e instanceof Error ? e.message : String(e);
  } finally {
    copyingPath.value = null;
  }
}

function filenameFromPath(path: string): string {
  const parts = path.split("/");
  return parts[parts.length - 1] || "image.png";
}

async function download(path: string, strip: boolean) {
  downloadingPath.value = path;
  errorText.value = "";
  try {
    const blob = await fetchImageBlob(path);
    const finalBlob = strip ? await stripMetadata(blob) : blob;
    const url = URL.createObjectURL(finalBlob);
    const a = document.createElement("a");
    const base = filenameFromPath(path);
    const name = strip ? base.replace(/\.png$/i, "") + "_clean.png" : base;
    a.href = url;
    a.download = name;
    document.body.appendChild(a);
    a.click();
    a.remove();
    URL.revokeObjectURL(url);
  } catch (e) {
    errorText.value = e instanceof Error ? e.message : String(e);
  } finally {
    downloadingPath.value = null;
  }
}

function openPreview(it: OutputItem) {
  previewItem.value = it;
}

function closePreview() {
  previewItem.value = null;
}

const previewList = computed(() => tabItems.value);

function movePreview(delta: number) {
  if (!previewItem.value) return;
  const list = previewList.value;
  const idx = list.findIndex((x) => x.path === previewItem.value?.path);
  if (idx === -1) return;
  const nextIdx = idx + delta;
  if (nextIdx < 0 || nextIdx >= list.length) return;
  previewItem.value = list[nextIdx];
}

useEventListener(window, "keydown", (e: KeyboardEvent) => {
  if (!previewItem.value) return;
  if (e.key === "Escape") {
    e.preventDefault();
    closePreview();
  } else if (e.key === "ArrowLeft") {
    e.preventDefault();
    movePreview(-1);
  } else if (e.key === "ArrowRight") {
    e.preventDefault();
    movePreview(1);
  }
});

function toggleActions(path: string) {
  openAction.value = openAction.value === path ? null : path;
}

onMounted(refresh);

watch(items, () => {
  if (
    openAction.value &&
    !items.value.some((it) => it.path === openAction.value)
  ) {
    openAction.value = null;
  }
});
</script>

<template>
  <PageShell
    title="输出列表"
    subtitle="按类型分栏，按时间倒序查看"
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

    <div
      v-if="categories.length"
      class="tabs tabs-boxed mt-4 w-full overflow-x-auto"
    >
      <button
        v-for="cat in categories"
        :key="cat"
        class="tab"
        :class="{ 'tab-active': cat === activeTab }"
        @click="activeTab = cat"
      >
        {{ cat }}
      </button>
    </div>

    <div v-if="loading && offset === 0" class="flex justify-center py-10">
      <div class="loading loading-spinner loading-lg" />
    </div>

    <div v-else-if="dateGroups.length" class="grid gap-6">
      <div v-for="g in dateGroups" :key="g.date" class="grid gap-3">
        <div class="font-medium">{{ g.date }}</div>
        <div class="grid grid-cols-2 gap-3 md:grid-cols-3 lg:grid-cols-4">
          <div
            v-for="it in g.items"
            :key="it.path"
            class="relative rounded-lg border border-base-300 bg-base-200/60 p-2"
          >
            <div
              class="thumb-wrap"
              :class="{ 'action-open': openAction === it.path }"
            >
              <button class="block w-full text-left" @click="openPreview(it)">
                <img
                  class="thumb-image w-full rounded bg-base-200 object-cover"
                  :class="{ shrink: openAction === it.path }"
                  :src="outputsUrl(it.path)"
                  loading="lazy"
                />
                <div class="mt-1 break-all text-xs opacity-70">
                  {{ it.filename }}
                </div>
              </button>

              <transition name="fade">
                <div v-if="openAction === it.path" class="action-panel">
                  <button
                    class="btn btn-outline btn-xs"
                    :class="{ 'btn-disabled': copyingPath === it.path }"
                    @click.prevent.stop="copyOriginal(it.path)"
                  >
                    {{ copyingPath === it.path ? "复制中..." : "复制原图" }}
                  </button>
                  <button
                    class="btn btn-outline btn-xs"
                    :class="{ 'btn-disabled': copyingPath === it.path }"
                    @click.prevent.stop="copyStripped(it.path)"
                  >
                    {{ copyingPath === it.path ? "复制中..." : "去元数据复制" }}
                  </button>
                  <button
                    class="btn btn-outline btn-xs"
                    :class="{ 'btn-disabled': downloadingPath === it.path }"
                    @click.prevent.stop="download(it.path, false)"
                  >
                    {{ downloadingPath === it.path ? "下载中..." : "下载原图" }}
                  </button>
                  <button
                    class="btn btn-outline btn-xs"
                    :class="{ 'btn-disabled': downloadingPath === it.path }"
                    @click.prevent.stop="download(it.path, true)"
                  >
                    {{
                      downloadingPath === it.path ? "下载中..." : "去元数据下载"
                    }}
                  </button>
                </div>
              </transition>
            </div>

            <div class="mt-2 flex items-center justify-between gap-2 text-xs">
              <label class="flex items-center gap-2">
                <input
                  class="checkbox checkbox-sm"
                  type="checkbox"
                  v-model="selected[it.path]"
                  @click.stop
                />
                <span class="truncate opacity-70">{{ it.op_type }}</span>
              </label>
              <button
                class="btn btn-ghost btn-xs"
                @click.stop="toggleActions(it.path)"
              >
                {{ openAction === it.path ? "收起" : "操作" }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="text-sm opacity-70">暂无输出</div>

    <div ref="loadMoreEl" class="h-4"></div>

    <div v-if="loadingMore" class="flex justify-center py-4">
      <div class="loading loading-spinner loading-md" />
    </div>

    <div v-else-if="hasMore && items.length" class="flex justify-center py-4">
      <button class="btn btn-outline" @click="loadMore()">继续加载</button>
    </div>

    <teleport to="body">
      <div
        v-if="previewItem"
        class="fixed inset-0 z-9999 flex items-center justify-center bg-black/80 backdrop-blur-sm px-2 sm:px-4"
        @click.self="closePreview"
      >
        <div
          class="relative max-h-[95vh] w-full max-w-6xl rounded-xl bg-base-100 p-4 shadow-2xl"
        >
          <div class="flex flex-wrap items-center justify-between gap-2">
            <div class="flex flex-col text-sm opacity-80">
              <span class="font-medium">{{ previewItem.filename }}</span>
              <span class="truncate"
                >{{ previewItem.op_type }} · {{ previewItem.date }}</span
              >
            </div>
            <div class="flex flex-wrap gap-2 text-sm">
              <a
                class="btn btn-ghost btn-sm"
                :href="outputsUrl(previewItem.path)"
                target="_blank"
                rel="noreferrer"
              >
                源链接
              </a>
              <button
                class="btn btn-outline btn-sm"
                :class="{ 'btn-disabled': copyingPath === previewItem.path }"
                @click.prevent="copyOriginal(previewItem.path)"
              >
                {{
                  copyingPath === previewItem.path ? "复制中..." : "复制原图"
                }}
              </button>
              <button
                class="btn btn-outline btn-sm"
                :class="{ 'btn-disabled': copyingPath === previewItem.path }"
                @click.prevent="copyStripped(previewItem.path)"
              >
                {{
                  copyingPath === previewItem.path
                    ? "复制中..."
                    : "去元数据复制"
                }}
              </button>
              <button
                class="btn btn-primary btn-sm"
                :class="{
                  'btn-disabled': downloadingPath === previewItem.path,
                }"
                @click.prevent="download(previewItem.path, false)"
              >
                {{
                  downloadingPath === previewItem.path
                    ? "下载中..."
                    : "下载原图"
                }}
              </button>
              <button
                class="btn btn-primary btn-sm"
                :class="{
                  'btn-disabled': downloadingPath === previewItem.path,
                }"
                @click.prevent="download(previewItem.path, true)"
              >
                {{
                  downloadingPath === previewItem.path
                    ? "下载中..."
                    : "去元数据下载"
                }}
              </button>
              <button class="btn btn-ghost btn-sm" @click="closePreview">
                关闭
              </button>
            </div>
          </div>

          <div class="mt-4 flex justify-center overflow-hidden rounded-lg">
            <img
              class="max-h-[82vh] w-full max-w-6xl object-contain"
              :src="outputsUrl(previewItem.path)"
              :alt="previewItem.filename"
            />
          </div>

          <div
            class="pointer-events-none absolute inset-x-0 bottom-0 h-16 bg-linear-to-t from-base-100/95 to-transparent"
          ></div>

          <div class="mt-4 flex items-center justify-between gap-2 text-sm">
            <span class="opacity-70">方向键切换；Esc 关闭</span>
            <div class="flex gap-2">
              <button
                class="btn btn-outline btn-sm"
                @click.stop="movePreview(-1)"
              >
                上一张
              </button>
              <button
                class="btn btn-outline btn-sm"
                @click.stop="movePreview(1)"
              >
                下一张
              </button>
            </div>
          </div>
        </div>
      </div>
    </teleport>
  </PageShell>
</template>

<style scoped>
.thumb {
  height: 220px;
}

@media (min-width: 768px) {
  .thumb {
    height: 260px;
  }
}

@media (min-width: 1024px) {
  .thumb {
    height: 280px;
  }
}

.thumb-wrap {
  position: relative;
  min-height: 260px;
}

.thumb-image {
  height: 240px;
  transition: height 0.2s ease;
}

.thumb-image.shrink {
  height: 220px;
}

.action-panel {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 10px;
  background: linear-gradient(
    180deg,
    rgba(0, 0, 0, 0) 0%,
    rgba(0, 0, 0, 0.65) 70%
  );
  backdrop-filter: blur(2px);
  border-radius: 10px;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
