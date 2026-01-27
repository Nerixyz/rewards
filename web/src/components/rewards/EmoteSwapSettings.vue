<template>
  <CSwitch :model-value="sliderEnabled" label="Limit emotes" @update:model-value="updateSliderEnabled" />
  <NumberField
    v-if="state.limit !== null"
    v-model="state.limit"
    label="Limit"
    class="mt-2"
    :min="1"
    :max="isSeventv ? 1000 : 400"
  />
  <CSwitch v-model="state.allow_unlisted" label="Allow unlisted emotes" />
  <CSwitch v-model="state.reply" label="Reply after successful redemption" />
  <OutlinedButton v-if="!isNew" @click="editSwapEmotes">Manage Emotes</OutlinedButton>
</template>

<script setup lang="ts">
import { computed, reactive, watch } from 'vue';
import NumberField from '../core/NumberField.vue';
import { SwapRewardData } from '../../api/types';
import CSwitch from '../core/CSwitch.vue';
import OutlinedButton from '../core/OutlinedButton.vue';
import { useRouter } from 'vue-router';

defineProps<{ isSeventv: boolean; isNew: boolean }>();
const [modelValue] = defineModel<SwapRewardData | null>({ required: true });

const state = reactive({ limit: null, allow_unlisted: true, reply: true, ...modelValue.value });
const sliderEnabled = computed(() => state.limit !== null);
const updateSliderEnabled = (enabled: boolean) => {
  // set the "default" to 1
  state.limit = enabled ? 1 : null;
};

watch(modelValue, newValue => {
  state.limit = newValue?.limit ?? null;
  state.allow_unlisted = newValue?.allow_unlisted ?? true;
  state.reply = newValue?.reply ?? true;
});
watch(state, value => {
  modelValue.value = value;
});

const router = useRouter();
const editSwapEmotes = () => {
  router.push(router.currentRoute.value.path + '/swap-emotes');
};
</script>
