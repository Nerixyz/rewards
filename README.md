# Rewards

This is an application that manages custom Twitch rewards.

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
Here, you can specify the duration like this: `1hour 3min 4ms`. 
For more examples go [here](https://docs.rs/humantime/2.1.0/humantime/fn.parse_duration.html).

You can also provide the duration like this: `rand( 10m ; 10h )`. 
This would timeout the user for a random duration between `10m` and `1h`.

### Available Rewards

* Timeout user (set or random duration)
* Enter Subonly-mode (set or random duration)
* Enter Emoteonly-mode (set or random duration)
* Swap/Add BTTV emotes (requires `RewardMore` to be an editor)
* Swap/Add FFZ emotes (requires `RewardMore` to be an editor)
* Add BTTV emotes to slots that expire after a set amount of time (+/- 2min)
* Skip a Spotify track
* Play a Spotify track
* Queue a Spotify track


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

## Roadmap

Things to-do are tracked [here](https://github.com/Nerixyz/rewards/projects).

# Internal - web API

All endpoints are on `/api/v1`. All endpoints (except `/auth/twich-auth-url` and the auth-callbacks) require authentication.

Authentication is done through the `Authorization` header 
that has to be set to `Bearer { cookie(auth_token) }`.
