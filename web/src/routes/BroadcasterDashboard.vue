<template>
  <div class="px-20 pt-5 xl:max-w-7xl mx-auto">
    <div v-if="loading">Loading...</div>
    <div v-else-if="error">
      Failed!
      <br />
      <pre>{{ error }}</pre>
    </div>

    <div v-else>
      <div v-if="broadcasters.length" class="flex flex-wrap gap-5">
        <RouterLink v-for="broadcaster of broadcasters" :key="broadcaster.id" :to="makeBroadcasterLink(broadcaster.id)">
          <div
            class="
              flex
              items-center
              flex-col
              gap-4
              bg-gray-300
              rounded-lg
              overflow-hidden
              border border-gray-900 border-opacity-30
              pt-4
              select-none
              cursor-pointer
              transform
              transition-transform transition-shadow
              drop-shadow-none
              hover:scale-105
              hover:shadow-light
            "
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
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue';
import { useApi } from '../api/plugin';
import { asyncRefs, tryAsync } from '../utilities';
import { TwitchUser } from '../api/types';

export default defineComponent({
  name: 'BroadcastersDashboard',
  components: {},
  setup() {
    const api = useApi();

    const { loading, error } = asyncRefs();

    const broadcasters = ref<TwitchUser[]>([]);

    tryAsync(
      async () => {
        broadcasters.value = await api.getBroadcasters();
      },
      loading,
      error,
    );

    const makeBroadcasterLink = (id: string) => {
      return `/rewards/${encodeURIComponent(id)}`;
    };

    return { loading, error, broadcasters, makeBroadcasterLink };
  },
});
</script>
