<script setup lang="ts">
import { computed } from "vue";
import type { Center, CharacterPrompt } from "@/api/types";

const props = defineProps<{
  form: { character_prompts?: CharacterPrompt[] | null };
  positionOptions: string[];
  centerToPosition: (c: Center) => string;
  positionToCenter: (p: string) => Center;
}>();

const characters = computed(
  () => (props.form.character_prompts ?? []) as CharacterPrompt[]
);
</script>

<template>
  <div class="collapse collapse-arrow bg-base-200">
    <input type="checkbox" />
    <div class="collapse-title font-medium">角色分区（V4.5）</div>
    <div class="collapse-content">
      <div class="grid gap-3">
        <div
          v-for="(c, idx) in characters"
          :key="idx"
          class="card bg-base-100 border border-base-300"
        >
          <div class="card-body gap-3">
            <div class="flex flex-wrap items-center justify-between gap-3">
              <div class="font-medium badge badge-neutral">
                角色 {{ idx + 1 }}
              </div>
              <label
                class="flex items-center gap-3 rounded-lg border border-base-300/70 bg-base-100/70 px-3 py-2"
              >
                <input v-model="c.enabled" type="checkbox" class="toggle" />
                <span class="text-sm">{{ c.enabled ? "开启" : "关闭" }}</span>
              </label>
            </div>

            <template v-if="c.enabled">
              <fieldset class="fieldset">
                <span>位置</span>
                <select
                  class="select select-bordered w-full"
                  :value="props.centerToPosition(c.center)"
                  @change="(e) => (c.center = props.positionToCenter((e.target as HTMLSelectElement).value))"
                  :disabled="!c.enabled"
                >
                  <option
                    v-for="p in props.positionOptions"
                    :key="p"
                    :value="p"
                  >
                    {{ p }}
                  </option>
                </select>
              </fieldset>

              <div class="grid grid-cols-1 gap-3 lg:grid-cols-2">
                <fieldset class="fieldset">
                  <span>正向（角色）</span>
                  <textarea
                    v-model="c.prompt"
                    class="textarea textarea-bordered h-28 w-full"
                    placeholder="character positive prompt"
                    :disabled="!c.enabled"
                  />
                </fieldset>

                <fieldset class="fieldset">
                  <span>反向（角色）</span>
                  <textarea
                    v-model="c.uc"
                    class="textarea textarea-bordered h-28 w-full"
                    placeholder="character negative prompt"
                    :disabled="!c.enabled"
                  />
                </fieldset>
              </div>
            </template>
          </div>
        </div>

        <div class="text-xs opacity-70">
          位置 A1..E5 映射到中心坐标 0.1/0.3/0.5/0.7/0.9
        </div>
      </div>
    </div>
  </div>
</template>
