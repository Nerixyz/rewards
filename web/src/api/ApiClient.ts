import {
  Connections,
  InputReward,
  InternalCustomReward,
  LogEntry,
  Reward,
  SpotifySettings,
  TwitchReward,
  TwitchUser,
} from './types';
import { BaseClient } from './BaseClient';

class HttpClient extends BaseClient {
  async deleteAccount(): Promise<void> {
    await this.delete('auth').catch(console.error);
    this.logout();
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

  getConnections() {
    return this.get<Connections>('connections');
  }

  updateSpotifySettings(settings: SpotifySettings) {
    return this.patch(settings, 'connections', 'spotify');
  }

  removeConnection(name: 'spotify') {
    return this.delete('connections', name);
  }

  getSpotifyUrl() {
    return this.get<string>('connections', 'spotify-auth-url');
  }

  async getRewards(id: string): Promise<Reward[]> {
    const response = await this.get<{ twitch: TwitchReward[]; data: InternalCustomReward[] }>('rewards', id);

    const map = new Map<string, Partial<Reward>>(response.twitch.map(r => [r.id, { twitch: r }]));
    for (const internal of response.data) {
      const el = map.get(internal.id);
      if (el) {
        el.data = internal.data;
        el.live_delay = internal.live_delay || '';
        el.auto_accept = internal.auto_accept;
      }
    }

    return [...map.values()] as Reward[];
  }

  async getLogs(id: string): Promise<LogEntry[]> {
    const response = await this.get<{ date: string; content: string }[]>('logs', id);
    const fmt = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'medium' });
    return response
      .map(({ date, content }) => ({ date: new Date(date), content }))
      .sort(({ date: a }, { date: b }) => Number(b) - Number(a))
      .map(({ date, content }) => {
        try {
          return { date: fmt.format(date), content };
        } catch (e) {
          return { date: '?', content };
        }
      });
  }

  addReward(broadcasterId: string, reward: InputReward) {
    return this.put<Reward>(reward, 'rewards', broadcasterId);
  }

  updateReward(broadcasterId: string, reward: InputReward, id: string) {
    return this.patch<Reward>(reward, 'rewards', broadcasterId, id);
  }

  deleteReward(broadcasterId: string, reward: Reward) {
    return this.delete('rewards', broadcasterId, reward.twitch.id);
  }

  setDiscordUrl(broadcasterId: string, url: string) {
    return this.patch({ url }, 'logs', broadcasterId, 'discord');
  }

  getDiscordUrl(broadcasterId: string): Promise<string> {
    return this.get<{ url: string } | null>('logs', broadcasterId, 'discord').then(x => x?.url || '');
  }

  deleteDiscordUrl(broadcasterId: string) {
    return this.delete('logs', broadcasterId, 'discord');
  }
}

const ApiClient = new HttpClient();

export default ApiClient;
