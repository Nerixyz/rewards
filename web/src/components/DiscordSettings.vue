<template>
  <OutlinedButton @click="open"> <DiscordIcon /> Discord Logging </OutlinedButton>
  <CDialog title="Discord Settings" :open="state.value">
    <div class="flex flex-col gap-4">
      <div v-if="state.loading">
        <CLoader />
      </div>
      <div v-else-if="state.error">
        {{ state.error }}
      </div>
      <div v-else-if="state.success">
        <TickIcon />
      </div>
      <div v-else class="p-4">
        <TextField v-model="url" label="Webhook URL" />
      </div>
      <div class="flex justify-center">
        <OutlinedButton v-if="!state.error && !state.loading" class="mr-20" @click="deleteUrl">Delete</OutlinedButton>
        <OutlinedButton @click="close">Cancel</OutlinedButton>
        <CButton v-if="!state.error && !state.loading" @click="setUrl">Save</CButton>
      </div>
    </div>
  </CDialog>
</template>

<script lang="ts">
import { defineComponent, ref, toRefs } from 'vue';
import OutlinedButton from './core/OutlinedButton.vue';
import CDialog from './core/CDialog.vue';
import { asyncDialog, tryAsync } from '../async-state';
import TextField from './core/TextField.vue';
import CButton from './core/CButton.vue';
import CLoader from './core/CLoader.vue';
import TickIcon from './icons/TickIcon.vue';
import { useApi } from '../api/plugin';
import DiscordIcon from './icons/DiscordIcon.vue';

export default defineComponent({
  name: 'DiscordSettings',
  components: { DiscordIcon, TickIcon, CLoader, CButton, TextField, CDialog, OutlinedButton },
  props: {
    broadcasterId: {
      type: String,
      required: true,
    },
  },
  setup(props) {
    const { broadcasterId } = toRefs(props);
    const api = useApi();

    const { state, reset } = asyncDialog(ref(false));

    const url = ref('');

    const open = () => {
      reset();
      state.value = true;
      tryAsync(async () => {
        state.success = false;
        url.value = await api.getDiscordUrl(broadcasterId.value);
      }, state);
    };
    const close = () => {
      state.value = false;
    };
    const setUrl = () => {
      tryAsync(async () => {
        await api.setDiscordUrl(broadcasterId.value, url.value);
        state.value = false;
        state.success = true;
      }, state);
    };
    const deleteUrl = () => {
      tryAsync(async () => {
        await api.deleteDiscordUrl(broadcasterId.value);
        state.value = false;
        state.success = true;
      }, state);
    };

    return { state, reset, url, open, close, setUrl, deleteUrl };
  },
});
</script>
