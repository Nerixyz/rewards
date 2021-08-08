<template>
  <router-link :to="`/rewards/${reward.twitch.broadcaster_id}/${reward.twitch.id}`">
    <div
      class="
        flex flex-col
        gap-2
        transition transition-transform transition-opacity
        transform
        bg-gray-300
        rounded-xl
        overflow-hidden
        cursor-pointer
        border border-transparent border-opacity-5 border-white
        hover:scale-105 hover:border-opacity-20
      "
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
          class="
            py-2
            px-3
            uppercase
            font-bold
            flex
            items-center
            w-full
            justify-center
            transition transition-colors
            border-t border-red
            hover:bg-red-dark hover:text-black hover:border-transparent
          "
          @click.prevent="$emit('deleteReward', reward)"
        >
          <TrashIcon /> Delete
        </button>
      </div>
    </div>
  </router-link>
</template>

<script lang="ts">
import { computed, defineComponent, PropType, toRefs } from 'vue';
import { Reward, RewardDataMap } from '../api/types';
import TrashIcon from './icons/TrashIcon.vue';

export default defineComponent({
  name: 'Reward',
  components: { TrashIcon },
  props: {
    reward: {
      type: Object as PropType<Reward>,
      required: true,
    },
  },
  emits: {
    deleteReward: (reward: Reward) => reward,
  },
  setup(props) {
    const { reward } = toRefs(props);

    const imageUrl = computed(() => {
      const twitch = reward.value.twitch;
      return twitch.image?.url_4x ?? twitch.default_image?.url_4x ?? null;
    });

    const actionDescription = computed(() => {
      const data = reward.value.data.data;
      let description = '';
      switch (reward.value.data.type) {
        case 'Timeout':
          description = data as RewardDataMap['Timeout'];
          break;
        case 'SubOnly':
        case 'EmoteOnly':
          description = data as RewardDataMap['SubOnly' | 'EmoteOnly'];
          break;
        case 'BttvSwap':
        case 'FfzSwap':
        case 'SevenTvSwap':
          const limit = (data as RewardDataMap['BttvSwap' | 'FfzSwap' | 'SevenTvSwap'])?.limit ?? null;
          if (limit !== null) {
            description = `limit = ${limit}`;
          }
          break;
        case 'BttvSlot':
        case 'FfzSlot':
        case 'SevenTvSlot':
          const sData = data as RewardDataMap['BttvSlot' | 'FfzSlot' | 'SevenTvSlot'];
          description = `slots = ${sData.slots}, expiration = ${sData.expiration}`;
          break;
        case 'SpotifySkip':
          break;
        case 'SpotifyPlay':
        case 'SpotifyQueue':
          const explicit = (data as RewardDataMap['SpotifyPlay' | 'SpotifyQueue']).allow_explicit;
          description = `explicit = ${explicit}`;
          break;
      }
      return description;
    });

    return { imageUrl, actionDescription };
  },
});
</script>
