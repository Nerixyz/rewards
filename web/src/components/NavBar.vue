<template>
  <nav v-if="isAuthenticated" class="w-full h-14 bg-gray-dark border-b border-opacity-10 border-white">
    <div class="mx-auto w-full max-w-7xl flex justify-between items-center h-full">
      <div class="flex flex-wrap divide-opacity-20 divide-white divide-solid divide-x">
        <div v-for="route of routes" :key="route.path" class="flex">
          <router-link
            :to="route.path.replace(/\/:.+$/, '')"
            :class="`mx-4 ${currentRoute.name === route.name ? 'nerix-underline-active' : ''} nerix-underline`"
          >
            {{ route.name }}
          </router-link>
        </div>
      </div>
      <div class="flex items-center gap-4 h-5/6">
        <a href="https://github.com/Nerixyz/rewards" target="_blank" title="GitHub Project">
          <GithubIcon />
        </a>
        <div
          v-if="!userLoading"
          class="flex gap-2 h-5/6 px-2 rounded-lg justify-center items-center select-none cursor-pointer hover:bg-gray-350"
          @click="toggleMenu"
        >
          <span>{{ userName }}</span>
          <img :src="userImage" alt="Profile Image" class="h-8 w-8 rounded-full" />
          <ChevronDown :class="[state.menuOpen ? 'rotate-180' : '', 'transform transition transition-transform']" />
          <div v-if="state.menuOpen" class="absolute block top-16 bg-gray-350 rounded-lg flex flex-col">
            <button
              class="uppercase bg-red m-2 rounded-md py-1 px-3 text-black font-bold hover:bg-red-dark"
              @click.stop="openLogout"
            >
              Logout
            </button>
            <button
              class="uppercase m-2 rounded-md py-1 px-3 text-sm text-red font-bold hover:bg-gray-400"
              @click.stop="openDelete"
            >
              Delete Account
            </button>
          </div>

          <!-- Logout Dialog -->
          <CDialog title="Are you sure?" :open="state.logoutDialogOpen">
            <DialogButtons>
              <OutlinedButton @click="closeAll">Cancel</OutlinedButton>
              <CButton @click="logout">Logout</CButton>
            </DialogButtons>
          </CDialog>

          <!-- Delete Account Dialog -->
          <CDialog
            title="Are you sure?"
            subtitle="All rewards and connections will be deleted!"
            :open="state.deleteDialogOpen"
          >
            <DialogButtons>
              <OutlinedButton @click="closeAll">Cancel</OutlinedButton>
              <CButton @click="deleteAccount">Delete Account</CButton>
            </DialogButtons>
          </CDialog>
        </div>
      </div>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { computed, reactive } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useApi } from '../api/plugin';
import { useDataStore } from '../store';
import ChevronDown from './icons/ChevronDown.vue';
import CDialog from './core/CDialog.vue';
import DialogButtons from './DialogButtons.vue';
import OutlinedButton from './core/OutlinedButton.vue';
import CButton from './core/CButton.vue';
import GithubIcon from './icons/GithubIcon.vue';

const router = useRouter();
const currentRoute = useRoute();
const store = useDataStore();
const api = useApi();

const state = reactive({
  menuOpen: false,
  deleteDialogOpen: false,
  logoutDialogOpen: false,
});
const withClose = (fn: () => unknown) => () => {
  state.menuOpen = false;
  state.deleteDialogOpen = false;
  state.logoutDialogOpen = false;
  fn();
};

const openDelete = withClose(() => (state.deleteDialogOpen = true));
const openLogout = withClose(() => (state.logoutDialogOpen = true));

const logout = withClose(() => {
  api.logout();
  router.replace('/');
});
const deleteAccount = withClose(() => {
  api.deleteAccount().then(() => {
    store.user.value = null;
    router.replace('/');
  });
});

const toggleMenu = () => (state.menuOpen = !state.menuOpen);

const closeAll = withClose(() => undefined);

const routes = router.getRoutes().filter(r => !!r.name && !r.meta['ignoreNav']);
const userImage = computed(() => store.user.value?.profile_image_url);
const userLoading = computed(() => !store.user.value);
const userName = computed(() => store.user.value?.login);
const isAuthenticated = api.isAuthenticated;
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
