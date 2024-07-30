<template>
  <CDropdown
    v-model="state.platform"
    label="Slots"
    :options="[
      { display: '7TV', value: 'SevenTv' },
      { display: 'BTTV', value: 'Bttv' },
      { display: 'FFZ', value: 'Ffz' },
    ]"
  />
  <CSwitch v-model="state.reply" label="Reply after successful redemption" />
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue';
import CDropdown from '../core/CDropdown.vue';
import CSwitch from '../core/CSwitch.vue';
import { RemEmoteRewardData } from '../../api/types';

const [modelValue] = defineModel<RemEmoteRewardData>({ required: true });

const state = reactive({ reply: true, ...modelValue.value });

watch(modelValue, newValue => {
  state.platform = newValue.platform ?? 'SevenTv';
  state.reply = newValue.reply ?? true;
});
watch(state, value => {
  modelValue.value = value;
});
</script>
