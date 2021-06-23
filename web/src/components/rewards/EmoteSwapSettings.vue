<template>
  <CSwitch :model-value="sliderEnabled" @update:model-value="updateSliderEnabled" label="Limit emotes" />
  <CSlider class="mt-2" v-if="state.limit !== null" v-model="state.limit" :min="1" :max="20" />
</template>

<script lang="ts">
import { computed, defineComponent, PropType, reactive, toRefs, watch } from 'vue';
import CSlider from '../core/CSlider.vue';
import { SwapRewardData } from '../../api/types';
import CSwitch from '../core/CSwitch.vue';

export default defineComponent({
  name: 'EmoteSwapSettings',
  components: { CSwitch, CSlider },
  props: {
    modelValue: {
      type: Object as PropType<SwapRewardData | null>,
      required: true,
    },
  },
  emits: ['update:modelValue'],
  setup(props, { emit }) {
    const { modelValue } = toRefs(props);

    const state = reactive(modelValue.value ?? { limit: null });
    const sliderEnabled = computed(() => state.limit !== null);
    const updateSliderEnabled = (enabled: boolean) => {
      // set the "default" to 1
      state.limit = enabled ? 1 : null;
    };

    watch(modelValue, newValue => {
      state.limit = newValue?.limit ?? null;
    });
    watch(state, value => {
      emit('update:modelValue', value);
    });

    return { state, sliderEnabled, updateSliderEnabled };
  },
});
</script>
