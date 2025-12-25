<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import { useMetaStore } from "@/stores/meta";
import { endpoints } from "@/api/endpoints";
import AdvancedJsonCollapse from "./generate/form/AdvancedJsonCollapse.vue";
import CharacterCollapse from "./generate/form/CharacterCollapse.vue";
import CoreParamsGrid from "./generate/form/CoreParamsGrid.vue";
import ModelSamplerRow from "./generate/form/ModelSamplerRow.vue";
import PromptFields from "./generate/form/PromptFields.vue";
import SamplingParamsGrid from "./generate/form/SamplingParamsGrid.vue";
import TogglesRow from "./generate/form/TogglesRow.vue";
import GeneratePresetApplyRow from "./generate/presets/GeneratePresetApplyRow.vue";
import PromptPresetApplyRow from "./generate/presets/PromptPresetApplyRow.vue";
import CharacterPresetApplyRow from "./generate/presets/CharacterPresetApplyRow.vue";
import type {
  BaseGenerateRequest,
  Center,
  CharacterPrompt,
  GeneratePreset,
} from "@/api/types";
import { fileToBase64Payload } from "./fileBase64";

const SIZE_PRESETS = [
  "832x1216",
  "1216x832",
  "1024x1024",
  "1024x1536",
  "1536x1024",
  "1472x1472",
  "1088x1920",
  "1920x1088",
  "512x768",
  "768x768",
  "640x640",
] as const;
const metaStore = useMetaStore();

const props = defineProps<{
  modelHint?: string;
  hideSubmit?: boolean;
  presetMode?: "apply" | "manage";
}>();

const emit = defineEmits<{
  (e: "submit", req: BaseGenerateRequest): void;
}>();

const CHARACTER_SLOTS = 6;
const POSITION_OPTIONS = (() => {
  const opts: string[] = [];
  for (let li = 0; li < 5; li++) {
    const letter = String.fromCharCode(65 + li);
    for (let ni = 1; ni <= 5; ni++) {
      opts.push(`${letter}${ni}`);
    }
  }
  return opts;
})();

function emptyCharacterPrompt(): CharacterPrompt {
  return {
    prompt: "",
    uc: "",
    center: { x: 0.5, y: 0.5 },
    enabled: false,
  };
}

function positionToCenter(position: string): Center {
  const offset = 0.1;
  const letter = position[0] ?? "C";
  const number = position[1] ?? "3";
  const li = Math.min(4, Math.max(0, letter.toUpperCase().charCodeAt(0) - 65));
  const ni = Math.min(4, Math.max(0, parseInt(number, 10) - 1));
  return {
    x: Math.round((li * 0.2 + offset) * 10) / 10,
    y: Math.round((ni * 0.2 + offset) * 10) / 10,
  };
}

function centerToPosition(center: Center): string {
  const offset = 0.1;
  const grid = [0, 1, 2, 3, 4].map(
    (i) => Math.round((i * 0.2 + offset) * 10) / 10
  );
  const nearestIndex = (v: number) => {
    let bestIdx = 0;
    let bestDist = Number.POSITIVE_INFINITY;
    for (let i = 0; i < grid.length; i++) {
      const d = Math.abs(grid[i] - v);
      if (d < bestDist) {
        bestDist = d;
        bestIdx = i;
      }
    }
    return bestIdx;
  };
  const li = nearestIndex(center.x);
  const ni = nearestIndex(center.y);
  const letter = String.fromCharCode(65 + li);
  const number = String(ni + 1);
  return `${letter}${number}`;
}

function inflateCharacterSlots(
  enabled: CharacterPrompt[] | null | undefined
): CharacterPrompt[] {
  const slots: CharacterPrompt[] = Array.from(
    { length: CHARACTER_SLOTS },
    emptyCharacterPrompt
  );
  const src = Array.isArray(enabled) ? enabled : [];
  for (let i = 0; i < Math.min(CHARACTER_SLOTS, src.length); i++) {
    slots[i] = { ...src[i] };
  }
  return slots;
}

const form = reactive<BaseGenerateRequest>({
  model: props.modelHint ?? "nai-diffusion-4-5-full",
  positive: "",
  negative: "",
  quantity: 1,
  width: 832,
  height: 1216,
  steps: 27,
  scale: 5,
  sampler: "k_euler_ancestral",
  noise_schedule: null,
  cfg_rescale: null,
  seed: -1,
  add_quality_tags: true,
  undesired_content_preset: "None",
  sm: false,
  sm_dyn: false,
  use_coords: true,
  legacy_uc: false,
  character_prompts: inflateCharacterSlots([]),
  reference_image_multiple: [],
  reference_information_extracted_multiple: [],
  reference_strength_multiple: [],
});

const isV3Model = computed(
  () =>
    form.model === "nai-diffusion-3" || form.model === "nai-diffusion-furry-3"
);

