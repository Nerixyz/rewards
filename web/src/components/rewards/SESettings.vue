<template>
  <TextField :model-value="modelValue" label="Duration" :warn="warn" @update:model-value="onUpdate" />
</template>

<script setup lang="ts">
import { computed, watch } from 'vue';
import TextField from '../core/TextField.vue';
import { isValidRewardDurationExpression } from '../../utilities';

const emit = defineEmits<{
  'update:warn': [warn: boolean];
}>();
const [modelValue] = defineModel<string>({ required: true });

const warn = computed(() => !isValidRewardDurationExpression(modelValue.value.trim()));
const onUpdate = (value: string) => {
  modelValue.value = value;
};
watch(warn, v => emit('update:warn', v));
</script>
