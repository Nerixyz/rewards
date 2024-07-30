<template>
  <MainLayout>
    <div v-if="rewards.loading || saveState.loading">
      <CLoader />
    </div>
    <div v-else-if="rewards.error || saveState.error">
      Something went wrong.
      <br />
      <br />
      <span class="break-words font-mono">{{ rewards.error || saveState.error }}</span>
      <CButton v-if="saveState.error" @click="resetState">Okay</CButton>
    </div>
    <div v-else>
      <div class="flex gap-4 items-center mb-10">
        <BackIcon
          class="text-red bg-white bg-opacity-0 w-10 h-10 p-2 rounded-full cursor-pointer hover:bg-opacity-20"
          @click="$router.back()"
        />
        <h1 class="font-bold font-mono text-5xl">New Reward</h1>
      </div>
      <div class="flex flex-col">
        <RewardEditor
          :can-update="false"
          :is-new="true"
          main-action="Create"
          :reward-model="undefined"
          @done="saveReward"
        />
      </div>
    </div>
  </MainLayout>
</template>

<script setup lang="ts">
import { onBeforeMount } from 'vue';
import MainLayout from '../components/MainLayout.vue';
import { useDataStore } from '../store';
import { useApi } from '../api/plugin';
import { useRewards } from '../hooks/use-rewards';
import { useBroadcaster } from '../hooks/use-broadcaster';
import { useRoute, useRouter } from 'vue-router';
import CLoader from '../components/core/CLoader.vue';
import RewardEditor from '../components/RewardEditor.vue';
import { InputReward } from '../api/types';
import { asyncState, tryAsync } from '../async-state';
import CButton from '../components/core/CButton.vue';
import BackIcon from '../components/icons/BackIcon.vue';

const store = useDataStore();
const api = useApi();
const route = useRoute();
const router = useRouter();
const { broadcasterId } = useBroadcaster({ store, route });
const { updateRewards, rewards } = useRewards({ store, api, broadcasterId });

const { state: saveState, reset: resetState } = asyncState(0, false);

onBeforeMount(() => resetState());

const saveReward = (target: InputReward) => {
  tryAsync(async () => {
    const newOne = await api.addReward(broadcasterId.value ?? '', target);
    updateRewards([...rewards.value, newOne]);
    await router.replace(`/rewards/${encodeURIComponent(broadcasterId.value ?? '')}`);
  }, saveState);
};
</script>
