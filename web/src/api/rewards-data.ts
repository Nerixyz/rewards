import { VRewardModel } from './model-conversion';
import { RewardDataMap } from './types';

export const StaticRewardData: Record<keyof RewardDataMap, { display: string, inputRequired: boolean }> = {
  Timeout: {
    display: 'Timeout',
    inputRequired: true,
  },
  SubOnly: {
    display: 'Subonly',
    inputRequired: false,
  },
  EmoteOnly: {
    display: 'Emoteonly',
    inputRequired: false,
  },
};

export const RewardTypes = Object.entries(StaticRewardData).map(([key, {display}]) => ({value: key, display }));

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
      data: '1s'
    }
  };
}
