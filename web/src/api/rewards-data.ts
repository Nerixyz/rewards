import { VRewardModel } from './model-conversion';
import { SlotRewardData, RewardDataMap, SpotifyPlayOptions, SwapRewardData } from './types';
interface StaticData<K extends keyof RewardDataMap> {
  display: string;
  inputRequired: boolean;
  validOptions: (opts: unknown) => boolean;
  defaultOptions: RewardDataMap[K];
}
export const StaticRewardData: { [K in keyof RewardDataMap]: StaticData<K> } = {
  Timeout: {
    display: 'Timeout',
    inputRequired: true,
    validOptions: TSEValid,
    defaultOptions: '1s',
  },
  SubOnly: {
    display: 'Subonly',
    inputRequired: false,
    validOptions: TSEValid,
    defaultOptions: '1s',
  },
  EmoteOnly: {
    display: 'Emoteonly',
    inputRequired: false,
    validOptions: TSEValid,
    defaultOptions: '1s',
  },
  BttvSwap: {
    display: 'Add/Swap Bttv Emote',
    inputRequired: true,
    validOptions: emoteSwapValid,
    defaultOptions: { limit: null },
  },
  FfzSwap: {
    display: 'Add/Swap Ffz Emote',
    inputRequired: true,
    validOptions: emoteSwapValid,
    defaultOptions: { limit: null },
  },
  SevenTvSwap: {
    display: 'Add/Swap 7TV Emote',
    inputRequired: true,
    validOptions: emoteSwapValid,
    defaultOptions: { limit: null },
  },
  BttvSlot: {
    display: 'Bttv Slots',
    inputRequired: true,
    validOptions: emoteSlotValid,
    defaultOptions: {
      slots: 2,
      expiration: '2d',
    },
  },
  FfzSlot: {
    display: 'Ffz Slots',
    inputRequired: true,
    validOptions: emoteSlotValid,
    defaultOptions: {
      slots: 2,
      expiration: '2d',
    },
  },
  SevenTvSlot: {
    display: '7TV Slots',
    inputRequired: true,
    validOptions: emoteSlotValid,
    defaultOptions: {
      slots: 2,
      expiration: '2d',
    },
  },
  SpotifySkip: {
    display: 'Skip Spotify Track',
    inputRequired: false,
    validOptions: opts => opts === null,
    defaultOptions: null,
  },
  SpotifyPlay: {
    display: 'Play Spotify Track',
    inputRequired: true,
    validOptions: spotifyPlayValid,
    defaultOptions: {
      allow_explicit: false,
    },
  },
  SpotifyQueue: {
    display: 'Queue Spotify Track',
    inputRequired: true,
    validOptions: spotifyPlayValid,
    defaultOptions: {
      allow_explicit: false,
    },
  },
};

function TSEValid(opts: unknown): boolean {
  return typeof opts === 'string';
}

function emoteSlotValid(opts: unknown): boolean {
  if (typeof opts !== 'object' || opts === null) return false;
  return typeof (opts as SlotRewardData).slots === 'number' && typeof (opts as SlotRewardData).expiration === 'string';
}

function emoteSwapValid(opts: unknown): boolean {
  if (typeof opts !== 'object') return false;
  return opts === null || typeof (opts as SwapRewardData).limit === 'number' || (opts as SwapRewardData).limit === null;
}

function spotifyPlayValid(opts: unknown): boolean {
  if (typeof opts !== 'object' || opts === null) return false;
  return typeof (opts as SpotifyPlayOptions).allow_explicit === 'boolean';
}

export const RewardTypes = Object.entries(StaticRewardData).map(([key, { display }]) => ({ value: key, display }));

export function defaultNewReward(): VRewardModel {
  return {
    title: '',
    cost: '',
    usesPerStream: '',
    usesPerUser: '',
    cooldown: '0',
    color: '',
    prompt: '',
    action: {
      type: 'Timeout',
      data: StaticRewardData.Timeout.defaultOptions,
    },
    liveDelay: '',
    imageUrl: null,
  };
}
