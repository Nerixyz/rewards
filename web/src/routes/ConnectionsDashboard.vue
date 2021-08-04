<template>
  <MainLayout>
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
            <div class="mx-1 my-3 text-white">
              <h4 class="font-serif text-xl mb-3 border-b border-white border-opacity-30">Settings</h4>
              <CSwitch
                v-model="state.value.spotify.only_while_live"
                label="Allow control only while live"
                @update:model-value="sendSpotifySettings"
              />
            </div>
            <OutlinedButton @click="revokeSpotifyAuth">Revoke</OutlinedButton>
          </div>
          <CButton v-else :href="spotifyUrl.value ?? ''" :disabled="!!(spotifyUrl.loading || spotifyUrl.error)"
            >Authorize</CButton
          >
        </div>
      </div>
    </div>
  </MainLayout>
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
import CSwitch from '../components/core/CSwitch.vue';
import MainLayout from '../components/MainLayout.vue';

export default defineComponent({
  name: 'ConnectionsDashboard',
  components: { MainLayout, CSwitch, OutlinedButton, Heading, SpotifyIcon, CButton, CLoader },
  setup() {
    const api = useApi();
    const { state } = asyncState<Connections>({ spotify: undefined });
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

    const sendSpotifySettings = () => {
      tryAsync(async state => {
        if (state.value.spotify) {
          await api.updateSpotifySettings(state.value.spotify);
        }
      }, state);
    };

    return { state, spotifyUrl, revokeSpotifyAuth, sendSpotifySettings };
  },
});
</script>
