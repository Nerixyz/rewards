create table spotify
(
    user_id varchar(16) not null
        constraint spotify_users_id_fk
            references users
            on delete cascade,
    access_token text not null,
    refresh_token text not null
);

create unique index spotify_user_id_uindex
    on spotify (user_id);

alter table spotify
    add constraint spotify_pk
        primary key (user_id);
