create table if not exists users
(
    id            varchar(16) not null
        constraint users_pk
            primary key,
    access_token  varchar(30) not null,
    refresh_token text        not null,
    scopes        text        not null,
    name          varchar(25) not null,
    eventsub_id   varchar(36)
);

comment on column users.id is 'The twitch user-id';

comment on column users.refresh_token is 'The length of the refresh-token isn''t set';

create unique index if not exists users_id_uindex
    on users (id);

create table if not exists rewards
(
    id      varchar(36) not null
        constraint rewards_pk
            primary key,
    user_id varchar(16) not null
        constraint rewards_users_id_fk
            references users
            on delete cascade,
    data    jsonb       not null
);

create unique index if not exists rewards_id_uindex
    on rewards (id);

create table if not exists editors
(
    editor_id      varchar(16) not null
        constraint editors_users_id_fk
            references users
            on delete cascade,
    broadcaster_id varchar(16) not null
        constraint editors_users_id_fk_2
            references users
            on delete cascade,
    constraint editors_pk
        primary key (editor_id, broadcaster_id)
);

create table if not exists config
(
    key   varchar(16) not null
        constraint config_pk
            primary key,
    value jsonb       not null
);

create unique index if not exists config_key_uindex
    on config (key);
