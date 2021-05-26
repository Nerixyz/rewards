<template>
  <div class="mt-8 flex justify-end">
    <slot />
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
  name: 'DialogButtons',
});
</script>
