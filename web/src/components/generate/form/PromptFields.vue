<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { endpoints } from "@/api/endpoints";
import type {
  BaseGenerateRequest,
  PromptSnippetPreviewResponse,
  PromptSnippetSummary,
} from "@/api/types";

type SnippetTarget =
  | { type: "positive" }
  | { type: "negative" }
  | { type: "character"; index: number; field: "prompt" | "uc" };

type TargetOption = {
  key: string;
  label: string;
  target: SnippetTarget;
};

type CharacterPreviewResult = {
  index: number;
  positive: string;
  negative: string;
  warnings: string[];
};

const props = defineProps<{
  form: BaseGenerateRequest;
}>();

const positiveRef = ref<HTMLTextAreaElement | null>(null);
const negativeRef = ref<HTMLTextAreaElement | null>(null);

const snippetQuery = ref("");
const snippetTag = ref("");
const snippets = ref<PromptSnippetSummary[]>([]);
const snippetsLoading = ref(false);

const activeTargetKey = ref<string>("positive");

const preview = ref<PromptSnippetPreviewResponse | null>(null);
const previewLoading = ref(false);
const characterPreviews = ref<CharacterPreviewResult[]>([]);

const showTools = ref(false);
const isDesktop = ref(false);
let mediaQuery: MediaQueryList | null = null;
const panelBaseClasses =
  "rounded-xl border border-base-300/70 bg-base-100 shadow-sm p-4";

const filteredSnippets = computed(() => {
  const q = snippetQuery.value.trim().toLowerCase();
  const tag = snippetTag.value.trim().toLowerCase();
  return (snippets.value ?? []).filter((s) => {
    const matchQ = !q
      ? true
      : s.name.toLowerCase().includes(q) ||
        (s.description ?? "").toLowerCase().includes(q) ||
        s.tags.some((t) => t.toLowerCase().includes(q));
    const matchTag = !tag ? true : s.tags.some((t) => t.toLowerCase() === tag);
    return matchQ && matchTag;
  });
});

const targetOptions = computed<TargetOption[]>(() => {
  const options: TargetOption[] = [
    { key: "positive", label: "正向提示词", target: { type: "positive" } },
    { key: "negative", label: "反向提示词", target: { type: "negative" } },
  ];

  (props.form.character_prompts ?? []).forEach((c, idx) => {
    const hasContent = (c?.prompt ?? "").trim() || (c?.uc ?? "").trim();
    if (!c || !(c.enabled || hasContent)) return;
    const humanIndex = idx + 1;
    options.push(
      {
        key: `char-${idx}-prompt`,
        label: `角色${humanIndex} 正向`,
        target: { type: "character", index: idx, field: "prompt" },
      },
      {
        key: `char-${idx}-uc`,
        label: `角色${humanIndex} 反向`,
        target: { type: "character", index: idx, field: "uc" },
      }
    );
  });

  return options;
});

const activeTarget = computed<SnippetTarget>(
  () =>
    targetOptions.value.find((o) => o.key === activeTargetKey.value)
      ?.target ?? { type: "positive" }
);

const activeTargetLabel = computed(
  () =>
    targetOptions.value.find((o) => o.key === activeTargetKey.value)?.label ??
    "正向提示词"
);

const characterTargetCount = computed(
  () => targetOptions.value.filter((o) => o.target.type === "character").length
);

watch(
  () => targetOptions.value,
  (opts) => {
    if (!opts.length) return;
    if (!opts.some((o) => o.key === activeTargetKey.value)) {
      activeTargetKey.value = opts[0].key;
    }
  },
  { deep: true }
);

async function loadSnippets() {
  snippetsLoading.value = true;
  try {
    const r = await endpoints.promptSnippetsList({
      q: snippetQuery.value || undefined,
      tags: snippetTag.value ? [snippetTag.value] : undefined,
    });
    snippets.value = r.items ?? [];
  } catch {
    snippets.value = [];
  } finally {
    snippetsLoading.value = false;
  }
}

function appendToken(value: string, token: string) {
  if (!value?.trim()) return token;
  return /\s$/.test(value) ? `${value}${token}` : `${value} ${token}`;
}

