<script setup lang="ts">
import { onMounted, ref } from "vue";
import { RouterLink } from "vue-router";
import { endpoints } from "@/api/endpoints";
import type { Anlas, Health } from "@/api/types";
import { useMetaStore } from "@/stores/meta";
import PageShell from "@/components/layout/PageShell.vue";

const metaStore = useMetaStore();

const health = ref<Health | null>(null);
const anlas = ref<Anlas | null>(null);
const errorText = ref<string>("");

async function refresh() {
  errorText.value = "";
  try {
    await metaStore.ensureLoaded();
    health.value = await endpoints.health();
    anlas.value = await endpoints.anlas();
  } catch (e) {
    errorText.value = e instanceof Error ? e.message : String(e);
  }
}

onMounted(() => {
  void refresh();
});
</script>

<template>
  <PageShell title="控制台" subtitle="后端状态与快速入口" max-width="lg">
    <template #actions>
      <button class="btn btn-primary" @click="refresh">刷新</button>
    </template>

    <div class="grid gap-6 lg:grid-cols-[1.3fr,1fr]">
      <div class="space-y-4">
        <div
          class="rounded-xl border border-base-300 bg-base-200/60 p-4 shadow-sm"
        >
          <div class="flex items-center justify-between">
            <div>
              <p class="text-xs uppercase tracking-[0.2em] text-primary">
                Backend
              </p>
              <h2 class="text-lg font-semibold">运行状态</h2>
            </div>
            <div class="badge badge-neutral text-xs">/api</div>
          </div>

          <div v-if="errorText" class="mt-3 alert alert-error">
            <span>{{ errorText }}</span>
          </div>

          <div class="mt-4 grid gap-3 md:grid-cols-3">
            <div
              class="rounded-lg border border-base-300/70 bg-base-100/80 p-3"
            >
              <div class="text-xs uppercase tracking-[0.2em] text-primary">
                健康
              </div>
              <div class="mt-1 text-xl font-semibold">
                {{ health?.ok ? "OK" : "—" }}
              </div>
              <div class="text-xs opacity-70">/api/health</div>
            </div>
            <div
              class="rounded-lg border border-base-300/70 bg-base-100/80 p-3"
            >
              <div class="text-xs uppercase tracking-[0.2em] text-primary">
                Anlas
              </div>
              <div class="mt-1 text-xl font-semibold">
                {{ anlas?.anlas ?? "—" }}
              </div>
              <div class="text-xs opacity-70">/api/anlas</div>
            </div>
            <div
              class="rounded-lg border border-base-300/70 bg-base-100/80 p-3"
            >
              <div class="text-xs uppercase tracking-[0.2em] text-primary">
                模型
              </div>
              <div class="mt-1 text-xl font-semibold">
                {{ metaStore.models.length }}
              </div>
              <div class="text-xs opacity-70">/api/meta</div>
            </div>
          </div>

          <div class="mt-4 grid gap-2 text-sm opacity-80">
            <div>
              已对接页面：生成（t2i/i2i/inpaint/character）、导演、任务、输出列表
            </div>
            <div>开发期：Vite 代理 /api 与 /outputs 到后端</div>
          </div>
        </div>
      </div>

      <div class="grid gap-4">
        <div
          class="rounded-xl border border-base-300 bg-base-100/80 p-4 shadow"
        >
          <div class="flex items-center justify-between">
            <h3 class="text-base font-semibold">快速入口</h3>
            <div class="badge badge-primary badge-outline">导航</div>
          </div>
          <div class="mt-3 grid gap-3">
            <RouterLink
              class="btn btn-outline justify-start"
              to="/generate/t2i"
            >
              生成中心（t2i/i2i/inpaint/character）
            </RouterLink>
            <RouterLink
              class="btn btn-outline justify-start"
              to="/presets/generate"
            >
              预设管理（生成 / 提示词 / 角色）
            </RouterLink>
            <RouterLink class="btn btn-outline justify-start" to="/director">
              导演（增强）与输出列表
            </RouterLink>
          </div>
        </div>

        <div
          class="rounded-xl border border-base-300 bg-base-100/80 p-4 shadow"
        >
          <h3 class="text-base font-semibold">提示</h3>
          <div class="mt-2 text-sm opacity-80">
            生成任务提交后自动跳转至“任务”页可随时刷新状态；输出会在“输出列表”按类型与日期聚合。
          </div>
        </div>
      </div>
    </div>
  </PageShell>
</template>
