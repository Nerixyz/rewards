# Rewards

This is an application that manages custom Twitch rewards.

The priorities for the project are _P1_ .. _Px_ where _P1_ is the most important.

## TODO

This may be managed in the issues.

### P1

* [ ] WebInterface
* [ ] EventSub
* [ ] IRC
* [ ] Verify index.html
* [ ] Use migrations  
* [ ] Document setup

<details>
<summary>Example config ('user_token')</summary>

```json5
{
    "type": "UserToken",
    "data": {
        "access_token": "MY_ACCESS_TOKEN",
        "refresh_token": "MY_REFRESH_TOKEN",
      // this is just new Date(timestamp).toISOString() okayge
        "created_at": "2021-05-24T15:04:47.000Z",
        "expires_at": "2021-05-24T19:25:51.000Z"
    }
  // scopes: Scope::ChatEdit,
  //         Scope::ChatRead,
  //         Scope::ChannelModerate,
  //         Scope::ModeratorManageAutoMod,
  //         Scope::ModerationRead,
  //         Scope::UserManageBlockedUsers,
  //         Scope::UserReadBlockedUsers,
  //         Scope::UserEditFollows,
  //         Scope::UserReadFollows,
  //         Scope::ChannelReadRedemptions,
  //         Scope::ChannelManageRedemptions,
  //         Scope::WhispersEdit,
  //         Scope::WhispersRead,
  //         Scope::ChannelEditCommercial,
  //         Scope::ChannelManageBroadcast,
}
```
</details>

* [ ] Check TODOs throughout the code

### P2
* [ ] IRC Bot
* [ ] Automatic auth flow (use `postMessage` on auth)
* [ ] Check token-scopes
* [ ] Pass the auth token via a hash (like twitch does it).
* [ ] Reduce cloning, consider editing the twitch library


## Possible Rewards

### P1
* [ ] Timeout user (for x-seconds)
* [ ] Enable SubMode (for x-seconds)
* [ ] Enable EmoteOnly (for x-seconds)
  
### P2
* [ ] Change FFZ Emote (requires the bot to be an **editor**)
* [ ] Change BTTV Emote (requires the bot to be an **editor**)
  
### P3
* [ ] Queue Spotify song

# Internal - web API

All endpoints are on `/api/v1`. All endpoints (except `/auth/twich-auth-url` and the auth-callbacks) require authentication.

Authentication is done through the `Authorization` header 
that has to be set to `Bearer { cookie(auth_token) }`.

## Auth

### GET `auth/twitch-auth-url`

Returns the _full_ url for the user to be redirected to.

### GET `auth/twitch-callback`

Callback for the twitch-auth. This sets the `auth_token` cookie for the user. 
This cookie isn't checked by the api, only the header.

### DELETE `auth`

Deletes the user data and revokes the token.

## Users

### GET `users/me`

Gets user info. It's just the twitch user info.

## Rewards

### PUT `rewards/{broadcaster_id}`

Creates a new reward.

**Body**
`{ twitch: TwitchRewardData, data: RewardData }`

**Returns**
`{ id: string }`

### PATCH `rewards/{broadcaster_id}/{reward_id}`

Updates a reward.

**Body**
`{ twitch: TwitchRewardData, data: RewardData }`

**Returns**
Nothing

### GET `rewards/{broadcaster_id}`

Gets all rewards managed by this application.

**Returns**
`{ twitch: TwitchRewardData[], data: RewardData[] }`

### GET `rewards/{broadcaster_id}/{reward_id}`

Gets the reward.

**Returns**
`{ twitch: TwitchRewardData, data: RewardData }`

### DELETE `rewards/{broadcaster_id}/{reward_id}`

Removes the reward.

## Editors

### GET `editors`

Returns the editors of the current user.

**Returns**
`Array<{ editor_id: string, editor_name: string }>`

### GET `editors/broadcasters`

Returns the broadcasters this user manages.

**Returns**
`Array<{ broadcaster_id: string, broadcaster_name: string }>`

### PUT `editors/{name}`

Adds an editor

### DELETE `editors/{name}`

Removes an editor

