import { DataStore } from '../store';
import { ComputedRef, watch } from 'vue';
import { Reward } from '../api/types';
import { AsyncState, asyncState, tryAsync } from '../async-state';
import ApiClient from '../api/ApiClient';

export function useRewards({
  store,
  broadcasterId,
  api,
}: {
  store: DataStore;
  broadcasterId: ComputedRef<string | undefined>;
  api: typeof ApiClient;
}) {
  const { state } = asyncState<Reward[]>([]);

  watch(
    () => broadcasterId.value,
    id => {
      if (!id) {
        // this should only be at the start
        state.loading = true;
        return;
      }

      if (store.rewards.value?.id === id) {
        // use the "cached" rewards
        state.value = store.rewards.value.items;
        state.loading = false;
        state.error = null;
      } else {
        tryAsync(async rewards => {
          const items = await api.getRewards(id);
          rewards.value = items;
          store.rewards.value = { id, items };
        }, state).catch(console.error);
      }
    },
    { immediate: true },
  );

  return {
    rewards: state as Readonly<AsyncState<Reward[]>>,
    updateRewards: (items: Reward[]) => {
      state.value = items;
      store.rewards.value = { id: broadcasterId.value ?? '?', items };
    },
  };
}
