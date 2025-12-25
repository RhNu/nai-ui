import { ref } from "vue";
import { endpoints } from "@/api/endpoints";
import type {
  BaseGenerateRequest,
  PromptPreset,
  CharacterPrompt,
} from "@/api/types";

export function usePromptPresets(opts: {
  form: BaseGenerateRequest;
  inflateCharacterSlots: (
    enabled: CharacterPrompt[] | null | undefined
  ) => CharacterPrompt[];
}) {
  const promptPresetName = ref<string>("");
  const promptPresetNames = ref<string[]>([]);

  async function refreshPromptPresetNames() {
    try {
      const resp = await endpoints.promptPresetsList();
      const names = (resp.names ?? []).slice();
      promptPresetNames.value = names;

      if (!names.includes(promptPresetName.value)) {
        promptPresetName.value = names[0] ?? "";
      }
    } catch {
      promptPresetNames.value = [];
      promptPresetName.value = "";
    }
  }

  function extractPromptPresetFromForm(): PromptPreset {
    return {
      positive: opts.form.positive ?? "",
      negative: opts.form.negative ?? "",
      add_quality_tags: (opts.form.add_quality_tags ?? null) as any,
      undesired_content_preset: (opts.form.undesired_content_preset ??
        null) as any,
      character_prompts: opts.inflateCharacterSlots(
        opts.form.character_prompts ?? []
      ),
    };
  }

  function applyPromptPresetToForm(p: PromptPreset) {
    opts.form.positive = p.positive ?? "";
    opts.form.negative = p.negative ?? "";
    opts.form.add_quality_tags = (p.add_quality_tags ??
      opts.form.add_quality_tags) as any;
    opts.form.undesired_content_preset = (p.undesired_content_preset ??
      opts.form.undesired_content_preset) as any;
    opts.form.character_prompts = opts.inflateCharacterSlots(
      p.character_prompts ?? []
    );
  }

  async function applyPromptPresetOnce() {
    const name = promptPresetName.value.trim();
    if (!name) return;
    try {
      const resp = await endpoints.promptPresetGet(name);
      if (!resp.preset) return;
      applyPromptPresetToForm(resp.preset);
    } catch {
      // ignore
    }
  }

  async function savePromptPreset() {
    const name = promptPresetName.value.trim();
    if (!name) {
      alert("请输入预设名");
      return;
    }
    try {
      await endpoints.promptPresetPut({
        name,
        preset: extractPromptPresetFromForm(),
      });
      promptPresetName.value = name;
      await refreshPromptPresetNames();
    } catch (e: any) {
      alert(e?.message ?? String(e));
    }
  }

  async function deletePromptPreset() {
    const name = promptPresetName.value.trim();
    if (!name) return;
    if (!confirm(`确定删除提示词预设：${name}？`)) return;
    try {
      await endpoints.promptPresetDelete(name);
      await refreshPromptPresetNames();
      promptPresetName.value = promptPresetNames.value[0] ?? "";
    } catch (e: any) {
      alert(e?.message ?? String(e));
    }
  }

  async function renamePromptPreset() {
    const from = promptPresetName.value.trim();
    if (!from) return;
    const to = prompt("新预设名：", from)?.trim();
    if (!to || to === from) return;
    try {
      await endpoints.promptPresetRename({ from, to });
      promptPresetName.value = to;
      await refreshPromptPresetNames();
    } catch (e: any) {
      alert(e?.message ?? String(e));
    }
  }

  return {
    promptPresetName,
    promptPresetNames,
    refreshPromptPresetNames,
    applyPromptPresetOnce,
    savePromptPreset,
    deletePromptPreset,
    renamePromptPreset,
  };
}
