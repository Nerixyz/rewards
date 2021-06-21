drop type if exists timed_mode;
create type timed_mode as enum ('emoteonly', 'subonly');
create table if not exists timed_modes
(
    id serial not null,
    user_id varchar(16) not null
        constraint timed_mode_users_id_fk
            references users,
    mode timed_mode not null,
    end_ts timestamptz not null
);

create unique index if not exists timed_mode_id_uindex
    on timed_modes (id);

alter table timed_modes
    add constraint timed_mode_pk
        primary key (id);

