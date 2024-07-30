<template>
  <component
    :is="href ? 'a' : 'button'"
    :href="href"
    :disabled="disabled"
    class="inline-flex justify-center items-center px-8 m-2 h-10 select-none uppercase bg-red rounded-md text-black font-bold shadow-md disabled:bg-gray-600 disabled:cursor-not-allowed disabled:ring-gray-600 hover:bg-red-dark transition-colors focus:ring-2 focus:ring-offset-2 focus:ring-pink-700 focus:ring-offset-gray-darkest focus:outline-none"
    @click.capture="tryClick"
  >
    <span class="flex items-center justify-center"><slot /></span>
  </component>
</template>

<script setup lang="ts">
import { toRefs } from 'vue';

const props = withDefaults(defineProps<{ disabled?: boolean; href?: string }>(), { href: undefined, disabled: false });

const { disabled } = toRefs(props);

const tryClick = (e: Event) => {
  if (disabled.value) {
    e.stopPropagation();
  }
};
</script>
