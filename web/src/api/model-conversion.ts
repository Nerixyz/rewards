import { InputReward, Reward, RewardData } from './types';
import { parseDuration } from '../utilities';
import { defaultNewReward } from './rewards-data';

export interface VRewardModel {
  title: string;
  prompt: string;
  cost: string;
  usesPerStream: string;
  usesPerUser: string;
  cooldown: string;
  inputRequired: boolean;
  color: string;

  action: RewardData
}


export function assignToVRewardModel(reward: Reward, model: VRewardModel) {
    model.cost = reward.twitch.cost.toString();
    model.cooldown = reward.twitch.global_cooldown_setting?.global_cooldown_seconds.toString() ?? '';
    model.title = reward.twitch.title;
    model.usesPerStream = reward.twitch.max_per_stream_setting?.max_per_stream.toString() ?? '';
    model.usesPerUser = reward.twitch.max_per_user_per_stream_setting?.max_per_user_per_stream.toString() ?? '';
    model.prompt = reward.twitch.prompt;
    model.inputRequired = reward.twitch.is_user_input_required;
    model.color = reward.twitch.background_color;
    model.action = reward.data;
}

export function toInputReward(vmodel: VRewardModel): InputReward {
  return {
    twitch: {
      title: vmodel.title,
      cost: Number(vmodel.cost),
      global_cooldown_seconds: parseDuration(vmodel.cooldown) ?? undefined,
      is_global_cooldown_enabled: !!vmodel.cooldown,
      max_per_user_per_stream: Number(vmodel.usesPerUser) || undefined,
      is_max_per_user_per_stream_enabled: !!vmodel.usesPerUser,
      max_per_stream: Number(vmodel.usesPerStream) || undefined,
      is_max_per_stream_enabled: !!vmodel.usesPerStream,
      prompt: vmodel.prompt,
      is_user_input_required: vmodel.inputRequired,
      should_redemptions_skip_request_queue: false,
      background_color: vmodel.color || undefined,
    },
    data: vmodel.action,
  }
}

export function assignDefaultToModel(model: VRewardModel) {
  for(const [key, value] of Object.entries(defaultNewReward())) {
    (model as any)[key] = value;
  }
}
