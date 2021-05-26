<template>
  <div class="flex flex-col justify-center items-center w-full h-full gap-4">
    <Heading>Some cool heading...</Heading>
    <SubHeading>An interesting description</SubHeading>
    <CButton v-if="!auth" :href="url"> <TwitchIcon />Login with twitch </CButton>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref } from 'vue';
import TwitchIcon from '../components/icons/TwitchIcon.vue';
import Heading from '../components/core/Heading.vue';
import SubHeading from '../components/core/SubHeading.vue';
import CButton from '../components/core/CButton.vue';
import { useApi } from '../api/plugin';

export default defineComponent({
  name: 'Home',
  components: { SubHeading, Heading, TwitchIcon, CButton },
  setup() {
    const url = ref('/');
    const api = useApi();
    onMounted(() =>
      api
        .getTwitchAuthUrl()
        .then(({ url: authUrl }) => (url.value = authUrl))
        .catch(console.error),
    );

    return {
      url,
      auth: api.isAuthenticated,
    };
  },
});
</script>
