<template>
  <CDialog :open="openModel" title="Choose an Action">
    <ul class="overflow-y-auto max-w-3xl max-h-75vh flex flex-col gap-3 p-2">
      <ActionType v-model="rewardAction" action="Timeout" description="Timeout someone" />
      <ActionCategory name="Timed Modes">
        <ActionType v-model="rewardAction" action="EmoteOnly" description="Turn on emote-only mode for some time" />
        <ActionType v-model="rewardAction" action="SubOnly" description="Turn on sub-only mode for some time" />
      </ActionCategory>
      <ActionCategory
        name="Emote Swaps"
        description="Adds emotes until a specified limit is reached. Then removes the oldest emotes in order. In contrast to Slots, this reward is more dynamic."
      >
        <ActionType v-model="rewardAction" action="BttvSwap" action-name="BTTV" description="Swap emotes on BTTV" />
        <ActionType v-model="rewardAction" action="FfzSwap" action-name="FFZ" description="Swap emotes on FFZ" />
        <ActionType v-model="rewardAction" action="SevenTvSwap" action-name="7TV" description="Swap emotes on 7TV" />
      </ActionCategory>
      <ActionCategory
        name="Emote Slots"
        description="Add emotes into slots. A slot has an expiration time for how long an emote will be in it. In contrast to Swaps, here the reward will be locked until an emote expires, so emotes have a fixed duration for how long they will be added."
      >
        <ActionType v-model="rewardAction" action="BttvSlot" action-name="BTTV" description="Emote slots on BTTV" />
        <ActionType v-model="rewardAction" action="FfzSlot" action-name="FFZ" description="Emote slots on FFZ" />
        <ActionType v-model="rewardAction" action="SevenTvSlot" action-name="7TV" description="Emote slots on 7TV" />
      </ActionCategory>
      <ActionType
        v-model="rewardAction"
        action="RemEmote"
        action-name="Remove Emote"
        description="Remove a 7TV/BTTV/FFZ emote from your channel."
      />
      <ActionCategory
        name="Spotify"
        description="These rewards require Spotify Premium since they control the Spotify player."
      >
        <ActionType
          v-model="rewardAction"
          action="SpotifySkip"
          action-name="Skip"
          description="Skip the currently playing track on Spotify"
        />
        <ActionType
          v-model="rewardAction"
          action="SpotifyPlay"
          action-name="Play"
          description="Play a track on Spotify"
        />
        <ActionType
          v-model="rewardAction"
          action="SpotifyQueue"
          action-name="Queue"
          description="Queue a track on Spotify"
        />
      </ActionCategory>
    </ul>
    <div class="flex justify-end mt-3">
      <OutlinedButton @click="closeDialog"> Cancel </OutlinedButton>
      <CButton @click="done"> Done </CButton>
    </div>
  </CDialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import CDialog from './core/CDialog.vue';
import { RewardDataMap } from '../api/types';
import ActionType from './ActionType.vue';
import ActionCategory from './ActionCategory.vue';
import CButton from './core/CButton.vue';
import OutlinedButton from './core/OutlinedButton.vue';

const props = defineProps<{ open: boolean; action: keyof RewardDataMap }>();
const [openModel] = defineModel<boolean>('open', { required: true });
const [actionModel] = defineModel<keyof RewardDataMap>('action', { required: true });

const rewardAction = ref<keyof RewardDataMap>('Timeout');
watch(
  () => props.action,
  () => {
    rewardAction.value = props.action;
  },
);

const closeDialog = () => {
  openModel.value = false;
};
const done = () => {
  actionModel.value = rewardAction.value;
  closeDialog();
};
</script>
