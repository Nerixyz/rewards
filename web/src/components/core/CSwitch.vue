<template>
  <SwitchGroup>
    <div class="flex items-center cursor-pointer">
      <SwitchLabel class="mr-4 select-none cursor-pointer">{{ label }}</SwitchLabel>
      <Switch
        :model-value="modelValue"
        :class="modelValue ? 'bg-red' : 'bg-gray-500'"
        class="
          relative
          inline-flex
          flex-shrink-0
          h-6
          w-10
          border-2 border-transparent
          rounded-full
          cursor-pointer
          transition-colors
          ease-in-out
          duration-200
          focus:outline-none
        "
        @update:model-value="onUpdate"
      >
        <span
          aria-hidden="true"
          :class="modelValue ? 'translate-x-4' : 'translate-x-0'"
          class="
            pointer-events-none
            inline-block
            h-5
            w-5
            rounded-full
            bg-white
            shadow-lg
            transform
            ring-0
            transition
            ease-in-out
            duration-200
          "
        />
      </Switch>
    </div>
  </SwitchGroup>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import { Switch, SwitchLabel, SwitchGroup } from '@headlessui/vue';

export default defineComponent({
  name: 'CSwitch',
  components: { Switch, SwitchLabel, SwitchGroup },
  props: {
    modelValue: {
      type: Boolean,
      required: true,
    },
    label: {
      type: String,
      required: true,
    },
  },
  emits: ['update:modelValue'],
  setup(_, { emit }) {
    const onUpdate = (v: boolean) => {
      emit('update:modelValue', v);
    };
    return { onUpdate };
  },
});
</script>
