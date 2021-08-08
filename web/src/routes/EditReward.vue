<template>
  <MainLayout>
    <div v-if="rewards.loading || updateState.loading">
      <CLoader />
    </div>
    <div v-else-if="rewards.error || updateState.error">
      Something went wrong.
      <br />
      <br />
      <span class="break-words font-mono">{{ rewards.error || updateState.error }}</span>
      <CButton v-if="updateState.error" @click="resetUpdate">Okay</CButton>
    </div>
    <div v-else-if="!reward.value">
      <h1>There's no reward with this id</h1>
    </div>
    <div v-else>
      <div class="flex gap-4 items-center mb-10">
        <BackIcon
          class="text-red bg-white bg-opacity-0 w-10 h-10 p-2 rounded-full cursor-pointer hover:bg-opacity-20"
          @click="$router.back()"
        />
        <h1 class="font-bold font-mono text-5xl">Edit Reward</h1>
      </div>
      <div class="flex flex-col">
        <RewardEditor
          :reward-model="reward.value"
          :can-update="true"
          main-action="Done"
          @update="onUpdate"
          @done="onDone"
        />
      </div>
    </div>
  </MainLayout>
</template>

<script lang="ts">
import { defineComponent, reactive, watch } from 'vue';
import MainLayout from '../components/MainLayout.vue';
import { useDataStore } from '../store';
import { useApi } from '../api/plugin';
import { useRewards } from '../hooks/use-rewards';
import { useBroadcaster } from '../hooks/use-broadcaster';
import { useRoute, useRouter } from 'vue-router';
import CLoader from '../components/core/CLoader.vue';
import RewardEditor from '../components/RewardEditor.vue';
import { InputReward, Reward } from '../api/types';
import { asyncState, tryAsync } from '../async-state';
import CButton from '../components/core/CButton.vue';
import BackIcon from '../components/icons/BackIcon.vue';

export default defineComponent({
  name: 'EditReward',
  components: { BackIcon, CButton, RewardEditor, CLoader, MainLayout },
  setup() {
    const store = useDataStore();
    const api = useApi();
    const route = useRoute();
    const router = useRouter();
    const { broadcasterId } = useBroadcaster({ store, route });
    const { updateRewards, rewards } = useRewards({ store, api, broadcasterId });

    const { state: updateState, reset: resetUpdate } = asyncState(0, false);

    const reward = reactive<{ value: null | Reward }>({ value: null });
    watch(
      () => [rewards.value, route.params.rewardId] as const,
      ([rewards, rewardId]) => {
        reward.value = rewards.find(x => x.twitch.id === rewardId) ?? null;
        resetUpdate();
      },
      { immediate: true },
    );

    const tryUpdate = (toUpdate: InputReward, post?: () => void) => {
      tryAsync(async () => {
        const updated = await api.updateReward(broadcasterId.value ?? '', toUpdate, reward.value?.twitch.id ?? '');
        updateRewards(rewards.value.map(r => (r.twitch.id === updated.twitch.id ? updated : r)));
        post?.();
      }, updateState);
    };

    const handlers = {
      onUpdate: (reward: InputReward) => {
        tryUpdate(reward);
      },
      onDone: (reward: InputReward) => {
        tryUpdate(reward, () => router.push(`/rewards/${encodeURIComponent(broadcasterId.value ?? '')}`));
      },
    };
    return { reward, rewards, updateState, resetUpdate, ...handlers };
  },
});
</script>
