<template>
  <div class="px-20 pt-5 xl:max-w-7xl mx-auto">
    <div v-if="state.loading"><CLoader /></div>
    <div v-else-if="state.error">
      Failed!
      <br />
      <span class="break-words font-mono">{{ state.error }}</span>
    </div>
    <div v-else class="flex flex-col gap-6">
      <Heading>Connections</Heading>
      <div class="flex flex-wrap">
        <div
          class="
            flex flex-col
            items-center
            justify-center
            gap-4
            bg-gray-350
            p-4
            rounded-lg
            border border-gray-900 border-opacity-30
            text-red
          "
        >
          <SpotifyIcon class="h-14 w-auto" />
          <div v-if="state.value.spotify" class="flex flex-col items-center gap-3">
            <span>Authorized</span>
            <OutlinedButton @click="revokeSpotifyAuth">Revoke</OutlinedButton>
          </div>
          <CButton v-else :href="spotifyUrl.value ?? ''" :disabled="!!(spotifyUrl.loading || spotifyUrl.error)"
            >Authorize</CButton
          >
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, watch } from 'vue';
import CLoader from '../components/core/CLoader.vue';
import CButton from '../components/core/CButton.vue';
import SpotifyIcon from '../components/icons/SpotifyIcon.vue';
import { useApi } from '../api/plugin';
import { asyncState, tryAsync } from '../async-state';
import { Connections } from '../api/types';
import Heading from '../components/core/Heading.vue';
import OutlinedButton from '../components/core/OutlinedButton.vue';

export default defineComponent({
  name: 'ConnectionsDashboard',
  components: { OutlinedButton, Heading, SpotifyIcon, CButton, CLoader },
  setup() {
    const api = useApi();
    const { state } = asyncState<Connections>({ spotify: false });
    const { state: spotifyUrl } = asyncState<string | null>(null);

    watch(
      () => state.value,
      state => {
        if (!state.spotify) {
          tryAsync(async url => {
            url.value = await api.getSpotifyUrl();
          }, spotifyUrl);
        }
      },
    );

    onMounted(() => {
      tryAsync(async state => {
        state.value = await api.getConnections();
      }, state);
    });

    const revokeSpotifyAuth = () =>
      tryAsync(async state => {
        await api.removeConnection('spotify');
        state.value = await api.getConnections();
      }, state);

    return { state, spotifyUrl, revokeSpotifyAuth };
  },
});
</script>
