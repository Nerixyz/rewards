<template>
  <svg xmlns="http://www.w3.org/2000/svg" version="1.1" width="20" height="20" viewBox="0 0 24 24">
    <path
      v-if="$props.dir === false"
      fill="currentColor"
      d="M18 21L14 17H17V7H14L18 3L22 7H19V17H22M2 19V17H12V19M2 13V11H9V13M2 7V5H6V7H2Z"
    />
    <path
      v-if="$props.dir === 'asc'"
      fill="currentColor"
      d="M19 17H22L18 21L14 17H17V3H19M2 17H12V19H2M6 5V7H2V5M2 11H9V13H2V11Z"
    />
    <path
      v-if="$props.dir === 'desc'"
      fill="currentColor"
      d="M19 7H22L18 3L14 7H17V21H19M2 17H12V19H2M6 5V7H2V5M2 11H9V13H2V11Z"
    />
  </svg>
</template>
<script lang="ts" setup>
import { SortDirection } from '@tanstack/vue-table';

defineProps<{ dir: SortDirection | false }>();
</script>
