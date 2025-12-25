<script setup lang="ts">
import { useRouter } from "vue-router";
import PageShell from "@/components/layout/PageShell.vue";

const router = useRouter();
void router.replace("/presets/generate");
</script>

<template>
  <PageShell title="预设" subtitle="正在跳转到：/presets/generate" />
</template>
