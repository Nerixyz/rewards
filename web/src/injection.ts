import { TwitchUser } from './api/types';
import { inject, provide } from 'vue';

const userKey = 're:user';
export function provideUser(user: TwitchUser) {
  provide(userKey, user);
}

export function useUser() {
  return inject(userKey) as TwitchUser;
}
