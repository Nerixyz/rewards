<template>
  <Listbox :model-value="modelValue" @update:model-value="onUpdate">
    <div class="relative min-w-10rem w-full outline-none">
      <ListboxButton
        class="
          relative
          w-full
          py-1
          pl-3
          pr-5
          text-left
          bg-transparent
          border border-gray-900 border-opacity-30
          focus:border-opacity-100 focus:border-red
          focus:bg-gray-350
          hover:bg-gray-350
          hover:border-red
          transition-colors
          rounded-md
          shadow-md
          outline-none
          focus:outline-none
        "
      >
        <span class="block truncate">{{ modelValue }}</span>
        <span class="absolute inset-0 left-auto right-0 flex items-center pr-2 pointer-events-none"
          ><ChevronDown class="w-5 h-5 text-white"
        /></span>
      </ListboxButton>

      <transition
        enter-active-class="transition duration-100 ease-out"
        enter-from-class="opacity-0 scale-90"
        enter-to-class="opacity-100 scale-100"
        leave-active-class="transition duration-100 ease-in"
        leave-from-class="opacity-100 scale-100"
        leave-to-class="opacity-0 scale-75"
      >
        <ListboxOptions
          class="
            transform
            transition-transform
            absolute
            origin-top
            w-full
            mt-1
            text-base
            bg-gray-500 bg-opacity-20
            backdrop-filter backdrop-blur-md
            border border-gray-900 border-opacity-20
            rounded-md
            shadow-lg
          "
        >
          <ListboxOption
            v-for="option of options"
            :key="option.value"
            v-slot="{ active, selected }"
            :value="option.value"
          >
            <li
              :class="[
                active ? 'text-red bg-grey-500' : 'bg-grey-400',
                'cursor-pointer select-none relative py-1 pl-5 pr-4',
              ]"
            >
              <span :class="[selected ? 'font-medium' : 'font-normal', 'block truncate']">{{ option.display }}</span>
            </li>
          </ListboxOption>
        </ListboxOptions>
      </transition>
    </div>
  </Listbox>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue';
import ChevronDown from '../icons/ChevronDown.vue';
import { Listbox, ListboxButton, ListboxOption, ListboxOptions } from '@headlessui/vue';

export interface CDropdownOption {
  display: string;
  value: string;
}

export default defineComponent({
  name: 'CDropdown',
  components: { ChevronDown, Listbox, ListboxOptions, ListboxOption, ListboxButton },
  props: {
    modelValue: {
      type: String,
      required: true,
    },
    options: {
      type: Array as PropType<CDropdownOption[]>,
      required: true,
    },
  },
  emits: ['update:modelValue'],
  methods: {
    onUpdate(value: string) {
      this.$emit('update:modelValue', value);
    },
  },
});
</script>
