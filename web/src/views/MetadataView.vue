<script setup lang="ts">
import { computed, ref } from "vue";
import PageShell from "@/components/layout/PageShell.vue";

type TextEntry = {
  key: string;
  value: string;
  source: string;
  json?: string;
};

type ParsedImage = {
  id: number;
  name: string;
  size: number;
  type: string;
  width?: number;
  height?: number;
  previewUrl: string;
  parameters?: string;
  entries: TextEntry[];
  warnings: string[];
  error?: string;
};

const results = ref<ParsedImage[]>([]);
const loading = ref(false);
const dragActive = ref(false);
const copyHint = ref<string>("");
const clipboardStatus = ref<string>("");
let idSeed = 0;

const decoder = new TextDecoder("utf-8", { fatal: false });

const hasResult = computed(() => results.value.length > 0);
const clipboardSupported = computed(
  () =>
    typeof navigator !== "undefined" &&
    !!navigator.clipboard &&
    "read" in navigator.clipboard
);

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
}

function chunkType(view: DataView, offset: number): string {
  return String.fromCharCode(
    view.getUint8(offset),
    view.getUint8(offset + 1),
    view.getUint8(offset + 2),
    view.getUint8(offset + 3)
  );
}

function isPng(buffer: ArrayBuffer): boolean {
  const signature = new Uint8Array(buffer, 0, 8);
  const pngSig = new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10]);
  return pngSig.every((v, i) => signature[i] === v);
}

function decodeString(bytes: Uint8Array): string {
  return decoder.decode(bytes);
}

function tryFormatJson(value: string): string | undefined {
  const trimmed = value.trim().replace(/^\ufeff/, "");
  if (!trimmed.startsWith("{") && !trimmed.startsWith("[")) return undefined;
  try {
    const parsed = JSON.parse(trimmed);
    return JSON.stringify(parsed, null, 2);
  } catch {
    return undefined;
  }
}

async function inflate(data: Uint8Array): Promise<Uint8Array> {
  if (typeof DecompressionStream === "undefined") {
    throw new Error("浏览器缺少 DecompressionStream，无法解压 zTXt / iTXt");
  }
  const stream = new Blob([data])
    .stream()
    .pipeThrough(new DecompressionStream("deflate"));
  const buffer = await new Response(stream).arrayBuffer();
  return new Uint8Array(buffer);
}

function parseTextChunk(data: Uint8Array): { key: string; value: string } {
  const sep = data.indexOf(0);
  if (sep === -1) {
    return { key: "tEXt", value: decodeString(data) };
  }
  const key = decodeString(data.slice(0, sep)).trim() || "tEXt";
  const value = decodeString(data.slice(sep + 1));
  return { key, value };
}

async function parseZTxtChunk(
  data: Uint8Array
): Promise<{ key: string; value: string }> {
  const sep = data.indexOf(0);
  if (sep === -1 || sep + 2 > data.length) {
    throw new Error("无效的 zTXt 块");
  }
  const key = decodeString(data.slice(0, sep)).trim() || "zTXt";
  const method = data[sep + 1];
  if (method !== 0) {
    throw new Error(`未知压缩方式 ${method}`);
  }
  const inflated = await inflate(data.slice(sep + 2));
  return { key, value: decodeString(inflated) };
}

async function parseITxtChunk(
  data: Uint8Array
): Promise<{ key: string; value: string }> {
  let cursor = 0;
  const keywordEnd = data.indexOf(0, cursor);
  if (keywordEnd === -1) throw new Error("iTXt 缺少关键字");
  const keyword = decodeString(data.slice(cursor, keywordEnd)).trim() || "iTXt";
  cursor = keywordEnd + 1;

  const compressionFlag = data[cursor];
  const compressionMethod = data[cursor + 1];
  cursor += 2;

  const langEnd = data.indexOf(0, cursor);
  if (langEnd === -1) throw new Error("iTXt 缺少语言标签");
  cursor = langEnd + 1;

  const translatedEnd = data.indexOf(0, cursor);
  if (translatedEnd === -1) throw new Error("iTXt 缺少译名");
  cursor = translatedEnd + 1;

  let textBytes = data.slice(cursor);
  if (compressionFlag === 1) {
    if (compressionMethod !== 0) {
      throw new Error(`未知压缩方式 ${compressionMethod}`);
    }
    textBytes = await inflate(textBytes);
  }
  return { key: keyword, value: decodeString(textBytes) };
}

