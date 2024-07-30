import { reactive, Ref, UnwrapRef } from 'vue';

export interface AsyncState<T> {
  loading: boolean;
  error: null | Error;
  value: UnwrapRef<T>;
}

export function asyncState<T>(initialValue: T, initialLoading = true): { state: AsyncState<T>; reset: () => void } {
  const state = reactive({
    loading: initialLoading,
    error: null,
    value: initialValue,
  });

  return {
    state,
    reset: () => {
      state.loading = false;
      state.error = null;
    },
  };
}

export interface AsyncDialog extends AsyncState<boolean> {
  success: boolean;
}

export function asyncDialog(openRef: Ref<boolean>): { state: AsyncDialog; reset: () => void } {
  const state = reactive({
    loading: false,
    error: null,
    value: openRef,
    success: false,
  });

  return {
    state,
    reset: () => {
      state.loading = false;
      state.error = null;
      state.success = false;
    },
  };
}

export async function tryAsync<T>(fn: (state: AsyncState<T>) => Promise<unknown>, state: AsyncState<T>): Promise<void> {
  try {
    state.loading = true;
    state.error = null;
    await fn(state);
  } catch (e) {
    state.error = e instanceof Error ? e : new Error('unknown error');
  } finally {
    state.loading = false;
  }
}

export async function tryAsyncDialog(fn: () => Promise<unknown>, dialog: AsyncDialog): Promise<void> {
  try {
    dialog.loading = true;
    dialog.error = null;
    await fn();
  } catch (e) {
    dialog.error = e instanceof Error ? e : new Error('unknown error');
  } finally {
    dialog.loading = false;
    dialog.success = true;
  }
}
