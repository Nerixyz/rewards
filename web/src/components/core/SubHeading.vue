<template>
  <h1 class="font-light font-serif text-4xl text-white"><slot /></h1>
</template>

<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
  name: 'SubHeading',
});
</script>
