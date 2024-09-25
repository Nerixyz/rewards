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
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  label: string;
  isOccupied: boolean;
  isFocused: boolean;
}>();
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
