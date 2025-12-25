export type Health = { ok: boolean };

export type Meta = {
  models: string[];
  samplers: string[];
  noise_schedules: string[];
  uc_presets: string[];
};

export type Anlas = { anlas: number };

export type Center = { x: number; y: number };

export type CharacterPrompt = {
  prompt: string;
  uc: string;
  center: Center;
  enabled: boolean;
};

export type BaseGenerateRequest = {
  model: string;
  positive: string;
  negative: string;
  quantity?: number | null;
  width: number;
  height: number;
  steps: number;
  scale: number;
  sampler: string;
  noise_schedule?: string | null;
  cfg_rescale?: number | null;
  seed: number;
  add_quality_tags?: boolean | null;
  undesired_content_preset?: string | null;
  sm?: boolean | null;
  sm_dyn?: boolean | null;
  use_coords?: boolean | null;
  legacy_uc?: boolean | null;
  character_prompts?: CharacterPrompt[] | null;
  reference_image_multiple?: string[] | null;
  reference_information_extracted_multiple?: number[] | null;
  reference_strength_multiple?: number[] | null;
};

export type Img2ImgRequest = BaseGenerateRequest & {
  image_base64: string;
  strength: number;
  noise: number;
  extra_noise_seed?: number | null;
  color_correct?: boolean | null;
};

export type InpaintRequest = BaseGenerateRequest & {
  image_base64: string;
  mask_base64: string;
  strength: number;
  noise: number;
  extra_noise_seed?: number | null;
  color_correct?: boolean | null;
};

export type CharacterRequest = BaseGenerateRequest & {
  character_reference_image_base64: string;
  style_aware: boolean;
  fidelity: number;
};

export type GenerateResponse = {
  seed: number;
  output_path: string;
  url: string;
};

export type OutputItem = {
  path: string;
  op_type: string;
  date: string;
  filename: string;
};

export type OutputsResponse = {
  items: OutputItem[];
  next_offset: number;
  has_more: boolean;
};

export type OutputsDeleteRequest = {
  items: string[];
};

export type OutputsDeleteResponse = {
  deleted: number;
};

export type JobSubmitResponse = {
  job_id: string;
};

export type JobStatus =
  | { status: "queued" }
  | { status: "running" }
  | { status: "cancelled" }
  | { status: "failed"; error: string }
  | { status: "succeeded"; outputs: GenerateResponse[] };

export type JobSummary = {
  id: string;
  kind: string;
  created_at_ms: number;
  started_at_ms: number | null;
  finished_at_ms: number | null;
  updated_at_ms: number;
  status: JobStatus;
};

export type JobsListResponse = {
  items: JobSummary[];
};

export type DirectorRequest = {
  width: number;
  height: number;
  image_base64: string;
};

export type DirectorPromptRequest = DirectorRequest & {
  prompt: string;
  defry: number;
};

export type DirectorResponse = {
  output_paths: string[];
};

export type LastGenerationRecord = {
  updated_at_ms: number;
  base: BaseGenerateRequest;
};

export type LastGenerationGetResponse = {
  record: LastGenerationRecord | null;
};

export type LastGenerationPutRequest = BaseGenerateRequest;

export type GeneratePreset = {
  quantity: number;
  width: number;
  height: number;
  steps: number;
  scale: number;
  sampler: string;
  noise_schedule: string | null;
  cfg_rescale: number | null;
  seed: number;
  add_quality_tags: boolean;
  undesired_content_preset: string;
  sm: boolean;
  sm_dyn: boolean;
  use_coords: boolean;
  legacy_uc: boolean;
};

export type PresetsListResponse = {
  names: string[];
};

export type PresetGetResponse = {
  preset: GeneratePreset | null;
};

export type PresetPutRequest = {
  model: string;
  name: string;
  preset: GeneratePreset;
};

export type PresetRenameRequest = {
  model: string;
  from: string;
  to: string;
};

export type PromptPreset = {
  positive: string;
  negative: string;
  add_quality_tags: boolean | null;
  undesired_content_preset: string | null;
  character_prompts: CharacterPrompt[];
};

export type PromptPresetsListResponse = {
  names: string[];
};

export type PromptPresetGetResponse = {
  preset: PromptPreset | null;
};

export type PromptPresetPutRequest = {
  name: string;
  preset: PromptPreset;
};

export type PromptPresetRenameRequest = {
  from: string;
  to: string;
};

export type PromptSnippet = {
  body: string;
  tags: string[];
  description?: string | null;
};

export type PromptSnippetSummary = {
  name: string;
  tags: string[];
  description?: string | null;
};

export type PromptSnippetsListResponse = {
  items: PromptSnippetSummary[];
};

export type PromptSnippetGetResponse = {
  snippet: PromptSnippet | null;
};

export type PromptSnippetPutRequest = {
  name: string;
  snippet: PromptSnippet;
};

export type PromptSnippetRenameRequest = {
  from: string;
  to: string;
};

export type PromptSnippetPreviewRequest = {
  positive: string;
  negative: string;
};

export type PromptSnippetPreviewResponse = {
  positive: string;
  negative: string;
  warnings: string[];
};

export type CharacterSlotPreset = {
  prompt: string;
  uc: string;
  center: Center;
};

export type CharacterPresetsListResponse = {
  names: string[];
};

export type CharacterPresetGetResponse = {
  preset: CharacterSlotPreset | null;
};

export type CharacterPresetPutRequest = {
  name: string;
  preset: CharacterSlotPreset;
};

export type CharacterPresetRenameRequest = {
  from: string;
  to: string;
};
