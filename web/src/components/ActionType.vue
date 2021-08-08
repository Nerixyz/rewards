<template>
  <li
    tabindex="0"
    :class="[
      `
      flex flex-col
      border-2 border-transparent
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
      selected ? 'border-red' : '',
    ]"
    @click="setCurrent"
    @keydown.enter="setCurrent"
  >
    <h3 class="font-bold font-mono text-xd">{{ action }}</h3>
    <span class="ml-3">{{ description }}</span>
  </li>
</template>

<script lang="ts">
import { computed, defineComponent, PropType, toRefs } from 'vue';
import { RewardDataMap } from '../api/types';

export default defineComponent({
  name: 'ActionType',
  props: {
    action: {
      type: String as PropType<keyof RewardDataMap>,
      required: true,
    },
    modelValue: {
      type: String as PropType<keyof RewardDataMap>,
      required: true,
    },
    description: {
      type: String,
      required: true,
    },
  },
  emits: ['update:model-value'],
  setup(props, { emit }) {
    const { action } = toRefs(props);

    const setCurrent = () => {
      emit('update:model-value', action.value);
    };
    const selected = computed(() => props.modelValue === props.action);
    return { setCurrent, selected };
  },
});
</script>
