<template>
  <CSwitch label="Allow Explicit" :model-value="modelValue.allow_explicit" @update:model-value="updateExplicit" />
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue';
import { SpotifyPlayOptions } from '../../api/types';
import CSwitch from '../core/CSwitch.vue';

export default defineComponent({
  name: 'SpotifyPlayOptions',
  components: { CSwitch },
  props: {
    modelValue: {
      type: Object as PropType<SpotifyPlayOptions>,
      required: true,
    },
  },
  emits: ['update:modelValue'],
  setup(_, { emit }) {
    const updateExplicit = (v: boolean) => {
      emit('update:modelValue', { allow_explicit: v });
    };

    return { updateExplicit };
  },
});
</script>