const sizePreset = computed({
  get() {
    const k = `${form.width}x${form.height}`;
    return (SIZE_PRESETS as readonly string[]).includes(k) ? k : "自定义";
  },
  set(v: string) {
    if (!v || v === "自定义") return;
    const [w, h] = v.split("x").map((x) => parseInt(x, 10));
    if (!Number.isFinite(w) || !Number.isFinite(h)) return;
    form.width = w;
    form.height = h;
  },
});

// ---- Presets (prompt/character). Generate preset is now fully separated. ----

watch(
  () => props.modelHint,
  (v) => {
    if (v) form.model = v;
  }
);

watch(
  () => form.model,
  (m, prev) => {
    if (m === prev) return;
    applyPresetForModel(m);
  }
);

function applyPresetForModel(model: string) {
  // NOTE: 继承只包含提示词与角色；其余字段视为 preset（模型切换强制覆盖）。
  const isV3 = model === "nai-diffusion-3" || model === "nai-diffusion-furry-3";

  // Shared defaults (v4.5 tuned)
  form.quantity = 1;
  form.width = 832;
  form.height = 1216;
  form.steps = 27;
  form.scale = 5;
  form.sampler = "k_euler_ancestral";
  form.noise_schedule = null;
  form.cfg_rescale = null;
  form.seed = -1;
  form.add_quality_tags = true;
  form.undesired_content_preset = "None";

  // Clear reference (vibe transfer) when switching model.
  form.reference_image_multiple = [];
  form.reference_information_extracted_multiple = [];
  form.reference_strength_multiple = [];

  // Model-specific toggles
  if (isV3) {
    form.sm = false;
    form.sm_dyn = false;
    // keep v4 fields as-is; build() will null them out for v3.
  } else {
    form.use_coords = true;
    form.legacy_uc = false;
    // keep character_prompts (roles) for inheritance
  }
}

function applyPresetToForm(model: string, p: GeneratePreset) {
  const isV3 = model === "nai-diffusion-3" || model === "nai-diffusion-furry-3";

  form.quantity = p.quantity;
  form.width = p.width;
  form.height = p.height;
  form.steps = p.steps;
  form.scale = p.scale;
  form.sampler = p.sampler;
  form.noise_schedule = p.noise_schedule;
  form.cfg_rescale = p.cfg_rescale;
  form.seed = p.seed;
  form.add_quality_tags = p.add_quality_tags;
  form.undesired_content_preset = p.undesired_content_preset;

  // Clear reference (vibe transfer) when applying preset.
  form.reference_image_multiple = [];
  form.reference_information_extracted_multiple = [];
  form.reference_strength_multiple = [];

  if (isV3) {
    form.sm = p.sm;
    form.sm_dyn = p.sm_dyn;
  } else {
    form.use_coords = p.use_coords;
    form.legacy_uc = p.legacy_uc;
  }
}

// model/preset watchers are handled inside useGeneratePresets()

function onSubmit() {
  emit("submit", build());
}

function build(): BaseGenerateRequest {
  const enabledCharacters = !isV3Model.value
    ? (form.character_prompts ?? []).filter(
        (c) => c.enabled && (c.prompt.trim() || c.uc.trim())
      )
    : [];

  return {
    ...form,
    quantity: form.quantity && form.quantity > 1 ? form.quantity : null,
    noise_schedule: form.noise_schedule || null,
    cfg_rescale: form.cfg_rescale ?? null,
    add_quality_tags: form.add_quality_tags ?? null,
    undesired_content_preset: form.undesired_content_preset ?? null,
    sm: isV3Model.value ? form.sm ?? null : null,
    sm_dyn: isV3Model.value ? form.sm_dyn ?? null : null,
    use_coords: !isV3Model.value ? form.use_coords ?? null : null,
    legacy_uc: !isV3Model.value ? form.legacy_uc ?? null : null,
    character_prompts: !isV3Model.value
      ? enabledCharacters.length
        ? enabledCharacters
        : []
      : null,
    reference_image_multiple: form.reference_image_multiple?.length
      ? form.reference_image_multiple
      : null,
    reference_information_extracted_multiple: form
      .reference_information_extracted_multiple?.length
      ? form.reference_information_extracted_multiple
      : null,
    reference_strength_multiple: form.reference_strength_multiple?.length
      ? form.reference_strength_multiple
      : null,
  };
}

async function ensureMeta() {
  await metaStore.ensureLoaded();
  if (metaStore.models.length && !metaStore.models.includes(form.model)) {
    form.model = metaStore.models[0];
  }
  if (metaStore.samplers.length && !metaStore.samplers.includes(form.sampler)) {
    form.sampler = metaStore.samplers[0];
  }
}

void ensureMeta();

