<template>
  <img :src="url" loading="lazy" class="max-w-10 h-10 object-contain m-auto" />
</template>
<script lang="ts" setup>
import { computed } from 'vue';
import { SlotPlatform } from '../api/types';

const props = defineProps<{ platform: SlotPlatform; id: string }>();
const url = computed(() => {
  switch (props.platform) {
    case 'Bttv':
      return `https://cdn.betterttv.net/emote/${encodeURIComponent(props.id)}/2x.webp`;
    case 'Ffz':
      return `https://cdn.frankerfacez.com/emote/${encodeURIComponent(props.id)}/2`;
    case 'SevenTv':
      return `https://cdn.7tv.app/emote/${encodeURIComponent(props.id)}/2x.webp`;
  }
  return '';
});
</script>
