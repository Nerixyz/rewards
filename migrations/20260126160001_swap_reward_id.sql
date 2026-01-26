alter table swap_emotes
    add reward_id varchar(36)
        constraint swap_emotes_rewards_id_fk
            references rewards
            on delete cascade;

alter table swap_emotes
    add constraint swap_emotes_users_id_fk
        foreign key (user_id) references users
            on delete cascade;

update
    swap_emotes s
set reward_id = r.id
from rewards r
where s.platform = '7tv'
  and s.user_id = r.user_id
  and r.data ->> 'type' = 'SevenTvSwap';

update
    swap_emotes s
set reward_id = r.id
from rewards r
where s.platform = 'bttv'
  and s.user_id = r.user_id
  and r.data ->> 'type' = 'BttvSwap';

update
    swap_emotes s
set reward_id = r.id
from rewards r
where s.platform = 'ffz'
  and s.user_id = r.user_id
  and r.data ->> 'type' = 'FfzSwap';

alter table swap_emotes
    alter column reward_id set not null;
