<template>
  <CDialog :title="`${isAdding ? 'Add' : 'Edit'} Reward`" :open="open">
    <div v-if='loading'>
      Loading...
    </div>
    <div v-else-if='error'>
      docNotl an error occurred!
      <br/>
      <pre>{{error}}</pre>
      <DialogButtons><OutlinedButton @click="closeAll"> Cancel </OutlinedButton></DialogButtons>
    </div>
    <form v-else @submit.prevent='onSubmit'>
      <div class='flex gap-4 min-w-40vw'>
        <div class='flex flex-col gap-3 flex-grow w-full'>
          <TextField label="Title" v-model='rewardState.title' :warn='v$.title.$invalid' />
          <TextField label="Cost" v-model='rewardState.cost' :warn='v$.cost.$invalid' />
          <TextField label="Prompt" v-model='rewardState.prompt' :warn='v$.prompt.$invalid' />
          <TextField label="Uses per Stream" v-model='rewardState.usesPerStream' :warn='v$.usesPerStream.$invalid' />
          <TextField label="Uses per User" v-model='rewardState.usesPerUser' :warn='v$.usesPerUser.$invalid' />
          <TextField label="Cooldown" v-model='rewardState.cooldown' :warn='v$.cooldown.$invalid' />
        </div>
        <div class='flex-grow w-full'>
          <CDropdown v-model='rewardState.action.type' :options='RewardTypes' class='z-30 pb-5'/>

          <TimeoutSettings v-if='rewardState.action.type === "Timeout"' v-model='rewardState.action.data'/>
        </div>
      </div>
      <DialogButtons>
        <OutlinedButton @click.prevent="closeAll"> Cancel </OutlinedButton>
        <CButton :disabled='v$.$invalid'> {{ isAdding ? 'Add' : 'Edit' }} </CButton>
      </DialogButtons>
    </form>
  </CDialog>
</template>

<script lang="ts">
import { computed, defineComponent, reactive, toRefs, watch } from 'vue';
import CDialog from './core/CDialog.vue';
import OutlinedButton from './core/OutlinedButton.vue';
import CButton from './core/CButton.vue';
import { asyncRefs, isValidDuration, tryAsync } from '../utilities';
import { useApi } from '../api/plugin';
import TextField from './core/TextField.vue';
import DialogButtons from './DialogButtons.vue';
import useVuelidate from '@vuelidate/core';
import { required , numeric,  } from '@vuelidate/validators';
import { assignDefaultToModel, assignToVRewardModel, toInputReward, VRewardModel } from '../api/model-conversion';
import CDropdown from './core/CDropdown.vue';
import TimeoutSettings from './rewards/TimeoutSettings.vue';
import { defaultNewReward, RewardTypes } from '../api/rewards-data';
import { Reward } from '../api/types';

export default defineComponent<{open: boolean,
  broadcasterId: string,
  rewardData?: Reward}>({
  name: 'AddOrEditRewardDialog',
  components: { TimeoutSettings, CDropdown, DialogButtons, TextField, CButton, OutlinedButton, CDialog },
  props: {
    open: Boolean,
    broadcasterId: String,
    rewardData: Object,
  },
  emits: ['update:open', 'close', 'added', 'updated'],
  setup(props, { emit }) {
    const { broadcasterId, rewardData } = toRefs(props);
    const api = useApi();

    const { loading, error } = asyncRefs(false);
    const clearAsyncRefs = () => {
      loading.value = false;
      error.value = null;
    };

    const closeAll = () => {
      emit('update:open', false);
      clearAsyncRefs();
    };

    const rewardState = reactive<VRewardModel>(defaultNewReward());
    watch(rewardData, (newData: Reward) => {
      if(!newData) {
        assignDefaultToModel(rewardState);
      } else {
        assignToVRewardModel(newData, rewardState);
      }
    });

    const v$ = useVuelidate({
      title: {required},
      cost: {required, numeric},
      usesPerStream: {numeric},
      usesPerUser: {numeric},
      cooldown: {isValidDuration},
      prompt: {required}
    }, rewardState);

    const isAdding = computed(() => !rewardData.value);

    const onSubmit = () => {
      tryAsync(async () => {
        let response;
        if(isAdding.value) {
          response = await api.addReward(broadcasterId.value, toInputReward(rewardState));
        } else {
          response = await api.updateReward(broadcasterId.value, toInputReward(rewardState), (rewardData.value as Reward).twitch.id);
        }

        // not really needed but not bad anyways
        assignToVRewardModel(response, response);

        emit(isAdding.value ? 'added' : 'updated', response);
        closeAll();
      }, loading, error);
    }

    return { loading, error, closeAll, v$, rewardState, RewardTypes, onSubmit, isAdding };
  },
});
</script>
