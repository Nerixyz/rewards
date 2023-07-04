<template>
  <CSlider v-model="state.slots" label="Slots" :min="1" :max="10" />
  <TextField v-model="state.expiration" label="Expiration" />
  <CSwitch v-model="state.allow_unlisted" label="Allow unlisted emotes" />
  <CSwitch v-model="state.reply" label="Reply after successful redemption" />
</template>

<script lang="ts">
import { defineComponent, PropType, reactive, toRefs, watch } from 'vue';
import CSlider from '../core/CSlider.vue';
import CSwitch from '../core/CSwitch.vue';
import { SlotRewardData } from '../../api/types';
import TextField from '../core/TextField.vue';

export default defineComponent({
  name: 'EmoteSlotSettings',
  components: { CSlider, CSwitch, TextField },
  props: {
    modelValue: {
      type: Object as PropType<SlotRewardData>,
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
      state.allow_unlisted = newValue.allow_unlisted ?? true;
      state.reply = newValue.reply ?? true;
    });
    watch(state, value => {
      emit('update:modelValue', value);
    });

    return { state };
  },
});
</script>
