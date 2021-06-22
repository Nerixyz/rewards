alter table users
    add if not exists seventv_id varchar(24) default null;

alter table users
    add if not exists seventv_history jsonb default '[]' not null;

create unique index if not exists users_seventv_id_uindex
    on users (seventv_id);