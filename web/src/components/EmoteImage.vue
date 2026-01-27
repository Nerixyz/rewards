<template>
  <a :href="url" target="_blank"><img :src="imageUrl" loading="lazy" class="max-w-10 h-10 object-contain m-auto" /></a>
</template>
<script lang="ts" setup>
import { computed } from 'vue';
import { SlotPlatform } from '../api/types';

const props = defineProps<{ platform: SlotPlatform; id: string }>();
const imageUrl = computed(() => {
  switch (props.platform) {
    case 'Bttv':
      return `https://cdn.betterttv.net/emote/${encodeURIComponent(props.id)}/2x.webp`;
    case 'Ffz':
      return `https://cdn.frankerfacez.com/emote/${encodeURIComponent(props.id)}/2`;
    case 'SevenTv':
      return `https://cdn.7tv.app/emote/${encodeURIComponent(props.id)}/2x.webp`;
  }
  // @ts-expect-error
  return '';
});
const url = computed(() => {
  switch (props.platform) {
    case 'Bttv':
      return `https://betterttv.com/emotes/${encodeURIComponent(props.id)}`;
    case 'Ffz':
      return `https://www.frankerfacez.com/emoticon/${encodeURIComponent(props.id)}`;
    case 'SevenTv':
      return `https://7tv.app/emotes/${encodeURIComponent(props.id)}`;
  }
  // @ts-expect-error
  return '';
});
</script>
