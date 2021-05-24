import { inject, Plugin } from 'vue';
import ApiClient  from './ApiClient';

const injectionKey = 're:api';

const ApiPlugin: Plugin = {
  install(app) {
    app.provide(injectionKey, ApiClient);
  }
};

export default ApiPlugin;

export function useApi() {
  return inject(injectionKey) as typeof ApiClient;
}
