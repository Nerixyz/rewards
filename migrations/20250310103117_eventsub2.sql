create table if not exists eventsubs
(
    id      varchar(63)  not null
        constraint eventsubs_pk
            primary key,
    user_id varchar(16)  not null
        constraint eventsubs_users_id_fk
            references users
            on delete cascade,
    name    varchar(255) not null
);

create unique index if not exists eventsubs_user_id_name_uindex
    on eventsubs (user_id, name);

insert into eventsubs (id, user_id, name)
select eventsub_id, id, 'channel.channel_points_custom_reward_redemption.add'
from users
where eventsub_id is not null;

alter table users drop column eventsub_id;
