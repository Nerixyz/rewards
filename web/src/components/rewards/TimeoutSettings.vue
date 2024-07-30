<template>
  <TextField v-model="modelValue.duration" label="Duration" :warn="warn" />
  <CSwitch v-model="modelValue.vip" label="Block VIP Timeouts" />
</template>

<script setup lang="ts">
import { computed, watch } from 'vue';
import TextField from '../core/TextField.vue';
import { isValidRewardDurationExpression } from '../../utilities';
import { TimeoutRewardData } from '../../api/types';
import CSwitch from '../core/CSwitch.vue';

const emit = defineEmits<{
  'update:warn': [warn: boolean];
}>();
const [modelValue] = defineModel<TimeoutRewardData>({ required: true });

const warn = computed(() => !isValidRewardDurationExpression(modelValue.value.duration.trim() ?? '-'));
watch(warn, v => emit('update:warn', v));
</script>
