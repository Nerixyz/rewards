<template>
  <div class="flex flex-col items-center justify-center w-full h-full">
    <Heading>Not Found</Heading>
    <CButton href="/">Go Home</CButton>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import Heading from '../components/core/Heading.vue';
import CButton from '../components/core/CButton.vue';

export default defineComponent({
  name: 'NotFound',
  components: { CButton, Heading },
});
</script>
