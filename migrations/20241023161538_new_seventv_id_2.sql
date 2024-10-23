alter table slots
    alter column emote_id type varchar(32) using emote_id::varchar(32);

alter table swap_emotes
    alter column emote_id type varchar(32) using emote_id::varchar(32);