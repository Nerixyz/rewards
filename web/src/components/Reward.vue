<template>
  <div class="flex">
    <div>
      <h3 class="font-serif text-2xl">{{ reward.twitch.title }}</h3>
      <h4 class="font-serif italic text-sm">{{ reward.data.type }}</h4>
    </div>
    <div class="ml-auto">
      <CButton @click="$emit('editReward', reward)">
        <EditIcon />
        edit
      </CButton>
      <OutlinedButton @click="$emit('deleteReward', reward)">
        <TrashIcon />
        delete
      </OutlinedButton>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue';
import { Reward } from '../api/types';
import CButton from './core/CButton.vue';
import OutlinedButton from './core/OutlinedButton.vue';
import TrashIcon from './icons/TrashIcon.vue';
import EditIcon from './icons/EditIcon.vue';

export default defineComponent({
  name: 'Reward',
  components: { CButton, OutlinedButton, TrashIcon, EditIcon },
  props: {
    reward: {
      type: Object as PropType<Reward>,
      required: true,
    },
  },
  emits: {
    deleteReward: (reward: Reward) => reward,
    editReward: (reward: Reward) => reward,
  },
});
</script>
