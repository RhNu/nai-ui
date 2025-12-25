import { apiDelete, apiGet, apiPost, apiPut } from "./client";
import type {
  Anlas,
  BaseGenerateRequest,
  CharacterPresetGetResponse,
  CharacterPresetPutRequest,
  CharacterPresetRenameRequest,
  CharacterPresetsListResponse,
  CharacterRequest,
  DirectorPromptRequest,
  DirectorRequest,
  DirectorResponse,
  GenerateResponse,
  Health,
  Img2ImgRequest,
  InpaintRequest,
  JobStatus,
  JobSubmitResponse,
  JobsListResponse,
  LastGenerationGetResponse,
  LastGenerationPutRequest,
  Meta,
  OutputsDeleteRequest,
  OutputsDeleteResponse,
  OutputsResponse,
  PresetGetResponse,
  PresetPutRequest,
  PresetRenameRequest,
  PresetsListResponse,
  PromptPresetGetResponse,
  PromptPresetPutRequest,
  PromptPresetRenameRequest,
  PromptPresetsListResponse,
} from "./types";

export const endpoints = {
  health: () => apiGet<Health>("/api/health"),
  meta: () => apiGet<Meta>("/api/meta"),
  anlas: () => apiGet<Anlas>("/api/anlas"),
  outputs: () => apiGet<OutputsResponse>("/api/outputs"),
  outputsDelete: (req: OutputsDeleteRequest) =>
    apiPost<OutputsDeleteRequest, OutputsDeleteResponse>(
      "/api/outputs/delete",
      req
    ),

  lastGenerationGet: () =>
    apiGet<LastGenerationGetResponse>("/api/last_generation"),
  lastGenerationPut: (req: LastGenerationPutRequest) =>
    apiPut<LastGenerationPutRequest, { ok: boolean }>(
      "/api/last_generation",
      req
    ),
  lastGenerationDelete: () =>
    apiDelete<{ ok: boolean }>("/api/last_generation"),

  presetsList: (model: string) =>
    apiGet<PresetsListResponse>(`/api/presets/${encodeURIComponent(model)}`),
  presetGet: (model: string, name: string) =>
    apiGet<PresetGetResponse>(
      `/api/preset?model=${encodeURIComponent(model)}&name=${encodeURIComponent(
        name
      )}`
    ),
  presetPut: (req: PresetPutRequest) =>
    apiPut<PresetPutRequest, { ok: boolean }>("/api/preset", req),
  presetDelete: (model: string, name: string) =>
    apiDelete<{ ok: boolean }>(
      `/api/preset?model=${encodeURIComponent(model)}&name=${encodeURIComponent(
        name
      )}`
    ),
  presetRename: (req: PresetRenameRequest) =>
    apiPost<PresetRenameRequest, { ok: boolean }>("/api/preset/rename", req),

  promptPresetsList: () =>
    apiGet<PromptPresetsListResponse>("/api/prompt_presets"),
  promptPresetGet: (name: string) =>
    apiGet<PromptPresetGetResponse>(
      `/api/prompt_preset?name=${encodeURIComponent(name)}`
    ),
  promptPresetPut: (req: PromptPresetPutRequest) =>
    apiPut<PromptPresetPutRequest, { ok: boolean }>("/api/prompt_preset", req),
  promptPresetDelete: (name: string) =>
    apiDelete<{ ok: boolean }>(
      `/api/prompt_preset?name=${encodeURIComponent(name)}`
    ),
  promptPresetRename: (req: PromptPresetRenameRequest) =>
    apiPost<PromptPresetRenameRequest, { ok: boolean }>(
      "/api/prompt_preset/rename",
      req
    ),

  characterPresetsList: () =>
    apiGet<CharacterPresetsListResponse>("/api/character_presets"),
  characterPresetGet: (name: string) =>
    apiGet<CharacterPresetGetResponse>(
      `/api/character_preset?name=${encodeURIComponent(name)}`
    ),
  characterPresetPut: (req: CharacterPresetPutRequest) =>
    apiPut<CharacterPresetPutRequest, { ok: boolean }>(
      "/api/character_preset",
      req
    ),
  characterPresetDelete: (name: string) =>
    apiDelete<{ ok: boolean }>(
      `/api/character_preset?name=${encodeURIComponent(name)}`
    ),
  characterPresetRename: (req: CharacterPresetRenameRequest) =>
    apiPost<CharacterPresetRenameRequest, { ok: boolean }>(
      "/api/character_preset/rename",
      req
    ),

  generateT2i: (req: BaseGenerateRequest) =>
    apiPost<BaseGenerateRequest, GenerateResponse>("/api/generate/t2i", req),
  generateI2i: (req: Img2ImgRequest) =>
    apiPost<Img2ImgRequest, GenerateResponse>("/api/generate/i2i", req),
  generateInpaint: (req: InpaintRequest) =>
    apiPost<InpaintRequest, GenerateResponse>("/api/generate/inpaint", req),
  generateCharacter: (req: CharacterRequest) =>
    apiPost<CharacterRequest, GenerateResponse>("/api/generate/character", req),

  jobT2i: (req: BaseGenerateRequest) =>
    apiPost<BaseGenerateRequest, JobSubmitResponse>("/api/jobs/t2i", req),
  jobI2i: (req: Img2ImgRequest) =>
    apiPost<Img2ImgRequest, JobSubmitResponse>("/api/jobs/i2i", req),
  jobInpaint: (req: InpaintRequest) =>
    apiPost<InpaintRequest, JobSubmitResponse>("/api/jobs/inpaint", req),
  jobCharacter: (req: CharacterRequest) =>
    apiPost<CharacterRequest, JobSubmitResponse>("/api/jobs/character", req),
  jobsList: () => apiGet<JobsListResponse>("/api/jobs"),
  jobStatus: (id: string) => apiGet<JobStatus>(`/api/jobs/${id}`),

  directorRemoveBg: (req: DirectorRequest) =>
    apiPost<DirectorRequest, DirectorResponse>("/api/director/remove_bg", req),
  directorLineArt: (req: DirectorRequest) =>
    apiPost<DirectorRequest, DirectorResponse>("/api/director/line_art", req),
  directorSketch: (req: DirectorRequest) =>
    apiPost<DirectorRequest, DirectorResponse>("/api/director/sketch", req),
  directorDeclutter: (req: DirectorRequest) =>
    apiPost<DirectorRequest, DirectorResponse>("/api/director/declutter", req),
  directorColorize: (req: DirectorPromptRequest) =>
    apiPost<DirectorPromptRequest, DirectorResponse>(
      "/api/director/colorize",
      req
    ),
  directorEmotion: (req: DirectorPromptRequest) =>
    apiPost<DirectorPromptRequest, DirectorResponse>(
      "/api/director/emotion",
      req
    ),
};
