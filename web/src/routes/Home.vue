<template>
  <div class="flex flex-col justify-center items-center w-full h-full gap-4">
    <Heading>Rewards</Heading>
    <SubHeading>Enhance your stream with custom rewards</SubHeading>
    <CButton v-if="!auth" href="/api/v1/auth/twitch-auth"> <TwitchIcon />Login with twitch </CButton>
    <div v-else class="inline">
      Make sure to mod <b class="border-red border-b">RewardMore</b> in chat to be able to use rewards that require
      mod-access!
    </div>
    <a
      href="https://github.com/Nerixyz/rewards"
      target="_blank"
      title="GitHub Project"
      class="mt-8 transition-transform ease-cubic-out hover:scale-150"
    >
      <GithubIcon />
    </a>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import TwitchIcon from '../components/icons/TwitchIcon.vue';
import Heading from '../components/core/Heading.vue';
import SubHeading from '../components/core/SubHeading.vue';
import CButton from '../components/core/CButton.vue';
import { useApi } from '../api/plugin';
import GithubIcon from '../components/icons/GithubIcon.vue';

export default defineComponent({
  name: 'Home',
  components: { GithubIcon, SubHeading, Heading, TwitchIcon, CButton },
  setup() {
    const api = useApi();
    return {
      auth: api.isAuthenticated,
    };
  },
});
</script>
