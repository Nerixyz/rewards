import { VRewardModel } from './model-conversion';
import { SlotRewardData, RewardDataMap, SpotifyPlayOptions, SwapRewardData, TimeoutRewardData } from './types';

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
    validOptions: timeoutValid,
    defaultOptions: { duration: '1s', vip: false },
  },
  SubOnly: {
    display: 'Subonly',
    inputRequired: false,
    validOptions: SEValid,
    defaultOptions: '1s',
  },
  EmoteOnly: {
    display: 'Emoteonly',
    inputRequired: false,
    validOptions: SEValid,
    defaultOptions: '1s',
  },
  BttvSwap: {
    display: 'Add/Swap Bttv Emote',
    inputRequired: true,
    validOptions: emoteSwapValid,
    defaultOptions: { limit: null, allow_unlisted: true, reply: true },
  },
  FfzSwap: {
    display: 'Add/Swap Ffz Emote',
    inputRequired: true,
    validOptions: emoteSwapValid,
    defaultOptions: { limit: null, allow_unlisted: true, reply: true },
  },
  SevenTvSwap: {
    display: 'Add/Swap 7TV Emote',
    inputRequired: true,
    validOptions: emoteSwapValid,
    defaultOptions: { limit: null, allow_unlisted: true, reply: true },
  },
  BttvSlot: {
    display: 'Bttv Slots',
    inputRequired: true,
    validOptions: emoteSlotValid,
    defaultOptions: {
      slots: 2,
      expiration: '2d',
      allow_unlisted: true,
      reply: true,
    },
  },
  FfzSlot: {
    display: 'Ffz Slots',
    inputRequired: true,
    validOptions: emoteSlotValid,
    defaultOptions: {
      slots: 2,
      expiration: '2d',
      allow_unlisted: true,
      reply: true,
    },
  },
  SevenTvSlot: {
    display: '7TV Slots',
    inputRequired: true,
    validOptions: emoteSlotValid,
    defaultOptions: {
      slots: 2,
      expiration: '2d',
      allow_unlisted: true,
      reply: true,
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

function timeoutValid(opts: unknown): boolean {
  return (
    typeof opts === 'object' &&
    typeof (opts as TimeoutRewardData).duration === 'string' &&
    typeof (opts as TimeoutRewardData).vip === 'boolean'
  );
}

function SEValid(opts: unknown): boolean {
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
    cooldown: '0m',
    color: '',
    prompt: '',
    action: {
      type: 'Timeout',
      data: StaticRewardData.Timeout.defaultOptions,
    },
    liveDelay: '',
    imageUrl: null,
    autoAccept: true,
  };
}
