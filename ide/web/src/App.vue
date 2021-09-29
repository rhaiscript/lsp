<template>
  <q-layout view="hHh lpR lff">
    <q-header class="main-header">
      <q-toolbar>
        <div class="main-logo" v-html="rhaiLogo" />
        <q-toolbar-title> </q-toolbar-title>
      </q-toolbar>
    </q-header>

    <q-page-container>
      <q-page>
        <router-view />
      </q-page>
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import rhaiLogo from "@/assets/rhai-logo-transparent-colour-white.svg?raw";
import { useQuasar } from "quasar";

const q = useQuasar();

q.dark.set(true);
</script>

<style lang="scss">
.main-header {
  background-color: #3a2e24 !important;
}

.main-logo {
  width: auto;
  height: 2rem;
}
</style>
