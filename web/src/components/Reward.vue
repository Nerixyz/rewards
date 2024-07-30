<template>
  <router-link :to="`/rewards/${reward.twitch.broadcaster_id}/${reward.twitch.id}`">
    <div
      class="flex flex-col gap-2 transition transition transform bg-gray-300 rounded-xl overflow-hidden cursor-pointer border border-opacity-5 border-white hover:scale-105 hover:border-opacity-20 focus:scale-105 focus:border-opacity-20"
    >
      <div class="flex flex-col p-4 w-60">
        <div v-if="imageUrl" class="flex items-center justify-center">
          <img :src="imageUrl" alt="Reward Icon" class="w-24 h-auto" />
        </div>
        <div>
          <h3 class="font-serif text-2xl mb-4 whitespace-nowrap overflow-hidden overflow-ellipsis">
            {{ reward.twitch.title }}
          </h3>
          <h4 class="font-serif text-sm mt-2 text-gray-700">Prompt</h4>
          <h4 class="font-serif italic text-sm whitespace-nowrap overflow-hidden overflow-ellipsis">
            {{ reward.twitch.prompt }}
          </h4>
          <h4 class="font-serif text-sm mt-2 text-gray-700">Action</h4>
          <h4 class="font-serif italic text-sm whitespace-nowrap overflow-hidden overflow-ellipsis">
            {{ reward.data.type }} <span class="ml-2 text-gray-700">{{ actionDescription }}</span>
          </h4>
        </div>
      </div>
      <div class="">
        <button
          class="py-2 px-3 uppercase font-bold flex items-center w-full justify-center transition transition-colors border-t border-red hover:bg-red-dark hover:text-black hover:border-transparent"
          @click.prevent="emit('deleteReward', reward)"
        >
          <TrashIcon /> Delete
        </button>
      </div>
    </div>
  </router-link>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Reward } from '../api/types';
import TrashIcon from './icons/TrashIcon.vue';

const props = defineProps<{ reward: Reward }>();
const emit = defineEmits<{ deleteReward: [reward: Reward] }>();

const imageUrl = computed(() => {
  const twitch = props.reward.twitch;
  return twitch.image?.url_4x ?? twitch.default_image?.url_4x ?? null;
});

const actionDescription = computed(() => {
  const { data, type } = props.reward.data;
  let description = '';
  switch (type) {
    case 'Timeout':
      description = `duration = ${data.duration}s, vip = ${data.vip}`;
      break;
    case 'SubOnly':
    case 'EmoteOnly':
      description = data;
      break;
    case 'BttvSwap':
    case 'FfzSwap':
    case 'SevenTvSwap':
      const limit = data?.limit ?? null;
      if (limit !== null) {
        description = `limit = ${limit}`;
      }
      break;
    case 'BttvSlot':
    case 'FfzSlot':
    case 'SevenTvSlot':
      description = `slots = ${data.slots}, expiration = ${data.expiration}`;
      break;
    case 'SpotifySkip':
      break;
    case 'SpotifyPlay':
    case 'SpotifyQueue':
      description = `explicit = ${data.allow_explicit}`;
      break;
  }
  return description;
});
</script>
