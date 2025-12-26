<script setup lang="ts">
import { computed, ref } from "vue";
import type { PropType } from "vue";
import {
  defaultHighlightConfig,
  renderHighlighted,
  type WeightHighlightConfig,
} from "./promptHighlighter";

const props = defineProps({
  modelValue: { type: String, default: "" },
  placeholder: { type: String, default: "" },
  rows: { type: Number, default: 6 },
  disabled: { type: Boolean, default: false },
  textareaClass: { type: String, default: "" },
  config: {
    type: Object as PropType<Partial<WeightHighlightConfig>>,
    default: () => ({}),
  },
});

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
  (e: "focus", ev: FocusEvent): void;
  (e: "blur", ev: FocusEvent): void;
  (e: "input", value: string): void;
}>();

const textareaRef = ref<HTMLTextAreaElement | null>(null);
const mergedConfig = computed<WeightHighlightConfig>(() => ({
  ...defaultHighlightConfig,
  ...(props.config ?? {}),
}));

const cssVars = computed(() => ({
  "--prompt-highlight-neutral": mergedConfig.value.neutralColor,
  "--prompt-highlight-colon": mergedConfig.value.colonColor,
}));

const renderedHtml = computed(() =>
  renderHighlighted(props.modelValue ?? "", mergedConfig.value)
);

function onInput(ev: Event) {
  const value = (ev.target as HTMLTextAreaElement).value;
  emit("update:modelValue", value);
  emit("input", value);
}

defineExpose({
  textareaEl: textareaRef,
  getTextarea: () => textareaRef.value,
});
</script>

<template>
  <div class="relative w-full space-y-2" :style="cssVars">
    <textarea
      ref="textareaRef"
      :value="modelValue"
      :rows="rows"
      :disabled="disabled"
      :placeholder="placeholder"
      class="textarea textarea-bordered w-full resize-y px-3 py-2 text-sm leading-relaxed min-h-28"
      :class="textareaClass"
      spellcheck="false"
      @input="onInput"
      @focus="(e) => emit('focus', e)"
      @blur="(e) => emit('blur', e)"
    />

    <div
      class="rounded-box border border-base-300/70 bg-base-100 px-3 py-2 text-sm leading-relaxed whitespace-pre-wrap wrap-break-word text-base-content/80"
    >
      <span v-if="!modelValue && placeholder" class="text-base-content/50">
        {{ placeholder }}
      </span>
      <span v-else v-html="renderedHtml" />
    </div>
  </div>
</template>

<style scoped>
.prompt-bracket {
  color: var(--prompt-highlight-neutral, #6b7280);
}

.prompt-mix-sep {
  color: var(--prompt-highlight-neutral, #6b7280);
}

.prompt-weight-close {
  color: var(--prompt-highlight-colon, #16a34a);
  font-weight: 600;
}

.prompt-random,
.prompt-random-sep {
  color: var(--prompt-highlight-colon, #16a34a);
  font-weight: 600;
}
</style>
