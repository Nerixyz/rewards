alter table slots
    alter column emote_id type varchar(25) using emote_id::varchar(25);

alter table swap_emotes
    alter column emote_id type varchar(25) using emote_id::varchar(25);