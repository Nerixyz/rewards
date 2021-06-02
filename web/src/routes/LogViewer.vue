<template>
  <div class="px-20 pt-5 xl:max-w-7xl mx-auto">
    <div class="flex flex-col gap-5">
      <div class="w-full pb-5 border-b border-opacity-30 border-gray-900">
        <router-link :to="`/rewards/${broadcasterId}`">
          <OutlinedButton :disabled="!broadcasterId"><BackIcon /> Back </OutlinedButton>
        </router-link>
        <OutlinedButton :disabled="logs.loading" @click="reload"><ReloadIcon /> Reload </OutlinedButton>
      </div>
      <!-- Loading handler -->
      <div v-if="logs.loading"><CLoader /></div>

      <!-- Error handler -->
      <div v-else-if="logs.error">
        Something went wrong.
        <br />
        <span class="break-words font-mono">{{ logs.error }}</span>
      </div>
      <div v-else-if="!logs.value.length" class="w-full flex flex-col gap-5">No logs :/</div>
      <div v-else class="w-full grid grid-cols-logs gap-x-3 gap-y-2">
        <template v-for="log in logs.value" :key="log.date">
          <div class="text-gray-700">
            {{ log.date }}
          </div>
          <div>
            {{ log.content }}
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, watch } from 'vue';
import OutlinedButton from '../components/core/OutlinedButton.vue';
import { useRoute } from 'vue-router';
import { useDataStore } from '../store';
import { useApi } from '../api/plugin';
import { asyncState, tryAsync } from '../async-state';
import { LogEntry } from '../api/types';
import CLoader from '../components/core/CLoader.vue';
import BackIcon from '../components/icons/BackIcon.vue';
import ReloadIcon from '../components/icons/ReloadIcon.vue';

export default defineComponent({
  name: 'LogViewer',
  components: { ReloadIcon, BackIcon, CLoader, OutlinedButton },
  setup() {
    const route = useRoute();
    const api = useApi();
    const dataStore = useDataStore();

    // core stuff to ensure we have a user id

    const getCurrentId = () => String(route.params.id || dataStore.user.value?.id || '');
    const broadcasterId = ref(getCurrentId());
    watch(
      () => route.params.id,
      () => (broadcasterId.value = getCurrentId()),
    );
    if (!route.params.id) {
      const stop = watch(dataStore.user, () => {
        broadcasterId.value = getCurrentId();
        stop();
      });
    }

    const { state: logs } = asyncState<LogEntry[]>([]);
    const getLogs = (id: string) =>
      tryAsync(async state => {
        state.value = await api.getLogs(id);
      }, logs);

    watch(broadcasterId, id => getLogs(id));

    if (broadcasterId.value) getLogs(broadcasterId.value);

    const reload = () => {
      console.log('reload');
      getLogs(broadcasterId.value);
    };

    return { logs, reload, broadcasterId };
  },
});
</script>
