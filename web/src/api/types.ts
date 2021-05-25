export interface TwitchUser {
  id: string;
  login: string;
  profile_image_url: string;
}

export interface TwitchInputReward {
  /** 	The title of the reward. */
  title: string;
  /** 	The cost of the reward. */
  cost: number;
  /** 	The prompt for the viewer when redeeming the reward. */
  prompt?: string;
  /** 	Is the reward currently enabled, if false the reward won’t show up to viewers. Default: true */
  is_enabled?: boolean;
  /** 	Custom background color for the reward. Format: Hex with # prefix. Example: #00E5CB. */
  background_color?: string;
  /** 	Does the user need to enter information when redeeming the reward. Default: false. */
  is_user_input_required?: boolean;
  /** 	Whether a maximum per stream is enabled. Default: false. */
  is_max_per_stream_enabled?: boolean;
  /** 	The maximum number per stream if enabled. Required when any value of is_max_per_stream_enabled is included. */
  max_per_stream?: number;
  /** 	Whether a maximum per user per stream is enabled. Default: false. */
  is_max_per_user_per_stream_enabled?: boolean;
  /** 	The maximum number per user per stream if enabled. Required when any value of is_max_per_user_per_stream_enabled is included. */
  max_per_user_per_stream?: number;
  /** 	Whether a cooldown is enabled. Default: false. */
  is_global_cooldown_enabled?: boolean;
  /** 	The cooldown in seconds if enabled. Required when any value of is_global_cooldown_enabled is included. */
  global_cooldown_seconds?: number;
  /** 	Should redemptions be set to FULFILLED status immediately when redeemed and skip the request queue instead of the normal UNFULFILLED status. Default: false. */
  should_redemptions_skip_request_queue?: boolean;
}

export interface TwitchReward {
  /** 	ID of the channel the reward is for */
  broadcaster_id: string;
  /** 	Login of the channel the reward is for */
  broadcaster_login: string;
  /** 	Display name of the channel the reward is for */
  broadcaster_name: string;
  /** 	ID of the reward */
  id: string;
  /** 	The title of the reward */
  title: string;
  /** 	The prompt for the viewer when they are redeeming the reward */
  prompt: string;
  /** 	The cost of the reward */
  cost: number;
  /** 	Set of custom images of 1x, 2x and 4x sizes for the reward, can be null if no images have been uploaded */
  image?: TwitchRewardImage;
  /** 	Set of default images of 1x, 2x and 4x sizes for the reward */
  default_image?: TwitchRewardImage;
  /** 	Custom background color for the reward. Format: Hex with # prefix. Example: #00E5CB. */
  background_color: string;
  /** 	Is the reward currently enabled, if false the reward won’t show up to viewers */
  is_enabled: boolean;
  /** 	Does the user need to enter information when redeeming the reward */
  is_user_input_required: boolean;
  /** 	Whether a maximum per stream is enabled and what the maximum is. */
  max_per_stream_setting: { is_enabled: boolean; max_per_stream: number };
  /** 	Whether a maximum per user per stream is enabled and what the maximum is. */
  max_per_user_per_stream_setting: { is_enabled: boolean; max_per_user_per_stream: number };
  /** 	Whether a cooldown is enabled and what the cooldown is. */
  global_cooldown_setting: { is_enabled: boolean; global_cooldown_seconds: number };
  /** 	Is the reward currently paused, if true viewers can’t redeem */
  is_paused: boolean;
  /** 	Is the reward currently in stock, if false viewers can’t redeem */
  is_in_stock: boolean;
  /** 	Should redemptions be set to FULFILLED status immediately when redeemed and skip the request queue instead of the normal UNFULFILLED status. */
  should_redemptions_skip_request_queue: boolean;
  /** 	The number of redemptions redeemed during the current live stream. Counts against the max_per_stream_setting limit. Null if the broadcasters stream isn’t live or max_per_stream_setting isn’t enabled. */
  redemptions_redeemed_current_stream: number | null;
  /** 	Timestamp of the cooldown expiration. Null if the reward isn’t on cooldown. */
  cooldown_expires_at: string | null;
}

interface TwitchRewardImage {
  url_1x: string;
  url_2x: string;
  url_4x: string;
}

export type RewardData<K extends keyof RewardDataMap = keyof RewardDataMap> = { type: K; data: RewardDataMap[K] };

export interface RewardDataMap {
  Timeout: string;
  SubOnly: string;
  EmoteOnly: string;
}

export interface InternalCustomReward {
  id: string;
  user_id: string;
  data: RewardData;
}

export interface Reward {
  twitch: TwitchReward;
  data: RewardData;
}

export interface InputReward {
  twitch: TwitchInputReward;
  data: RewardData;
}
