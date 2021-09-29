<template>
  <div style="width: 100%; min-height: inherit; height: 100%">
    <editor style="min-height: inherit" v-model="src" />
  </div>
</template>

<script setup lang="ts">
import Editor from "@/components/Editor.vue";
import { ref } from "vue";

const src = ref("");
</script>
