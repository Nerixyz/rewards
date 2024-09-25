<template>
  <BaseInput :is-focused="isFocused" :is-occupied="isOccupied" :label="label">
    <input
      :value="modelValue"
      :disabled="disabled"
      class="bg-transparent w-full h-full px-3 py-2 border-none mt-2 outline-none"
      type="number"
      :min="min"
      :max="max"
      required
      @input="onInput"
      @focus="onFocus"
      @blur="onBlur"
    />
    <WarnIcon
      class="text-yellow-400 on-invalid-warn absolute left-auto right-10 max-w-full overflow-hidden overflow-ellipsis whitespace-nowrap pointer-events-none origin-top-left top-4"
    />
  </BaseInput>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import WarnIcon from '../icons/WarnIcon.vue';
import BaseInput from './BaseInput.vue';

withDefaults(
  defineProps<{
    placeholder?: string;
    disabled?: boolean;
    label: string;
    min: number;
    max: number;
  }>(),
  { placeholder: '', disabled: false },
);
const [modelValue] = defineModel<number>({ default: 0 });

const onInput = (e: Event) => {
  const el = e.target as HTMLInputElement;
  if (el.validity.valid) {
    modelValue.value = (e.target as HTMLInputElement).valueAsNumber;
  }
};

const isFocused = ref(false);
const onFocus = () => {
  isFocused.value = true;
};
const onBlur = () => {
  isFocused.value = false;
};

const isOccupied = computed(() => !!(isFocused.value || modelValue.value));
</script>
<style scoped>
input:not(:invalid) + .on-invalid-warn {
  display: none;
}
</style>
