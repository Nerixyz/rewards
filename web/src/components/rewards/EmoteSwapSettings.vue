<template>
  <CSwitch :model-value="sliderEnabled" label="Limit emotes" @update:model-value="updateSliderEnabled" />
  <CSlider v-if="state.limit !== null" v-model="state.limit" class="mt-2" :min="1" :max="400" />
  <CSwitch v-model="state.allow_unlisted" label="Allow unlisted emotes" />
  <CSwitch v-model="state.reply" label="Reply after successful redemption" />
</template>

<script setup lang="ts">
import { computed, reactive, watch } from 'vue';
import CSlider from '../core/CSlider.vue';
import { SwapRewardData } from '../../api/types';
import CSwitch from '../core/CSwitch.vue';

const [modelValue] = defineModel<SwapRewardData | null>({ required: true });

const state = reactive({ limit: null, allow_unlisted: true, reply: true, ...modelValue.value });
const sliderEnabled = computed(() => state.limit !== null);
const updateSliderEnabled = (enabled: boolean) => {
  // set the "default" to 1
  state.limit = enabled ? 1 : null;
};

watch(modelValue, newValue => {
  state.limit = newValue?.limit ?? null;
  state.allow_unlisted = newValue?.allow_unlisted ?? true;
  state.reply = newValue?.reply ?? true;
});
watch(state, value => {
  modelValue.value = value;
});
</script>
