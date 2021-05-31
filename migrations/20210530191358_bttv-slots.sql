create table bttv_slots
(
    id serial not null,
    user_id varchar(16) not null
        constraint bttv_slots_users_id_fk
            references users
            on delete cascade,
    emote_id varchar(24) default null,
    expires timestamptz default null,
    reward_id varchar(36) not null
        constraint bttv_slots_rewards_id_fk
            references rewards
            on delete cascade
);

comment on column bttv_slots.emote_id is 'If this is null, the slot is free, else it''s occupied';

create unique index bttv_slots_id_uindex
    on bttv_slots (id);

alter table bttv_slots
    add constraint bttv_slots_pk
        primary key (id);