async function parsePng(
  buffer: ArrayBuffer
): Promise<Omit<ParsedImage, "id" | "name" | "size" | "type" | "previewUrl">> {
  const view = new DataView(buffer);
  let offset = 8;
  let width: number | undefined;
  let height: number | undefined;
  const entries: TextEntry[] = [];
  const warnings: string[] = [];
  let parameters: string | undefined;

  const addEntry = (key: string, value: string, source: string) => {
    entries.push({ key, value, source, json: tryFormatJson(value) });
    if (!parameters && key.toLowerCase() === "parameters") {
      parameters = value;
    }
  };

  while (offset + 8 <= view.byteLength) {
    const length = view.getUint32(offset);
    const type = chunkType(view, offset + 4);
    const dataStart = offset + 8;
    const dataEnd = dataStart + length;
    if (dataEnd + 4 > view.byteLength) {
      warnings.push(`块 ${type} 超出文件长度`);
      break;
    }
    const data = new Uint8Array(buffer, dataStart, length);

    if (type === "IHDR" && length >= 8) {
      width = view.getUint32(dataStart);
      height = view.getUint32(dataStart + 4);
    } else if (type === "tEXt") {
      const { key, value } = parseTextChunk(data);
      addEntry(key, value, "tEXt");
    } else if (type === "zTXt") {
      try {
        const { key, value } = await parseZTxtChunk(data);
        addEntry(key, value, "zTXt");
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        warnings.push(`zTXt 解析失败：${msg}`);
      }
    } else if (type === "iTXt") {
      try {
        const { key, value } = await parseITxtChunk(data);
        addEntry(key, value, "iTXt");
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        warnings.push(`iTXt 解析失败：${msg}`);
      }
    }

    offset = dataEnd + 4; // 跳过 CRC
    if (type === "IEND") break;
  }

  return { width, height, entries, warnings, parameters };
}

async function parseFile(file: File): Promise<ParsedImage> {
  const buffer = await file.arrayBuffer();
  const base: ParsedImage = {
    id: idSeed++,
    name: file.name,
    size: file.size,
    type: file.type || "unknown",
    previewUrl: URL.createObjectURL(file),
    entries: [],
    warnings: [],
  };

  try {
    if (!isPng(buffer)) {
      return {
        ...base,
        error:
          "暂仅支持解析 PNG（tEXt / zTXt / iTXt），其他格式请先转 PNG 再试",
      };
    }
    const parsed = await parsePng(buffer);
    return { ...base, ...parsed };
  } catch (e) {
    return {
      ...base,
      error: e instanceof Error ? e.message : String(e),
    };
  }
}

function revokePreviews(list: ParsedImage[]) {
  for (const item of list) {
    URL.revokeObjectURL(item.previewUrl);
  }
}

async function handleFiles(fileList: FileList | File[]) {
  const files = Array.from(fileList ?? []).filter((f) =>
    f.type.startsWith("image/")
  );
  if (!files.length) return;
  loading.value = true;
  copyHint.value = "";
  try {
    const parsed = await Promise.all(files.map((f) => parseFile(f)));
    results.value = [...parsed, ...results.value];
  } finally {
    loading.value = false;
  }
}

function onFileChange(ev: Event) {
  const input = ev.target as HTMLInputElement;
  if (!input.files?.length) return;
  void handleFiles(input.files);
  input.value = "";
}

function onDrop(ev: DragEvent) {
  ev.preventDefault();
  dragActive.value = false;
  if (ev.dataTransfer?.files?.length) {
    void handleFiles(ev.dataTransfer.files);
  }
}

function onDragOver(ev: DragEvent) {
  ev.preventDefault();
  dragActive.value = true;
}

function onDragLeave() {
  dragActive.value = false;
}

function clearResults() {
  revokePreviews(results.value);
  results.value = [];
}

async function copyText(text: string) {
  await navigator.clipboard.writeText(text);
  copyHint.value = "已复制到剪贴板";
  setTimeout(() => {
    copyHint.value = "";
  }, 1600);
}

