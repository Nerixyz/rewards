import { VRewardModel } from './model-conversion';

export const RewardTypes: Array<{value: string, display: string}> = [{value: 'Timeout', display: 'Timeout for n seconds'}];

export function defaultNewReward(): VRewardModel {
  return {
    title: '',
    cost: '',
    usesPerStream: '',
    usesPerUser: '',
    cooldown: '',
    color: '',
    inputRequired: false,
    prompt: '',
    action: {
      type: 'Timeout',
      data: '1s'
    }
  };
}
