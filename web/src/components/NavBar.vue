<template>
  <nav v-if="userImage" class="w-full h-14 flex bg-gray-light shadow-light justify-between px-10 items-center">
    <div class="flex">
      <router-link
        v-for="route of routes"
        :key="route.path"
        :to="route.path.replace(/\/:.+$/, '')"
        :class="`mx-3 border-red ${route.name === currentRoute.name ? 'border-b-2' : ''} hover:border-b-2`"
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
import { useDataStore } from '../store';

export default defineComponent({
  name: 'NavBar',
  setup() {
    const router = useRouter();
    const route = useRoute();
    const store = useDataStore();

    return {
      routes: router.getRoutes().filter(r => !!r.name),
      currentRoute: route,
      userImage: computed(() => store.user.value?.profile_image_url),
      userLoading: computed(() => !store.user.value),
    };
  },
});
</script>
