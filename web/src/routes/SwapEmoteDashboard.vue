<template>
  <MainLayout>
    <div v-if="state.loading">
      <CLoader />
    </div>
    <div v-else-if="state.error">
      Something went wrong.
      <br />
      <br />
      <span class="break-words font-mono">{{ state.error }}</span>
      <CButton @click="$router.back()">Go Back</CButton>
    </div>
    <div v-else-if="!state.value">No data.</div>
    <div v-else>
      <div class="flex gap-4 items-center mb-5">
        <BackIcon
          class="text-red bg-white bg-opacity-0 w-10 h-10 p-2 rounded-full cursor-pointer hover:bg-opacity-20"
          @click="$router.back()"
        />
        <h1 class="font-bold font-mono text-5xl">Emote Dashboard</h1>
      </div>
      <h2 class="font-mono text-xl bg-white bg-opacity-10 p-4 rounded-md overflow-hidden text-ellipsis text-nowrap">
        {{ state.value.twitch.title }}
      </h2>
      <div class="flex items-center">
        <OutlinedButton @click="loadState"><ReloadIcon class="mr-2" /> Refresh </OutlinedButton>
        <OutlinedButton @click="onSync"><SyncIcon class="mr-2" /> Sync with {{ platformName }}</OutlinedButton>
        <span class="ml-auto mr-6 text-xl">{{ state.value.emotes.length }}/{{ limitStr }}</span>
      </div>
      <SwapEmotesTable
        :items="state.value.emotes"
        :broadcaster-id="state.value.twitch.broadcaster_id"
        :reward-id="state.value.twitch.id"
        @untrack="onUntrack"
      />
    </div>

    <CDialog title="Untracking Emote" :open="untrackDialog.value" @dialog-closed="clearUntrackDialog">
      <div v-if="untrackDialog.loading"><CLoader /></div>
      <div v-else-if="untrackDialog.error">
        Failed to untrack emote: {{ untrackDialog.error }}
        <DialogButtons>
          <CButton @click="clearUntrackDialog">OK</CButton>
        </DialogButtons>
      </div>
      <div v-else-if="untrackDialog.success"><TickIcon /></div>
    </CDialog>

    <CDialog title="Syncing Emotes" :open="syncDialog.value" @dialog-closed="clearSyncDialog">
      <div v-if="syncDialog.loading"><CLoader /></div>
      <div v-else-if="syncDialog.error">
        Failed to sync emotes: {{ syncDialog.error }}
        <DialogButtons>
          <CButton @click="clearSyncDialog">OK</CButton>
        </DialogButtons>
      </div>
      <div v-else-if="syncDialog.success">
        <TickIcon />
        Untracked {{ nRemovedEmotes }} emotes that are already removed on {{ platformName }}.
        <DialogButtons>
          <CButton @click="clearSyncDialog">OK</CButton>
        </DialogButtons>
      </div>
    </CDialog>
  </MainLayout>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import MainLayout from '../components/MainLayout.vue';
import { useDataStore } from '../store';
import { useApi } from '../api/plugin';
import { useBroadcaster } from '../hooks/use-broadcaster';
import { useRoute } from 'vue-router';
import CLoader from '../components/core/CLoader.vue';
import CDialog from '../components/core/CDialog.vue';
import DialogButtons from '../components/DialogButtons.vue';
import { ListSwapEmotesResponse } from '../api/types';
import { asyncDialog, asyncState, tryAsync, tryAsyncDialog } from '../async-state';
import CButton from '../components/core/CButton.vue';
import BackIcon from '../components/icons/BackIcon.vue';
import SwapEmotesTable from '../components/SwapEmotesTable.vue';
import OutlinedButton from '../components/core/OutlinedButton.vue';
import TickIcon from '../components/icons/TickIcon.vue';
import ReloadIcon from '../components/icons/ReloadIcon.vue';
import SyncIcon from '../components/icons/SyncIcon.vue';

const store = useDataStore();
const api = useApi();
const route = useRoute();
const { broadcasterId } = useBroadcaster({ store, route });

const { state } = asyncState<ListSwapEmotesResponse | null>(null, true);
const { state: untrackDialog, tryClear: clearUntrackDialog } = asyncDialog(ref(false));
const { state: syncDialog, tryClear: clearSyncDialog } = asyncDialog(ref(false));
const nRemovedEmotes = ref(0);

const loadState = () => {
  const rewardId = route.params['rewardId'];
  if (!rewardId || typeof rewardId !== 'string') {
    return;
  }
  tryAsync(async state => {
    const res = await api.listSwapEmotes(broadcasterId.value ?? '', rewardId);
    state.value = res;
  }, state).catch(console.error);
};
watch(
  () => [broadcasterId, route.params['rewardId']] as const,
  ([broadcasterId, rewardId]) => {
    if (!broadcasterId.value || !rewardId || typeof broadcasterId.value !== 'string' || typeof rewardId !== 'string') {
      state.loading = true;
      return;
    }
    loadState();
  },
  { immediate: true },
);
const platformName = computed(() => {
  if (!state.value) {
    return '(unknown)';
  }
  switch (state.value.data.data.type) {
    case 'BttvSwap':
      return 'BTTV';
    case 'FfzSwap':
      return 'FFZ';
    case 'SevenTvSwap':
      return '7TV';
    default:
      return '(unknown)';
  }
});
const limitStr = computed(() => {
  if (state.value) {
    const d = state.value.data.data.data;
    if (typeof d === 'object' && d !== null && 'limit' in d) {
      if (d.limit !== null) {
        return d.limit.toString();
      }
    }
  }
  return '∞';
});

const onUntrack = (id: number) => {
  if (!state.value) {
    return;
  }

  untrackDialog.value = true;
  tryAsyncDialog(async () => {
    await api.untrackSwapEmote(state.value?.twitch.broadcaster_id ?? '', state.value?.twitch.id ?? '', id);
    untrackDialog.value = false;

    if (state.value) {
      state.value.emotes = state.value.emotes.filter(e => e.id !== id);
    }
  }, untrackDialog);
};
const onSync = () => {
  if (!state.value) {
    return;
  }

  syncDialog.value = true;
  tryAsyncDialog(async () => {
    const { n_removed } = await api.refreshEmotes(state.value?.twitch.broadcaster_id ?? '');
    nRemovedEmotes.value = n_removed;
    if (n_removed > 0) {
      loadState();
    }
  }, syncDialog);
};
</script>
