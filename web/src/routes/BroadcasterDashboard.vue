<template>
  <MainLayout>
    <div v-if="state.loading"><CLoader /></div>
    <div v-else-if="state.error">
      Failed!
      <br />
      <span class="break-words font-mono">{{ state.error }}</span>
    </div>

    <div v-else>
      <div v-if="state.value.length" class="flex flex-wrap gap-5">
        <RouterLink v-for="broadcaster of state.value" :key="broadcaster.id" :to="makeBroadcasterLink(broadcaster.id)">
          <div
            class="flex items-center flex-col gap-4 bg-gray-300 rounded-lg overflow-hidden border border-gray-900 border-opacity-30 pt-4 select-none cursor-pointer transition drop-shadow-none hover:scale-105 hover:shadow-light"
          >
            <img
              :src="broadcaster.profile_image_url"
              :alt="`Profile image of ${broadcaster.login}`"
              class="w-10 h-10 rounded-full"
            />
            <h3>{{ broadcaster.login }}</h3>
            <div class="bg-red text-black font-bold uppercase w-full px-4 py-2">Edit Rewards</div>
          </div>
        </RouterLink>
      </div>
      <div v-else>
        It seems like noone has added you as an editor. Don't be sad
        <img class="w-5 h-5 inline" alt="FeelsOkayMan" src="https://cdn.frankerfacez.com/emote/145947/2" />
      </div>
    </div>
  </MainLayout>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import { useApi } from '../api/plugin';
import { TwitchUser } from '../api/types';
import { asyncState, tryAsync } from '../async-state';
import CLoader from '../components/core/CLoader.vue';
import MainLayout from '../components/MainLayout.vue';

export default defineComponent({
  name: 'BroadcastersDashboard',
  components: { MainLayout, CLoader },
  setup() {
    const api = useApi();

    const { state } = asyncState<TwitchUser[]>([]);

    tryAsync(async state => {
      state.value = await api.getBroadcasters();
    }, state);

    const makeBroadcasterLink = (id: string) => {
      return `/rewards/${encodeURIComponent(id)}`;
    };

    return { state, makeBroadcasterLink };
  },
});
</script>
