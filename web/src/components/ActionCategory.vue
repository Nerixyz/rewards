<template>
  <div class="flex flex-col gap-3 border border-white border-opacity-20 rounded-md m-1 p-3">
    <h2 class="font-mono font-bold italic text-xl text-red-200">{{ name }}</h2>
    <h3 v-if="description">{{ description }}</h3>
    <slot />
  </div>
</template>

<script setup lang="ts">
withDefaults(defineProps<{ name: string; description?: string }>(), { description: '' });
</script>
