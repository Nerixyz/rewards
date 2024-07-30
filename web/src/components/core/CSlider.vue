<template>
  <div class="w-full mb-2">
    <span v-if="label">{{ label }}</span>
    <div class="flex w-full gap-3 px-1">
      <!-- eslint-disable-next-line vue/html-self-closing -->
      <input class="flex-grow" type="range" :value="modelValue" :min="min" :max="max" :step="step" @input="onUpdate" />
      <span>{{ currentValueStr }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, toRefs } from 'vue';

const props = withDefaults(
  defineProps<{
    label?: string;
    min?: number;
    max?: number;
    step?: number;
  }>(),
  { min: 0, max: 100, step: 1, label: undefined },
);
const [modelValue] = defineModel<number>({ required: true });

const { min, step } = toRefs(props);

const onUpdate = (e: Event) => {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  modelValue.value = Number((e.target as any).value) || min.value;
};
const currentValueStr = computed(() => (Math.round(modelValue.value / step.value) * step.value).toString());
</script>
