import { ref, Plugin, inject } from 'vue';
import { Reward, TwitchUser } from './api/types';

export class DataStore {
  user = ref<null | TwitchUser>(null);
  rewards = ref<null | { id: string; items: Reward[] }>(null);
}

const injectKey = 're:data-store';
export const DataStorePlugin: Plugin = {
  install(app) {
    app.provide(injectKey, new DataStore());
  },
};

export function useDataStore(): DataStore {
  return inject(injectKey) as DataStore;
}
