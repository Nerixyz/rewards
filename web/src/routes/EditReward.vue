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

    <CDialog title="Are You sure?" :open="updateWarningShown" @dialog-closed="abortWarning">
      <span class="max-w-xs">
        {{ updateWarningText }}
      </span>
      <DialogButtons>
        <OutlinedButton @click="abortWarning">Abort</OutlinedButton>
        <CButton @click="replayUpdate">Update</CButton>
      </DialogButtons>
    </CDialog>
  </MainLayout>
</template>

<script setup lang="ts">
import { reactive, ref, watch } from 'vue';
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
import CDialog from '../components/core/CDialog.vue';
import DialogButtons from '../components/DialogButtons.vue';
import OutlinedButton from '../components/core/OutlinedButton.vue';

const store = useDataStore();
const api = useApi();
const route = useRoute();
const router = useRouter();
const { broadcasterId } = useBroadcaster({ store, route });
const { updateRewards, rewards } = useRewards({ store, api, broadcasterId });

const { state: updateState, reset: resetUpdate } = asyncState(0, false);
const updateWarningText = ref<string>('');
const updateWarningShown = ref(false);
let acknowledgedWarning = false;
let lastUpdateParams: null | { reward: InputReward; cb?: () => void } = null;

const reward = reactive<{ value: null | Reward }>({ value: null });
watch(
  () => [rewards.value, route.params['rewardId']] as const,
  ([rewards, rewardId]) => {
    reward.value = rewards.find(x => x.twitch.id === rewardId) ?? null;
    resetUpdate();
  },
  { immediate: true },
);

const checkWarning = async (toUpdate: InputReward) => {
  if (acknowledgedWarning) {
    return true; // can continue
  }
  if (toUpdate.data.type !== 'BttvSwap' && toUpdate.data.type !== 'FfzSwap' && toUpdate.data.type !== 'SevenTvSwap') {
    return true;
  }

  const limit = toUpdate.data.data?.limit;
  if (!limit) {
    return true;
  }

  const { usage } = await api.getSwapEmotesUsage(broadcasterId.value ?? '', reward.value?.twitch.id ?? '');
  if (usage > limit) {
    updateWarningText.value = `There are currently ${usage} emotes tracked with this reward. Updating the reward will cause the last ${usage - limit} emotes to be removed.`;
    updateWarningShown.value = true;
    return false;
  }

  return true;
};

const replayUpdate = () => {
  acknowledgedWarning = true;
  updateWarningShown.value = false;
  if (lastUpdateParams) {
    tryUpdate(lastUpdateParams.reward, lastUpdateParams.cb);
  }
};
const abortWarning = () => {
  updateWarningShown.value = false;
};

const tryUpdate = (toUpdate: InputReward, post?: () => void) => {
  tryAsync(async () => {
    lastUpdateParams = { reward: toUpdate, cb: post };
    if (!(await checkWarning(toUpdate).catch(() => true))) {
      return;
    }

    const updated = await api.updateReward(broadcasterId.value ?? '', toUpdate, reward.value?.twitch.id ?? '');
    updateRewards(rewards.value.map(r => (r.twitch.id === updated.twitch.id ? updated : r)));
    post?.();
    lastUpdateParams = null;
    acknowledgedWarning = false;
  }, updateState);
};
const onUpdate = (reward: InputReward) => {
  tryUpdate(reward);
};
const onDone = (reward: InputReward) => {
  tryUpdate(reward, () => router.push(`/rewards/${encodeURIComponent(broadcasterId.value ?? '')}`));
};
</script>
