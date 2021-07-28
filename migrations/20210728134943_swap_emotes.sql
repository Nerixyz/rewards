-- same as slot_platform
create type swap_platform as enum ('bttv', 'ffz', '7tv');

create table swap_emotes
(
    id bigserial not null,
    user_id varchar(16) not null,
    emote_id varchar(24) not null,
    platform slot_platform not null,
    name varchar(100) not null,
    added_by varchar(40) not null,
    added_at timestamptz not null
);

create unique index swap_emotes_id_uindex
    on swap_emotes (id);

create unique index swap_emotes_name_uindex
    on swap_emotes (user_id, platform, name);

alter table swap_emotes
    add constraint swap_emotes_pk
        primary key (id);

insert into swap_emotes (user_id, emote_id, platform, name, added_by, added_at)
select
       id as user_id,
       jsonb_array_elements_text(seventv_history) as emote_id,
       '7tv' as platform,
       '{?:' || jsonb_array_elements_text(seventv_history) || '}' as name,
       'unknown' as added_by,
       now() as added_at
from users;

insert into swap_emotes (user_id, emote_id, platform, name, added_by, added_at)
select
    id as user_id,
    jsonb_array_elements_text(ffz_history) as emote_id,
    'ffz' as platform,
    '{?:' || jsonb_array_elements_text(ffz_history) || '}' as name,
    'unknown' as added_by,
    now() as added_at
from users;

insert into swap_emotes (user_id, emote_id, platform, name, added_by, added_at)
select
    id as user_id,
    jsonb_array_elements_text(bttv_history) as emote_id,
    'bttv' as platform,
    '{?:' || jsonb_array_elements_text(bttv_history) || '}' as name,
    'unknown' as added_by,
    now() as added_at
from users;
