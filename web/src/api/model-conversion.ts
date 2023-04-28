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
  imageUrl: string | null;

  liveDelay: string;
  autoAccept: boolean;

  action: RewardData;
}

function getImageUrl(reward: Reward): string | null {
  return reward.twitch.image?.url_4x ?? reward.twitch.default_image?.url_4x ?? null;
}

export function toVRewardModel(reward: Reward): VRewardModel {
  return {
    cost: reward.twitch.cost.toString(),
    cooldown: reward.twitch.global_cooldown_setting?.global_cooldown_seconds.toString() ?? '',
    title: reward.twitch.title,
    usesPerStream: reward.twitch.max_per_stream_setting?.max_per_stream.toString() ?? '',
    usesPerUser: reward.twitch.max_per_user_per_stream_setting?.max_per_user_per_stream.toString() ?? '',
    prompt: reward.twitch.prompt,
    color: reward.twitch.background_color,
    imageUrl: getImageUrl(reward),

    action: {
      type: reward.data.type,
      data:
        typeof reward.data.data === 'object' && reward.data.data !== null
          ? {
              ...reward.data.data,
            }
          : reward.data.data,
    },

    liveDelay: reward.live_delay ?? '',
    autoAccept: reward.auto_accept,
  };
}

export function assignToVRewardModel(reward: Reward, model: VRewardModel): void {
  model.cost = reward.twitch.cost.toString();
  model.cooldown = reward.twitch.global_cooldown_setting?.global_cooldown_seconds.toString() ?? '';
  model.title = reward.twitch.title;
  model.usesPerStream = reward.twitch.max_per_stream_setting?.max_per_stream.toString() ?? '';
  model.usesPerUser = reward.twitch.max_per_user_per_stream_setting?.max_per_user_per_stream.toString() ?? '';
  model.prompt = reward.twitch.prompt;
  model.color = reward.twitch.background_color;
  model.imageUrl = getImageUrl(reward);

  model.action = {
    type: reward.data.type,
    data:
      typeof reward.data.data === 'object' && reward.data.data !== null
        ? {
            ...reward.data.data,
          }
        : reward.data.data,
  };

  model.liveDelay = reward.live_delay ?? '';
  model.autoAccept = reward.auto_accept;
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
    data: {
      type: vmodel.action.type,
      data:
        typeof vmodel.action.data === 'object' && vmodel.action.data !== null
          ? {
              ...vmodel.action.data,
            }
          : vmodel.action.data,
    },
    live_delay: vmodel.liveDelay.trim() || undefined,
    auto_accept: vmodel.autoAccept,
  };
}

export function assignDefaultToModel(model: VRewardModel): void {
  copyModel(defaultNewReward(), model);
}

export function copyModel(from: VRewardModel, to: VRewardModel): void {
  for (const [key, value] of Object.entries(from)) {
    (to as any)[key] = value; // TODO: this is ugly
  }
}

export function simpleClone<T>(value: T): T {
  if (typeof value === 'object' && value !== null) {
    return { ...value };
  }
  return value;
}
