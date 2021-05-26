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
      disabled:bg-gray-lightest
      hover:border-red-dark
      hover:text-red-dark
      transition-colors
      focus:ring-2 focus:ring-offset-2 focus:ring-pink-700 focus:ring-offset-gray-darkest
      focus:outline-none
    "
  >
    <span class="flex items-center justify-center"><slot /></span>
  </button>
</template>

<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
  name: 'OutlinedButton',
  props: {
    disabled: {
      type: Boolean,
      required: false,
    },
  },
});
</script>
