# Rewards

This is an application that manages custom Twitch rewards.

The priorities for the project are _P1_ .. _Px_ where _P1_ is the most important.

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


# Development

## Setup

* Setup a [`postgres`](https://www.postgresql.org/) database.
* Create a new application on the [Twitch Console](https://dev.twitch.tv/console/apps).
* Copy `.env.example` to `.env` and set the appropriate values.
* Run `setup.sh` or `setup.bat` depending on your platform.
* Now you're done!

### Setting up a development environment

* Use [`ngrok`](https://ngrok.com/) to create a tunnel to your machine for _eventsub_.
* Edit the `SERVER_URL` to the ngrok-https-url.
* In the `web` directory set the `VITE_API_BASE_URL` to the ngrok-https-url.

## TODO

This may be managed in the issues.

### P1

* [x] WebInterface
* [x] EventSub
* [x] IRC
* [ ] Verify index.html
* [x] Use migrations  
* [x] Document setup
* [ ] Check TODOs throughout the code
* [ ] Get rid of cloning everywhere
* [ ] Handle timeout errors

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