<template>
  <div class="bg-gray-dark text-white w-screen h-screen overflow-hidden">
      <nav-bar/>
      <router-view/>
  </div>
</template>

<script lang="ts">
import { defineComponent, onBeforeMount } from 'vue';
import NavBar from './components/NavBar.vue';
import { useApi } from './api/plugin';
import { useDataStore } from './store';

export default defineComponent({
  name: 'App',
  components: {
    NavBar,
  },
  setup() {
    const api = useApi();
    const store = useDataStore();
    onBeforeMount(() => {
      if(api.isAuthenticated.value) {
        api.getCurrentUser().then(user => store.user.value = user);
      }
    });

    return {auth: api.isAuthenticated};
  }
})
</script>
