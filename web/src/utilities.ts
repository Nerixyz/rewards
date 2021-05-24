import { ref, Ref, } from 'vue';

export async function tryAsync(fn: () => Promise<any>, loading: Ref<boolean>, error: Ref<string | null>) {
  try {
    loading.value = true;
    error.value = null;
    await fn();
  } catch (e) {
    error.value = e.toString();
  } finally {
    loading.value = false;
  }
}

export function asyncRefs(initialLoading = true) {
  return {loading: ref(initialLoading), error: ref<null | string>(null)};
}

export function parseDuration(d: string): number | null {
  if(!d.length) {
    return null;
  }
  if(/^\d+$/.test(d)) {
    return Number(d);
  }

  const [fullMatch,num, unit] = d.match(/^(\d+\.?\d*)([smhd])$/) ?? [];
  if(!fullMatch) return null;

  const base = Number(num);

  return base * DURATIONS[unit as keyof typeof DURATIONS];
}

export function isValidDuration(s: unknown) {
  if(typeof s === 'number') return true;
  if(typeof s !== 'string') return false;

  const dur = parseDuration(s);
  return dur !== null;
}

const DURATIONS = {s: 1, m: 60, h: 60 * 60, d: 24 * 60 * 60};
