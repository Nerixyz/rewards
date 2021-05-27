<template>
  <nav v-if="isAuthenticated" class="w-full h-14 flex bg-gray-dark shadow-light justify-between px-10 items-center">
    <div class="flex">
      <router-link
        v-for="route of routes"
        :key="route.path"
        :to="route.path.replace(/\/:.+$/, '')"
        :class="`mx-3 ${route.name === currentRoute.name ? 'nerix-underline-active' : ''} nerix-underline`"
      >
        {{ route.name }}
      </router-link>
    </div>
    <img v-if="!userLoading" :src="userImage" alt="Profile Image" class="h-8 w-8 rounded-full" />
  </nav>
</template>

<script lang="ts">
import { computed, defineComponent } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useApi } from '../api/plugin';
import { useDataStore } from '../store';

export default defineComponent({
  name: 'NavBar',
  setup() {
    const router = useRouter();
    const route = useRoute();
    const store = useDataStore();
    const api = useApi();

    return {
      routes: router.getRoutes().filter(r => !!r.name),
      currentRoute: route,
      userImage: computed(() => store.user.value?.profile_image_url),
      userLoading: computed(() => !store.user.value),
      isAuthenticated: api.isAuthenticated,
    };
  },
});
</script>
<style scoped>
.nerix-underline::after {
  content: '';
  position: relative;
  display: block;
  width: 100%;
  height: 0.15rem;
  background-color: #ff4151;
  transform: scaleX(0);
  transition: transform 150ms;
  transform-origin: right;
  border-radius: 2px;
}

.nerix-underline:hover::after, .nerix-underline-active::after {
  transform: scaleX(1);
  transform-origin: left;
}
</style>
