import { InputReward, Reward, RewardData } from './types';
import { parseDuration } from '../utilities';
import { defaultNewReward, StaticRewardData } from './rewards-data';

export interface VRewardModel {
  title: string;
  prompt: string;
  cost: string;
  usesPerStream: string;
  usesPerUser: string;
  cooldown: string;
  color: string;

  action: RewardData;
}

export function assignToVRewardModel(reward: Reward, model: VRewardModel): void {
  model.cost = reward.twitch.cost.toString();
  model.cooldown = reward.twitch.global_cooldown_setting?.global_cooldown_seconds.toString() ?? '';
  model.title = reward.twitch.title;
  model.usesPerStream = reward.twitch.max_per_stream_setting?.max_per_stream.toString() ?? '';
  model.usesPerUser = reward.twitch.max_per_user_per_stream_setting?.max_per_user_per_stream.toString() ?? '';
  model.prompt = reward.twitch.prompt;
  model.color = reward.twitch.background_color;
  model.action = {
    type: reward.data.type,
    // use spread on object -- no objects currently - may change
    data: reward.data.data,
  };
}

export function toInputReward(vmodel: VRewardModel): InputReward {
  const cooldown = parseDuration(vmodel.cooldown) || 0;
  return {
    twitch: {
      title: vmodel.title,
      cost: Number(vmodel.cost),
      prompt: vmodel.prompt,

      global_cooldown_seconds: cooldown,
      is_global_cooldown_enabled: !!cooldown,

      max_per_user_per_stream: Number(vmodel.usesPerUser),
      is_max_per_user_per_stream_enabled: !!Number(vmodel.usesPerUser),

      max_per_stream: Number(vmodel.usesPerStream),
      is_max_per_stream_enabled: !!Number(vmodel.usesPerStream),

      is_user_input_required: StaticRewardData[vmodel.action.type].inputRequired,

      should_redemptions_skip_request_queue: false,
      background_color: vmodel.color || undefined,
    },
    data: vmodel.action,
  };
}

export function assignDefaultToModel(model: VRewardModel): void {
  for (const [key, value] of Object.entries(defaultNewReward())) {
    model[key as keyof VRewardModel] = value;
  }
}
