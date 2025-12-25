<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import { endpoints } from "@/api/endpoints";
import type { PromptSnippet, PromptSnippetSummary } from "@/api/types";

const snippetName = ref("");
const snippets = ref<PromptSnippetSummary[]>([]);
const busy = ref(false);
const filterText = ref("");

const form = reactive<PromptSnippet>({
  body: "",
  tags: [],
  description: "",
});

const tagsInput = computed({
  get() {
    return (form.tags ?? []).join(", ");
  },
  set(v: string) {
    form.tags = v
      .split(",")
      .map((s) => s.trim())
      .filter((s) => s.length);
  },
});

async function refreshList() {
  busy.value = true;
  try {
    const r = await endpoints.promptSnippetsList({
      q: filterText.value || undefined,
    });
    snippets.value = r.items ?? [];
    if (!snippets.value.find((s) => s.name === snippetName.value)) {
      snippetName.value = snippets.value[0]?.name ?? "";
    }
  } finally {
    busy.value = false;
  }
}

async function loadSelected(skipRefresh = false) {
  const name = snippetName.value.trim();
  if (!name) return;
  busy.value = true;
  try {
    if (!skipRefresh) await refreshList();
    const r = await endpoints.promptSnippetGet(name);
    if (r.snippet) {
      form.body = r.snippet.body ?? "";
      form.tags = (r.snippet.tags ?? []).slice();
      form.description = r.snippet.description ?? "";
    }
  } finally {
    busy.value = false;
  }
}

async function save() {
  const name = snippetName.value.trim();
  if (!name) {
    alert("请输入片段名");
    return;
  }
  busy.value = true;
  try {
    await endpoints.promptSnippetPut({ name, snippet: { ...form } });
    snippetName.value = name;
    await refreshList();
  } finally {
    busy.value = false;
  }
}

async function renameSnippet() {
  const from = snippetName.value.trim();
  if (!from) return;
  const to = prompt("新片段名：", from)?.trim();
  if (!to || to === from) return;
  busy.value = true;
  try {
    await endpoints.promptSnippetRename({ from, to });
    snippetName.value = to;
    await refreshList();
  } finally {
    busy.value = false;
  }
}

async function remove() {
  const name = snippetName.value.trim();
  if (!name) return;
  if (!confirm(`确定删除片段：${name}？`)) return;
  busy.value = true;
  try {
    await endpoints.promptSnippetDelete(name);
    await refreshList();
    snippetName.value = snippets.value[0]?.name ?? "";
    await loadSelected(true);
  } finally {
    busy.value = false;
  }
}

void refreshList().then(() => loadSelected(true));
</script>

<template>
  <div class="grid gap-4">
    <div class="grid grid-cols-1 gap-3 lg:grid-cols-3">
      <fieldset class="fieldset lg:col-span-2">
        <legend class="fieldset-legend">片段名</legend>
        <div class="join w-full">
          <input
            v-model="snippetName"
            class="input input-bordered join-item w-full"
            list="snippetNames"
            placeholder="输入或选择片段名"
            :disabled="busy"
          />
          <datalist id="snippetNames">
            <option v-for="s in snippets" :key="s.name" :value="s.name" />
          </datalist>
          <button
            class="btn join-item"
            type="button"
            :class="{ 'btn-disabled': busy || !snippetName.trim() }"
            @click="save"
          >
            保存/更新
          </button>
          <button
            class="btn join-item"
            type="button"
            :class="{ 'btn-disabled': busy || !snippetName.trim() }"
            @click="renameSnippet"
          >
            重命名
          </button>
          <button
            class="btn btn-error join-item"
            type="button"
            :class="{ 'btn-disabled': busy || !snippetName.trim() }"
            @click="remove"
          >
            删除
          </button>
        </div>
        <div class="label">
          <span class="label-text-alt"
            >支持 "&lt;snippet:名称&gt;" 语法引用</span
          >
          <span v-if="busy" class="label-text-alt">处理中…</span>
        </div>
      </fieldset>
      <fieldset class="fieldset">
        <legend class="fieldset-legend">搜索/过滤</legend>
        <div class="join w-full">
          <input
            v-model="filterText"
            class="input input-bordered join-item w-full"
            placeholder="按名称/标签/描述过滤"
            :disabled="busy"
          />
          <button
            class="btn join-item"
            type="button"
            :class="{ 'btn-disabled': busy }"
            @click="refreshList"
          >
            刷新
          </button>
          <button
            class="btn join-item"
            type="button"
            :class="{ 'btn-disabled': busy }"
            @click="() => loadSelected()"
          >
            重载当前
          </button>
        </div>
      </fieldset>
    </div>

    <div class="grid grid-cols-1 gap-3 lg:grid-cols-3">
      <fieldset class="fieldset">
        <legend class="fieldset-legend">标签（逗号分隔）</legend>
        <input
          v-model="tagsInput"
          class="input input-bordered w-full"
          placeholder="例如：场景, 服装"
        />
      </fieldset>
      <fieldset class="fieldset lg:col-span-2">
        <legend class="fieldset-legend">描述（可选）</legend>
        <input
          v-model="form.description"
          class="input input-bordered w-full"
          placeholder="用于搜索/备注"
        />
      </fieldset>
    </div>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">片段内容</legend>
      <textarea
        v-model="form.body"
        class="textarea textarea-bordered h-48 w-full"
        placeholder="可使用 <snippet:名称> 继续嵌套"
      />
    </fieldset>
  </div>
</template>
