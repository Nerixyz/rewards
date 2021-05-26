<template>
  <div
    class="
      bg-transparent
      rounded-t-lg rounded-b-sm
      border-b-2 border-red
      transition-colors
      hover:bg-gray-350
      focus:bg-gray-500
      relative
    "
  >
    <span
      class="
        transform
        -translate-y-3
        scale-75
        text-pink-600
        absolute
        left-1
        right-auto
        max-w-full
        overflow-hidden overflow-ellipsis
        whitespace-nowrap
        pointer-events-none
        origin-top-left
        top-3
      "
    >
      {{ label }}
    </span>
    <WarnIcon
      v-if="warn"
      class="
        text-yellow-400
        absolute
        left-auto
        right-1
        max-w-full
        overflow-hidden overflow-ellipsis
        whitespace-nowrap
        pointer-events-none
        origin-top-left
        top-5
      "
    />
    <input
      :value="modelValue"
      :disabled="disabled"
      @input="onInput"
      class="bg-transparent w-full h-full px-3 py-2 border-none mt-2 outline-none"
    />
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import WarnIcon from '../icons/WarnIcon.vue';

export default defineComponent({
  name: 'TextField',
  props: {
    placeholder: {
      type: String,
      required: false,
    },
    modelValue: {
      type: String,
      required: false,
    },
    disabled: {
      type: Boolean,
      required: false,
    },
    label: {
      type: String,
      required: true,
    },
    warn: {
      type: Boolean,
      required: false,
    },
  },
  components: { WarnIcon },
  emits: ['update:modelValue'],
  setup(_props, { emit }) {
    const onInput = (e: Event) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      emit('update:modelValue', (e.target as any).value);
    };

    return { onInput };
  },
});
</script>
