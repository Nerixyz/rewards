<template>
  <MainLayout>
    <div>
      <div class="flex gap-4 items-center mb-10">
        <BackIcon
          class="text-red bg-white bg-opacity-0 w-10 h-10 p-2 rounded-full cursor-pointer hover:bg-opacity-20"
          @click="$router.back()"
        />
        <h1 class="font-bold font-mono text-5xl">New Reward (Test)</h1>
      </div>
      <div class="flex flex-col">
        <RewardEditor
          :can-update="false"
          :is-new="true"
          main-action="Create"
          :reward-model="undefined"
          @done="saveReward"
        />
      </div>
    </div>
  </MainLayout>
</template>

<script setup lang="ts">
import MainLayout from '../components/MainLayout.vue';
import RewardEditor from '../components/RewardEditor.vue';
import { InputReward } from '../api/types';
import BackIcon from '../components/icons/BackIcon.vue';

const saveReward = (target: InputReward) => {
  console.log({ target });
};
</script>
