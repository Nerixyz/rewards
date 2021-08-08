import { RouteLocationNormalized, useRoute } from 'vue-router';
import { DataStore, useDataStore } from '../store';
import { computed, ComputedRef } from 'vue';

export function useBroadcaster(pre: { route?: RouteLocationNormalized; store?: DataStore } = {}): {
  broadcasterId: ComputedRef<string | undefined>;
  thisUserId: ComputedRef<string | undefined>;
} {
  const route = pre.route ?? useRoute();
  const store = pre.store ?? useDataStore();

  const broadcasterId = computed(() => {
    const routeId = route.params.id as string | undefined;
    const storeId = store.user.value?.id;
    return routeId || storeId || undefined;
  });
  const thisUserId = computed(() => store.user.value?.id);

  return { broadcasterId, thisUserId };
}
