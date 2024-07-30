<template>
  <div class="flex flex-col p-1 gap-4 max-w-2xl w-full self-center mt-5">
    <h2 class="font-bold font-mono text-xl text-gray-700">Action</h2>
    <div class="flex gap-4 items-center">
      <h1 class="font-mono font-bold text-3xl">{{ reward.type }}</h1>
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
    <TimeoutSettings v-if="reward.type === 'Timeout'" v-model="reward.data" @update:warn="updateWarn" />
    <SESettings
      v-else-if="reward.type === 'SubOnly' || reward.type === 'EmoteOnly'"
      v-model="reward.data"
      @update:warn="updateWarn"
    />
    <EmoteSlotSettings
      v-else-if="reward.type === 'BttvSlot' || reward.type === 'FfzSlot' || reward.type === 'SevenTvSlot'"
      v-model="reward.data"
    />
    <EmoteSwapSettings
      v-else-if="reward.type === 'BttvSwap' || reward.type === 'FfzSwap' || reward.type === 'SevenTvSwap'"
      v-model="reward.data"
    />
    <SpotifyPlayOptions
      v-else-if="reward.type === 'SpotifyPlay' || reward.type === 'SpotifyQueue'"
      v-model="reward.data"
    />
    <RemEmoteSettings v-else-if="reward.type === 'RemEmote'" v-model="reward.data" />
  </div>

  <ActionDialog v-model:action="reward.type" v-model:open="dialogOpen" />
</template>

<script setup lang="ts">
import { onBeforeMount, ref, watch } from 'vue';
import EditIcon from './icons/EditIcon.vue';
import SESettings from './rewards/SESettings.vue';
import TimeoutSettings from './rewards/TimeoutSettings.vue';
import EmoteSlotSettings from './rewards/EmoteSlotSettings.vue';
import EmoteSwapSettings from './rewards/EmoteSwapSettings.vue';
import SpotifyPlayOptions from './rewards/SpotifyPlayOptions.vue';
import RemEmoteSettings from './rewards/RemEmoteSettings.vue';
import ActionDialog from './ActionDialog.vue';
import { StaticRewardData } from '../api/rewards-data';
import { RewardData } from '../api/types';
import { simpleClone } from '../api/model-conversion';

const [reward] = defineModel<RewardData>({
  required: true,
});
defineProps<{ isNew?: boolean }>();
const emit = defineEmits<{
  'update:warn': [warn: boolean];
}>();

const updateWarn = (warn: boolean) => {
  emit('update:warn', warn);
};

watch(
  () => reward.value,
  v => {
    if (!StaticRewardData[v.type].validOptions(v.data)) {
      v.data = simpleClone(StaticRewardData[v.type].defaultOptions);
      console.log('default');
    }
  },
);

const dialogOpen = ref(false);
const hasOpened = ref(false);
onBeforeMount(() => (hasOpened.value = false));
const openDialog = () => {
  hasOpened.value = true;
  dialogOpen.value = true;
};
</script>
