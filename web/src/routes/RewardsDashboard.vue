<template>
  <div class="px-20 pt-5 xl:max-w-7xl mx-auto">
    <!-- Loading handler -->
    <div v-if="rewards.loading"><CLoader /></div>

    <!-- Error handler -->
    <div v-else-if="rewards.error">
      Something went wrong.
      <span v-if="broadcasterId !== thisUserId"
        >Make sure the broadcaster has added you as an editor and they have the feature available.</span
      >
      <span v-else>It seems like you don't have the rewards feature available.</span>
      <br />
      <br />
      <span class="break-words font-mono">Error: {{ rewards.error }}</span>
    </div>

    <!-- The main page -->
    <div v-else class="flex flex-col gap-5">
      <div class="w-full pb-5 border-b border-opacity-30 border-gray-900">
        <OutlinedButton @click="openAddDialog"> <PlusIcon /> Add Reward </OutlinedButton>
        <router-link :to="`/rewards/logs/${encodeURIComponent(broadcasterId || thisUserId || '')}`">
          <OutlinedButton><LogIcon /> Logs</OutlinedButton>
        </router-link>
      </div>
      <div class="w-full flex flex-col gap-5">
        <div v-if="rewards.value.length">
          <div v-for="reward of rewards.value" :key="reward.twitch.id" class="flex">
            <div>
              <h3 class="font-serif text-2xl">{{ reward.twitch.title }}</h3>
              <h4 class="font-serif italic text-sm">{{ reward.data.type }}</h4>
            </div>
            <div class="ml-auto">
              <CButton @click="openEditDialog(reward)">
                <EditIcon />
                edit
              </CButton>
              <OutlinedButton @click="openDeleteDialogForReward(reward)">
                <TrashIcon />
                delete
              </OutlinedButton>
            </div>
          </div>
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

      <AddOrEditRewardDialog
        v-model:open="addEditDialogOpen"
        :broadcaster-id="broadcasterId"
        :reward-data="editRewardData"
        @added="rewardAdded"
        @updated="rewardUpdated"
      />

      <CDialog title="Delete Reward" :open="deleteDialog.value" @dialogClosed="clearDeleteDialog">
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
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, watch } from 'vue';
import { useRoute } from 'vue-router';
import { useDataStore } from '../store';
import { useApi } from '../api/plugin';
import { Reward } from '../api/types';
import OutlinedButton from '../components/core/OutlinedButton.vue';
import PlusIcon from '../components/icons/PlusIcon.vue';
import AddOrEditRewardDialog from '../components/AddOrEditRewardDialog.vue';
import CButton from '../components/core/CButton.vue';
import EditIcon from '../components/icons/EditIcon.vue';
import TrashIcon from '../components/icons/TrashIcon.vue';
import CDialog from '../components/core/CDialog.vue';
import DialogButtons from '../components/DialogButtons.vue';
import { asyncDialog, asyncState, tryAsync, tryAsyncDialog } from '../async-state';
import CLoader from '../components/core/CLoader.vue';
import TickIcon from '../components/icons/TickIcon.vue';
import LogIcon from '../components/icons/LogIcon.vue';

export default defineComponent({
  name: 'RewardsDashboard',
  components: {
    LogIcon,
    TickIcon,
    CLoader,
    DialogButtons,
    CDialog,
    TrashIcon,
    EditIcon,
    CButton,
    AddOrEditRewardDialog,
    PlusIcon,
    OutlinedButton,
  },
  setup() {
    // TODO: explain
    const route = useRoute();
    const store = useDataStore();
    const api = useApi();

    // core stuff to ensure we have a user id

    const { state: rewards } = asyncState<Reward[]>([]);
    const broadcasterId = ref<string>(((route.params.id as string | undefined) || store.user.value?.id) ?? '');
    const thisUserId = ref<undefined | string>(undefined);

    const updateBroadcaster = () =>
      tryAsync(async rewards => {
        const id = (route.params.id as string | undefined) || store.user.value?.id;

        thisUserId.value = store.user.value?.id;
        broadcasterId.value = id ?? '';

        rewards.value = await api.getRewards(id ?? '');
      }, rewards);

    watch(() => route.params.id, updateBroadcaster);

    if (!store.user.value) {
      const stop = watch(store.user, () => {
        stop();
        updateBroadcaster();
      });
    } else {
      updateBroadcaster();
    }

    const coreExports = { rewards, broadcasterId, thisUserId };

    // Add/Edit Dialog

    const editRewardData = ref<undefined | Reward>(undefined);
    const addEditDialogOpen = ref(false);

    const openAddDialog = () => {
      editRewardData.value = undefined; // important!
      addEditDialogOpen.value = true;
    };
    const openEditDialog = (reward: Reward) => {
      editRewardData.value = reward;
      addEditDialogOpen.value = true;
    };

    const rewardAdded = (reward: Reward) => {
      rewards.value = [...rewards.value, reward];
    };
    const rewardUpdated = (reward: Reward) => {
      // replace the old one
      rewards.value = rewards.value.map(r => (r.twitch.id === reward.twitch.id ? reward : r));
    };

    const addExports = { addEditDialogOpen, openAddDialog, openEditDialog, rewardAdded, rewardUpdated, editRewardData };

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
        rewards.value = rewards.value.filter(r => r.twitch.id !== reward.twitch.id);

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

    const deleteExports = {
      deleteDialog,
      closeDeleteDialog,
      deleteCurrentReward,
      openDeleteDialogForReward,
      clearDeleteDialog,
    };

    return { ...coreExports, ...addExports, ...deleteExports };
  },
});
</script>
