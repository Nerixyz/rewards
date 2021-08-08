<template>
  <CDialog :open="dialogOpen" title="Choose an Action">
    <ul class="overflow-y-auto max-w-5xl max-h-75vh flex flex-col gap-3 p-2">
      <ActionType v-model="rewardAction" action="Timeout" description="Timeout someone" />
      <ActionCategory name="Timed Modes">
        <ActionType v-model="rewardAction" action="EmoteOnly" description="Turn on emote-only mode for some time" />
        <ActionType v-model="rewardAction" action="SubOnly" description="Turn on sub-only mode for some time" />
      </ActionCategory>
      <ActionCategory
        name="Emote Swaps"
        description="Swap emotes on some platform. This will add emotes until the limit
      is reached and then remove the last added emotes."
      >
        <ActionType v-model="rewardAction" action="BttvSwap" description="Swap emotes on BTTV" />
        <ActionType v-model="rewardAction" action="FfzSwap" description="Swap emotes on FFZ" />
        <ActionType v-model="rewardAction" action="SevenTvSwap" description="Swap emotes on 7TV" />
      </ActionCategory>
      <ActionCategory
        name="Emote Slots"
        description="Create slots for emotes to be added to. A slot has an expiration time for how long an emote will be in it."
      >
        <ActionType v-model="rewardAction" action="BttvSlot" description="Emote slots on BTTV" />
        <ActionType v-model="rewardAction" action="FfzSlot" description="Emote slots on FFZ" />
        <ActionType v-model="rewardAction" action="SevenTvSlot" description="Emote slots on 7TV" />
      </ActionCategory>
      <ActionCategory name="Spotify">
        <ActionType
          v-model="rewardAction"
          action="SpotifySkip"
          description="Skip the currently playing track on Spotify"
        />
        <ActionType v-model="rewardAction" action="SpotifyPlay" description="Play a track on Spotify" />
        <ActionType v-model="rewardAction" action="SpotifyQueue" description="Queue a track on Spotify" />
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
