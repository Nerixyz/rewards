<template>
  <TextField v-model="state.duration" label="Duration" :warn="warn" />
  <CSwitch v-model="state.vip" label="Block VIP Timeouts" />
</template>

<script lang="ts">
import { computed, defineComponent, PropType, reactive, toRefs, watch } from 'vue';
import TextField from '../core/TextField.vue';
import { isValidRewardDurationExpression } from '../../utilities';
import { TimeoutRewardData } from '../../api/types';
import CSwitch from '../core/CSwitch.vue';
import { StaticRewardData } from '../../api/rewards-data';

export default defineComponent({
  name: 'TimeoutSettings',
  components: { CSwitch, TextField },
  props: {
    modelValue: {
      type: Object as PropType<TimeoutRewardData | null>,
      required: true,
    },
  },
  emits: ['update:modelValue', 'update:warn'],
  setup(props, { emit }) {
    const { modelValue } = toRefs(props);

    const state = reactive(modelValue.value ?? { ...StaticRewardData.Timeout.defaultOptions });

    const warn = computed(() => !isValidRewardDurationExpression(state.duration.trim() ?? '-'));
    watch(warn, v => emit('update:warn', v));

    watch(modelValue, newValue => {
      state.duration = newValue?.duration ?? StaticRewardData.Timeout.defaultOptions.duration;
      state.vip = newValue?.vip ?? StaticRewardData.Timeout.defaultOptions.vip;
    });
    watch(state, value => {
      emit('update:modelValue', value);
    });

    return { state, warn };
  },
});
</script>
