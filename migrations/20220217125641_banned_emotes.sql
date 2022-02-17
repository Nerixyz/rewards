create table banned_emotes
(
    channel_id varchar(16)   not null
        constraint banned_emotes_users_id_fk
            references users
            on delete cascade,
    emote_id   varchar(24)   not null,
    platform   slot_platform not null
);

create unique index banned_emotes_channel_id_emote_id_platform_uindex
    on banned_emotes (channel_id, emote_id, platform);