async function hydrateFromBackendLastGeneration() {
  try {
    const r = await endpoints.lastGenerationGet();
    if (!r.record) return;

    // 继承完整参数（其它参数默认由上一次生成固定）。
    const b = r.record.base;
    form.model = b.model ?? form.model;
    form.positive = b.positive ?? "";
    form.negative = b.negative ?? "";
    form.quantity = (b.quantity ?? 1) as any;
    form.width = b.width ?? form.width;
    form.height = b.height ?? form.height;
    form.steps = b.steps ?? form.steps;
    form.scale = b.scale ?? form.scale;
    form.sampler = b.sampler ?? form.sampler;
    form.noise_schedule = (b.noise_schedule ?? null) as any;
    form.cfg_rescale = (b.cfg_rescale ?? null) as any;
    form.seed = (b.seed ?? form.seed) as any;
    form.add_quality_tags = (b.add_quality_tags ??
      form.add_quality_tags) as any;
    form.undesired_content_preset = (b.undesired_content_preset ??
      form.undesired_content_preset) as any;
    form.sm = (b.sm ?? form.sm) as any;
    form.sm_dyn = (b.sm_dyn ?? form.sm_dyn) as any;
    form.use_coords = (b.use_coords ?? form.use_coords) as any;
    form.legacy_uc = (b.legacy_uc ?? form.legacy_uc) as any;

    form.character_prompts = inflateCharacterSlots(b.character_prompts ?? []);
    form.reference_image_multiple = b.reference_image_multiple ?? [];
    form.reference_information_extracted_multiple =
      b.reference_information_extracted_multiple ?? [];
    form.reference_strength_multiple = b.reference_strength_multiple ?? [];
  } catch {
    // ignore
  }
}

void hydrateFromBackendLastGeneration();

const advancedJson = computed({
  get() {
    return JSON.stringify(
      {
        reference_image_multiple: form.reference_image_multiple ?? [],
        reference_information_extracted_multiple:
          form.reference_information_extracted_multiple ?? [],
        reference_strength_multiple: form.reference_strength_multiple ?? [],
      },
      null,
      2
    );
  },
  set(v: string) {
    try {
      const parsed = JSON.parse(v) as any;
      form.reference_image_multiple = Array.isArray(
        parsed.reference_image_multiple
      )
        ? parsed.reference_image_multiple
        : [];
      form.reference_information_extracted_multiple = Array.isArray(
        parsed.reference_information_extracted_multiple
      )
        ? parsed.reference_information_extracted_multiple
        : [];
      form.reference_strength_multiple = Array.isArray(
        parsed.reference_strength_multiple
      )
        ? parsed.reference_strength_multiple
        : [];
    } catch {
      // ignore until user fixes json
    }
  },
});

async function onPickReferenceFiles(ev: Event) {
  const input = ev.target as HTMLInputElement;
  const files = Array.from(input.files ?? []);
  if (!files.length) return;

  for (const f of files) {
    const b64 = await fileToBase64Payload(f);
    form.reference_image_multiple ??= [];
    form.reference_information_extracted_multiple ??= [];
    form.reference_strength_multiple ??= [];
    form.reference_image_multiple.push(b64);
    form.reference_information_extracted_multiple.push(0);
    form.reference_strength_multiple.push(0.6);
  }

  // reset input so selecting same file again triggers change
  input.value = "";
}

function removeReference(idx: number) {
  form.reference_image_multiple?.splice(idx, 1);
  form.reference_information_extracted_multiple?.splice(idx, 1);
  form.reference_strength_multiple?.splice(idx, 1);
}
</script>

<template>
  <form class="grid gap-3" @submit.prevent="onSubmit">
    <ModelSamplerRow :form="form" />

    <PromptPresetApplyRow
      :form="form"
      :inflate-character-slots="inflateCharacterSlots"
    />

    <PromptFields :form="form" />

    <GeneratePresetApplyRow
      :model="form.model"
      :on-apply-preset-to-form="applyPresetToForm"
      :on-apply-defaults-for-model="applyPresetForModel"
    />

    <CoreParamsGrid
      :form="form"
      :size-preset="sizePreset"
      :size-presets="SIZE_PRESETS"
      @update:sizePreset="(v) => (sizePreset = v)"
    />
    <SamplingParamsGrid :form="form" />
    <TogglesRow :form="form" :is-v3-model="isV3Model" />

    <CharacterPresetApplyRow
      v-if="!isV3Model"
      :form="form"
      :slots="CHARACTER_SLOTS"
    />

    <CharacterCollapse
      v-if="!isV3Model"
      :position-options="POSITION_OPTIONS"
      :center-to-position="centerToPosition"
      :position-to-center="positionToCenter"
      :form="form"
    />

    <AdvancedJsonCollapse
      :form="form"
      :advanced-json="advancedJson"
      :on-pick-reference-files="onPickReferenceFiles"
      :on-remove-reference="removeReference"
      @update:advancedJson="(v) => (advancedJson = v)"
    />

    <div v-if="!props.hideSubmit">
      <button class="btn btn-primary" type="submit">生成</button>
    </div>
    <div v-else>
      <slot name="actions" :submit="onSubmit" :build="build" />
    </div>
  </form>
</template>
