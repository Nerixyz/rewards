create table discord_settings
(
    user_id varchar(16) not null
        constraint discord_settings_users_id_fk
            references users
            on delete cascade,
    url varchar(255) not null,
    log_emotes bool default true not null
);

create unique index discord_settings_user_id_uindex
    on discord_settings (user_id);

alter table discord_settings
    add constraint discord_settings_pk
        primary key (user_id);

