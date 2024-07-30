<template>
  <div class="bg-gray-dark text-white w-screen h-screen overflow-x-hidden">
    <nav-bar />
    <router-view />
  </div>
</template>

<script setup lang="ts">
import { onBeforeMount } from 'vue';
import NavBar from './components/NavBar.vue';
import { useApi } from './api/plugin';
import { useDataStore } from './store';

const api = useApi();
const store = useDataStore();
onBeforeMount(() => {
  if (api.isAuthenticated.value) {
    api.getCurrentUser().then(user => (store.user.value = user));
  }
});
</script>
