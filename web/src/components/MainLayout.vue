<template>
  <div class="px-20 pt-5 xl:max-w-7xl mx-auto">
    <slot />
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
  name: 'MainLayout',
});
</script>