async function parseFromClipboard() {
  clipboardStatus.value = "";
  if (!clipboardSupported.value) {
    clipboardStatus.value = "当前浏览器不支持读取剪贴板图像";
    return;
  }
  try {
    const items = await navigator.clipboard.read();
    const files: File[] = [];
    for (const item of items) {
      const type = item.types.find((t) => t.startsWith("image/"));
      if (!type) continue;
      const blob = await item.getType(type);
      const name = `clipboard-${Date.now()}-${idSeed}.png`;
      files.push(new File([blob], name, { type: blob.type || "image/png" }));
    }
    if (!files.length) {
      clipboardStatus.value = "剪贴板中没有图片内容";
      return;
    }
    await handleFiles(files);
    clipboardStatus.value = `已解析 ${files.length} 张剪贴板图片`;
  } catch (e) {
    clipboardStatus.value = e instanceof Error ? e.message : String(e);
  }
}
</script>

<template>
  <PageShell
    title="图片元数据解析"
    subtitle="灵感来自 looyun/spell，纯前端解析 PNG 中的 Stable Diffusion 参数块"
    max-width="2xl"
    :loading="loading"
    loading-text="解析中"
  >
    <div class="grid gap-6 lg:grid-cols-[1.15fr,0.85fr]">
      <div class="space-y-4">
        <div
          class="rounded-2xl border border-dashed border-primary/60 bg-primary/5 p-5 shadow-inner transition"
          :class="{ 'border-primary bg-primary/10 shadow-lg': dragActive }"
          @dragover="onDragOver"
          @drop="onDrop"
          @dragleave="onDragLeave"
        >
          <div
            class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between"
          >
            <div class="space-y-1">
              <div class="text-sm font-semibold">拖拽或选择图片</div>
              <div class="text-xs text-base-content/70">
                支持 PNG 的 tEXt / zTXt /
                iTXt；其他格式会保留预览但不解析元数据。
              </div>
            </div>
            <div class="flex items-center gap-2">
              <label class="btn btn-primary" for="metadata-upload"
                >选择文件</label
              >
              <input
                id="metadata-upload"
                class="hidden"
                type="file"
                accept="image/png,image/jpeg,image/webp"
                multiple
                @change="onFileChange"
              />
              <button
                class="btn btn-outline"
                type="button"
                :class="{ 'btn-disabled': loading || !clipboardSupported }"
                @click="parseFromClipboard"
              >
                从剪贴板解析
              </button>
              <button
                class="btn btn-ghost"
                type="button"
                :class="{ 'btn-disabled': !hasResult }"
                @click="clearResults"
              >
                清空
              </button>
            </div>
          </div>
          <div v-if="clipboardStatus" class="mt-2 text-xs text-base-content/70">
            {{ clipboardStatus }}
          </div>
        </div>

        <div v-if="hasResult" class="space-y-4">
          <div class="text-sm font-semibold text-base-content/70">
            解析结果（最新在前）
          </div>
          <div class="space-y-4">
            <div
              v-for="item in results"
              :key="item.id"
              class="rounded-xl border border-base-300/70 bg-base-200/40 p-4 shadow-sm"
            >
              <div class="flex flex-col gap-4 md:flex-row">
                <div class="w-full md:w-48">
                  <div
                    class="relative overflow-hidden rounded-lg border border-base-300/80 bg-base-100"
                  >
                    <img
                      :src="item.previewUrl"
                      alt="preview"
                      class="aspect-square w-full object-cover"
                      loading="lazy"
                    />
                  </div>
                  <div
                    class="mt-2 flex flex-wrap items-center gap-2 text-xs text-base-content/70"
                  >
                    <span class="badge badge-outline">{{
                      formatSize(item.size)
                    }}</span>
                    <span class="badge badge-outline">{{
                      item.type || "image"
                    }}</span>
                    <span
                      v-if="item.width && item.height"
                      class="badge badge-neutral"
                    >
                      {{ item.width }} × {{ item.height }}
                    </span>
                  </div>
                </div>

                <div class="flex-1 space-y-3">
                  <div
                    class="flex flex-wrap items-center justify-between gap-2"
                  >
                    <div>
                      <div class="text-base font-semibold">{{ item.name }}</div>
                      <div class="text-xs text-base-content/60">
                        {{ item.entries.length }} 条元数据
                      </div>
                    </div>
                    <div
                      class="flex items-center gap-2 text-xs"
                      v-if="item.warnings.length"
                    >
                      <span class="badge badge-warning"
                        >{{ item.warnings.length }} 条警告</span
                      >
                    </div>
                  </div>

                  <div v-if="item.error" class="alert alert-error">
                    <span>{{ item.error }}</span>
                  </div>

                  <div
                    v-if="item.parameters"
                    class="rounded-lg border border-primary/40 bg-primary/5 p-3"
                  >
                    <div class="flex items-center justify-between gap-2">
                      <div class="text-sm font-semibold">parameters</div>
                      <div
                        class="flex items-center gap-2 text-xs text-base-content/70"
                      >
                        <span v-if="copyHint" class="text-success">{{
                          copyHint
                        }}</span>
                        <button
                          class="btn btn-xs"
                          type="button"
                          @click="copyText(item.parameters)"
                        >
                          复制
                        </button>
                      </div>
                    </div>
                    <pre
                      class="mt-2 whitespace-pre-wrap break-words font-mono text-xs leading-relaxed"
                      >{{ item.parameters }}</pre
                    >
                  </div>

                  <div v-if="item.entries.length" class="space-y-2">
                    <div
                      class="text-xs uppercase tracking-[0.18em] text-base-content/50"
                    >
                      tEXt / zTXt / iTXt
                    </div>
                    <div class="space-y-2">
                      <div
                        v-for="entry in item.entries"
                        :key="`${entry.source}-${entry.key}-${entry.value.slice(
                          0,
                          16
                        )}`"
                        class="rounded-lg border border-base-300/70 bg-base-100/80 p-3"
                      >
                        <div
                          class="flex items-center justify-between gap-2 text-xs text-base-content/60"
                        >
                          <span class="font-semibold text-base-content">{{
                            entry.key
                          }}</span>
                          <span class="badge badge-ghost">{{
                            entry.source
                          }}</span>
                        </div>
                        <pre
                          v-if="entry.json"
                          class="mt-1 whitespace-pre-wrap break-words font-mono text-xs leading-relaxed"
                          >{{ entry.json }}</pre
                        >
                        <pre
                          v-else
                          class="mt-1 whitespace-pre-wrap break-words font-mono text-xs leading-relaxed"
                          >{{ entry.value }}</pre
                        >
                      </div>
                    </div>
                  </div>

                  <div
                    v-if="item.warnings.length"
                    class="rounded-lg border border-warning/50 bg-warning/10 p-3 text-sm"
                  >
                    <div class="font-semibold">解析警告</div>
                    <ul class="mt-1 list-disc space-y-1 pl-4">
                      <li v-for="w in item.warnings" :key="w">{{ w }}</li>
                    </ul>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div
          v-else
          class="rounded-xl border border-base-300/70 bg-base-200/50 p-6 text-sm text-base-content/70"
        >
          暂无文件，拖拽图片到上方区域即可解析。推荐将 PNG
          原图直接拖入，可完整提取 Stable Diffusion 的 parameters 字段。
        </div>
      </div>

      <div
        class="space-y-3 rounded-2xl border border-base-300/70 bg-base-100/90 p-4 shadow"
      >
        <div class="flex items-center justify-between">
          <div>
            <div class="text-sm font-semibold">提示</div>
            <div class="text-xs text-base-content/60">前端离线解析</div>
          </div>
          <div class="badge badge-primary">spell 风格</div>
        </div>
        <ul class="list-disc space-y-1 pl-5 text-sm text-base-content/70">
          <li>PNG 会读取 tEXt、zTXt、iTXt；parameters 会单独展示并可复制。</li>
          <li>
            若使用压缩文本块，需要浏览器支持 DecompressionStream（Chrome/Edge
            新版可用）。
          </li>
          <li>JPEG/WebP 目前仅做预览，不解析 EXIF；如需解析请先转 PNG。</li>
          <li>解析时不上传文件，所有逻辑在浏览器本地完成。</li>
        </ul>
      </div>
    </div>
  </PageShell>
</template>
