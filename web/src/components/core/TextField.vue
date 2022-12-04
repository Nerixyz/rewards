<template>
  <div class="block">
    <div
      :class="[
        `bg-transparent
        rounded-lg overflow-hidden
      border border-gray-900
      border-opacity-30
      bg-opacity-50
      transition-colors
      hover:bg-gray-350
      hover:bg-opacity-50
      relative nerix-underline-tf`,
        isFocused ? 'nerix-underline-tf-active bg-gray-350' : '',
      ]"
    >
      <span
        :class="[
          isOccupied
            ? `-translate-y-3
        scale-75`
            : '',
          isFocused ? 'text-red' : 'text-gray-700',
          `transform
        transition-transform
        transition-colors
        duration-200
        ease-cubic-out
        absolute
        left-2
        right-auto
        max-w-full
        overflow-hidden overflow-ellipsis
        whitespace-nowrap
        pointer-events-none
        origin-top-left
        top-3`,
        ]"
      >
        {{ label }}
      </span>
      <WarnIcon
        v-if="warn"
        class="text-yellow-400 absolute left-auto right-2 max-w-full overflow-hidden overflow-ellipsis whitespace-nowrap pointer-events-none origin-top-left top-4"
      />
      <input
        :value="modelValue"
        :disabled="disabled"
        class="bg-transparent w-full h-full px-3 py-2 border-none mt-2 outline-none"
        @input="onInput"
        @focus="onFocus"
        @blur="onBlur"
      />
    </div>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, ref, toRefs } from 'vue';
import WarnIcon from '../icons/WarnIcon.vue';

export default defineComponent({
  name: 'TextField',
  components: { WarnIcon },
  props: {
    placeholder: {
      type: String,
      required: false,
      default: '',
    },
    modelValue: {
      type: String,
      required: false,
      default: '',
    },
    disabled: {
      type: Boolean,
      required: false,
    },
    label: {
      type: String,
      required: true,
    },
    warn: {
      type: Boolean,
      required: false,
    },
  },
  emits: ['update:modelValue'],
  setup(props, { emit }) {
    const { modelValue } = toRefs(props);

    const onInput = (e: Event) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      emit('update:modelValue', (e.target as any).value);
    };

    const isFocused = ref(false);
    const onFocus = () => {
      isFocused.value = true;
    };
    const onBlur = () => {
      isFocused.value = false;
    };

    const isOccupied = computed(() => isFocused.value || modelValue.value);

    return { onInput, isOccupied, onFocus, onBlur, isFocused };
  },
});
</script>
<style scoped>
.nerix-underline-tf::after {
  content: '';
  position: relative;
  display: block;
  width: 100%;
  height: 0.1rem;
  background-color: #ff4151;
  transform: scaleX(0);
  transition: transform 150ms;
  transform-origin: right;
  top: 0.05rem;
}

.nerix-underline-tf.overflow-hidden::after {
  top: 0;
}

.nerix-underline-tf:hover::after,
.nerix-underline-tf-active::after {
  transform: scaleX(1);
  transform-origin: left;
}
</style>
