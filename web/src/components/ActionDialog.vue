<template>
  <CDialog :open="dialogOpen" title="Choose an Action">
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

<script lang="ts">
import { computed, defineComponent, PropType, ref, watchEffect } from 'vue';
import CDialog from './core/CDialog.vue';
import { RewardDataMap } from '../api/types';
import ActionType from './ActionType.vue';
import ActionCategory from './ActionCategory.vue';
import CButton from './core/CButton.vue';
import OutlinedButton from './core/OutlinedButton.vue';

export default defineComponent({
  name: 'ActionDialog',
  components: { OutlinedButton, CButton, ActionCategory, ActionType, CDialog },
  props: {
    open: {
      type: Boolean,
      required: true,
    },
    action: {
      type: String as PropType<keyof RewardDataMap>,
      required: true,
    },
  },
  emits: ['update:open', 'update:action'],
  setup(props, { emit }) {
    const dialogOpen = computed({ get: () => props.open, set: v => emit('update:open', v) });

    const rewardAction = ref<keyof RewardDataMap>('Timeout');
    watchEffect(() => {
      rewardAction.value = props.action;
    });

    const setAction = (action: keyof RewardDataMap) => {
      rewardAction.value = action;
    };

    const closeDialog = () => {
      dialogOpen.value = false;
    };
    const done = () => {
      emit('update:action', rewardAction.value);
      closeDialog();
    };

    return { rewardAction, dialogOpen, setAction, closeDialog, done };
  },
});
</script>
