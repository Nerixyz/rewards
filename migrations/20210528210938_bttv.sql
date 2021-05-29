alter table users
    add if not exists bttv_id varchar(24) default null;

alter table users
    add if not exists bttv_history jsonb default '[]' not null;

create unique index if not exists users_bttv_id_uindex
    on users (bttv_id);
