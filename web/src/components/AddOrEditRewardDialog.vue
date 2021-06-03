<template>
  <CDialog :title="`${isAdding ? 'Add' : 'Edit'} Reward`" :open="open" @dialog-closed="resetDialog">
    <div v-if="dialogState.loading">
      <CLoader />
      <DialogButtons>
        <OutlinedButton @click.prevent="closeAll"> Cancel </OutlinedButton>
      </DialogButtons>
    </div>
    <div v-else-if="dialogState.error">
      docNotl an error occurred!
      <br />
      <span class="break-words font-mono">{{ dialogState.error }}</span>
      <DialogButtons><OutlinedButton @click="closeAll"> Cancel </OutlinedButton></DialogButtons>
    </div>
    <div v-else-if="dialogState.success">
      <TickIcon />
      <DialogButtons><OutlinedButton @click="closeAll"> Close </OutlinedButton></DialogButtons>
    </div>
    <form v-else @submit.prevent="onSubmit">
      <div class="flex gap-4 min-w-40vw">
        <div class="flex flex-col gap-3 flex-grow w-full">
          <TextField v-model="rewardState.title" label="Title" :warn="v$.title.$invalid" />
          <TextField v-model="rewardState.cost" label="Cost" :warn="v$.cost.$invalid" />
          <TextField v-model="rewardState.prompt" label="Prompt" :warn="v$.prompt.$invalid" />
          <TextField v-model="rewardState.usesPerStream" label="Uses per Stream" :warn="v$.usesPerStream.$invalid" />
          <TextField v-model="rewardState.usesPerUser" label="Uses per User" :warn="v$.usesPerUser.$invalid" />
          <TextField v-model="rewardState.cooldown" label="Cooldown" :warn="v$.cooldown.$invalid" />
        </div>
        <div class="flex-grow w-full">
          <CDropdown v-model="rewardState.action.type" :options="RewardTypes" class="z-30 pb-5" />

          <TSESettings
            v-if="['Timeout', 'SubOnly', 'EmoteOnly'].includes(rewardState.action.type)"
            v-model="rewardState.action.data"
            @update:warn="updateRewardWarning"
          />
          <BttvSlotSettings v-else-if="rewardState.action.type === 'BttvSlot'" v-model="rewardState.action.data" />
          <SpotifyPlayOptions
            v-else-if="['SpotifyPlay', 'SpotifyQueue'].includes(rewardState.action.type)"
            v-model="rewardState.action.data"
          />
        </div>
      </div>
      <DialogButtons>
        <OutlinedButton @click.prevent="closeAll"> Cancel </OutlinedButton>
        <CButton :disabled="!maySubmit"> {{ isAdding ? 'Add' : 'Edit' }} </CButton>
      </DialogButtons>
    </form>
  </CDialog>
</template>

<script lang="ts">
import { computed, defineComponent, PropType, reactive, ref, toRefs, watch } from 'vue';
import CDialog from './core/CDialog.vue';
import OutlinedButton from './core/OutlinedButton.vue';
import CButton from './core/CButton.vue';
import { isValidDuration } from '../utilities';
import { useApi } from '../api/plugin';
import TextField from './core/TextField.vue';
import DialogButtons from './DialogButtons.vue';
import useVuelidate from '@vuelidate/core';
import { required, numeric } from '@vuelidate/validators';
import { assignDefaultToModel, assignToVRewardModel, toInputReward, VRewardModel } from '../api/model-conversion';
import CDropdown from './core/CDropdown.vue';
import { defaultNewReward, RewardTypes, StaticRewardData } from '../api/rewards-data';
import { Reward } from '../api/types';
import TSESettings from './rewards/TSESettings.vue';
import { asyncDialog, tryAsyncDialog } from '../async-state';
import CLoader from './core/CLoader.vue';
import BttvSlotSettings from './rewards/BttvSlotSettings.vue';
import TickIcon from './icons/TickIcon.vue';
import SpotifyPlayOptions from './rewards/SpotifyPlayOptions.vue';

export default defineComponent({
  name: 'AddOrEditRewardDialog',
  components: {
    SpotifyPlayOptions,
    TickIcon,
    BttvSlotSettings,
    CLoader,
    TSESettings,
    CDropdown,
    DialogButtons,
    TextField,
    CButton,
    OutlinedButton,
    CDialog,
  },
  props: {
    open: {
      type: Boolean,
      required: false,
    },
    broadcasterId: {
      type: String,
      required: true,
    },
    rewardData: {
      type: Object as PropType<Reward | undefined>,
      required: true,
    },
  },
  emits: ['update:open', 'close', 'added', 'updated'],
  setup(props, { emit }) {
    const { broadcasterId, rewardData, open } = toRefs(props);
    const api = useApi();

    const { state: dialogState, reset: resetDialog } = asyncDialog(open);

    const closeAll = () => {
      emit('update:open', false);
    };

    const rewardState = reactive<VRewardModel>(defaultNewReward());
    const assignToState = (newData?: Reward) => {
      console.log('update data', newData);
      if (!newData) {
        assignDefaultToModel(rewardState);
      } else {
        assignToVRewardModel(newData, rewardState);
      }
    };
    watch(rewardData, assignToState);
    watch(open, value => {
      if (value && rewardData.value) {
        assignToState(rewardData.value);
      }
    });
    watch(
      () => rewardState.action.type,
      newType => {
        console.log('check new type');
        if (!StaticRewardData[newType].validOptions(rewardState.action.data)) {
          rewardState.action.data = StaticRewardData[newType].defaultOptions;
          rewardInvalid.value = false;
        }
      },
    );

    const v$ = useVuelidate(
      {
        title: { required },
        cost: { required, numeric },
        usesPerStream: { numeric },
        usesPerUser: { numeric },
        cooldown: { isValidDuration },
        prompt: { required },
      },
      rewardState,
    );
    const rewardInvalid = ref(false);
    const updateRewardWarning = (valid: boolean) => {
      rewardInvalid.value = valid;
    };
    const maySubmit = computed(() => !v$.value.$invalid && !rewardInvalid.value);

    const isAdding = computed(() => !rewardData.value);

    const onSubmit = () => {
      tryAsyncDialog(async () => {
        let response;
        if (isAdding.value) {
          response = await api.addReward(broadcasterId.value ?? '', toInputReward(rewardState));
        } else {
          response = await api.updateReward(
            broadcasterId.value ?? '',
            toInputReward(rewardState),
            rewardData.value?.twitch?.id ?? '',
          );
        }

        // clear the dialog
        assignDefaultToModel(rewardState);

        emit(isAdding.value ? 'added' : 'updated', response);
        closeAll();
      }, dialogState);
    };

    return {
      dialogState,
      resetDialog,
      closeAll,

      v$,
      rewardState,

      RewardTypes,
      onSubmit,
      isAdding,
      open,
      maySubmit,
      updateRewardWarning,
    };
  },
});
</script>
