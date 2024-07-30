<template>
  <CSwitch label="Allow Explicit" :model-value="modelValue.allow_explicit" @update:model-value="updateExplicit" />
</template>

<script setup lang="ts">
import { SpotifyPlayOptions } from '../../api/types';
import CSwitch from '../core/CSwitch.vue';

const [modelValue] = defineModel<SpotifyPlayOptions>({ required: true });

const updateExplicit = (v: boolean) => {
  modelValue.value = { allow_explicit: v };
};
</script>
