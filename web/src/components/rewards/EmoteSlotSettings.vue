<template>
  <CSlider v-model="state.slots" label="Slots" :min="1" :max="10" />
  <TextField v-model="state.expiration" label="Expiration" />
  <CSwitch v-model="state.allow_unlisted" label="Allow unlisted emotes" />
  <CSwitch v-model="state.reply" label="Reply after successful redemption" />
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue';
import CSlider from '../core/CSlider.vue';
import CSwitch from '../core/CSwitch.vue';
import { SlotRewardData } from '../../api/types';
import TextField from '../core/TextField.vue';

const [modelValue] = defineModel<SlotRewardData>({ required: true });

const state = reactive({ allow_unlisted: true, reply: true, ...modelValue.value });

watch(modelValue, newValue => {
  state.expiration = newValue.expiration;
  state.slots = newValue.slots;
  state.allow_unlisted = newValue.allow_unlisted ?? true;
  state.reply = newValue.reply ?? true;
});
watch(state, value => {
  modelValue.value = value;
});
</script>
