import { createRouter, createWebHistory } from "vue-router";

import HomeView from "@/views/HomeView.vue";
import GenerateT2IView from "@/views/generate/GenerateT2IView.vue";
import GenerateI2IView from "@/views/generate/GenerateI2IView.vue";
import GenerateInpaintView from "@/views/generate/GenerateInpaintView.vue";
import GenerateCharacterView from "@/views/generate/GenerateCharacterView.vue";
import GeneratePresetsView from "@/views/generate/GeneratePresetsView.vue";
import PresetsGenerateView from "@/views/presets/PresetsGenerateView.vue";
import PresetsPromptView from "@/views/presets/PresetsPromptView.vue";
import PresetsCharacterView from "@/views/presets/PresetsCharacterView.vue";
import PresetsSnippetView from "@/views/presets/PresetsSnippetView.vue";
import DirectorView from "@/views/DirectorView.vue";
import JobsView from "@/views/JobsView.vue";
import OutputsView from "@/views/OutputsView.vue";

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", name: "home", component: HomeView },
    {
      path: "/generate",
      redirect: "/generate/t2i",
    },
    { path: "/generate/t2i", name: "generate-t2i", component: GenerateT2IView },
    { path: "/generate/i2i", name: "generate-i2i", component: GenerateI2IView },
    {
      path: "/generate/inpaint",
      name: "generate-inpaint",
      component: GenerateInpaintView,
    },
    {
      path: "/generate/character",
      name: "generate-character",
      component: GenerateCharacterView,
    },
    {
      path: "/generate/presets",
      name: "generate-presets",
      component: GeneratePresetsView,
    },
    {
      path: "/presets",
      redirect: "/presets/generate",
    },
    {
      path: "/presets/generate",
      name: "presets-generate",
      component: PresetsGenerateView,
    },
    {
      path: "/presets/prompt",
      name: "presets-prompt",
      component: PresetsPromptView,
    },
    {
      path: "/presets/snippet",
      name: "presets-snippet",
      component: PresetsSnippetView,
    },
    {
      path: "/presets/character",
      name: "presets-character",
      component: PresetsCharacterView,
    },
    { path: "/director", name: "director", component: DirectorView },
    { path: "/jobs", name: "jobs", component: JobsView },
    { path: "/outputs", name: "outputs", component: OutputsView },
  ],
});
