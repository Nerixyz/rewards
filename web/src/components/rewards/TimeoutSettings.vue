<template>
  <TextField v-model="state.duration" label="Duration" :warn="warn" />
  <CSwitch v-model="state.vip" label="Block VIP Timeouts" />
</template>

<script setup lang="ts">
import { computed, reactive, watch } from 'vue';
import TextField from '../core/TextField.vue';
import { isValidRewardDurationExpression } from '../../utilities';
import { TimeoutRewardData } from '../../api/types';
import CSwitch from '../core/CSwitch.vue';
import { StaticRewardData } from '../../api/rewards-data';

const emit = defineEmits<{
  'update:warn': [warn: boolean];
}>();
const [modelValue] = defineModel<TimeoutRewardData | null>({ required: true });

const state = reactive(modelValue.value ?? { ...StaticRewardData.Timeout.defaultOptions });

const warn = computed(() => !isValidRewardDurationExpression(state.duration.trim() ?? '-'));
watch(warn, v => emit('update:warn', v));

watch(modelValue, newValue => {
  state.duration = newValue?.duration ?? StaticRewardData.Timeout.defaultOptions.duration;
  state.vip = newValue?.vip ?? StaticRewardData.Timeout.defaultOptions.vip;
});
watch(state, value => {
  modelValue.value = value;
});
</script>
