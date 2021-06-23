create table timeouts
(
    channel_id varchar(16) not null
        constraint timeouts_pk
            primary key
        constraint timeouts_users_id_fk
            references users
            on delete cascade,
    user_id varchar(16) not null,
    expires_at timestamptz not null
);
