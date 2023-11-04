<template>
  <CDropdown
    v-model="state.platform"
    label="Slots"
    :options="[
      { display: '7TV', value: 'SevenTv' },
      { display: 'BTTV', value: 'Bttv' },
      { display: 'FFZ', value: 'Ffz' },
    ]"
  />
  <CSwitch v-model="state.reply" label="Reply after successful redemption" />
</template>

<script lang="ts">
import { defineComponent, PropType, reactive, toRefs, watch } from 'vue';
import CDropdown from '../core/CDropdown.vue';
import CSwitch from '../core/CSwitch.vue';
import { RemEmoteRewardData } from '../../api/types';

export default defineComponent({
  name: 'RemEmoteSettings',
  components: { CSwitch, CDropdown },
  props: {
    modelValue: {
      type: Object as PropType<RemEmoteRewardData>,
      required: true,
    },
  },
  emits: ['update:modelValue'],
  setup(props, { emit }) {
    const { modelValue } = toRefs(props);

    const state = reactive(modelValue.value);

    watch(modelValue, newValue => {
      state.platform = newValue.platform ?? 'SevenTv';
      state.reply = newValue.reply ?? true;
    });
    watch(state, value => {
      emit('update:modelValue', value);
    });

    return { state };
  },
});
</script>
