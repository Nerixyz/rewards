<template>
  <MainLayout>
    <!-- Loading handler -->
    <div v-if="rewards.loading"><CLoader /></div>

    <!-- Error handler -->
    <div v-else-if="rewards.error">
      Something went wrong.
      <span v-if="broadcasterId !== thisUserId">
        Make sure the broadcaster has added you as an editor and they have the feature available.
      </span>
      <span v-else>It seems like you don't have the rewards feature available.</span>
      <br />
      <br />
      <span class="break-words font-mono">{{ rewards.error }}</span>
    </div>

    <!-- The main page -->
    <div v-else class="flex flex-col gap-5">
      <div class="w-full flex pb-5 border-b border-opacity-30 border-gray-900">
        <router-link :to="`/rewards/${encodeURIComponent(broadcasterId || '')}/new`">
          <OutlinedButton>
            <PlusIcon />
            New Reward
          </OutlinedButton>
        </router-link>
        <router-link :to="`/rewards/logs/${encodeURIComponent(broadcasterId || thisUserId || '')}`">
          <OutlinedButton><LogIcon /> Logs</OutlinedButton>
        </router-link>
        <DiscordSettings v-if="broadcasterId" :broadcaster-id="broadcasterId" />
      </div>
      <div class="w-full flex flex-col">
        <div v-if="rewards.value.length" class="flex flex-wrap justify-center gap-6">
          <RewardComponent
            v-for="reward of rewards.value"
            :key="reward.twitch.id"
            :reward="reward"
            @delete-reward="openDeleteDialogForReward"
          />
        </div>
        <div v-else>
          It looks like you haven't created any rewards here yet. How about creating some?
          <img
            class="w-5 h-auto inline"
            alt="KKona"
            src="https://cdn.betterttv.net/emote/566ca04265dbbdab32ec054a/2x"
          />
        </div>
      </div>

      <CDialog title="Delete Reward" :open="deleteDialog.value" @dialog-closed="clearDeleteDialog">
        <div v-if="deleteDialog.loading"><CLoader /></div>
        <div v-else-if="deleteDialog.error">
          Could not delete :/
          <br />
          <span class="break-words font-mono">{{ deleteDialog.error }}</span>
        </div>
        <div v-else-if="deleteDialog.success"><TickIcon /></div>
        <div v-else>Are you sure about that?</div>
        <DialogButtons>
          <OutlinedButton @click="closeDeleteDialog">Cancel</OutlinedButton>
          <CButton @click="deleteCurrentReward">Delete</CButton>
        </DialogButtons>
      </CDialog>
    </div>
  </MainLayout>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useApi } from '../api/plugin';
import { Reward } from '../api/types';
import OutlinedButton from '../components/core/OutlinedButton.vue';
import PlusIcon from '../components/icons/PlusIcon.vue';
import CButton from '../components/core/CButton.vue';
import CDialog from '../components/core/CDialog.vue';
import DialogButtons from '../components/DialogButtons.vue';
import { asyncDialog, tryAsyncDialog } from '../async-state';
import CLoader from '../components/core/CLoader.vue';
import TickIcon from '../components/icons/TickIcon.vue';
import LogIcon from '../components/icons/LogIcon.vue';
import MainLayout from '../components/MainLayout.vue';
import RewardComponent from '../components/Reward.vue';
import { useBroadcaster } from '../hooks/use-broadcaster';
import { useDataStore } from '../store';
import { useRewards } from '../hooks/use-rewards';
import DiscordSettings from '../components/DiscordSettings.vue';

const api = useApi();
const store = useDataStore();

// core stuff to ensure we have a user id

const { thisUserId, broadcasterId } = useBroadcaster({ store });
const { rewards, updateRewards } = useRewards({ broadcasterId, store, api });

// Delete actions
const { state: deleteDialog, reset: resetDeleteDialog } = asyncDialog(ref(false));

const currentRewardToDelete = ref<null | Reward>(null);
const openDeleteDialogForReward = (reward: Reward) => {
  currentRewardToDelete.value = reward;
  deleteDialog.value = true;
};

const deleteReward = (reward: Reward) => {
  tryAsyncDialog(async () => {
    await api.deleteReward(broadcasterId.value ?? '', reward);
    updateRewards(rewards.value.filter(r => r.twitch.id !== reward.twitch.id));

    closeDeleteDialog();
  }, deleteDialog);
};
const deleteCurrentReward = () => {
  if (!currentRewardToDelete.value) {
    closeDeleteDialog();
    return;
  }
  deleteReward(currentRewardToDelete.value);
};
const clearDeleteDialog = () => {
  resetDeleteDialog();
  currentRewardToDelete.value = null;
};
const closeDeleteDialog = () => {
  deleteDialog.value = false;
};
</script>
