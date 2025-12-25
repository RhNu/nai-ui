<script setup lang="ts">
import { computed, ref } from "vue";
import PageShell from "@/components/layout/PageShell.vue";
import { endpoints } from "@/api/endpoints";
import type { DirectorResponse } from "@/api/types";
import { fileToBase64Payload } from "@/components/fileBase64";
import { outputsUrl } from "@/components/urls";

type Mode =
  | "remove_bg"
  | "line_art"
  | "sketch"
  | "declutter"
  | "colorize"
  | "emotion";

const mode = ref<Mode>("remove_bg");
const width = ref(832);
const height = ref(1216);
const prompt = ref("");
const defry = ref(0);

const imageFile = ref<File | null>(null);
const loading = ref(false);
const errorText = ref("");
const resp = ref<DirectorResponse | null>(null);

const needsPrompt = computed(
  () => mode.value === "colorize" || mode.value === "emotion"
);

function onPick(ev: Event) {
  const input = ev.target as HTMLInputElement;
  imageFile.value = input.files?.[0] ?? null;
}

async function run() {
  if (!imageFile.value) {
    errorText.value = "请选择图片";
    return;
  }
  loading.value = true;
  errorText.value = "";
  resp.value = null;
  try {
    const image_base64 = await fileToBase64Payload(imageFile.value);

    if (mode.value === "colorize") {
      resp.value = await endpoints.directorColorize({
        width: width.value,
        height: height.value,
        image_base64,
        prompt: prompt.value,
        defry: defry.value,
      });
    } else if (mode.value === "emotion") {
      resp.value = await endpoints.directorEmotion({
        width: width.value,
        height: height.value,
        image_base64,
        prompt: prompt.value,
        defry: defry.value,
      });
    } else if (mode.value === "remove_bg") {
      resp.value = await endpoints.directorRemoveBg({
        width: width.value,
        height: height.value,
        image_base64,
      });
    } else if (mode.value === "line_art") {
      resp.value = await endpoints.directorLineArt({
        width: width.value,
        height: height.value,
        image_base64,
      });
    } else if (mode.value === "sketch") {
      resp.value = await endpoints.directorSketch({
        width: width.value,
        height: height.value,
        image_base64,
      });
    } else if (mode.value === "declutter") {
      resp.value = await endpoints.directorDeclutter({
        width: width.value,
        height: height.value,
        image_base64,
      });
    }
  } catch (e) {
    errorText.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <PageShell
    title="导演"
    subtitle="图像增强 / 处理（augment-image）"
    max-width="2xl"
  >
    <template #actions>
      <RouterLink class="btn btn-outline btn-sm" to="/outputs"
        >查看输出</RouterLink
      >
    </template>

    <div v-if="errorText" class="alert alert-error">
      <span>{{ errorText }}</span>
    </div>

    <div class="grid gap-6 lg:grid-cols-2">
      <div
        class="grid gap-3 rounded-xl border border-base-300/70 bg-base-200/60 p-4"
      >
        <label class="form-control">
          <div class="label"><span class="label-text">模式</span></div>
          <select v-model="mode" class="select select-bordered">
            <option value="remove_bg">remove_bg (bg-removal)</option>
            <option value="line_art">line_art</option>
            <option value="sketch">sketch</option>
            <option value="declutter">declutter</option>
            <option value="colorize">colorize (prompt+defry)</option>
            <option value="emotion">emotion (prompt+defry)</option>
          </select>
        </label>

        <label class="form-control">
          <div class="label"><span class="label-text">图片</span></div>
          <input
            class="file-input file-input-bordered"
            type="file"
            accept="image/*"
            @change="onPick"
          />
        </label>

        <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
          <label class="form-control">
            <div class="label"><span class="label-text">宽</span></div>
            <input
              v-model.number="width"
              class="input input-bordered"
              type="number"
              min="64"
              step="64"
            />
          </label>
          <label class="form-control">
            <div class="label"><span class="label-text">高</span></div>
            <input
              v-model.number="height"
              class="input input-bordered"
              type="number"
              min="64"
              step="64"
            />
          </label>
        </div>

        <div v-if="needsPrompt" class="grid gap-3">
          <label class="form-control">
            <div class="label"><span class="label-text">prompt</span></div>
            <textarea
              v-model="prompt"
              class="textarea textarea-bordered h-24"
            />
          </label>
          <label class="form-control">
            <div class="label"><span class="label-text">defry</span></div>
            <input
              v-model.number="defry"
              class="input input-bordered"
              type="number"
              step="1"
            />
          </label>
        </div>

        <button
          class="btn btn-primary"
          :class="{ 'btn-disabled': loading }"
          type="button"
          @click="run"
        >
          运行
        </button>
      </div>

      <div class="grid gap-4">
        <div
          class="rounded-xl border border-base-300 bg-base-100/80 p-4 shadow"
        >
          <div class="flex items-center justify-between">
            <div class="font-medium">输出</div>
            <div v-if="loading" class="loading loading-spinner" />
          </div>

          <div
            v-if="resp?.output_paths?.length"
            class="mt-3 grid grid-cols-2 gap-3"
          >
            <a
              v-for="p in resp.output_paths"
              :key="p"
              class="block"
              :href="outputsUrl(p)"
              target="_blank"
              rel="noreferrer"
            >
              <img class="rounded bg-base-200" :src="outputsUrl(p)" />
              <div class="mt-1 text-xs opacity-70 break-all">{{ p }}</div>
            </a>
          </div>
          <div v-else class="text-sm opacity-70">暂无</div>
        </div>

        <div class="alert alert-info">
          <span>remove_bg 可能返回多张图（image_0/1/2）。</span>
        </div>
      </div>
    </div>
  </PageShell>
</template>
