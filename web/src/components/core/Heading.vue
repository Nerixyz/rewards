<template>
  <h1 class="font-bold font-serif text-7xl text-white"><slot /></h1>
</template>

<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
  name: 'Heading',
});
</script>
