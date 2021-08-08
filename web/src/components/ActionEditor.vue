<template>
  <div class="flex flex-col p-1 gap-4 max-w-2xl w-full self-center mt-5">
    <h2 class="font-bold font-mono text-xl text-gray-700">Action</h2>
    <div class="flex gap-4 items-center">
      <h1 class="font-mono font-bold text-3xl">{{ actionType }}</h1>
      <button
        :class="[
          `
            text-black
            bg-red
            rounded-full
            p-2
            transition transition-colors
            ring-offset-2 ring-offset-gray-dark ring ring-transparent
            hover:bg-red-dark
            focus:ring-red
          `,
          isNew && !hasOpened ? 'animate-bing' : '',
        ]"
        @click="openDialog"
      >
        <EditIcon />
      </button>
    </div>
    <TSESettings
      v-if="['Timeout', 'SubOnly', 'EmoteOnly'].includes(actionType)"
      v-model="actionData"
      @update:warn="updateWarn"
    />
    <EmoteSlotSettings v-else-if="['BttvSlot', 'FfzSlot', 'SevenTvSlot'].includes(actionType)" v-model="actionData" />
    <EmoteSwapSettings v-else-if="['BttvSwap', 'FfzSwap', 'SevenTvSwap'].includes(actionType)" v-model="actionData" />
    <SpotifyPlayOptions v-else-if="['SpotifyPlay', 'SpotifyQueue'].includes(action)" v-model="actionData" />
  </div>

  <ActionDialog v-model:action="actionType" v-model:open="dialogOpen" />
</template>

<script lang="ts">
import { computed, defineComponent, onBeforeMount, PropType, ref } from 'vue';
import EditIcon from './icons/EditIcon.vue';
import TSESettings from './rewards/TSESettings.vue';
import EmoteSlotSettings from './rewards/EmoteSlotSettings.vue';
import EmoteSwapSettings from './rewards/EmoteSwapSettings.vue';
import SpotifyPlayOptions from './rewards/SpotifyPlayOptions.vue';
import ActionDialog from './ActionDialog.vue';
import { StaticRewardData } from '../api/rewards-data';
import { RewardDataMap } from '../api/types';
import { simpleClone } from '../api/model-conversion';

export default defineComponent({
  name: 'ActionEditor',
  components: { ActionDialog, EditIcon, TSESettings, EmoteSlotSettings, EmoteSwapSettings, SpotifyPlayOptions },
  props: {
    action: {
      type: String as PropType<keyof RewardDataMap>,
      required: true,
    },
    data: {
      type: [Object, String],
      required: true,
    },
    isNew: {
      type: Boolean,
      default: false,
    },
  },
  emits: ['update:action', 'update:data', 'update:warn'],
  setup(props, { emit }) {
    const actionData = computed({
      get: () => props.data,
      set: data => {
        emit('update:data', data);
      },
    });
    const actionType = computed({
      get: () => props.action,
      set: (act: keyof RewardDataMap) => {
        if (!StaticRewardData[act].validOptions(actionData.value)) {
          actionData.value = simpleClone(StaticRewardData[act].defaultOptions);
        }

        emit('update:action', act);
      },
    });

    const updateWarn = (warn: boolean) => {
      emit('update:warn', warn);
    };

    const dialogOpen = ref(false);
    const hasOpened = ref(false);
    onBeforeMount(() => (hasOpened.value = false));
    const openDialog = () => {
      hasOpened.value = true;
      dialogOpen.value = true;
    };
    return { actionData, actionType, updateWarn, dialogOpen, hasOpened, openDialog };
  },
});
</script>
