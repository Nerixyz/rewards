<template>
  <div class="px-20 pt-5">
    <div v-if="loading">Loading...</div>
    <div v-else-if="error">
      Failed!
      <br />
      <pre>{{ error }}</pre>
    </div>

    <div v-else>
      <div
        v-if="broadcasters.length"
        v-for="broadcaster of broadcasters"
        :key="broadcaster.id"
        class="flex items-center gap-4"
      >
        <img
          :src="broadcaster.profile_image_url"
          :alt="`Profile image of ${broadcaster.login}`"
          class="w-10 h-10 rounded-full"
        />
        <h3>{{ broadcaster.login }}</h3>
        <CButton @click="goToBroadcaster(broadcaster.id)" :disabled="loading" class="ml-auto">Edit Rewards</CButton>
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
import CButton from '../components/core/CButton.vue';
import { useApi } from '../api/plugin';
import { useRouter } from 'vue-router';
import { asyncRefs, tryAsync } from '../utilities';
import { TwitchUser } from '../api/types';

export default defineComponent({
  name: 'BroadcastersDashboard',
  components: { CButton },
  setup() {
    const api = useApi();
    const router = useRouter();

    const { loading, error } = asyncRefs();

    const broadcasters = ref<TwitchUser[]>([]);

    tryAsync(
      async () => {
        broadcasters.value = await api.getBroadcasters();
      },
      loading,
      error,
    );

    const goToBroadcaster = (id: string) => {
      router.push(`/rewards/${encodeURIComponent(id)}`);
    };

    return { loading, error, broadcasters, goToBroadcaster };
  },
});
</script>