function insertIntoTextArea(
  el: HTMLTextAreaElement | null,
  currentValue: string,
  setValue: (v: string) => void,
  token: string
) {
  if (!el) {
    setValue(appendToken(currentValue, token));
    return;
  }

  const start = el.selectionStart ?? el.value.length;
  const end = el.selectionEnd ?? el.value.length;
  const value = el.value;
  const next = value.slice(0, start) + token + value.slice(end);
  el.value = next;
  setValue(next);
  el.focus();
  const cursor = start + token.length;
  el.setSelectionRange(cursor, cursor);
}

function insertSnippet(name: string, targetKey?: string) {
  const token = `<snippet:${name}>, `;
  const pickedTarget = targetKey
    ? targetOptions.value.find((o) => o.key === targetKey)?.target
    : null;
  const target = pickedTarget ?? activeTarget.value;

  if (target.type === "positive") {
    insertIntoTextArea(
      positiveRef.value,
      props.form.positive,
      (v) => (props.form.positive = v),
      token
    );
    activeTargetKey.value = "positive";
    return;
  }

  if (target.type === "negative") {
    insertIntoTextArea(
      negativeRef.value,
      props.form.negative,
      (v) => (props.form.negative = v),
      token
    );
    activeTargetKey.value = "negative";
    return;
  }

  if (target.type === "character") {
    const c = props.form.character_prompts?.[target.index];
    if (!c) return;
    const field = target.field === "uc" ? "uc" : "prompt";
    c[field] = appendToken(c[field] ?? "", token);
    activeTargetKey.value = `char-${target.index}-${field}`;
  }
}

async function runPreview() {
  previewLoading.value = true;
  try {
    const characters = (props.form.character_prompts ?? [])
      .map((c, idx) => ({ c, idx }))
      .filter(({ c }) =>
        Boolean(
          c && c.enabled && ((c.prompt ?? "").trim() || (c.uc ?? "").trim())
        )
      );

    const [base, charResults] = await Promise.all([
      endpoints.promptSnippetPreview({
        positive: props.form.positive,
        negative: props.form.negative,
      }),
      Promise.all(
        characters.map(({ c, idx }) =>
          endpoints
            .promptSnippetPreview({
              positive: c.prompt ?? "",
              negative: c.uc ?? "",
            })
            .then((r) => ({
              index: idx,
              positive: r.positive,
              negative: r.negative,
              warnings: r.warnings ?? [],
            }))
            .catch((e) => ({
              index: idx,
              positive: c?.prompt ?? "",
              negative: c?.uc ?? "",
              warnings: [
                `角色${idx + 1} 预览失败: ${
                  e instanceof Error ? e.message : String(e)
                }`,
              ],
            }))
        )
      ),
    ]);

    preview.value = base;
    characterPreviews.value = charResults;
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    preview.value = {
      positive: props.form.positive,
      negative: props.form.negative,
      warnings: [`预览失败: ${msg}`],
    };
    characterPreviews.value = [];
  } finally {
    previewLoading.value = false;
  }
}

function applyPreviewResult() {
  if (!preview.value) return;
  props.form.positive = preview.value.positive;
  props.form.negative = preview.value.negative;
  for (const r of characterPreviews.value) {
    const c = props.form.character_prompts?.[r.index];
    if (!c) continue;
    c.prompt = r.positive;
    c.uc = r.negative;
  }
}

function syncMediaQuery() {
  const desktop = mediaQuery?.matches ?? false;
  isDesktop.value = desktop;
  showTools.value = desktop ? true : false;
}

onMounted(() => {
  mediaQuery = window.matchMedia("(min-width: 1280px)");
  mediaQuery.addEventListener("change", syncMediaQuery);
  syncMediaQuery();
  void loadSnippets();
});

onBeforeUnmount(() => {
  mediaQuery?.removeEventListener("change", syncMediaQuery);
});
</script>

