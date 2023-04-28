<template>
  <form class="p-4" @submit.prevent="">
    <div class="flex flex-col gap-4">
      <div class="flex bg-white bg-opacity-10 p-4 rounded-md items-center">
        <input
          v-model="reward.title"
          placeholder="Title"
          class="bg-transparent flex-grow outline-none placeholder-gray-700 font-mono font-bold text-white text-2xl"
        />
        <WarnIcon v-if="v$.title.$invalid" class="text-yellow-400" />
      </div>

      <TextField v-model="reward.prompt" label="Prompt" :warn="v$.prompt.$invalid" />
      <div class="flex items-center gap-10 flex-wrap">
        <div class="flex flex-col gap-3 flex-grow">
          <TextField v-model="reward.cost" label="Cost" :warn="v$.cost.$invalid" />
          <TextField v-model="reward.cooldown" label="Cooldown" :warn="v$.cooldown.$invalid" />
          <TextField v-model="reward.usesPerStream" label="Uses per Stream" :warn="v$.usesPerStream.$invalid" />
          <TextField v-model="reward.usesPerUser" label="Uses per User" :warn="v$.usesPerUser.$invalid" />
          <TextField v-model="reward.liveDelay" label="Live Delay" :warn="v$.liveDelay.$invalid" />
          <CSwitch v-model="reward.autoAccept" label="Automatically Accept Redemptions" />
        </div>
        <div class="flex flex-col items-center justify-center gap-5 p-5">
          <div
            class="flex flex-col p-4 rounded-md w-36 h-36 justify-center items-center gap-5"
            :style="{ 'background-color': reward.color }"
          >
            <img
              class="w-12 h-12"
              :src="reward.imageUrl ?? 'https://static-cdn.jtvnw.net/custom-reward-images/default-4.png'"
              alt="Reward Image"
            />
            <div class="bg-black bg-opacity-50 px-3 py-0.5 rounded-md">
              💵<span class="ml-2">{{ v$.cost.$invalid ? '?' : reward.cost }}</span>
            </div>
          </div>
          <div class="flex items-center gap-2">
            Background Color:
            <input v-model="reward.color" type="color" class="rounded-md ring ring-transparent focus:ring-red" />
          </div>
        </div>
      </div>

      <ActionEditor
        v-model:action="reward.action.type"
        v-model:data="reward.action.data"
        :is-new="isNew"
        @update:warn="updateActionWarn"
      />
    </div>

    <div class="flex flex-wrap mt-5 justify-end">
      <OutlinedButton v-if="canUpdate" :disabled="isInvalid" @click="onUpdate"> Update </OutlinedButton>
      <CButton :disabled="isInvalid" @click="onDone">
        {{ mainAction }}
      </CButton>
    </div>
  </form>
</template>

<script lang="ts">
import { computed, defineComponent, PropType, reactive, ref, watch } from 'vue';
import { assignDefaultToModel, assignToVRewardModel, toInputReward } from '../api/model-conversion';
import { defaultNewReward } from '../api/rewards-data';
import OutlinedButton from './core/OutlinedButton.vue';
import CButton from './core/CButton.vue';
import CSwitch from './core/CSwitch.vue';
import WarnIcon from './icons/WarnIcon.vue';
import { Reward } from '../api/types';
import useVuelidate from '@vuelidate/core';
import { numeric, required } from '@vuelidate/validators';
import { isValidDuration } from '../utilities';
import TextField from './core/TextField.vue';
import ActionEditor from './ActionEditor.vue';

export default defineComponent({
  name: 'RewardEditor',
  components: { ActionEditor, TextField, CButton, OutlinedButton, WarnIcon },
  props: {
    rewardModel: {
      type: Object as PropType<Reward>,
      required: false,
      default: undefined,
    },
    canUpdate: {
      type: Boolean,
      required: false,
    },
    mainAction: {
      type: String,
      required: true,
    },
    isNew: {
      type: Boolean,
      default: false,
    },
  },
  emits: ['update', 'done'],
  setup(props, { emit }) {
    const reward = reactive(defaultNewReward());
    watch(
      () => props.rewardModel,
      v => {
        if (v) {
          assignToVRewardModel(v, reward);
        } else {
          assignDefaultToModel(reward);
        }
      },
      { immediate: true },
    );

    const v$ = useVuelidate(
      {
        title: { required },
        cost: { required, numeric },
        usesPerStream: { numeric },
        usesPerUser: { numeric },
        cooldown: { isValidDuration },
        prompt: { required },
        liveDelay: {},
        autoAccept: {},
      },
      reward,
    );

    const actionWarn = ref(false);
    const updateActionWarn = (warn: boolean) => (actionWarn.value = warn);
    const isInvalid = computed(() => actionWarn.value || v$.value.$invalid);

    const buttons = {
      onUpdate: () => {
        if (isInvalid.value) return;

        emit('update', toInputReward(reward));
      },
      onDone: () => {
        if (isInvalid.value) return;

        emit('done', toInputReward(reward));
        assignDefaultToModel(reward);
      },
    };
    return { reward, v$, isInvalid, updateActionWarn, ...buttons };
  },
});
</script>
