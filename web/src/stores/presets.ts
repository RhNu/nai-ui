import { defineStore } from "pinia";
import { reactive, ref } from "vue";
import { endpoints } from "@/api/endpoints";
import type {
  CharacterSlotPreset,
  GeneratePreset,
  PromptPreset,
} from "@/api/types";

type GeneratePresetCache = Record<string, Record<string, GeneratePreset>>;

export const usePresetStore = defineStore("presets", () => {
  const generateNames = reactive<Record<string, string[]>>({});
  const promptNames = ref<string[]>([]);
  const characterNames = ref<string[]>([]);

  const generateCache = reactive<GeneratePresetCache>({});
  const promptCache = reactive<Record<string, PromptPreset>>({});
  const characterCache = reactive<Record<string, CharacterSlotPreset>>({});

  const lastError = ref("");

  async function refreshGenerateNames(model: string) {
    lastError.value = "";
    try {
      const res = await endpoints.presetsList(model);
      generateNames[model] = res.names ?? [];
    } catch (e) {
      lastError.value = e instanceof Error ? e.message : String(e);
      generateNames[model] ??= [];
    }
    return generateNames[model] ?? [];
  }

  async function refreshPromptNames() {
    lastError.value = "";
    try {
      const res = await endpoints.promptPresetsList();
      promptNames.value = res.names ?? [];
    } catch (e) {
      lastError.value = e instanceof Error ? e.message : String(e);
    }
    return promptNames.value ?? [];
  }

  async function refreshCharacterNames() {
    lastError.value = "";
    try {
      const res = await endpoints.characterPresetsList();
      characterNames.value = res.names ?? [];
    } catch (e) {
      lastError.value = e instanceof Error ? e.message : String(e);
    }
    return characterNames.value ?? [];
  }

  async function fetchGeneratePreset(model: string, name: string) {
    lastError.value = "";
    try {
      const res = await endpoints.presetGet(model, name);
      if (res.preset) {
        (generateCache[model] ??= {})[name] = res.preset;
      }
      return res.preset ?? null;
    } catch (e) {
      lastError.value = e instanceof Error ? e.message : String(e);
      return generateCache[model]?.[name] ?? null;
    }
  }

  async function fetchPromptPreset(name: string) {
    lastError.value = "";
    try {
      const res = await endpoints.promptPresetGet(name);
      if (res.preset) {
        promptCache[name] = res.preset;
      }
      return res.preset ?? null;
    } catch (e) {
      lastError.value = e instanceof Error ? e.message : String(e);
      return promptCache[name] ?? null;
    }
  }

  async function fetchCharacterPreset(name: string) {
    lastError.value = "";
    try {
      const res = await endpoints.characterPresetGet(name);
      if (res.preset) {
        characterCache[name] = res.preset;
      }
      return res.preset ?? null;
    } catch (e) {
      lastError.value = e instanceof Error ? e.message : String(e);
      return characterCache[name] ?? null;
    }
  }

  function rememberGeneratePreset(
    model: string,
    name: string,
    preset: GeneratePreset
  ) {
    (generateCache[model] ??= {})[name] = preset;
  }

  function rememberPromptPreset(name: string, preset: PromptPreset) {
    promptCache[name] = preset;
  }

  function rememberCharacterPreset(name: string, preset: CharacterSlotPreset) {
    characterCache[name] = preset;
  }

  function evictGeneratePreset(model: string, name: string) {
    if (generateCache[model]) delete generateCache[model][name];
  }

  function evictPromptPreset(name: string) {
    delete promptCache[name];
  }

  function evictCharacterPreset(name: string) {
    delete characterCache[name];
  }

  return {
    generateNames,
    promptNames,
    characterNames,
    generateCache,
    promptCache,
    characterCache,
    lastError,
    refreshGenerateNames,
    refreshPromptNames,
    refreshCharacterNames,
    fetchGeneratePreset,
    fetchPromptPreset,
    fetchCharacterPreset,
    rememberGeneratePreset,
    rememberPromptPreset,
    rememberCharacterPreset,
    evictGeneratePreset,
    evictPromptPreset,
    evictCharacterPreset,
  };
});
