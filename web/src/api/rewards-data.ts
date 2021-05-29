import { VRewardModel } from './model-conversion';
import { RewardDataMap } from './types';
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
    validOptions: opts => opts === null,
    defaultOptions: null,
  },
  FfzSwap: {
    display: 'Add/Swap Ffz Emote',
    inputRequired: true,
    validOptions: opts => opts === null,
    defaultOptions: null,
  },
};

function TSEValid(opts: unknown): boolean {
  return typeof opts === 'string';
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
  };
}
