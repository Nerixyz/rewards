# Rewards

This is an application that manages custom Twitch rewards.

The priorities for the project are _P1_ .. _Px_ where _P1_ is the most important.

## TODO

This may be managed in the issues.

### P1

* [x] WebInterface
* [x] EventSub
* [x] IRC
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
* [ ] Get rid of cloning everywhere

### P2
* [ ] IRC Bot
* [ ] Automatic auth flow (use `postMessage` on auth)
* [ ] Check token-scopes
* [ ] Pass the auth token via a hash (like twitch does it).
* [ ] Reduce cloning, consider editing the twitch library
* [ ] Get rid of uses of `actix_web::Error` where it's not useful

### P3

* [ ] Use Vuetify once it's out

## Possible Rewards

### P1
* [x] Timeout user (for x-seconds)
* [x] Enable SubMode (for x-seconds)
* [x] Enable EmoteOnly (for x-seconds)
  
### P2
* [ ] Change FFZ Emote (requires the bot to be an **editor**)
* [ ] Change BTTV Emote (requires the bot to be an **editor**)
  
### P3
* [ ] Queue Spotify song

# Internal - web API

All endpoints are on `/api/v1`. All endpoints (except `/auth/twich-auth-url` and the auth-callbacks) require authentication.

Authentication is done through the `Authorization` header 
that has to be set to `Bearer { cookie(auth_token) }`.

# Web Interface

## Editors and Broadcasters

A broadcaster can add multiple editors who then can manage the rewards.
This is done in the `Editors`-tab. 

**Important:** A broadcaster can **only** add editors that have registered on this app (like on BTTV).

The editors can access the rewards through the `Broadcasters` tab.

## Rewards

You can edit the rewards in the `Rewards` tab.

In the dialog there are two sides. 

On the left side you can edit Twitch specific parameters like the reward title or the cooldown.
The cooldown can be specified like this: `1h`, `3.5d`, `3m` or `123` (seconds).

On the right side you can edit the specific action done when this reward is redeemed.

Some rewards expose a `Duration` configuration.
Here, you can specify the duration like this: `1hour 3min 4ms`. For more examples go [here](https://docs.rs/humantime/2.1.0/humantime/fn.parse_duration.html).