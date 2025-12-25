<script setup lang="ts">
import type { BaseGenerateRequest } from "@/api/types";

const props = defineProps<{
  form: BaseGenerateRequest;
  advancedJson: string;
  onPickReferenceFiles: (ev: Event) => void;
  onRemoveReference: (idx: number) => void;
}>();

const emit = defineEmits<{
  (e: "update:advancedJson", v: string): void;
}>();
</script>

<template>
  <div class="collapse collapse-arrow bg-base-200">
    <input type="checkbox" />
    <div class="collapse-title font-medium">高级字段（JSON）</div>
    <div class="collapse-content">
      <div class="grid gap-3">
        <div class="font-medium text-sm">参考图（vibe transfer）</div>
        <input
          class="file-input file-input-bordered"
          type="file"
          accept="image/*"
          multiple
          @change="props.onPickReferenceFiles"
        />

        <div
          v-if="props.form.reference_image_multiple?.length"
          class="overflow-auto"
        >
          <table class="table table-zebra">
            <thead>
              <tr>
                <th>#</th>
                <th>info_extracted</th>
                <th>strength</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="(_, idx) in props.form.reference_image_multiple"
                :key="idx"
              >
                <td class="font-mono text-xs">{{ idx }}</td>
                <td>
                  <input
                    v-model.number="props.form.reference_information_extracted_multiple![idx]"
                    class="input input-bordered input-sm w-24"
                    type="number"
                    step="1"
                  />
                </td>
                <td>
                  <input
                    v-model.number="props.form.reference_strength_multiple![idx]"
                    class="input input-bordered input-sm w-28"
                    type="number"
                    step="0.01"
                    min="0"
                    max="2"
                  />
                </td>
                <td>
                  <button
                    class="btn btn-ghost btn-sm"
                    type="button"
                    @click="props.onRemoveReference(idx)"
                  >
                    移除
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
          <div class="text-xs opacity-70 mt-2">
            参考图会自动转成 raw base64（无 data: 前缀）。
          </div>
        </div>
      </div>

      <textarea
        :value="props.advancedJson"
        class="textarea textarea-bordered h-56 w-full font-mono text-xs"
        @input="
          emit(
            'update:advancedJson',
            ($event.target as HTMLTextAreaElement).value
          )
        "
      />
      <div class="text-xs opacity-70 mt-2">
        这里可直接编辑：reference_*（vibe transfer pass-through）。图片请填 raw
        base64（无 data: 前缀）。
      </div>
    </div>
  </div>
</template>
