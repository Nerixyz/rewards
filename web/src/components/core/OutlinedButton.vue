<template>
  <button
    :disabled="disabled"
    class="
      inline-flex
      justify-center
      items-center
      px-8
      m-2
      h-10
      select-none
      uppercase
      border-red border-2
      rounded-md
      text-red
      font-bold
      shadow-md
      disabled:text-gray-600
      disabled:border-gray-600
      disabled:cursor-not-allowed
      disabled:ring-gray-600
      hover:border-red-dark
      hover:text-red-dark
      transition transition-colors
      focus:ring-2 focus:ring-offset-2 focus:ring-pink-700 focus:ring-offset-gray-darkest
      focus:outline-none
    "
    @click.capture="tryClick"
  >
    <span class="flex items-center justify-center"><slot /></span>
  </button>
</template>

<script lang="ts">
import { defineComponent, toRefs } from 'vue';

export default defineComponent({
  name: 'OutlinedButton',
  props: {
    disabled: {
      type: Boolean,
      required: false,
    },
  },
  setup(props) {
    const { disabled } = toRefs(props);

    const tryClick = (e: Event) => {
      if (disabled.value) {
        e.stopPropagation();
      }
    };

    return { tryClick };
  },
});
</script>
