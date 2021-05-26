<template>
  <CDialog :title="`${isAdding ? 'Add' : 'Edit'} Reward`" :open="open" @dialog-closed="onDialogClosed">
    <div v-if="loading">
      <span>Loading...</span>
      <DialogButtons>
        <OutlinedButton @click.prevent="closeAll"> Cancel </OutlinedButton>
      </DialogButtons>
    </div>
    <div v-else-if="error">
      docNotl an error occurred!
      <br />
      <pre>{{ error }}</pre>
      <DialogButtons><OutlinedButton @click="closeAll"> Cancel </OutlinedButton></DialogButtons>
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
          />
        </div>
      </div>
      <DialogButtons>
        <OutlinedButton @click.prevent="closeAll"> Cancel </OutlinedButton>
        <CButton :disabled="v$.$invalid"> {{ isAdding ? 'Add' : 'Edit' }} </CButton>
      </DialogButtons>
    </form>
  </CDialog>
</template>

<script lang="ts">
import { computed, defineComponent, PropType, reactive, toRefs, watch } from 'vue';
import CDialog from './core/CDialog.vue';
import OutlinedButton from './core/OutlinedButton.vue';
import CButton from './core/CButton.vue';
import { asyncRefs, isValidDuration, tryAsync } from '../utilities';
import { useApi } from '../api/plugin';
import TextField from './core/TextField.vue';
import DialogButtons from './DialogButtons.vue';
import useVuelidate from '@vuelidate/core';
import { required, numeric } from '@vuelidate/validators';
import { assignDefaultToModel, assignToVRewardModel, toInputReward, VRewardModel } from '../api/model-conversion';
import CDropdown from './core/CDropdown.vue';
import { defaultNewReward, RewardTypes } from '../api/rewards-data';
import { Reward } from '../api/types';
import TSESettings from './rewards/TSESettings.vue';

export default defineComponent({
  name: 'AddOrEditRewardDialog',
  components: { TSESettings, CDropdown, DialogButtons, TextField, CButton, OutlinedButton, CDialog },
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

    const { loading, error } = asyncRefs(false);
    const clearAsyncRefs = () => {
      loading.value = false;
      error.value = null;
    };

    const closeAll = () => {
      emit('update:open', false);
    };

    const onDialogClosed = () => {
      clearAsyncRefs();
    };

    const rewardState = reactive<VRewardModel>(defaultNewReward());
    watch(rewardData, (newData?: Reward) => {
      if (!newData) {
        assignDefaultToModel(rewardState);
      } else {
        assignToVRewardModel(newData, rewardState);
      }
    });

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

    const isAdding = computed(() => !rewardData.value);

    const onSubmit = () => {
      tryAsync(
        async () => {
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
        },
        loading,
        error,
      );
    };

    return { loading, error, closeAll, v$, rewardState, RewardTypes, onSubmit, isAdding, onDialogClosed, open };
  },
});
</script>
