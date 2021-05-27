<template>
  <div class="flex flex-col justify-center items-center w-full h-full gap-2">
    <Heading>Authentication Failed</Heading>
    <SubHeading class="mt-2">You denied access or something broke</SubHeading>
    <div>
      Details:
      <span class="font-mono ml-1">{{ message }}</span>
    </div>
    <CButton href="/">Go home</CButton>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref } from 'vue';
import Heading from '../components/core/Heading.vue';
import SubHeading from '../components/core/SubHeading.vue';
import CButton from '../components/core/CButton.vue';

export default defineComponent({
  name: 'FailedAuth',
  components: { CButton, SubHeading, Heading },
  setup() {
    const message = ref('<no data>');
    onMounted(() => {
      message.value = decodeURIComponent(location.hash.substring(1)) || message.value;
    });

    return { message };
  },
});
</script>
