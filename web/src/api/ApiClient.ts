import { InternalCustomReward, Reward, TwitchReward, TwitchUser } from './types';
import { BaseClient } from './BaseClient';

class HttpClient extends BaseClient {

  getTwitchAuthUrl() {
    return this.get<{url: string}>('auth', 'twitch-auth-url');
  }

  getCurrentUser() {
    return this.get<TwitchUser>('users', 'me');
  }

  getUserInfo(login: string) {
    return this.get<TwitchUser>('users', login);
  }

  getEditors() {
    return this.get<TwitchUser[]>('editors');
  }

  addEditor(name: string) {
    return this.put(undefined, 'editors', name);
  }

  removeEditor(name: string) {
    return this.delete('editors', name);
  }

  getBroadcasters() {
    return this.get<TwitchUser[]>('editors', 'broadcasters');
  }

  async getRewards(id: string): Promise<Reward[]> {
    const response = await this.get<{twitch: TwitchReward[], data: InternalCustomReward[]}>('rewards', id);

    const map = new Map<string, Partial<Reward>>(response.twitch.map(r => [r.id, {twitch: r}]));
    for(const internal of response.data) {
      const el = map.get(internal.id);
      if(el) (el as any).data = internal.data;
    }

    return [...map.values()] as Reward[];
  }

  addReward(broadcasterId: string, reward: Reward) {
    return this.put<Reward>(reward, 'rewards', broadcasterId);
  }

  updateReward(broadcasterId: string, reward: Reward, id: string) {
    return this.patch<Reward>(reward, 'rewards', broadcasterId, id);
  }

  deleteReward(broadcasterId: string, reward: Reward){
    return this.delete('rewards', broadcasterId, reward.twitch.id);
  }
}

const ApiClient = new HttpClient();

export default ApiClient;
