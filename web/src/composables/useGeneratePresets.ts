import { ref, watch } from "vue";
import { endpoints } from "@/api/endpoints";
import type { BaseGenerateRequest, GeneratePreset } from "@/api/types";

export function useGeneratePresets(opts: {
  form: BaseGenerateRequest;
  defaultPresetName: string;
  applyPresetForModel: (model: string) => void;
  applyPresetToForm: (model: string, p: GeneratePreset) => void;
  extractPresetFromForm: () => GeneratePreset;
}) {
  const presetName = ref<string>(opts.defaultPresetName);
  const presetNames = ref<string[]>([opts.defaultPresetName]);
  const presetBusy = ref(false);
  const suppressAutoPresetApply = ref(true);

  async function refreshPresetNames(model: string) {
    try {
      const r = await endpoints.presetsList(model);
      presetNames.value = r.names.length ? r.names : [opts.defaultPresetName];
      if (!presetNames.value.includes(opts.defaultPresetName)) {
        presetNames.value.unshift(opts.defaultPresetName);
      }
    } catch {
      presetNames.value = [opts.defaultPresetName];
    }
  }

  async function loadAndApplyPreset(model: string, name: string) {
    presetBusy.value = true;
    try {
      await refreshPresetNames(model);
      const r = await endpoints.presetGet(model, name);
      if (r.preset) {
        opts.applyPresetToForm(model, r.preset);
      } else {
        opts.applyPresetForModel(model);
      }
    } catch {
      opts.applyPresetForModel(model);
    } finally {
      presetBusy.value = false;
    }
  }

  async function savePreset() {
    const name = presetName.value.trim() || opts.defaultPresetName;
    presetBusy.value = true;
    try {
      await endpoints.presetPut({
        model: opts.form.model,
        name,
        preset: opts.extractPresetFromForm(),
      });
      presetName.value = name;
      await refreshPresetNames(opts.form.model);
    } finally {
      presetBusy.value = false;
    }
  }

  async function deletePreset() {
    const name = presetName.value.trim() || opts.defaultPresetName;
    if (name === opts.defaultPresetName) return;
    if (!confirm(`确定删除预设：${name}？`)) return;
    presetBusy.value = true;
    try {
      await endpoints.presetDelete(opts.form.model, name);
      presetName.value = opts.defaultPresetName;
      await refreshPresetNames(opts.form.model);
      await loadAndApplyPreset(opts.form.model, presetName.value);
    } finally {
      presetBusy.value = false;
    }
  }

  async function renamePreset() {
    const from = presetName.value.trim() || opts.defaultPresetName;
    if (from === opts.defaultPresetName) return;
    const to = prompt("新预设名：", from)?.trim();
    if (!to || to === from) return;
    presetBusy.value = true;
    try {
      await endpoints.presetRename({ model: opts.form.model, from, to });
      presetName.value = to;
      await refreshPresetNames(opts.form.model);
    } finally {
      presetBusy.value = false;
    }
  }

  watch(
    () => opts.form.model,
    (m, prev) => {
      if (m === prev) return;
      presetName.value = opts.defaultPresetName;
      if (suppressAutoPresetApply.value) {
        void refreshPresetNames(m);
        return;
      }
      void loadAndApplyPreset(m, presetName.value);
    },
    { flush: "sync" }
  );

  watch(
    () => presetName.value,
    (n, prev) => {
      if (n === prev) return;
      const name = n.trim() || opts.defaultPresetName;
      void loadAndApplyPreset(opts.form.model, name);
    }
  );

  return {
    presetName,
    presetNames,
    presetBusy,
    suppressAutoPresetApply,
    refreshPresetNames,
    loadAndApplyPreset,
    savePreset,
    deletePreset,
    renamePreset,
  };
}
