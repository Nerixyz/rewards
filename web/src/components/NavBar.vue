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
    <div
      v-if="!userLoading"
      class="flex gap-2 h-5/6 px-2 rounded-lg justify-center items-center select-none cursor-pointer hover:bg-gray-350"
      @click="openMenu"
    >
      <span>{{ userName }}</span>
      <img :src="userImage" alt="Profile Image" class="h-8 w-8 rounded-full" />
      <ChevronDown :class="[userMenuState.menuOpen ? 'rotate-180' : '', 'transform transition transition-transform']" />
      <div v-if="userMenuState.menuOpen" class="absolute block top-16 bg-gray-350 rounded-lg">
        <button
          class="uppercase bg-red m-2 rounded-md py-1 px-3 text-black font-bold hover:bg-red-dark"
          @click.stop="toggleDialog"
        >
          Logout
        </button>
      </div>
      <CDialog
        title="Are you sure?"
        subtitle="All rewards and connections will be deleted"
        :open="userMenuState.dialogOpen"
      >
        <DialogButtons>
          <OutlinedButton @click="toggleDialog">Cancel</OutlinedButton>
          <CButton @click="logout">Logout</CButton>
        </DialogButtons>
      </CDialog>
    </div>
  </nav>
</template>

<script lang="ts">
import { computed, defineComponent, reactive } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useApi } from '../api/plugin';
import { useDataStore } from '../store';
import ChevronDown from './icons/ChevronDown.vue';
import CDialog from './core/CDialog.vue';
import DialogButtons from './DialogButtons.vue';
import OutlinedButton from './core/OutlinedButton.vue';
import CButton from './core/CButton.vue';

export default defineComponent({
  name: 'NavBar',
  components: { CButton, OutlinedButton, DialogButtons, CDialog, ChevronDown },
  setup() {
    const router = useRouter();
    const route = useRoute();
    const store = useDataStore();
    const api = useApi();

    const userMenuState = reactive({
      dialogOpen: false,
      menuOpen: false,
    });
    const openMenu = () => {
      userMenuState.menuOpen = !userMenuState.menuOpen;
    };
    const toggleDialog = () => {
      userMenuState.menuOpen = false;
      userMenuState.dialogOpen = !userMenuState.dialogOpen;
    };
    const logout = () => {
      api.logout().then(() => {
        store.user.value = null;
        router.replace('/');
      });
    };

    return {
      routes: router.getRoutes().filter(r => !!r.name && !r.meta.ignoreNav),
      currentRoute: route,
      userImage: computed(() => store.user.value?.profile_image_url),
      userLoading: computed(() => !store.user.value),
      userName: computed(() => store.user.value?.login),
      isAuthenticated: api.isAuthenticated,
      userMenuState,
      openMenu,
      toggleDialog,
      logout,
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

.nerix-underline:hover::after,
.nerix-underline-active::after {
  transform: scaleX(1);
  transform-origin: left;
}
</style>
