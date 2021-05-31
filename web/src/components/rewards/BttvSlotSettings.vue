<template>
  <CSlider v-model="state.slots" label="Slots" :min="0" :max="10" />
  <TextField v-model="state.expiration" label="Expiration" />
</template>

<script lang="ts">
import { defineComponent, PropType, reactive, toRefs, watch } from 'vue';
import CSlider from '../core/CSlider.vue';
import { BttvSlotRewardData } from '../../api/types';
import TextField from '../core/TextField.vue';

export default defineComponent({
  name: 'BttvSlotSettings',
  components: { CSlider, TextField },
  props: {
    modelValue: {
      type: Object as PropType<BttvSlotRewardData>,
      required: true,
    },
  },
  emits: ['update:modelValue'],
  setup(props, { emit }) {
    const { modelValue } = toRefs(props);

    const state = reactive(modelValue.value);

    watch(modelValue, newValue => {
      state.expiration = newValue.expiration;
      state.slots = newValue.slots;
    });
    watch(state, value => {
      emit('update:modelValue', value);
    });

    return { state };
  },
});
</script>
