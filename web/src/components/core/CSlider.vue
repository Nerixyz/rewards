<template>
  <div class="w-full mb-2">
    <span>{{ label }}</span>
    <div class="flex w-full gap-3 px-1">
      <input class='flex-grow' type="range" :value="value" :min="min" :max="max" :step="step" @input="onUpdate" />
      <span>{{ currentValueStr }}</span>
    </div>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, ref, toRefs, watch } from 'vue';

export default defineComponent({
  name: 'CSlider',
  props: {
    label: {
      type: String,
      required: true,
    },
    modelValue: {
      type: Number,
      required: true,
    },
    min: {
      type: Number,
      required: false,
      default: 0,
    },
    max: {
      type: Number,
      required: false,
      default: 100,
    },
    step: {
      type: Number,
      required: false,
      default: 1,
    },
  },
  emits: ['update:modelValue'],
  setup(props, { emit }) {
    const { modelValue, min, step } = toRefs(props);
    const value = ref(modelValue.value);
    watch(modelValue, v => {
      value.value = v;
    });
    const onUpdate = (e: Event) => {
      emit('update:modelValue', Number((e.target as any).value) || min.value);
    };
    const currentValueStr = computed(() => (Math.round(value.value / step.value) * step.value).toString());
    return { value, onUpdate, currentValueStr };
  },
});
</script>
