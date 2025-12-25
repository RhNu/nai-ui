import { ref } from "vue";
import { endpoints } from "@/api/endpoints";
import type {
  BaseGenerateRequest,
  CharacterPrompt,
  CharacterSlotPreset,
} from "@/api/types";

export function useCharacterPresets(opts: {
  form: BaseGenerateRequest;
  slots?: number;
}) {
  const slots = Math.max(1, Math.floor(opts.slots ?? 6));

  const characterPresetSlot = ref(1);
  const characterPresetName = ref<string>("");
  const characterPresetNames = ref<string[]>([]);
  const characterPresetSelected = ref<string>("");

  function currentSlotIndex(): number {
    return Math.min(slots, Math.max(1, characterPresetSlot.value)) - 1;
  }

  async function refreshCharacterPresetNames() {
    try {
      const resp = await endpoints.characterPresetsList();
      characterPresetNames.value = (resp.names ?? []).slice();
    } catch {
      characterPresetNames.value = [];
    }
  }

  async function saveCharacterPreset() {
    const name = characterPresetName.value.trim();
    if (!name) return;
    const idx = currentSlotIndex();
    const slot = (opts.form.character_prompts?.[idx] ??
      null) as CharacterPrompt | null;
    if (!slot) return;
    const preset: CharacterSlotPreset = {
      prompt: slot.prompt ?? "",
      uc: slot.uc ?? "",
      center: slot.center,
    };
    try {
      await endpoints.characterPresetPut({ name, preset });
      await refreshCharacterPresetNames();
      characterPresetSelected.value = name;
    } catch (e: any) {
      alert(e?.message ?? String(e));
    }
  }

  async function applyCharacterPreset() {
    const name = characterPresetSelected.value.trim();
    if (!name) return;
    let p: CharacterSlotPreset | null = null;
    try {
      const resp = await endpoints.characterPresetGet(name);
      p = resp.preset;
    } catch {
      p = null;
    }
    if (!p) return;
    const idx = currentSlotIndex();
    const slot = (opts.form.character_prompts?.[idx] ??
      null) as CharacterPrompt | null;
    if (!slot) return;
    slot.enabled = true;
    slot.prompt = p.prompt;
    slot.uc = p.uc;
    slot.center = p.center;
  }

  async function deleteCharacterPreset() {
    const name = characterPresetSelected.value.trim();
    if (!name) return;
    if (!confirm(`确定删除角色预设：${name}？`)) return;
    try {
      await endpoints.characterPresetDelete(name);
      await refreshCharacterPresetNames();
      if (characterPresetSelected.value === name)
        characterPresetSelected.value = "";
    } catch (e: any) {
      alert(e?.message ?? String(e));
    }
  }

  return {
    characterPresetSlot,
    characterPresetName,
    characterPresetNames,
    characterPresetSelected,
    refreshCharacterPresetNames,
    saveCharacterPreset,
    applyCharacterPreset,
    deleteCharacterPreset,
  };
}
