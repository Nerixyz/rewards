import { ref, Plugin, inject } from 'vue';
import { TwitchUser } from './api/types';

class DataStore {
  user = ref<null | TwitchUser>(null);
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