<template>
  <div class="relative">
    <div
      class="grid gap-3 xl:grid-cols-[minmax(0,1fr)_minmax(340px,0.9fr)] xl:items-start"
    >
      <div class="space-y-3">
        <div class="flex justify-end xl:hidden">
          <button class="btn btn-sm" type="button" @click="showTools = true">
            片段/预览
          </button>
        </div>

        <fieldset class="fieldset">
          <legend class="fieldset-legend">正向提示词</legend>
          <textarea
            ref="positiveRef"
            v-model="props.form.positive"
            class="textarea textarea-bordered h-28 w-full"
            placeholder="positive prompt"
            @focus="activeTargetKey = 'positive'"
          />
        </fieldset>

        <fieldset class="fieldset">
          <legend class="fieldset-legend">反向提示词</legend>
          <textarea
            ref="negativeRef"
            v-model="props.form.negative"
            class="textarea textarea-bordered h-28 w-full"
            placeholder="negative prompt"
            @focus="activeTargetKey = 'negative'"
          />
        </fieldset>
      </div>

      <aside
        v-if="isDesktop"
        :class="
          panelBaseClasses +
          ' xl:sticky xl:top-2 xl:max-h-[calc(100vh-140px)] xl:overflow-y-auto'
        "
        aria-label="片段与预览工具"
      >
        <div class="flex items-start justify-between gap-3">
          <div>
            <div class="font-semibold leading-tight">片段与预览</div>
            <div class="text-xs text-base-content/70">
              在提示框使用 &lt;snippet:名称&gt; 复用片段
            </div>
            <div class="text-xs text-base-content/70 mt-1">
              当前插入目标：{{ activeTargetLabel }}
            </div>
          </div>
        </div>

        <div class="mt-3 space-y-3">
          <div
            class="rounded-lg border border-base-300/70 bg-base-200/60 p-3 space-y-2"
          >
            <div class="flex flex-wrap items-center gap-2">
              <div class="text-sm font-semibold">插入目标</div>
              <select
                v-model="activeTargetKey"
                class="select select-bordered select-sm w-full sm:w-auto"
              >
                <option
                  v-for="opt in targetOptions"
                  :key="opt.key"
                  :value="opt.key"
                >
                  {{ opt.label }}
                </option>
              </select>
              <span class="text-xs text-base-content/70"
                >选择片段要落入的位置</span
              >
            </div>
            <div
              v-if="!characterTargetCount"
              class="text-xs text-base-content/60"
            >
              开启“角色分区”后，角色提示词也会出现在插入目标里。
            </div>
          </div>

          <div class="flex flex-wrap items-center gap-2">
            <input
              v-model="snippetQuery"
              class="input input-bordered w-full sm:w-48"
              placeholder="关键词"
              @change="loadSnippets"
            />
            <input
              v-model="snippetTag"
              class="input input-bordered w-full sm:w-40"
              placeholder="标签过滤"
              @change="loadSnippets"
            />
            <button
              class="btn btn-sm"
              type="button"
              :class="{ 'btn-disabled': snippetsLoading }"
              @click="loadSnippets"
            >
              刷新
            </button>
            <span v-if="snippetsLoading" class="text-sm text-base-content/70"
              >加载中…</span
            >
          </div>

          <div
            class="flex flex-col gap-2 rounded-lg border border-base-300/70 bg-base-200/60 p-3 max-h-60 overflow-y-auto"
          >
            <div
              v-if="!filteredSnippets.length"
              class="text-sm text-base-content/70"
            >
              暂无片段，先在“片段库”页创建。
            </div>
            <div
              v-for="s in filteredSnippets"
              :key="s.name"
              class="rounded-lg bg-base-100/80 p-2 shadow-sm"
            >
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <div class="font-semibold leading-tight">{{ s.name }}</div>
                  <div class="text-xs text-base-content/70">
                    {{ s.tags.length ? s.tags.join(", ") : "无标签" }}
                  </div>
                  <div
                    v-if="s.description"
                    class="text-xs text-base-content/70 line-clamp-2"
                  >
                    {{ s.description }}
                  </div>
                </div>
                <div class="flex flex-col gap-2">
                  <button
                    class="btn btn-xs btn-primary"
                    type="button"
                    @click="insertSnippet(s.name)"
                  >
                    插入到{{ activeTargetLabel }}
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div class="rounded-lg border border-base-300/70 bg-base-100/80 p-3">
            <div class="flex flex-wrap items-center gap-2">
              <div class="font-semibold">预览展开（dry run）</div>
              <div class="flex flex-wrap items-center gap-2">
                <button
                  class="btn btn-sm btn-primary"
                  type="button"
                  :class="{ 'btn-disabled': previewLoading }"
                  @click="runPreview"
                >
                  预览
                </button>
                <button
                  class="btn btn-sm"
                  type="button"
                  :class="{ 'btn-disabled': previewLoading || !preview }"
                  @click="applyPreviewResult"
                >
                  将预览结果写回
                </button>
                <div
                  v-if="previewLoading"
                  class="loading loading-spinner loading-xs"
                />
              </div>
            </div>

            <div
              v-if="preview?.warnings?.length"
              class="alert alert-warning mt-3"
            >
              <ul class="list-disc pl-4 text-sm">
                <li v-for="w in preview?.warnings" :key="w">{{ w }}</li>
              </ul>
            </div>

            <div class="mt-3 space-y-2 text-sm">
              <div>
                <div class="font-semibold">正向</div>
                <div
                  class="rounded bg-base-200/70 p-2 whitespace-pre-wrap wrap-break-word"
                >
                  {{ preview?.positive ?? "尚未预览" }}
                </div>
              </div>
              <div>
                <div class="font-semibold">反向</div>
                <div
                  class="rounded bg-base-200/70 p-2 whitespace-pre-wrap wrap-break-word"
                >
                  {{ preview?.negative ?? "尚未预览" }}
                </div>
              </div>
              <div v-if="characterPreviews.length" class="space-y-2">
                <div class="font-semibold">角色</div>
                <div
                  v-for="r in characterPreviews"
                  :key="r.index"
                  class="rounded border border-base-300/60 bg-base-200/40 p-2"
                >
                  <div class="text-xs font-semibold mb-1">
                    角色 {{ r.index + 1 }}
                  </div>
                  <div class="space-y-1 text-xs">
                    <div>
                      <div class="font-semibold">正向</div>
                      <div
                        class="rounded bg-base-100/80 p-2 whitespace-pre-wrap wrap-break-word"
                      >
                        {{ r.positive || "(空)" }}
                      </div>
                    </div>
                    <div>
                      <div class="font-semibold">反向</div>
                      <div
                        class="rounded bg-base-100/80 p-2 whitespace-pre-wrap wrap-break-word"
                      >
                        {{ r.negative || "(空)" }}
                      </div>
                    </div>
                    <div v-if="r.warnings?.length" class="text-warning">
                      <ul class="list-disc pl-4">
                        <li v-for="w in r.warnings" :key="w">{{ w }}</li>
                      </ul>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </aside>
    </div>

    <div
      v-if="showTools && !isDesktop"
      class="fixed inset-0 z-50 flex justify-center bg-base-content/30 backdrop-blur-sm p-3"
      @click.self="showTools = false"
    >
      <div
        class="w-full max-w-3xl space-y-3 overflow-y-auto rounded-2xl border border-base-300/80 bg-base-100 p-4 shadow-2xl"
      >
        <div class="flex items-start justify-between gap-3">
          <div>
            <div class="font-semibold leading-tight">片段与预览</div>
            <div class="text-xs text-base-content/70">
              在提示框使用 &lt;snippet:名称&gt; 复用片段
            </div>
            <div class="text-xs text-base-content/70 mt-1">
              当前插入目标：{{ activeTargetLabel }}
            </div>
          </div>
          <button
            class="btn btn-ghost btn-sm"
            type="button"
            @click="showTools = false"
          >
            ✕
          </button>
        </div>

        <div class="space-y-3">
          <div
            class="rounded-lg border border-base-300/70 bg-base-200/60 p-3 space-y-2"
          >
            <div class="flex flex-wrap items-center gap-2">
              <div class="text-sm font-semibold">插入目标</div>
              <select
                v-model="activeTargetKey"
                class="select select-bordered select-sm w-full"
              >
                <option
                  v-for="opt in targetOptions"
                  :key="opt.key"
                  :value="opt.key"
                >
                  {{ opt.label }}
                </option>
              </select>
              <span class="text-xs text-base-content/70"
                >选择片段要落入的位置</span
              >
            </div>
            <div
              v-if="!characterTargetCount"
              class="text-xs text-base-content/60"
            >
              开启“角色分区”后，角色提示词也会出现在插入目标里。
            </div>
          </div>

          <div class="flex flex-wrap items-center gap-2">
            <input
              v-model="snippetQuery"
              class="input input-bordered w-full"
              placeholder="关键词"
              @change="loadSnippets"
            />
            <input
              v-model="snippetTag"
              class="input input-bordered w-full"
              placeholder="标签过滤"
              @change="loadSnippets"
            />
            <div class="flex gap-2">
              <button
                class="btn btn-sm"
                type="button"
                :class="{ 'btn-disabled': snippetsLoading }"
                @click="loadSnippets"
              >
                刷新
              </button>
              <span
                v-if="snippetsLoading"
                class="text-sm text-base-content/70 self-center"
                >加载中…</span
              >
            </div>
          </div>

          <div
            class="flex flex-col gap-2 rounded-lg border border-base-300/70 bg-base-200/60 p-3 max-h-64 overflow-y-auto"
          >
            <div
              v-if="!filteredSnippets.length"
              class="text-sm text-base-content/70"
            >
              暂无片段，先在“片段库”页创建。
            </div>
            <div
              v-for="s in filteredSnippets"
              :key="s.name"
              class="rounded-lg bg-base-100/80 p-2 shadow-sm"
            >
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <div class="font-semibold leading-tight">{{ s.name }}</div>
                  <div class="text-xs text-base-content/70">
                    {{ s.tags.length ? s.tags.join(", ") : "无标签" }}
                  </div>
                  <div
                    v-if="s.description"
                    class="text-xs text-base-content/70 line-clamp-2"
                  >
                    {{ s.description }}
                  </div>
                </div>
                <div class="flex flex-col gap-2">
                  <button
                    class="btn btn-xs btn-primary"
                    type="button"
                    @click="insertSnippet(s.name)"
                  >
                    插入到{{ activeTargetLabel }}
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div class="rounded-lg border border-base-300/70 bg-base-100/80 p-3">
            <div class="flex flex-wrap items-center gap-2">
              <div class="font-semibold">预览展开（dry run）</div>
              <div class="flex flex-wrap items-center gap-2">
                <button
                  class="btn btn-sm btn-primary"
                  type="button"
                  :class="{ 'btn-disabled': previewLoading }"
                  @click="runPreview"
                >
                  预览
                </button>
                <button
                  class="btn btn-sm"
                  type="button"
                  :class="{ 'btn-disabled': previewLoading || !preview }"
                  @click="applyPreviewResult"
                >
                  将预览结果写回
                </button>
                <div
                  v-if="previewLoading"
                  class="loading loading-spinner loading-xs"
                />
              </div>
            </div>

            <div
              v-if="preview?.warnings?.length"
              class="alert alert-warning mt-3"
            >
              <ul class="list-disc pl-4 text-sm">
                <li v-for="w in preview?.warnings" :key="w">{{ w }}</li>
              </ul>
            </div>

            <div class="mt-3 space-y-2 text-sm">
              <div>
                <div class="font-semibold">正向</div>
                <div
                  class="rounded bg-base-200/70 p-2 whitespace-pre-wrap wrap-break-word"
                >
                  {{ preview?.positive ?? "尚未预览" }}
                </div>
              </div>
              <div>
                <div class="font-semibold">反向</div>
                <div
                  class="rounded bg-base-200/70 p-2 whitespace-pre-wrap wrap-break-word"
                >
                  {{ preview?.negative ?? "尚未预览" }}
                </div>
              </div>
              <div v-if="characterPreviews.length" class="space-y-2">
                <div class="font-semibold">角色</div>
                <div
                  v-for="r in characterPreviews"
                  :key="r.index"
                  class="rounded border border-base-300/60 bg-base-200/40 p-2"
                >
                  <div class="text-xs font-semibold mb-1">
                    角色 {{ r.index + 1 }}
                  </div>
                  <div class="space-y-1 text-xs">
                    <div>
                      <div class="font-semibold">正向</div>
                      <div
                        class="rounded bg-base-100/80 p-2 whitespace-pre-wrap wrap-break-word"
                      >
                        {{ r.positive || "(空)" }}
                      </div>
                    </div>
                    <div>
                      <div class="font-semibold">反向</div>
                      <div
                        class="rounded bg-base-100/80 p-2 whitespace-pre-wrap wrap-break-word"
                      >
                        {{ r.negative || "(空)" }}
                      </div>
                    </div>
                    <div v-if="r.warnings?.length" class="text-warning">
                      <ul class="list-disc pl-4">
                        <li v-for="w in r.warnings" :key="w">{{ w }}</li>
                      </ul>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
