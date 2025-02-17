<template>
  <BaseInput :is-focused="isFocused" :is-occupied="isOccupied" :label="label">
    <WarnIcon
      v-if="warn"
      class="text-yellow-400 absolute left-auto right-2 max-w-full overflow-hidden overflow-ellipsis whitespace-nowrap pointer-events-none origin-top-left top-4"
    />
    <input
      :value="modelValue"
      :disabled="disabled"
      class="bg-transparent w-full h-full px-3 py-2 border-none mt-2 outline-none"
      @input="onInput"
      @focus="onFocus"
      @blur="onBlur"
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
    warn?: boolean;
  }>(),
  { placeholder: '', disabled: false, warn: false },
);
const [modelValue] = defineModel<string>({ required: true });

const onInput = (e: Event) => {
  modelValue.value = (e.target as HTMLInputElement).value;
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
