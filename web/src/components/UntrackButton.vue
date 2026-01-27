<template>
  <div class="w-full flex">
    <button
      class="inline-flex justify-center items-center mx-auto w-8 h-8 select-none uppercase hover:border-red border-2 border-transparent rounded-full text-red font-bold shadow-md disabled:text-gray-600 disabled:border-gray-600 disabled:cursor-not-allowed disabled:ring-gray-600 transition-colors focus:ring-2 focus:ring-offset-2 focus:ring-pink-700 focus:ring-offset-gray-darkest focus:outline-none"
      @click="$emit('untrack', $props.emoteId)"
    >
      <DeleteIcon />
    </button>
  </div>
</template>
<script lang="ts" setup>
import DeleteIcon from './icons/DeleteIcon.vue';

defineProps<{ emoteId: number }>();
defineEmits<{ untrack: [number] }>();
</script>
