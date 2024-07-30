<template>
  <li
    tabindex="0"
    :class="[
      `
      flex flex-col
      border-2
      select-none
      cursor-pointer
      rounded-md
      p-2
      bg-white
      bg-opacity-0
      outline-none
      focus:bg-opacity-5
      hover:bg-opacity-10
      `,
      selected ? 'border-red' : 'border-transparent',
    ]"
    @click="setCurrent"
    @keydown.enter="setCurrent"
  >
    <h3 class="font-bold font-mono text-xd">{{ actionName ?? action }}</h3>
    <span class="ml-3">{{ description }}</span>
  </li>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { RewardDataMap } from '../api/types';

const props = defineProps<{ action: keyof RewardDataMap; actionName?: string; description: string }>();
const [modelValue] = defineModel<keyof RewardDataMap>({ required: true });

const setCurrent = () => {
  modelValue.value = props.action;
};
const selected = computed(() => modelValue.value === props.action);
</script>
